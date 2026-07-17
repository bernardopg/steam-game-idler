use crate::utils::{create_private_dir_all, get_cache_dir, get_user_data_dir};
use serde_json::{json, Value};
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

const LEGACY_APP_IDENTIFIER: &str = "com.zevnda.steam-game-idler";
const LEGACY_MIGRATION_MARKER: &str = ".legacy-profile-migration-v1.json";
const WEBKIT_DATA_DIRECTORIES: &[&str] = &[
    "CacheStorage",
    "WebKitCache",
    "cookies",
    "localstorage",
    "mediakeys",
    "storage",
];

// Default settings
fn get_default_settings() -> Value {
    json!({
        "gameSettings": null,
        "general": {
            "antiAway": false,
            "freeGameNotifications": true,
            "apiKey": null,
            "disableTooltips": false,
            "runAtStartup": false,
            "startMinimized": false,
            "closeToTray": true,
            "customBackground": null,
            "autoRedeemFreeGames": false,
            "autoUpdateGamesList": false,
            "showRecommendedCarousel": true,
            "showRecentCarousel": true,
            "showCardDropsCarousel": false
        },
        "cardFarming": {
            "listGames": false,
            "allGames": true,
            "nextTaskCheckbox": false,
            "nextTask": null,
            "credentials": null,
            "userSummary": null,
            "gamesWithDrops": 0,
            "totalDropsRemaining": 0,
            "blacklist": null,
            "skipNoPlaytime": false,
            "farmUnplayedOnly": false,
            "sortByHighestDrops": false,
            "sortByLowestDrops": false
        },
        "achievementUnlocker": {
            "idle": true,
            "hidden": false,
            "nextTaskCheckbox": false,
            "nextTask": null,
            "schedule": false,
            "scheduleFrom": {
                "hour": 8,
                "minute": 30,
                "second": 0,
                "millisecond": 0
            },
            "scheduleTo": {
                "hour": 23,
                "minute": 0,
                "second": 0,
                "millisecond": 0
            },
            "interval": [
                30,
                130
            ]
        },
        "tradingCards": {
            "sellOptions": "highestBuyOrder",
            "priceAdjustment": 0.0,
            "sellLimit": {
                "min": 0.01,
                "max": 10
            },
            "sellDelay": 10
        },
    })
}

// Recursively merge missing keys from default into user settings
fn merge_defaults(user: &mut Value, default: &Value) {
    match (user, default) {
        (Value::Object(user_map), Value::Object(default_map)) => {
            for (k, v) in default_map {
                if !user_map.contains_key(k) {
                    user_map.insert(k.clone(), v.clone());
                } else {
                    merge_defaults(user_map.get_mut(k).unwrap(), v);
                }
            }
        }
        // For arrays and other types, do nothing (or could handle arrays if needed)
        _ => {}
    }
}

fn copy_directory_contents(source_dir: &Path, target_dir: &Path) -> Result<(), String> {
    if !source_dir.exists() {
        return Ok(());
    }

    create_private_dir_all(target_dir)?;

    for entry in fs::read_dir(source_dir).map_err(|e| {
        format!(
            "Failed to read migration source directory {}: {}",
            source_dir.display(),
            e
        )
    })? {
        let entry = entry.map_err(|e| e.to_string())?;
        let source_path = entry.path();
        let target_path = target_dir.join(entry.file_name());
        let file_type = entry.file_type().map_err(|e| e.to_string())?;

        if file_type.is_dir() {
            copy_directory_contents(&source_path, &target_path)?;
            continue;
        }

        if file_type.is_file() && !target_path.exists() {
            fs::copy(&source_path, &target_path).map_err(|e| {
                format!(
                    "Failed to copy {} to {}: {}",
                    source_path.display(),
                    target_path.display(),
                    e
                )
            })?;
        }
    }

    Ok(())
}

fn copy_webkit_directory(
    source_dir: &Path,
    target_dir: &Path,
    has_current_profiles: bool,
) -> Result<(), String> {
    if !source_dir.exists() {
        return Ok(());
    }

    if target_dir.exists() {
        if has_current_profiles {
            return Ok(());
        }
        fs::remove_dir_all(target_dir).map_err(|error| {
            format!(
                "Failed to replace initialized WebKit directory {}: {}",
                target_dir.display(),
                error
            )
        })?;
    }

    copy_directory_contents(source_dir, target_dir)
}

fn is_missing_value(value: &Value) -> bool {
    matches!(value, Value::Null) || matches!(value, Value::String(text) if text.trim().is_empty())
}

