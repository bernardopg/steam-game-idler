use crate::command_runner::apply_hidden_command_style;
use crate::utils::{get_cache_dir, get_lib_path};
use serde_json::{json, Value};
use std::fs::File;
use std::io::Read;

#[tauri::command]
pub async fn get_achievement_data(
    steam_id: String,
    app_id: u32,
    refetch: Option<bool>,
    app_handle: tauri::AppHandle,
) -> Result<Value, String> {
    let app_data_dir = get_cache_dir(&app_handle)?
        .join(steam_id.clone())
        .join("achievement_data");

    let file_name = format!("{}.json", app_id);
    let achievement_file_path = app_data_dir.join(&file_name);

    let should_fetch_new = refetch.unwrap_or(false) || !achievement_file_path.exists();

    let achievement_data = if should_fetch_new {
        let cache_dir = get_cache_dir(&app_handle)?;
        let cache_dir_str = cache_dir.to_string_lossy().to_string();

        let exe_path = get_lib_path()?;
        let mut command = std::process::Command::new(exe_path);
        command.args(["get_achievement_data", &app_id.to_string(), &cache_dir_str]);
        let output = apply_hidden_command_style(&mut command)
            .output()
            .map_err(|e| format!("Failed to execute unlocker: {}", e))?;

        let output_str = String::from_utf8_lossy(&output.stdout);

        if output_str.contains("error") {
            return Ok(output_str.to_string().into());
        }

        if output_str.contains("success") {
            if achievement_file_path.exists() {
                let mut file = File::open(&achievement_file_path)
                    .map_err(|e| format!("Failed to open achievement file: {}", e))?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)
                    .map_err(|e| format!("Failed to read achievement file: {}", e))?;
                serde_json::from_str(&contents)
                    .map_err(|e| format!("Failed to parse achievement JSON: {}", e))?
            } else {
                json!({})
            }
        } else {
            json!({})
        }
    } else {
        let mut file = File::open(&achievement_file_path)
            .map_err(|e| format!("Failed to open achievement file: {}", e))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| format!("Failed to read achievement file: {}", e))?;
        serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse achievement JSON: {}", e))?
    };

    Ok(json!({"achievement_data": achievement_data}))
}

fn run_steam_utility_command(args: &[&str]) -> Result<String, String> {
    let exe_path = get_lib_path()?;
    let mut command = std::process::Command::new(exe_path);
    command.args(args);

    let output = apply_hidden_command_style(&mut command)
        .output()
        .map_err(|e| format!("Failed to execute SteamUtility command: {}", e))?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[tauri::command]
pub async fn unlock_achievement(app_id: u32, achievement_id: &str) -> Result<String, String> {
    run_steam_utility_command(&["unlock_achievement", &app_id.to_string(), achievement_id])
}

#[tauri::command]
pub async fn lock_achievement(app_id: u32, achievement_id: &str) -> Result<String, String> {
    run_steam_utility_command(&["lock_achievement", &app_id.to_string(), achievement_id])
}

#[tauri::command]
pub async fn toggle_achievement(app_id: u32, achievement_id: &str) -> Result<String, String> {
    run_steam_utility_command(&["toggle_achievement", &app_id.to_string(), achievement_id])
}

#[tauri::command]
pub async fn unlock_all_achievements(app_id: u32) -> Result<String, String> {
    run_steam_utility_command(&["unlock_all_achievements", &app_id.to_string()])
}

#[tauri::command]
pub async fn lock_all_achievements(app_id: u32) -> Result<String, String> {
    run_steam_utility_command(&["lock_all_achievements", &app_id.to_string()])
}

#[tauri::command]
pub async fn update_stats(app_id: u32, stats_arr: &str) -> Result<String, String> {
    run_steam_utility_command(&["update_stats", &app_id.to_string(), stats_arr])
}

#[tauri::command]
pub async fn reset_all_stats(app_id: u32) -> Result<String, String> {
    run_steam_utility_command(&["reset_all_stats", &app_id.to_string()])
}
