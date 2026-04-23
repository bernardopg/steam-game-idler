use crate::command_runner::apply_hidden_command_style;
use crate::process_handler::{cleanup_dead_processes, kill_all_steamutil_processes};
use crate::utils::get_lib_path;
use serde_json::{json, Value};
use std::process::Child;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug)]
pub struct ProcessInfo {
    pub child: Child,
    pub app_id: u32,
    pub app_name: String,
    pub pid: u32,
}

lazy_static::lazy_static! {
    pub static ref SPAWNED_PROCESSES: Arc<Mutex<Vec<ProcessInfo>>> = Arc::new(Mutex::new(Vec::new()));
}

#[tauri::command]
pub async fn start_idle(app_id: u32, app_name: String) -> Result<Value, String> {
    cleanup_dead_processes().map_err(|e| e.to_string())?;

    let exe_path = get_lib_path()?;
    let mut command = Command::new(exe_path);
    command.args(["idle", &app_id.to_string(), app_name.as_str()]);
    let child = apply_hidden_command_style(&mut command)
        .spawn()
        .map_err(|e| e.to_string())?;

    let pid = child.id();

    {
        let mut processes = SPAWNED_PROCESSES.lock().map_err(|e| e.to_string())?;
        processes.push(ProcessInfo {
            child,
            app_id,
            app_name,
            pid,
        });
    }

    tokio::time::sleep(Duration::from_millis(1000)).await;

    let mut processes = SPAWNED_PROCESSES.lock().map_err(|e| e.to_string())?;
    if let Some(process) = processes.last_mut() {
        match process.child.try_wait() {
            Ok(Some(_)) => Ok(json!({"error": "Failed to start idling game"})),
            Ok(None) => Ok(json!({"success": "Successfully started idling game"})),
            Err(e) => Ok(json!({"error": e.to_string()})),
        }
    } else {
        Ok(json!({"error": "No processes found"}))
    }
}

#[tauri::command]
pub async fn stop_idle(app_id: u32) -> Result<Value, String> {
    let mut processes = SPAWNED_PROCESSES.lock().map_err(|e| e.to_string())?;
    let position = processes
        .iter()
        .position(|p| p.app_id == app_id)
        .ok_or_else(|| "No matching process found".to_string())?;

    let mut process = processes.remove(position);
    process.child.kill().map_err(|e| e.to_string())?;

    Ok(json!({"success": "Successfully stopped idling game"}))
}

#[derive(serde::Deserialize)]
pub struct GameInfo {
    app_id: u32,
    name: String,
}

#[tauri::command]
pub async fn start_farm_idle(games_list: Vec<GameInfo>) -> Result<Value, String> {
    let exe_path = get_lib_path()?;

    cleanup_dead_processes().map_err(|e| e.to_string())?;

    let mut failed = false;
    let app_ids: Vec<u32> = games_list.iter().map(|game| game.app_id).collect();

    for game in &games_list {
        let mut command = Command::new(&exe_path);
        command.args(["idle", &game.app_id.to_string(), &game.name]);
        let child = apply_hidden_command_style(&mut command)
            .spawn()
            .map_err(|e| e.to_string())?;

        let pid = child.id();

        {
            let mut processes = SPAWNED_PROCESSES.lock().map_err(|e| e.to_string())?;
            processes.push(ProcessInfo {
                child,
                app_id: game.app_id,
                app_name: game.name.clone(),
                pid,
            });
        }
    }

    tokio::time::sleep(Duration::from_millis(1000)).await;

    {
        let mut processes = SPAWNED_PROCESSES.lock().map_err(|e| e.to_string())?;
        for process in processes.iter_mut() {
            if app_ids.contains(&process.app_id) {
                match process.child.try_wait() {
                    Ok(Some(_)) => {
                        failed = true;
                        break;
                    }
                    Err(_) => {
                        failed = true;
                        break;
                    }
                    Ok(None) => {}
                }
            }
        }
    }

    if failed {
        let _ = kill_all_steamutil_processes().await;
        Ok(json!({"error": "Failed to start one or more idle processes"}))
    } else {
        Ok(json!({"success": "Successfully started idling games"}))
    }
}

#[tauri::command]
pub async fn stop_farm_idle() -> Result<Value, String> {
    let mut processes = SPAWNED_PROCESSES.lock().map_err(|e| e.to_string())?;

    for process in processes.iter_mut() {
        process.child.kill().map_err(|e| e.to_string())?;
    }

    processes.clear();

    Ok(json!({"success": "Successfully stopped idling games"}))
}
