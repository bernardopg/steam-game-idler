use crate::idling::SPAWNED_PROCESSES;
use serde_json::{json, Value};
use std::time::Duration;
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

#[tauri::command]
pub async fn get_running_processes() -> Result<Value, String> {
    cleanup_dead_processes().map_err(|e| e.to_string())?;

    #[cfg(windows)]
    {
        use sysinfo::{ProcessesToUpdate, System};

        let mut system = System::new_all();
        system.refresh_processes(ProcessesToUpdate::All, true);

        let mut processes = Vec::new();

        for (_pid, process) in system.processes() {
            let proc_name = process.name().to_ascii_lowercase();
            if proc_name == "steamutility" || proc_name == "steamutility.exe" {
                let pid = process.pid().as_u32();
                let window_title = get_any_window_title_for_pid(pid).unwrap_or_default();

                let (game_name, app_id) = if let Some(start) = window_title.find('[') {
                    if let Some(end) = window_title[start..].find(']') {
                        let app_id_str = &window_title[start + 1..start + end];
                        let name = window_title[..start].trim().trim_end_matches(" -");
                        (name.to_string(), app_id_str.parse::<u32>().unwrap_or(0))
                    } else {
                        ("".to_string(), 0)
                    }
                } else {
                    ("".to_string(), 0)
                };

                if app_id > 0 {
                    processes.push(json!({
                        "appid": app_id,
                        "pid": pid,
                        "name": game_name,
                    }));
                }
            }
        }

        return Ok(json!({"processes": processes}));
    }

    #[cfg(not(windows))]
    {
        let processes = SPAWNED_PROCESSES
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
        return Ok(json!({"success": "Successfully killed process with PID"}));
    }

    Ok(json!({"error": "Failed to kill process with PID"}))
}

#[tauri::command]
pub async fn kill_all_steamutil_processes() -> Result<Value, String> {
    cleanup_dead_processes().map_err(|e| e.to_string())?;

    let mut processes = SPAWNED_PROCESSES.lock().map_err(|e| e.to_string())?;
    if processes.is_empty() {
        return Ok(json!({"error": "No SteamUtility processes found"}));
    }

    let mut killed_count = 0;
    for process in processes.iter_mut() {
        if process.child.kill().is_ok() {
            killed_count += 1;
        }
    }
    processes.clear();

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
                processes.remove(i);
            } else {
                i += 1;
            }
        } else {
            processes.remove(i);
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
