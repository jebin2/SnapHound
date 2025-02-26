use crate::image_processor::process_thumbnail;
use crate::initialise::fetch_config;
use crate::utils::{expand_paths, get_file_type, send_to_frontend};
use rayon::prelude::*;
use serde_json::{json, Value};
use std::collections::HashSet;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::AppHandle;
use uuid::Uuid;
use walkdir::WalkDir;
use dashmap::DashSet;

lazy_static::lazy_static! {
    static ref CANCEL_FLAG: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    static ref SENT_FILES: DashSet<String> = DashSet::new();
}

#[tauri::command]
pub async fn list_files(app: AppHandle) {
    CANCEL_FLAG.store(false, Ordering::SeqCst);
    let app_clone = app.clone();
    let config = crate::initialise::fetch_config().await.unwrap();
    thread::spawn(move || {
        let priority_paths: Vec<String> =
            serde_json::from_value(config["priority_paths"].clone()).unwrap_or_default();
        if priority_paths.is_empty() {
            send_to_frontend(
                &app_clone,
                "No paths configured for listing".to_string(),
                "status_update",
            );
            return;
        }

        let mut directories = expand_paths(priority_paths);
        directories.sort();

        for expanded_path in directories {
            if CANCEL_FLAG.load(Ordering::SeqCst) {
                send_to_frontend(
                    &app_clone,
                    "File listing operation cancelled".to_string(),
                    "remove_all_data",
                );
                return;
            }
            send_to_frontend(
                &app_clone,
                format!("Fetching data from: {:?}", expanded_path),
                "status_update",
            );

            if expanded_path.is_dir() {
                let walkdir = WalkDir::new(&expanded_path).max_depth(
                    if config["recursive"].as_bool().unwrap_or(false) {
                        usize::MAX
                    } else {
                        1
                    },
                );

                let mut file_paths: Vec<_> = walkdir
                    .into_iter()
                    .filter_map(|e| {
                        if CANCEL_FLAG.load(Ordering::SeqCst) {
                            return None;
                        } // Check cancellation inside traversal
                        e.ok()
                    })
                    .filter(|entry| entry.path().is_file())
                    .map(|entry| entry.path().to_path_buf())
                    .collect();

                if CANCEL_FLAG.load(Ordering::SeqCst) {
                    send_to_frontend(
                        &app_clone,
                        "File listing operation cancelled".to_string(),
                        "remove_all_data",
                    );
                    return;
                } // Check after traversal
                file_paths.sort();
                
                const CHUNK_SIZE: usize = 10;
                
                // Process in chunks if there are enough files
                if file_paths.len() >= CHUNK_SIZE {
                    file_paths.par_chunks(CHUNK_SIZE).for_each(|chunk| {
                        if CANCEL_FLAG.load(Ordering::SeqCst) { return; }
                    
                        let files: Vec<_> = chunk.par_iter().filter_map(|file_path| {
                            let file_str = file_path.to_string_lossy().to_string();
                            if CANCEL_FLAG.load(Ordering::SeqCst) || SENT_FILES.contains(&file_str) {
                                return None;
                            }
                            
                            let file_type = get_file_type(&file_path);
                            if file_type != "unknown" {
                                SENT_FILES.insert(file_str.clone());
                                Some(json!({
                                    "id": Uuid::new_v4().to_string(),
                                    "file_path": file_path,
                                    "path": crate::image_processor::process_thumbnail(file_path.to_str().unwrap()),
                                    "type": file_type
                                }))
                            } else {
                                None
                            }
                        }).collect();
                    
                        if !files.is_empty() && !CANCEL_FLAG.load(Ordering::SeqCst) {
                            send_to_frontend(&app_clone, serde_json::to_string(&files).unwrap(), "file_path");
                        }
                    });
                } else {
                    // Handle case where there are fewer files than CHUNK_SIZE
                    if !file_paths.is_empty() {
                        let files: Vec<_> = file_paths.par_iter().filter_map(|file_path| {
                            let file_str = file_path.to_string_lossy().to_string();
                            if CANCEL_FLAG.load(Ordering::SeqCst) || SENT_FILES.contains(&file_str) {
                                return None;
                            }
                            
                            let file_type = get_file_type(&file_path);
                            if file_type != "unknown" {
                                SENT_FILES.insert(file_str.clone());
                                Some(json!({
                                    "id": Uuid::new_v4().to_string(),
                                    "file_path": file_path,
                                    "path": crate::image_processor::process_thumbnail(file_path.to_str().unwrap()),
                                    "type": file_type
                                }))
                            } else {
                                None
                            }
                        }).collect();
                        
                        if !files.is_empty() && !CANCEL_FLAG.load(Ordering::SeqCst) {
                            send_to_frontend(&app_clone, serde_json::to_string(&files).unwrap(), "file_path");
                        }
                    }
                }
            }

            if CANCEL_FLAG.load(Ordering::SeqCst) {
                send_to_frontend(
                    &app_clone,
                    "File listing operation cancelled".to_string(),
                    "remove_all_data",
                );
                return;
            } // Check after directory
        }

        send_to_frontend(
            &app_clone,
            "File listing completed".to_string(),
            "status_update",
        );
    });
}

#[tauri::command]
pub async fn cancel_list_files(app: AppHandle) -> Result<(), String> {
    CANCEL_FLAG.store(true, Ordering::SeqCst);
    SENT_FILES.clear(); // Reset sent file tracking
    send_to_frontend(
        &app,
        "Cancelling file listing operation...".to_string(),
        "status_update",
    );
    Ok(())
}
