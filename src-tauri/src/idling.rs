use crate::command_runner::apply_hidden_command_style;
use crate::process_handler::{cleanup_dead_processes, kill_all_steamutil_processes};
use crate::utils::get_lib_path;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Child;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct ProcessInfo {
    pub child: Child,
    pub app_id: u32,
    pub app_name: String,
    pub pid: u32,
    pub work_dir: PathBuf,
}

lazy_static::lazy_static! {
    pub static ref SPAWNED_PROCESSES: Arc<Mutex<Vec<ProcessInfo>>> = Arc::new(Mutex::new(Vec::new()));
}

#[cfg(windows)]
const MAX_FARM_IDLE_PROCESSES: usize = 32;
#[cfg(not(windows))]
const MAX_FARM_IDLE_PROCESSES: usize = 8;

#[cfg(windows)]
const FARM_IDLE_START_DELAY_MS: u64 = 100;
#[cfg(not(windows))]
const FARM_IDLE_START_DELAY_MS: u64 = 500;

fn idle_work_root() -> PathBuf {
    std::env::temp_dir().join("steam-game-idler").join("idlers")
}

pub fn cleanup_idle_work_root() {
    let _ = fs::remove_dir_all(idle_work_root());
}

fn create_idle_work_dir(app_id: u32) -> Result<PathBuf, String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let work_dir =
        idle_work_root().join(format!("{}-{}-{}", std::process::id(), app_id, timestamp));

    fs::create_dir_all(&work_dir).map_err(|e| e.to_string())?;
    fs::write(work_dir.join("steam_appid.txt"), app_id.to_string()).map_err(|e| e.to_string())?;

    Ok(work_dir)
}

fn remove_idle_work_dir(work_dir: &Path) {
    let _ = fs::remove_dir_all(work_dir);
}

fn build_idle_command(
    exe_path: &str,
    app_id: u32,
    app_name: &str,
) -> Result<(Command, PathBuf), String> {
    let work_dir = create_idle_work_dir(app_id)?;
    let mut command = Command::new(exe_path);
    command
        .args(["idle", &app_id.to_string(), app_name])
        .current_dir(&work_dir)
        .env("SteamAppId", app_id.to_string())
        .env("SteamGameId", app_id.to_string());

    Ok((command, work_dir))
}

#[tauri::command]
pub async fn start_idle(app_id: u32, app_name: String) -> Result<Value, String> {
    cleanup_dead_processes().map_err(|e| e.to_string())?;

    let exe_path = get_lib_path()?;
    let (mut command, work_dir) = build_idle_command(&exe_path, app_id, app_name.as_str())?;
    let child = match apply_hidden_command_style(&mut command).spawn() {
        Ok(child) => child,
        Err(e) => {
            remove_idle_work_dir(&work_dir);
            return Err(e.to_string());
        }
    };

    let pid = child.id();

    {
        let mut processes = SPAWNED_PROCESSES.lock().map_err(|e| e.to_string())?;
        processes.push(ProcessInfo {
            child,
            app_id,
            app_name,
            pid,
            work_dir,
        });
    }

    tokio::time::sleep(Duration::from_millis(1000)).await;

    let mut processes = SPAWNED_PROCESSES.lock().map_err(|e| e.to_string())?;
    if let Some(position) = processes.iter().position(|process| process.pid == pid) {
        let process = &mut processes[position];
        match process.child.try_wait() {
            Ok(Some(_)) => {
                let process = processes.remove(position);
                remove_idle_work_dir(&process.work_dir);
                Ok(json!({"error": "Failed to start idling game"}))
            }
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
    let _ = process.child.wait();
    remove_idle_work_dir(&process.work_dir);

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
    let games_to_start = games_list
        .iter()
        .take(MAX_FARM_IDLE_PROCESSES)
        .collect::<Vec<&GameInfo>>();
    let app_ids: Vec<u32> = games_to_start.iter().map(|game| game.app_id).collect();

    for game in games_to_start {
        let (mut command, work_dir) = build_idle_command(&exe_path, game.app_id, &game.name)?;
        let child = match apply_hidden_command_style(&mut command).spawn() {
            Ok(child) => child,
            Err(e) => {
                remove_idle_work_dir(&work_dir);
                return Err(e.to_string());
            }
        };

        let pid = child.id();

        {
            let mut processes = SPAWNED_PROCESSES.lock().map_err(|e| e.to_string())?;
            processes.push(ProcessInfo {
                child,
                app_id: game.app_id,
                app_name: game.name.clone(),
                pid,
                work_dir,
            });
        }

        tokio::time::sleep(Duration::from_millis(FARM_IDLE_START_DELAY_MS)).await;
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
        let _ = process.child.wait();
        remove_idle_work_dir(&process.work_dir);
    }

    processes.clear();

    Ok(json!({"success": "Successfully stopped idling games"}))
}
