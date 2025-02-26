use walkdir::WalkDir;
use crate::utils::{send_to_frontend, get_file_type, expand_paths};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::io::{BufRead, BufReader};
use std::sync::atomic::{AtomicUsize, Ordering};
use tauri::AppHandle;
use crate::initialise::fetch_config;
use crate::image_processor::process_thumbnail;
use serde_json::{json, Value};
use uuid::Uuid;
use rayon::prelude::*;

// List files operation state
// 0 = Not running (idle)
// 1 = Running
// 2 = Cancellation requested
lazy_static::lazy_static! {
    static ref LIST_FILES_STATE: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
}

// Constants for state values
const STATE_IDLE: usize = 0;
const STATE_RUNNING: usize = 1;
const STATE_CANCELLING: usize = 2;

#[tauri::command]
pub async fn list_files(app: AppHandle) {

    cancel_list_files(app.clone()).await.ok();
    
    // Set state to running
    LIST_FILES_STATE.store(STATE_RUNNING, Ordering::SeqCst);
    
    let config = fetch_config().await.unwrap();
    
    let priority_paths: Vec<String> = serde_json::from_value(config["priority_paths"].clone()).unwrap_or_default();
    
    if priority_paths.is_empty() {
        send_to_frontend(&app, "No paths configured for listing".to_string(), "status_update");
        LIST_FILES_STATE.store(STATE_IDLE, Ordering::SeqCst);
        return;
    }
    
    let mut directories = expand_paths(priority_paths);
    
    // Sort directories before processing
    directories.sort();
    
    for expanded_path in directories {
        // Check if cancellation was requested
        if LIST_FILES_STATE.load(Ordering::SeqCst) == STATE_CANCELLING {
            send_to_frontend(&app, "File listing operation cancelled".to_string(), "remove_all_data");
            LIST_FILES_STATE.store(STATE_IDLE, Ordering::SeqCst);
            return;
        }
        
        send_to_frontend(&app, format!("Fetching data from: {:?}", expanded_path), "status_update");
        
        if expanded_path.is_dir() {
            // Using collect to gather paths first
            let walkdir = WalkDir::new(&expanded_path)
                .max_depth(if config["recursive"].as_bool().unwrap_or(false) { usize::MAX } else { 1 });
                
            let mut file_paths: Vec<_> = walkdir
                .into_iter()
                .filter_map(|e| {
                    // Check for cancellation during directory traversal
                    if LIST_FILES_STATE.load(Ordering::SeqCst) == STATE_CANCELLING {
                        return None;
                    }
                    e.ok()
                })
                .filter(|entry| entry.path().is_file()) // Keep only files
                .map(|entry| entry.path().to_path_buf()) // Extract file paths
                .collect();
            
            // Check if cancellation was requested after collecting paths
            if LIST_FILES_STATE.load(Ordering::SeqCst) == STATE_CANCELLING {
                send_to_frontend(&app, "File listing operation cancelled".to_string(), "remove_all_data");
                LIST_FILES_STATE.store(STATE_IDLE, Ordering::SeqCst);
                return;
            }
            
            // Sort file paths before processing
            file_paths.sort();
            
            // Use larger chunks for better performance
            const CHUNK_SIZE: usize = 1;
            
            // Process files in parallel and send in chunks
            file_paths
                .par_chunks(CHUNK_SIZE)
                .for_each(|chunk| {
                    // Check for cancellation before processing each chunk
                    if LIST_FILES_STATE.load(Ordering::SeqCst) == STATE_CANCELLING {
                        return;
                    }
                    
                    let files: Vec<_> = chunk
                        .par_iter()
                        .filter_map(|file_path| {
                            // Check for cancellation during parallel processing
                            if LIST_FILES_STATE.load(Ordering::SeqCst) == STATE_CANCELLING {
                                return None;
                            }
                            
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
                    
                    if LIST_FILES_STATE.load(Ordering::SeqCst) != STATE_CANCELLING && !files.is_empty() {
                        let json_data = serde_json::to_string(&files).unwrap();
                        send_to_frontend(&app, json_data, "file_path");
                    }
                });
        }
        
        // Check if cancellation was requested after processing each directory
        if LIST_FILES_STATE.load(Ordering::SeqCst) == STATE_CANCELLING {
            send_to_frontend(&app, "File listing operation cancelled".to_string(), "remove_all_data");
            LIST_FILES_STATE.store(STATE_IDLE, Ordering::SeqCst);
            return;
        }
    }
    
    send_to_frontend(&app, "File listing completed".to_string(), "status_update");
    LIST_FILES_STATE.store(STATE_IDLE, Ordering::SeqCst);
}

#[tauri::command]
pub async fn cancel_list_files(app: AppHandle) -> Result<(), String> {
    let current_state = LIST_FILES_STATE.load(Ordering::SeqCst);
    
    if current_state == STATE_RUNNING {
        // Only send update message if an operation is actually running
        send_to_frontend(&app, "Cancelling file listing operation...".to_string(), "status_update");
        
        // Set the state to cancelling
        LIST_FILES_STATE.store(STATE_CANCELLING, Ordering::SeqCst);
        
        // Wait a brief moment to allow threads to notice cancellation
        let app_clone = app.clone();
        tauri::async_runtime::spawn(async move {
            // Wait for up to 1 second for the operation to get cleaned up
            for _ in 0..10 {
                if LIST_FILES_STATE.load(Ordering::SeqCst) == STATE_IDLE {
                    // Operation is no longer running
                    break;
                }
                // Sleep for 100ms
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
            
            // Force reset state if it hasn't been cleaned up
            if LIST_FILES_STATE.compare_exchange(STATE_CANCELLING, STATE_IDLE, 
                Ordering::SeqCst, Ordering::SeqCst).is_ok() {
                send_to_frontend(&app_clone, "File listing operation cancelled".to_string(), "remove_all_data");
            }
        });
    } else if current_state == STATE_CANCELLING {
        // Operation is already being cancelled, do nothing
    } else {
        // No operation running, just ensure state is reset
        LIST_FILES_STATE.store(STATE_IDLE, Ordering::SeqCst);
    }
    
    Ok(())
}