/// Merge persisted settings without replacing an explicit value from the current profile.
///
/// API keys and Steam credentials are represented as strings (or nested objects), so this
/// also recovers a partially saved credential set while keeping values the user entered in
/// the current application profile.
fn merge_missing_values(current: &mut Value, legacy: &Value) {
    match (current, legacy) {
        (Value::Object(current_map), Value::Object(legacy_map)) => {
            for (key, legacy_value) in legacy_map {
                match current_map.get_mut(key) {
                    Some(current_value) => merge_missing_values(current_value, legacy_value),
                    None => {
                        current_map.insert(key.clone(), legacy_value.clone());
                    }
                }
            }
        }
        (current_value, legacy_value) if is_missing_value(current_value) => {
            *current_value = legacy_value.clone();
        }
        _ => {}
    }
}

fn read_json(path: &Path) -> Result<Value, String> {
    let contents = fs::read_to_string(path)
        .map_err(|error| format!("Failed to read {}: {}", path.display(), error))?;
    serde_json::from_str(&contents)
        .map_err(|error| format!("Failed to parse {}: {}", path.display(), error))
}

fn write_json(path: &Path, value: &Value) -> Result<(), String> {
    let contents = serde_json::to_string_pretty(value)
        .map_err(|error| format!("Failed to serialize {}: {}", path.display(), error))?;
    fs::write(path, contents)
        .map_err(|error| format!("Failed to write {}: {}", path.display(), error))
}

fn merge_settings_file(legacy_path: &Path, current_path: &Path) -> Result<(), String> {
    let legacy_settings = read_json(legacy_path)?;
    if !current_path.exists() {
        return write_json(current_path, &legacy_settings);
    }

    let mut current_settings = read_json(current_path)?;
    let before = current_settings.clone();
    merge_missing_values(&mut current_settings, &legacy_settings);
    if current_settings != before {
        write_json(current_path, &current_settings)?;
    }

    Ok(())
}

fn is_steam_profile_dir(path: &Path) -> bool {
    path.join("settings.json").is_file()
}

fn has_steam_profiles(root: &Path) -> bool {
    fs::read_dir(root)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .any(|entry| {
            entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false)
                && is_steam_profile_dir(&entry.path())
        })
}

/// Migrate data created before the application identifier changed.
///
/// The migration is intentionally conservative: current values always win, WebKit databases
/// are copied only when their destination does not exist, and old logs are not imported.
/// Keeping it path-based makes this critical upgrade behavior unit-testable without Tauri.
fn migrate_legacy_app_data_at_paths(
    legacy_root: &Path,
    current_root: &Path,
) -> Result<bool, String> {
    let marker_path = current_root.join(LEGACY_MIGRATION_MARKER);
    if marker_path.exists() || !legacy_root.is_dir() || legacy_root == current_root {
        return Ok(false);
    }

    create_private_dir_all(current_root)?;
    // Tauri may initialize WebKit directories before `setup` runs. If the current
    // application has no saved SGI profile yet, those files cannot represent user data,
    // so replace the whole store and preserve the legacy `userSummary` localStorage.
    let has_current_profiles = has_steam_profiles(current_root);

    for entry in fs::read_dir(legacy_root).map_err(|error| {
        format!(
            "Failed to read legacy application data {}: {}",
            legacy_root.display(),
            error
        )
    })? {
        let entry = entry.map_err(|error| error.to_string())?;
        let source_path = entry.path();
        let target_path = current_root.join(entry.file_name());
        let file_type = entry.file_type().map_err(|error| error.to_string())?;
        let name = entry.file_name();
        let name = name.to_string_lossy();

        // Log history has no state needed to operate the app and would make an upgrade look
        // like it generated old errors again.
        if name == "log.txt" || name == LEGACY_MIGRATION_MARKER {
            continue;
        }

        if file_type.is_dir() {
            if is_steam_profile_dir(&source_path) {
                // Validate before copying. A malformed legacy settings file must never be
                // introduced into a previously clean current profile.
                read_json(&source_path.join("settings.json"))?;
                copy_directory_contents(&source_path, &target_path)?;
                merge_settings_file(
                    &source_path.join("settings.json"),
                    &target_path.join("settings.json"),
                )?;
            } else if WEBKIT_DATA_DIRECTORIES.contains(&name.as_ref()) {
                // SQLite/WebKit stores consist of multiple related files. Merging an already
                // initialized store could produce an inconsistent database, so transfer only
                // complete stores into a profile that has not created one yet.
                copy_webkit_directory(&source_path, &target_path, has_current_profiles)?;
            } else {
                copy_directory_contents(&source_path, &target_path)?;
            }
        } else if file_type.is_file() && !target_path.exists() {
            fs::copy(&source_path, &target_path).map_err(|error| {
                format!(
                    "Failed to copy {} to {}: {}",
                    source_path.display(),
                    target_path.display(),
                    error
                )
            })?;
        }
    }

    write_json(
        &marker_path,
        &json!({
            "source": LEGACY_APP_IDENTIFIER,
            "version": 1,
        }),
    )?;

    Ok(true)
}

