use rfd::FileDialog;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tauri::{AppHandle, Emitter, Manager};
use uuid::Uuid;
use walkdir::WalkDir;

use crate::initialise::fetch_config;

pub fn send_to_frontend(app_handle: &AppHandle, message: String, event_type: &str) {
    app_handle.emit(event_type, message).unwrap();
}

#[tauri::command]
pub async fn select_folder() -> String {
    let result = FileDialog::new().set_directory(".").pick_folder();

    match result {
        Some(path) => path.display().to_string(),
        None => String::new(),
    }
}

#[tauri::command]
pub async fn list_files(app: AppHandle) {
    println!("Fetching config...");
    let config = fetch_config().await.unwrap();
    println!("Config fetched: {:?}", config);

    if let Some(paths) = config["priority_path"].as_array() {
        for path_value in paths {
            if let Some(mut path_str) = path_value.as_str() {
                // Handle recursive flag and strip /*
                let recursive = path_str.ends_with("/*");
                if recursive {
                    path_str = &path_str[..path_str.len() - 2];
                }

                let expanded_path = if path_str.starts_with("~/") {
                    dirs::home_dir()
                        .map(|home| home.join(&path_str[2..]))
                        .unwrap_or_else(|| PathBuf::from(path_str))
                } else {
                    PathBuf::from(path_str)
                };

                println!("Processing path: {:?}", expanded_path);

                if expanded_path.is_dir() {
                    let walker = WalkDir::new(&expanded_path)
                        .max_depth(if recursive { usize::MAX } else { 1 })
                        .into_iter()
                        .filter_map(|e| e.ok());

                    for entry in walker {
                        let file_path = entry.path();
                        if file_path.is_file() {
                            let file_type = get_file_type(&file_path);
                            if file_type != "unknown" {
                                let file_info = json!({
                                    "id": Uuid::new_v4().to_string(), // Generate a unique ID
                                    "path": file_path,
                                    "type": file_type
                                });
                                // Send the JSON object directly instead of string
                                send_to_frontend(&app, file_info.to_string(), "file_path");
                            }
                        }
                    }
                }
            }
        }
    }
    send_to_frontend(&app, "file_path_end".to_string(), "file_path_end");
}

fn wsl_to_windows_path(path: String) -> String {
    let output = Command::new("wslpath")
        .arg("-w")
        .arg(path)
        .output()
        .expect("Failed to execute wslpath");

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn get_file_type(path: &Path) -> String {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("mp4") | Some("mkv") | Some("avi") => "video".to_string(),
        Some("jpg") | Some("jpeg") | Some("png") | Some("gif") => "image".to_string(),
        _ => "unknown".to_string(),
    }
}

#[tauri::command]
pub async fn read_image(path: String) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|e| e.to_string())
}
