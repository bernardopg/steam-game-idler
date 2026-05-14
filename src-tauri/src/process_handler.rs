use crate::idling::SPAWNED_PROCESSES;
use serde_json::{json, Value};
use std::ffi::OsString;
use std::fs;
use std::time::Duration;
use sysinfo::{Pid, Process, ProcessesToUpdate, System};
use tauri::Emitter;

#[cfg(windows)]
use windows::Win32::{
    Foundation::{HWND, LPARAM},
    UI::WindowsAndMessaging::{EnumWindows, GetWindowTextW, GetWindowThreadProcessId},
};

#[cfg(windows)]
fn get_any_window_title_for_pid(pid: u32) -> Option<String> {
    use windows::Win32::Foundation::BOOL;

    struct EnumData {
        target_pid: u32,
        title: Option<String>,
    }

    unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let data = &mut *(lparam.0 as *mut EnumData);
        let mut pid_buf: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid_buf));
        if pid_buf == data.target_pid {
            let mut buf = [0u16; 512];
            let len = GetWindowTextW(hwnd, &mut buf);
            if len > 0 {
                let title = String::from_utf16_lossy(&buf[..len as usize]);
                data.title = Some(title);
                return BOOL(0);
            }
        }
        BOOL(1)
    }

    let mut data = EnumData {
        target_pid: pid,
        title: None,
    };
    let lparam = LPARAM(&mut data as *mut _ as isize);
    unsafe {
        let _ = EnumWindows(Some(enum_windows_proc), lparam);
    }
    data.title
}

fn process_text(process: &Process) -> String {
    let name = process.name().to_string_lossy();
    let exe = process
        .exe()
        .map(|path| path.to_string_lossy())
        .unwrap_or_default();
    let cmd = process
        .cmd()
        .iter()
        .map(|arg| arg.to_string_lossy())
        .collect::<Vec<_>>()
        .join(" ");

    format!("{} {} {}", name, exe, cmd).to_ascii_lowercase()
}

fn is_steam_utility_process(process: &Process) -> bool {
    let text = process_text(process);
    text.contains("steamutility") || text.contains("steamutility.cli")
}

fn parse_idle_process_args(args: &[OsString]) -> Option<(u32, String)> {
    let idle_index = args
        .iter()
        .position(|arg| arg.to_string_lossy().eq_ignore_ascii_case("idle"))?;
    let app_id = args.get(idle_index + 1)?.to_string_lossy().parse().ok()?;
    let app_name = args
        .iter()
        .skip(idle_index + 2)
        .map(|arg| arg.to_string_lossy())
        .collect::<Vec<_>>()
        .join(" ");

    Some((app_id, app_name))
}

#[cfg(windows)]
fn parse_process_window_title(pid: u32) -> Option<(u32, String)> {
    let window_title = get_any_window_title_for_pid(pid)?;
    let start = window_title.find('[')?;
    let end = window_title[start..].find(']')?;
    let app_id_str = &window_title[start + 1..start + end];
    let app_id = app_id_str.parse::<u32>().ok()?;
    let name = window_title[..start]
        .trim()
        .trim_end_matches(" -")
        .to_string();

    Some((app_id, name))
}

fn kill_external_process(pid: u32) -> bool {
    let mut system = System::new_all();
    system.refresh_processes(ProcessesToUpdate::All, true);

    system
        .process(Pid::from_u32(pid))
        .filter(|process| is_steam_utility_process(process))
        .map(|process| process.kill())
        .unwrap_or(false)
}

fn external_idle_process_entries() -> Vec<(u32, u32, String)> {
    let mut system = System::new_all();
    system.refresh_processes(ProcessesToUpdate::All, true);

    system
        .processes()
        .values()
        .filter(|process| is_steam_utility_process(process))
        .filter_map(|process| {
            let pid = process.pid().as_u32();

            #[cfg(windows)]
            let parsed =
                parse_process_window_title(pid).or_else(|| parse_idle_process_args(process.cmd()));

            #[cfg(not(windows))]
            let parsed = parse_idle_process_args(process.cmd());

            parsed.map(|(app_id, game_name)| (pid, app_id, game_name))
        })
        .collect()
}

