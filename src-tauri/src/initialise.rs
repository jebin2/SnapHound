use std::env;
use std::fs;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use serde_json::Value;

use crate::utils::send_to_frontend;

const APP_TEMP_DIR: &str = "snaphound";
const VENV_DIR: &str = "venv";
const CONFIG_FILE: &str = "config.json";

struct EnvPaths {
    python_binary: PathBuf,
    config_path: PathBuf,
    temp_dir: PathBuf,
}

impl EnvPaths {
    fn new() -> Self {
        let temp_dir = env::temp_dir().join(APP_TEMP_DIR);
        fs::create_dir_all(&temp_dir).expect("Failed to create application directory");

        let python_binary = if cfg!(target_os = "windows") {
            temp_dir.join(VENV_DIR).join("Scripts").join("python.exe")
        } else {
            temp_dir.join(VENV_DIR).join("bin").join("python")
        };

        let config_path = temp_dir.join(CONFIG_FILE);

        Self {
            python_binary,
            config_path,
            temp_dir,
        }
    }
}

fn get_resource_path(app: &AppHandle, resource_type: &str) -> PathBuf {
    let resource_path = match resource_type {
        "venv" => "bin/dependency/venv",
        "config" => "bin/dependency/config.json",
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

async fn setup_virtual_environment(app: &AppHandle, paths: &EnvPaths) {
    if paths.python_binary.exists() {
        send_to_frontend(app, format!("Virtual environment already exists at {:?}", paths.python_binary), "error");
        return;
    }

    let venv_source = get_resource_path(app, "venv");
    if venv_source.as_os_str().is_empty() {
        return;
    }

    copy_resource(&app, &venv_source, &paths.temp_dir);
}

fn install_dependencies(app: &AppHandle, paths: &EnvPaths) {
    if !paths.python_binary.exists() {
        send_to_frontend(app, format!("Python binary not found at {:?}", paths.python_binary), "error");
        return;
    }

    let mut child = Command::new(&paths.python_binary)
        .args(&["-m", "pip", "install", "git+https://github.com/jebin2/SnapHoundPy.git"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start process");

	// Handle stdout
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(line) = line {
                println!("{}", line);
            }
        }
    }

    // Handle stderr
    if let Some(stderr) = child.stderr.take() {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            if let Ok(line) = line {
                eprintln!("{}", line);
            }
        }
    }

    // Wait for the process to complete
    let _ = child.wait();

    send_to_frontend(app, "Success.".to_string(), "success")
}

fn copy_resource(app: &AppHandle, source: &PathBuf, destination: &PathBuf) {
    let result = Command::new("cp")
        .args(&["-r", 
            source.to_str().unwrap(), 
            destination.to_str().unwrap()
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output();

    if let Err(e) = result {
        send_to_frontend(app, format!("Failed to copy resource: {}", e), "error");
    }
}

async fn setup_config(app: &AppHandle, paths: &EnvPaths) {
    let config_source = get_resource_path(app, "config");
    if config_source.as_os_str().is_empty() {
        return;
    }
    
    copy_resource(app, &config_source, &paths.config_path);
}

#[tauri::command]
pub async fn initialize_environment(app: AppHandle) {
    let paths = EnvPaths::new();

    setup_virtual_environment(&app, &paths).await;
    install_dependencies(&app, &paths);
    setup_config(&app, &paths).await;

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