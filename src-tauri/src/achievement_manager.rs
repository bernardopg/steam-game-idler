use crate::command_runner::apply_hidden_command_style;
use crate::utils::{get_cache_dir, get_lib_path};
use serde_json::{json, Value};
use std::fs::File;
use std::io::Read;
use tokio::sync::Semaphore;

// Caps concurrent SteamUtility child processes spawned for achievement/stat
// operations. Without a cap the unlocker can flood the async runtime with
// blocking process spawns and stall every other task.
lazy_static::lazy_static! {
    static ref ACHIEVEMENT_PROCESS_LIMIT: Semaphore = Semaphore::new(6);
}

// Run a SteamUtility subcommand off the runtime's worker threads via
// spawn_blocking, throttled to ACHIEVEMENT_PROCESS_LIMIT concurrent processes.
async fn spawn_lib_output(args: Vec<String>) -> Result<std::process::Output, String> {
    let _permit = ACHIEVEMENT_PROCESS_LIMIT
        .acquire()
        .await
        .map_err(|e| format!("Failed to acquire achievement process slot: {}", e))?;

    let exe_path = get_lib_path()?;
    tokio::task::spawn_blocking(move || {
        let mut command = std::process::Command::new(exe_path);
        command.args(&args);
        apply_hidden_command_style(&mut command).output()
    })
    .await
    .map_err(|e| format!("Failed to join blocking task: {}", e))?
    .map_err(|e| format!("Failed to execute SteamUtility command: {}", e))
}

fn parse_steam_utility_stdout(stdout: &[u8], stderr: &[u8]) -> Result<Value, String> {
    let output_str = String::from_utf8_lossy(stdout);
    let error_str = String::from_utf8_lossy(stderr);

    serde_json::from_str(output_str.trim()).map_err(|e| {
        format!(
            "Failed to parse SteamUtility JSON output: {}\nSTDOUT: {}\nSTDERR: {}",
            e, output_str, error_str
        )
    })
}

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

        let output = spawn_lib_output(vec![
            "get_achievement_data".to_string(),
            app_id.to_string(),
            cache_dir_str,
        ])
        .await?;

        let status = parse_steam_utility_stdout(&output.stdout, &output.stderr)?;

        if status.get("error").is_some() {
            return Ok(status.to_string().into());
        }

        if status.get("success").is_some() {
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

async fn run_steam_utility_command(args: &[&str]) -> Result<String, String> {
    let owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let output = spawn_lib_output(owned).await?;
    let status = parse_steam_utility_stdout(&output.stdout, &output.stderr)?;
    Ok(status.to_string())
}

#[tauri::command]
pub async fn unlock_achievement(app_id: u32, achievement_id: &str) -> Result<String, String> {
    run_steam_utility_command(&["unlock_achievement", &app_id.to_string(), achievement_id])
        .await
}

#[tauri::command]
pub async fn lock_achievement(app_id: u32, achievement_id: &str) -> Result<String, String> {
    run_steam_utility_command(&["lock_achievement", &app_id.to_string(), achievement_id])
        .await
}

#[tauri::command]
pub async fn toggle_achievement(app_id: u32, achievement_id: &str) -> Result<String, String> {
    run_steam_utility_command(&["toggle_achievement", &app_id.to_string(), achievement_id])
        .await
}

#[tauri::command]
pub async fn unlock_all_achievements(app_id: u32) -> Result<String, String> {
    run_steam_utility_command(&["unlock_all_achievements", &app_id.to_string()])
        .await
}

#[tauri::command]
pub async fn lock_all_achievements(app_id: u32) -> Result<String, String> {
    run_steam_utility_command(&["lock_all_achievements", &app_id.to_string()])
        .await
}

#[tauri::command]
pub async fn update_stats(app_id: u32, stats_arr: &str) -> Result<String, String> {
    run_steam_utility_command(&["update_stats", &app_id.to_string(), stats_arr])
        .await
}

#[tauri::command]
pub async fn reset_all_stats(app_id: u32) -> Result<String, String> {
    run_steam_utility_command(&["reset_all_stats", &app_id.to_string()])
        .await
}