#[tauri::command]
pub async fn get_running_processes() -> Result<Value, String> {
    cleanup_dead_processes().map_err(|e| e.to_string())?;

    {
        let mut processes = SPAWNED_PROCESSES
            .lock()
            .map_err(|e| e.to_string())?
            .iter()
            .map(|process| {
                json!({
                    "appid": process.app_id,
                    "pid": process.pid,
                    "name": process.app_name,
                })
            })
            .collect::<Vec<Value>>();

        for (pid, app_id, game_name) in external_idle_process_entries() {
            if processes
                .iter()
                .any(|known| known["pid"].as_u64() == Some(pid as u64))
            {
                continue;
            }

            processes.push(json!({
                "appid": app_id,
                "pid": pid,
                "name": game_name,
            }));
        }

        Ok(json!({"processes": processes}))
    }
}

#[tauri::command]
pub async fn kill_process_by_pid(pid: u32) -> Result<Value, String> {
    cleanup_dead_processes().map_err(|e| e.to_string())?;

    let mut processes = SPAWNED_PROCESSES.lock().map_err(|e| e.to_string())?;
    if let Some(position) = processes.iter().position(|process| process.pid == pid) {
        let mut process = processes.remove(position);
        process.child.kill().map_err(|e| e.to_string())?;
        let _ = process.child.wait();
        let _ = fs::remove_dir_all(&process.work_dir);
        return Ok(json!({"success": "Successfully killed process with PID"}));
    }

    if external_idle_process_entries()
        .iter()
        .any(|(external_pid, _, _)| *external_pid == pid)
        && kill_external_process(pid)
    {
        return Ok(json!({"success": "Successfully killed process with PID"}));
    }

    Ok(json!({"error": "Failed to kill process with PID"}))
}

#[tauri::command]
pub async fn kill_all_steamutil_processes() -> Result<Value, String> {
    cleanup_dead_processes().map_err(|e| e.to_string())?;

    let mut processes = SPAWNED_PROCESSES.lock().map_err(|e| e.to_string())?;
    if processes.is_empty() {
        drop(processes);
        let killed_count = external_idle_process_entries()
            .into_iter()
            .map(|(pid, _, _)| pid)
            .filter(|pid| kill_external_process(*pid))
            .count();

        if killed_count > 0 {
            return Ok(json!({
                "success": "Successfully killed all SteamUtility processes",
                "killed_count": killed_count
            }));
        }

        return Ok(json!({"error": "No SteamUtility processes found"}));
    }

    let mut killed_count = 0;
    for process in processes.iter_mut() {
        if process.child.kill().is_ok() {
            let _ = process.child.wait();
            let _ = fs::remove_dir_all(&process.work_dir);
            killed_count += 1;
        }
    }
    processes.clear();
    killed_count += external_idle_process_entries()
        .into_iter()
        .map(|(pid, _, _)| pid)
        .filter(|pid| kill_external_process(*pid))
        .count();

    Ok(json!({
        "success": "Successfully killed all SteamUtility processes",
        "killed_count": killed_count
    }))
}

pub fn cleanup_dead_processes() -> Result<(), String> {
    let mut processes = SPAWNED_PROCESSES.lock().map_err(|e| e.to_string())?;
    let mut i = 0;
    while i < processes.len() {
        if let Ok(status) = processes[i].child.try_wait() {
            if status.is_some() {
                let process = processes.remove(i);
                let _ = fs::remove_dir_all(&process.work_dir);
            } else {
                i += 1;
            }
        } else {
            let process = processes.remove(i);
            let _ = fs::remove_dir_all(&process.work_dir);
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn start_processes_monitor(app_handle: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut last_processes: Option<String> = None;
        loop {
            match get_running_processes().await {
                Ok(processes_value) => {
                    let current_json = processes_value.to_string();
                    if last_processes.as_ref() != Some(&current_json) {
                        last_processes = Some(current_json.clone());
                        let _ = app_handle.emit("running_processes_changed", processes_value);
                    }
                }
                Err(e) => {
                    eprintln!("Error getting running processes: {}", e);
                }
            }
            tokio::time::sleep(Duration::from_millis(1000)).await;
        }
    });
}