pub fn migrate_legacy_app_data(app_handle: &tauri::AppHandle) -> Result<bool, String> {
    if crate::utils::is_portable() {
        return Ok(false);
    }

    let current_root = get_user_data_dir(app_handle)?;
    let data_root = current_root.parent().ok_or_else(|| {
        format!(
            "Could not determine the parent data directory for {}",
            current_root.display()
        )
    })?;
    let legacy_root = data_root.join(LEGACY_APP_IDENTIFIER);

    migrate_legacy_app_data_at_paths(&legacy_root, &current_root)
}

#[tauri::command]
pub async fn migrate_local_dev_profile(
    source_steam_id: String,
    target_steam_id: String,
    app_handle: tauri::AppHandle,
) -> Result<Value, String> {
    if source_steam_id.is_empty()
        || target_steam_id.is_empty()
        || source_steam_id == target_steam_id
    {
        return Ok(json!({ "success": true, "migrated": false }));
    }

    let user_data_dir = get_user_data_dir(&app_handle)?;
    let cache_dir = get_cache_dir(&app_handle)?;

    let source_user_dir = user_data_dir.join(&source_steam_id);
    let target_user_dir = user_data_dir.join(&target_steam_id);
    copy_directory_contents(&source_user_dir, &target_user_dir)?;

    if cache_dir != user_data_dir {
        let source_cache_dir = cache_dir.join(&source_steam_id);
        let target_cache_dir = cache_dir.join(&target_steam_id);
        copy_directory_contents(&source_cache_dir, &target_cache_dir)?;
    }

    Ok(json!({
        "success": true,
        "migrated": true,
        "sourceSteamId": source_steam_id,
        "targetSteamId": target_steam_id,
    }))
}

fn get_settings_file_path(
    app_handle: &tauri::AppHandle,
    steam_id: &str,
) -> Result<PathBuf, String> {
    let app_data_dir = get_user_data_dir(app_handle)?.join(steam_id);

    if !app_data_dir.exists() {
        create_private_dir_all(&app_data_dir)?;
    }

    let settings_file_path = app_data_dir.join("settings.json");
    if !settings_file_path.exists() {
        let legacy_settings_file_path = get_cache_dir(app_handle)?
            .join(steam_id)
            .join("settings.json");

        if legacy_settings_file_path.exists() && legacy_settings_file_path != settings_file_path {
            fs::copy(&legacy_settings_file_path, &settings_file_path)
                .map_err(|e| format!("Failed to migrate settings file: {}", e))?;
        }
    }

    Ok(settings_file_path)
}

#[tauri::command]
pub async fn get_user_settings(
    steam_id: String,
    app_handle: tauri::AppHandle,
) -> Result<Value, String> {
    let settings_file_path = get_settings_file_path(&app_handle, &steam_id)?;

    // Get default settings
    let default_settings = get_default_settings();

    // Read the settings file or create with default settings if it doesn't exist
    let mut settings = if settings_file_path.exists() {
        let mut file = File::open(&settings_file_path)
            .map_err(|e| format!("Failed to open settings file: {}", e))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| format!("Failed to read settings file: {}", e))?;
        serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse settings JSON: {}", e))?
    } else {
        // Create a new file with default settings
        let json_string = serde_json::to_string_pretty(&default_settings)
            .map_err(|e| format!("Failed to serialize default settings JSON: {}", e))?;
        let mut file = File::create(&settings_file_path)
            .map_err(|e| format!("Failed to create settings file: {}", e))?;
        file.write_all(json_string.as_bytes())
            .map_err(|e| format!("Failed to write to settings file: {}", e))?;
        default_settings.clone()
    };

    // Merge missing keys from default settings
    let before = settings.clone();
    merge_defaults(&mut settings, &default_settings);
    if settings != before {
        // Write back if any changes
        let json_string = serde_json::to_string_pretty(&settings)
            .map_err(|e| format!("Failed to serialize settings JSON: {}", e))?;
        let mut file = File::create(&settings_file_path)
            .map_err(|e| format!("Failed to create settings file: {}", e))?;
        file.write_all(json_string.as_bytes())
            .map_err(|e| format!("Failed to write to settings file: {}", e))?;
    }

    Ok(json!({
        "settings": settings
    }))
}

