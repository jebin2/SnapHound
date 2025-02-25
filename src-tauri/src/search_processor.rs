use crate::initialise::EnvPaths;
use crate::utils::send_to_frontend;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::io::{BufRead, BufReader};
use tauri::AppHandle;
use crate::initialise::fetch_config;

// Global process tracker
lazy_static::lazy_static! {
    static ref SEARCH_PROCESS: Arc<Mutex<Option<std::process::Child>>> = Arc::new(Mutex::new(None));
}

// Helper to handle process output streams
fn handle_process_output(reader: impl BufRead, app: &AppHandle, event_type: &str, prefix: &str) {
    for line in reader.lines().flatten() {
        println!("{}: {}", prefix, line);
        send_to_frontend(app, format!("{}: {}", prefix, line), event_type);
    }
}

#[tauri::command]
pub async fn search_data(search_data: String, app: AppHandle) {
    // Cancel any existing search before starting a new one
    search_cancel(app.clone()).await;
    send_to_frontend(&app, format!("Searching for {}", search_data), "status_update");
    let paths = EnvPaths::new();
    let config = fetch_config().await.unwrap();
    
    match Command::new(&paths.python_binary)
        .arg("-u") // Forces unbuffered output
        .arg(paths.search_path) // Your Python script
        .arg(search_data.to_string()) // JSON string
        .arg(config["priority_path"].to_string()) // JSON string
        // .arg(paths.to_string()) // JSON string
        .env("PYTHONUNBUFFERED", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(mut child) => {
            // Capture stdout and stderr before storing the process
            let stdout = child.stdout.take().expect("Failed to capture stdout");
            let stderr = child.stderr.take().expect("Failed to capture stderr");
            
            // Store the process
            {
                let mut process_guard = SEARCH_PROCESS.lock().unwrap();
                *process_guard = Some(child);
            }
            
            // Handle stdout
            let app_clone = app.clone();
            tauri::async_runtime::spawn(async move {
                let reader = BufReader::new(stdout);
                handle_process_output(reader, &app_clone, "status_update", "Progress");
            });
            
            // Handle stderr
            let app_clone = app.clone();
            let app_for_completion = app.clone();
            tauri::async_runtime::spawn(async move {
                let reader = BufReader::new(stderr);
                handle_process_output(reader, &app_clone, "error", "Error");
                
                // Wait for process completion in this same task
                let mut process_guard = SEARCH_PROCESS.lock().unwrap();
                if let Some(mut child_process) = process_guard.take() {
                    match child_process.wait() {
                        Ok(exit_status) if exit_status.success() => {
                            send_to_frontend(&app_for_completion, "Search completed successfully.".to_string(), "success");
                        }
                        Ok(exit_status) => {
                            send_to_frontend(&app_for_completion, format!("Search failed with status: {}", exit_status), "error");
                        }
                        Err(e) => {
                            send_to_frontend(&app_for_completion, format!("Failed to wait for search process: {}", e), "error");
                        }
                    }
                }
            });
        },
        Err(e) => {
            send_to_frontend(&app, format!("Failed to start search process: {}", e), "error");
        }
    }
}

#[tauri::command]
pub async fn search_cancel(app: AppHandle) -> Result<(), String> {
    let mut process_guard = SEARCH_PROCESS.lock().map_err(|e| e.to_string())?;
    
    if let Some(mut child) = process_guard.take() {
        match child.kill() {
            Ok(_) => {
                send_to_frontend(&app, "Search process canceled.".to_string(), "status_update");
                Ok(())
            },
            Err(e) => {
                let error_msg = format!("Failed to cancel process: {}", e);
                send_to_frontend(&app, error_msg.clone(), "error");
                Err(error_msg)
            }
        }
    } else {
        Ok(()) // No process to cancel
    }
}