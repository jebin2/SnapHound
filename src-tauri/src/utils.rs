use rfd::FileDialog;
use serde_json::json;
use std::env::temp_dir;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{exit, Child, Command, Stdio};
use tauri::{AppHandle, Emitter, Manager};
use uuid::Uuid;
use rayon::prelude::*;
use walkdir::WalkDir;
use std::io::{BufRead, BufReader};

use crate::initialise::EnvPaths;
use crate::initialise::fetch_config;
use crate::image_processor::process_thumbnail;

pub fn send_to_frontend(app_handle: &AppHandle, message: String, event_type: &str) {
    println!("{}", message);
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

pub fn expand_paths(paths: Vec<String>) -> Vec<PathBuf> {
    let mut directories = Vec::new();

    for mut path_str in paths {
        let recursive = path_str.ends_with("/*");

        // Handle special cases for home directory
        let expanded_path = if path_str == "~/*" {
            // Just return home dir, we'll process the wildcard below
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"))
        } else if path_str.starts_with("~/") {
            // Trim pattern suffixes if present
            if recursive {
                path_str.truncate(path_str.len() - 2);
            }
            
            // Expand home directory
            dirs::home_dir()
                .map(|home| home.join(&path_str[2..]))
                .unwrap_or_else(|| PathBuf::from(&path_str))
        } else {
            // Trim pattern suffixes for non-home paths
            if recursive {
                path_str.truncate(path_str.len() - 2);
            }
            
            PathBuf::from(&path_str)
        };
        
        if expanded_path.is_dir() {
            directories.push(expanded_path.clone());

            // Special handling for ~/* pattern
            if path_str == "~/*" {
                let subdirs = list_subdirectories(&expanded_path);
                directories.extend(subdirs);
            } 
            // Normal recursive handling
            else if recursive {
                let subdirs = list_subdirectories(&expanded_path);
                directories.extend(subdirs);
            }
        }
    }

    directories
}

fn list_subdirectories(base_path: &Path) -> Vec<PathBuf> {
    let mut subdirs = Vec::new();

    if let Ok(entries) = fs::read_dir(base_path) {
        for entry in entries.flatten() {
            let sub_path = entry.path();
            if sub_path.is_dir() {
                subdirs.push(sub_path.clone());
            }
        }
    }

    subdirs
}

pub fn get_file_type(path: &Path) -> String {
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

pub fn execute_command(app: &AppHandle, command: &mut Command, cmd_type: String) -> std::io::Result<Child> {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    // Read and send stdout messages in real-time
    for line in stdout_reader.lines() {
        if let Ok(output) = line {
            send_to_frontend(app, format!("{}", output), "status_update");
        }
    }

    // Read and send stderr messages in real-time
    for line in stderr_reader.lines() {
        if let Ok(error) = line {
            send_to_frontend(app, format!("Error: {}", error), "status_update");
        }
    }

    Ok(child)
}

#[tauri::command]
pub async fn reset_all(app: AppHandle) -> Result<bool, String> {
    let paths = EnvPaths::new();
    let mut command = Command::new("rm");
    command.args(&["-rf", paths.temp_dir.to_str().unwrap()]);

    match execute_command(&app, &mut command, "copy_resource".to_string()) {
        Ok(mut child) => match child.wait() {
            Ok(exit_status) if exit_status.success() => {
                send_to_frontend(&app, "Removed All Cached Data".to_string(), "success_reset");
                Ok(true)  // Return `true` on success
            }
            Ok(exit_status) => {
                let error_msg = format!("Failed to remove: {}", exit_status);
                send_to_frontend(&app, error_msg.clone(), "error");
                Err(error_msg) // Return error as `Err`
            }
            Err(e) => {
                let error_msg = format!("Failed to remove: {}", e);
                send_to_frontend(&app, error_msg.clone(), "error");
                Err(error_msg)
            }
        },
        Err(e) => {
            let error_msg = format!("Failed to start copy process: {}", e);
            send_to_frontend(&app, error_msg.clone(), "error");
            Err(error_msg)
        }
    }
}

#[tauri::command]
pub fn relaunch(app: tauri::AppHandle) {
    app.restart();
}