#[tauri::command]
pub async fn update_user_settings(
    steam_id: String,
    key: String,
    value: Value,
    app_handle: tauri::AppHandle,
) -> Result<Value, String> {
    let settings_file_path = get_settings_file_path(&app_handle, &steam_id)?;

    // Read current settings or create with default settings if it doesn't exist
    let mut settings = if settings_file_path.exists() {
        let mut file = File::open(&settings_file_path)
            .map_err(|e| format!("Failed to open settings file: {}", e))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| format!("Failed to read settings file: {}", e))?;
        serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse settings JSON: {}", e))?
    } else {
        get_default_settings()
    };

    // Parse the key
    let path_parts: Vec<&str> = key.split('.').collect();
    if path_parts.is_empty() {
        return Err("Invalid key path".to_string());
    }

    // Navigate through the JSON structure and update the specified value
    let mut current = &mut settings;
    for (i, &part) in path_parts.iter().enumerate() {
        if i == path_parts.len() - 1 {
            // This is the final key to update
            if let Some(obj) = current.as_object_mut() {
                obj.insert(part.to_string(), value.clone());
            } else {
                return Err(format!(
                    "Cannot update key '{}': parent is not an object",
                    part
                ));
            }
        } else {
            // Navigate to the next level in the JSON structure
            if let Some(obj) = current.as_object_mut() {
                if !obj.contains_key(part) {
                    obj.insert(part.to_string(), json!({}));
                }
                current = obj.get_mut(part).unwrap();
            } else {
                return Err(format!(
                    "Cannot navigate to key '{}': parent is not an object",
                    part
                ));
            }
        }
    }

    // Write the updated settings back to the file
    let json_string = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings JSON: {}", e))?;
    let mut file = File::create(&settings_file_path)
        .map_err(|e| format!("Failed to create settings file: {}", e))?;
    file.write_all(json_string.as_bytes())
        .map_err(|e| format!("Failed to write to settings file: {}", e))?;

    Ok(json!({
        "settings": settings
    }))
}

#[tauri::command]
pub async fn reset_user_settings(
    steam_id: String,
    app_handle: tauri::AppHandle,
) -> Result<Value, String> {
    let settings_file_path = get_settings_file_path(&app_handle, &steam_id)?;

    // Get default settings
    let default_settings = get_default_settings();

    // Create or overwrite the settings file with default settings
    let json_string = serde_json::to_string_pretty(&default_settings)
        .map_err(|e| format!("Failed to serialize default settings JSON: {}", e))?;
    let mut file = File::create(&settings_file_path)
        .map_err(|e| format!("Failed to create settings file: {}", e))?;
    file.write_all(json_string.as_bytes())
        .map_err(|e| format!("Failed to write to settings file: {}", e))?;

    Ok(json!({
        "settings": default_settings
    }))
}

