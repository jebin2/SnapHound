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
use std::thread;
use std::sync::{atomic::{AtomicBool}};

lazy_static::lazy_static! {
    static ref CANCEL_FLAG: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
}

#[tauri::command]
pub async fn list_files(app: AppHandle) {
    cancel_list_files(app.clone()).await.ok(); // Cancel any existing operation

    CANCEL_FLAG.store(false, Ordering::SeqCst); // Reset cancellation flag
    crate::utils::send_to_frontend(&app, "File listing operation cancelled".to_string(), "remove_all_data");
    let app_clone = app.clone();
    let config = crate::initialise::fetch_config().await.unwrap(); // Asynchronous call
    thread::spawn(move || {
        // Run list_files logic here, but check CANCEL_FLAG periodically

        let priority_paths: Vec<String> = serde_json::from_value(config["priority_paths"].clone()).unwrap_or_default();

        if priority_paths.is_empty() {
            crate::utils::send_to_frontend(&app_clone, "No paths configured for listing".to_string(), "status_update");
            return;
        }

        let mut directories = crate::utils::expand_paths(priority_paths);
        directories.sort();

        for expanded_path in directories {
            if CANCEL_FLAG.load(Ordering::SeqCst) { // Check cancellation flag
                crate::utils::send_to_frontend(&app_clone, "File listing operation cancelled".to_string(), "remove_all_data");
                return; // Stop processing
            }

            crate::utils::send_to_frontend(&app_clone, format!("Fetching data from: {:?}", expanded_path), "status_update");

            if expanded_path.is_dir() {
                let walkdir = walkdir::WalkDir::new(&expanded_path)
                    .max_depth(if config["recursive"].as_bool().unwrap_or(false) { usize::MAX } else { 1 });

                let mut file_paths: Vec<_> = walkdir
                    .into_iter()
                    .filter_map(|e| {
                        if CANCEL_FLAG.load(Ordering::SeqCst) { return None; } // Check cancellation inside traversal
                        e.ok()
                    })
                    .filter(|entry| entry.path().is_file())
                    .map(|entry| entry.path().to_path_buf())
                    .collect();

                if CANCEL_FLAG.load(Ordering::SeqCst) { crate::utils::send_to_frontend(&app_clone, "File listing operation cancelled".to_string(), "remove_all_data"); return; } // Check after traversal
                file_paths.sort();

                const CHUNK_SIZE: usize = 1;
                file_paths
                    .par_chunks(CHUNK_SIZE)
                    .for_each(|chunk| {
                        if CANCEL_FLAG.load(Ordering::SeqCst) { return; } // Check cancellation before chunk

                        let files: Vec<_> = chunk
                            .par_iter()
                            .filter_map(|file_path| {
                                if CANCEL_FLAG.load(Ordering::SeqCst) { return None; } // Check cancellation inside parallel file loop

                                let file_type = crate::utils::get_file_type(&file_path);
                                if file_type != "unknown" {
                                    Some(serde_json::json!({
                                        "id": uuid::Uuid::new_v4().to_string(),
                                        "path": crate::image_processor::process_thumbnail(file_path.to_str().unwrap()),
                                        "type": file_type
                                    }))
                                } else {
                                    None
                                }
                            })
                            .collect();

                        if !files.is_empty() && !CANCEL_FLAG.load(Ordering::SeqCst) { // Check before sending
                            let json_data = serde_json::to_string(&files).unwrap();
                            crate::utils::send_to_frontend(&app_clone, json_data, "file_path");
                        }
                    });
            }

            if CANCEL_FLAG.load(Ordering::SeqCst) { crate::utils::send_to_frontend(&app_clone, "File listing operation cancelled".to_string(), "remove_all_data"); return; } // Check after directory
        }

        crate::utils::send_to_frontend(&app_clone, "File listing completed".to_string(), "status_update");
    });
}

#[tauri::command]
pub async fn cancel_list_files(app: AppHandle) -> Result<(), String> {
    CANCEL_FLAG.store(true, Ordering::SeqCst); // Set cancellation flag
    crate::utils::send_to_frontend(&app, "Cancelling file listing operation...".to_string(), "status_update");
    Ok(())
}