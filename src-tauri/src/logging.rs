use crate::utils::{get_cache_dir, get_user_data_dir};
use chrono::Local;
use std::fs::{copy, create_dir_all, OpenOptions};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

const MAX_LINES: usize = 500;

fn get_log_file_path(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = get_user_data_dir(app_handle)?;
    create_dir_all(&app_data_dir).map_err(|e| format!("Failed to create app directory: {}", e))?;

    let log_file_path = app_data_dir.join("log.txt");
    if !log_file_path.exists() {
        let legacy_log_file_path = get_cache_dir(app_handle)?.join("log.txt");

        if legacy_log_file_path.exists() && legacy_log_file_path != log_file_path {
            copy(&legacy_log_file_path, &log_file_path)
                .map_err(|e| format!("Failed to migrate log file: {}", e))?;
        }
    }

    Ok(log_file_path)
}

#[tauri::command]
pub fn log_event(message: String, app_handle: tauri::AppHandle) -> Result<(), String> {
    let log_file_path = get_log_file_path(&app_handle)?;
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&log_file_path)
        .map_err(|e| format!("Failed to open log file: {}", e))?;
    // Read existing log lines
    let reader = BufReader::new(&file);
    let mut lines: Vec<String> = reader
        .lines()
        .map(|line| line.unwrap_or_default())
        .collect();
    // Create a new log entry with a timestamp
    let timestamp = Local::now().format("%b %d %H:%M:%S%.3f").to_string();
    let mask_one = mask_sensitive_data(&message, "711B8063");
    let mask_two = mask_sensitive_data(&mask_one, "3DnyBUX");
    let mask_three = mask_sensitive_data(&mask_two, "5e2699aef2301b283");
    let new_log = format!("{} + {}", timestamp, mask_three);
    // Insert the new log entry at the beginning
    lines.insert(0, new_log);
    // Truncate the log if it exceeds the maximum number of lines
    if lines.len() > MAX_LINES {
        lines.truncate(MAX_LINES);
    }
    // Write the updated log back to the file
    file.seek(SeekFrom::Start(0))
        .map_err(|e| format!("Failed to seek to start of file: {}", e))?;
    file.set_len(0)
        .map_err(|e| format!("Failed to truncate file: {}", e))?;
    for line in lines {
        writeln!(file, "{}", line).map_err(|e| format!("Failed to write to log file: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub fn clear_log_file(app_handle: tauri::AppHandle) -> Result<(), String> {
    let log_file_path = get_log_file_path(&app_handle)?;
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&log_file_path)
        .map_err(|e| format!("Failed to open log file: {}", e))?;

    file.set_len(0)
        .map_err(|e| format!("Failed to truncate file: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn read_log_file(app_handle: tauri::AppHandle) -> Result<String, String> {
    let log_file_path = get_log_file_path(&app_handle)?;
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&log_file_path)
        .map_err(|e| format!("Failed to open log file: {}", e))?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| format!("Failed to read log file: {}", e))?;

    Ok(contents)
}

pub fn mask_sensitive_data(message: &str, sensitive_data: &str) -> String {
    // Mask sensitive data in log messages
    if let Some(start_index) = message.find(sensitive_data) {
        let end_index = start_index + sensitive_data.len();
        let mask_start = start_index.saturating_sub(5);
        let mask_end = (end_index + 5).min(message.len());
        let mask_length = mask_end - mask_start;

        let mut masked_message = message.to_string();
        masked_message.replace_range(mask_start..mask_end, &"*".repeat(mask_length));
        masked_message
    } else {
        message.to_string()
    }
}