pub async fn check_start_minimized_setting(app_handle: &tauri::AppHandle) -> Result<bool, String> {
    use serde_json::Value;
    use std::fs::File;
    use std::io::Read;
    use std::time::SystemTime;

    fn read_start_minimized_from_dir(app_data_dir: &PathBuf) -> Option<bool> {
        let mut candidates = Vec::new();

        if let Ok(entries) = std::fs::read_dir(app_data_dir) {
            for entry in entries.flatten() {
                if !entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                    continue;
                }

                let settings_file = entry.path().join("settings.json");
                let Ok(mut file) = File::open(&settings_file) else {
                    continue;
                };
                let mut contents = String::new();
                if file.read_to_string(&mut contents).is_err() {
                    continue;
                }
                let Ok(settings) = serde_json::from_str::<Value>(&contents) else {
                    continue;
                };
                let Some(start_minimized) = settings
                    .get("general")
                    .and_then(|general| general.get("startMinimized"))
                    .and_then(Value::as_bool)
                else {
                    continue;
                };

                let modified = settings_file
                    .metadata()
                    .and_then(|metadata| metadata.modified())
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                candidates.push((modified, entry.file_name(), start_minimized));
            }
        }

        candidates
            .into_iter()
            .max_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)))
            .map(|(_, _, start_minimized)| start_minimized)
    }

    let app_data_dir = get_user_data_dir(app_handle)?;
    if let Some(start_minimized) = read_start_minimized_from_dir(&app_data_dir) {
        return Ok(start_minimized);
    }

    let legacy_cache_dir = get_cache_dir(app_handle)?;
    if legacy_cache_dir != app_data_dir {
        if let Some(start_minimized) = read_start_minimized_from_dir(&legacy_cache_dir) {
            return Ok(start_minimized);
        }
    }

    // Default to false if no setting found
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_directory(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "sgi-settings-{name}-{}-{unique}",
            std::process::id()
        ))
    }

    fn settings(api_key: Value, credentials: Value, start_minimized: bool) -> Value {
        json!({
            "general": { "apiKey": api_key, "startMinimized": start_minimized },
            "cardFarming": { "credentials": credentials },
        })
    }

    #[test]
    fn migrates_a_legacy_profile_once_without_importing_logs() {
        let root = temporary_directory("legacy-migration");
        let legacy = root.join("legacy");
        let current = root.join("current");
        let steam_id = "76561198000000000";
        let legacy_profile = legacy.join(steam_id);
        create_private_dir_all(&legacy_profile).unwrap();
        write_json(
            &legacy_profile.join("settings.json"),
            &settings(
                json!("legacy-api-key"),
                json!({ "sid": "legacy-sid", "sls": "legacy-sls" }),
                true,
            ),
        )
        .unwrap();
        create_private_dir_all(&legacy.join("localstorage")).unwrap();
        fs::write(legacy.join("localstorage").join("state"), "legacy state").unwrap();
        create_private_dir_all(&current.join("localstorage")).unwrap();
        fs::write(
            current.join("localstorage").join("state"),
            "empty new state",
        )
        .unwrap();
        fs::write(legacy.join("log.txt"), "old log entry").unwrap();

        assert!(migrate_legacy_app_data_at_paths(&legacy, &current).unwrap());
        let migrated = read_json(&current.join(steam_id).join("settings.json")).unwrap();
        assert_eq!(migrated["general"]["apiKey"], json!("legacy-api-key"));
        assert_eq!(
            migrated["cardFarming"]["credentials"]["sid"],
            json!("legacy-sid")
        );
        assert!(current.join("localstorage").join("state").exists());
        assert_eq!(
            fs::read_to_string(current.join("localstorage").join("state")).unwrap(),
            "legacy state"
        );
        assert!(!current.join("log.txt").exists());
        assert!(current.join(LEGACY_MIGRATION_MARKER).exists());
        assert!(!migrate_legacy_app_data_at_paths(&legacy, &current).unwrap());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn retains_current_values_and_fills_only_missing_legacy_values() {
        let root = temporary_directory("merge-settings");
        let legacy = root.join("legacy");
        let current = root.join("current");
        let steam_id = "76561198000000001";
        let legacy_profile = legacy.join(steam_id);
        let current_profile = current.join(steam_id);
        create_private_dir_all(&legacy_profile).unwrap();
        create_private_dir_all(&current_profile).unwrap();

        write_json(
            &legacy_profile.join("settings.json"),
            &settings(
                json!("legacy-api-key"),
                json!({ "sid": "legacy-sid", "sls": "legacy-sls" }),
                true,
            ),
        )
        .unwrap();
        write_json(
            &current_profile.join("settings.json"),
            &settings(
                json!("current-api-key"),
                json!({ "sid": "current-sid", "sls": "" }),
                false,
            ),
        )
        .unwrap();
        create_private_dir_all(&legacy.join("localstorage")).unwrap();
        create_private_dir_all(&current.join("localstorage")).unwrap();
        fs::write(legacy.join("localstorage").join("state"), "legacy state").unwrap();
        fs::write(current.join("localstorage").join("state"), "current state").unwrap();

        assert!(migrate_legacy_app_data_at_paths(&legacy, &current).unwrap());
        let migrated = read_json(&current_profile.join("settings.json")).unwrap();
        assert_eq!(migrated["general"]["apiKey"], json!("current-api-key"));
        assert_eq!(migrated["general"]["startMinimized"], json!(false));
        assert_eq!(
            migrated["cardFarming"]["credentials"]["sid"],
            json!("current-sid")
        );
        assert_eq!(
            migrated["cardFarming"]["credentials"]["sls"],
            json!("legacy-sls")
        );
        assert_eq!(
            fs::read_to_string(current.join("localstorage").join("state")).unwrap(),
            "current state"
        );

        fs::remove_dir_all(root).unwrap();
    }
}
