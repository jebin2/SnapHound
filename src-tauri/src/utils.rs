use rfd::FileDialog;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tauri::{AppHandle, Emitter, Manager};
use uuid::Uuid;
use rayon::prelude::*;
use walkdir::WalkDir;

use crate::initialise::fetch_config;
use crate::image_processor::process_thumbnail;

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
    
    let mut directories = Vec::new();

    if let Some(paths) = config["priority_path"].as_array() {
        for path_value in paths {
            if let Some(mut path_str) = path_value.as_str() {
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

                directories.push(expanded_path);
            }
        }
    }

    // Sort directories before processing
    directories.sort();

    for expanded_path in directories {
        println!("Processing directory: {:?}", expanded_path);

        if expanded_path.is_dir() {
            let mut file_paths: Vec<_> = WalkDir::new(&expanded_path)
                .max_depth(if config["recursive"].as_bool().unwrap_or(false) { usize::MAX } else { 1 })
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|entry| entry.path().is_file()) // Keep only files
                .map(|entry| entry.path().to_path_buf()) // Extract file paths
                .collect();

            // Sort file paths before processing
            file_paths.sort();

            // Process files in parallel and send in chunks of 100
            file_paths
                .par_chunks(2) // Process in parallel in chunks of 100
                .for_each(|chunk| {
                    let files: Vec<_> = chunk
                        .par_iter()
                        .filter_map(|file_path| {
                            let file_type = get_file_type(&file_path);
                            if file_type != "unknown" {
                                Some(json!({
                                    "id": Uuid::new_v4().to_string(),
                                    "path": process_thumbnail(file_path.to_str().unwrap()),
                                    "type": file_type
                                }))
                            } else {
                                None
                            }
                        })
                        .collect();

                    if !files.is_empty() {
                        let json_data = serde_json::to_string(&files).unwrap();
                        send_to_frontend(&app, json_data, "file_path");
                    }
                });
        }
    }
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
