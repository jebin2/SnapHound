use std::env;
use std::fs;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use serde_json::Value;

use crate::utils::{send_to_frontend, execute_command};

const APP_TEMP_DIR: &str = "snaphound";
const VENV_DIR: &str = "venv";
const CONFIG_FILE: &str = "config.json";
const THUMBNAIL_DIR: &str = "thumbnail";
const SEARCH_PY: &str = "search.py";

pub struct EnvPaths {
    pub python_binary: PathBuf,
    config_path: PathBuf,
    pub temp_dir: PathBuf,
    pub thumbnail_path: PathBuf,
    pub search_path: PathBuf,
}

impl EnvPaths {
    pub fn new() -> Self {
        let temp_dir = env::temp_dir().join(APP_TEMP_DIR);
        fs::create_dir_all(&temp_dir).expect("Failed to create application directory");

        let python_binary = if cfg!(target_os = "windows") {
            temp_dir.join(VENV_DIR).join("Scripts").join("python.exe")
        } else {
            temp_dir.join(VENV_DIR).join("bin").join("python")
        };

        let config_path = temp_dir.join(CONFIG_FILE);
        let search_path = temp_dir.join(SEARCH_PY);
        let thumbnail_path = temp_dir.join(THUMBNAIL_DIR);
        fs::create_dir_all(&thumbnail_path).expect("Failed to create thumbnail directory");

        Self {
            python_binary,
            config_path,
            temp_dir,
            thumbnail_path,
            search_path
        }
    }
}

fn get_resource_path(app: &AppHandle, resource_type: &str) -> PathBuf {
    let resource_path = match resource_type {
        "venv" => "bin/dependency/venv",
        "config" => "bin/dependency/config.json",
        "search" => "bin/dependency/search.py",
        _ => {
            send_to_frontend(app, format!("Unsupported resource type: {}", resource_type), "error");
            return PathBuf::new();
        }
    };

    match app.path().resolve(resource_path, tauri::path::BaseDirectory::Resource) {
        Ok(path) => {
            if !path.exists() {
                send_to_frontend(app, format!("Resource not found at {:?}", path), "error");
                PathBuf::new()
            } else {
                path
            }
        }
        Err(e) => {
            send_to_frontend(app, format!("Failed to resolve path: {}", e), "error");
            PathBuf::new()
        }
    }
}

async fn setup_virtual_environment(app: &AppHandle, paths: &EnvPaths) -> Result<(), String> {
    if paths.python_binary.exists() {
        send_to_frontend(app, format!("Virtual environment already exists at {:?}", paths.python_binary), "info");
        return Ok(());
    }

    let venv_source = get_resource_path(app, "venv");
    if venv_source.as_os_str().is_empty() {
        return Err("Failed to locate virtual environment resource".to_string());
    }

    copy_resource(&app, &venv_source, &paths.temp_dir).await
}

async fn install_dependencies(app: &AppHandle, paths: &EnvPaths) -> Result<(), String> {
    if !paths.python_binary.exists() {
        send_to_frontend(app, format!("Python binary not found at {:?}", paths.python_binary), "error");
        return Err("Python binary not found".to_string());
    }

    let mut command = Command::new(&paths.python_binary);
    command.args(&["-m", "pip", "install", "--force-reinstall", "git+https://github.com/jebin2/SnapHoundPy.git"]);

    // Execute the command and wait for it to complete
    match execute_command(app, &mut command, "install_dependencies".to_string()) {
        Ok(mut child) => {
            // Properly wait for the child process to complete
            match child.wait() {
                Ok(exit_status) if exit_status.success() => {
                    send_to_frontend(app, "Dependencies installed successfully".to_string(), "success");
                    Ok(())
                }
                Ok(exit_status) => {
                    let error_msg = format!("Installation failed with status: {}", exit_status);
                    send_to_frontend(app, error_msg.clone(), "error");
                    Err(error_msg)
                }
                Err(e) => {
                    let error_msg = format!("Failed to wait for installation process: {}", e);
                    send_to_frontend(app, error_msg.clone(), "error");
                    Err(error_msg)
                }
            }
        }
        Err(e) => {
            let error_msg = format!("Failed to start installation process: {}", e);
            send_to_frontend(app, error_msg.clone(), "error");
            Err(error_msg)
        }
    }
}


async fn copy_resource(app: &AppHandle, source: &PathBuf, destination: &PathBuf) -> Result<(), String> {
    let mut command = Command::new("cp");
    command.args(&["-r", source.to_str().unwrap(), destination.to_str().unwrap()]);

    match execute_command(app, &mut command, "copy_resource".to_string()) {
        Ok(mut child) => {
            match child.wait() {
                Ok(exit_status) if exit_status.success() => {
                    send_to_frontend(app, "Copy completed successfully.".to_string(), "success");
                    Ok(())
                }
                Ok(exit_status) => {
                    let error_msg = format!("Copy failed with status: {}", exit_status);
                    send_to_frontend(app, error_msg.clone(), "error");
                    Err(error_msg)
                }
                Err(e) => {
                    let error_msg = format!("Failed to wait for copy process: {}", e);
                    send_to_frontend(app, error_msg.clone(), "error");
                    Err(error_msg)
                }
            }
        }
        Err(e) => {
            let error_msg = format!("Failed to start copy process: {}", e);
            send_to_frontend(app, error_msg.clone(), "error");
            Err(error_msg)
        }
    }
}

async fn setup_config(app: &AppHandle, paths: &EnvPaths) -> Result<(), String> {
    let config_source = get_resource_path(app, "config");
    if config_source.as_os_str().is_empty() {
        return Err("Failed to locate config resource".to_string());
    }
    
    let search_path = get_resource_path(app, "search");
    if search_path.as_os_str().is_empty() {
        return Err("Failed to locate search script resource".to_string());
    }
    
    // Copy the config file
    if let Err(e) = copy_resource(app, &config_source, &paths.config_path).await {
        return Err(format!("Failed to copy config file: {}", e));
    }
    
    // Copy the search script
    copy_resource(app, &search_path, &paths.search_path).await
}

#[tauri::command]
pub async fn initialize_environment(app: AppHandle) {
    let paths = EnvPaths::new();

    if let Err(e) = setup_virtual_environment(&app, &paths).await {
        send_to_frontend(&app, format!("Failed to setup virtual environment: {}", e), "error");
        return;
    }
    
    if let Err(e) = install_dependencies(&app, &paths).await {
        send_to_frontend(&app, format!("Failed to install dependencies: {}", e), "error");
        return;
    }
    
    if let Err(e) = setup_config(&app, &paths).await {
        send_to_frontend(&app, format!("Failed to setup config: {}", e), "error");
        return;
    }

    // Only send this if all previous steps succeeded
    send_to_frontend(&app, "can_fetch_list".to_string(), "can_fetch_list");
}

#[tauri::command]
pub async fn fetch_config() -> Result<Value, String> {
    let paths = EnvPaths::new();
    let file_content = fs::read_to_string(&paths.config_path)
        .map_err(|e| e.to_string())?;
    serde_json::from_str::<Value>(&file_content)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_config(priorityPath: Vec<String>, app: AppHandle) -> Result<(), String> {
    let paths = EnvPaths::new();

    // Convert the Vec<String> into JSON
    let json_data = serde_json::json!({ "priority_path": priorityPath });

    // Convert JSON to a string
    let json_string = serde_json::to_string_pretty(&json_data).map_err(|e| e.to_string())?;

    // Write to config file
    fs::write(&paths.config_path, json_string).map_err(|e| e.to_string())?;

    Ok(())
}