use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::collections::HashSet;
use std::io::{BufRead, BufReader, Write};
use std::thread;
use tauri::AppHandle;
use serde_json::json;
use crate::initialise::{EnvPaths, fetch_config};
use crate::utils::send_to_frontend;
use crate::file_processor::cancel_list_files;
use crate::image_processor::process_thumbnail;
use serde_json::Value;
use uuid::Uuid;

lazy_static::lazy_static! {
	static ref PYTHON_PROCESS: Arc<Mutex<Option<std::process::Child>>> = Arc::new(Mutex::new(None));
	static ref SENT_RESULTS: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
}

fn handle_process_output(reader: impl BufRead, app: &AppHandle, event_type: &str, prefix: &str) {
	for line in reader.lines().flatten() {
		if line.contains("\"searched_result\"") {
			if let Ok(parsed) = serde_json::from_str::<Value>(&line) {
				if let Some(paths) = parsed["searched_result"].as_array() {
                    let mut sent_results = SENT_RESULTS.lock().unwrap();

					let results: Vec<Value> = paths
						.iter()
                        .filter_map(|path| path.as_str())
                        .filter(|file_path| sent_results.insert(file_path.to_string())) // Ignore duplicates
						.map(|file_path| json!({
							"id": Uuid::new_v4().to_string(),
							"file_path": file_path,
							"path": process_thumbnail(file_path),
							"type": "image",
							"searched_result":true
						}))
						.collect();

                    if !results.is_empty() {
						send_to_frontend(app, json!(results).to_string(), "searched_result");
                    }
				}
			}
		} else if line.contains("Starting Index process") || line.contains("Index Completed") {
			send_to_frontend(app, format!("{}", line), event_type);
		}
	}
}

// Function to start the Python process
pub async fn start_python_process(app: AppHandle) {
	let paths = EnvPaths::new(); // Ensure this struct has `python_binary` and `search_path`
    let config = fetch_config().await.unwrap();

    // Construct the JSON argument for the Python script
    let python_args_json = json!({
        "index": true,
        "priority_paths": config["priority_paths"]
    }).to_string();
	
	match Command::new(&paths.python_binary)
		.arg("-u")
		.arg(&paths.search_path)
        .arg(python_args_json) // Pass JSON as argument
		.env("PYTHONUNBUFFERED", "1")
		.stdin(Stdio::piped())
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()
	{
		Ok(mut child) => {
			let stdout = child.stdout.take().unwrap();
			let stderr = child.stderr.take().unwrap();
			
			// Store the process (including stdin)
			{
				let mut process_guard = PYTHON_PROCESS.lock().unwrap();
				*process_guard = Some(child); // Store the whole child process, stdin is now part of it.
			}

			// Handle stdout asynchronously
			let app_clone = app.clone();
			thread::spawn(move || {
				let reader = BufReader::new(stdout);
                handle_process_output(reader, &app_clone, "index_status", "");
			});

			// Handle stderr asynchronously
			let app_clone = app.clone();
			thread::spawn(move || {
				let reader = BufReader::new(stderr);
				handle_process_output(reader, &app_clone, "index_status", "");
			});

			// No need for stdin keep-alive thread anymore. Stdin is accessed directly in send_command_to_python
		}
		Err(e) => {
			println!("Failed to start Python process: {}", e);
			send_to_frontend(&app, format!("Failed to start Python process: {}", e), "error");
		}
	}
}

// Function to send JSON commands to Python process
pub fn send_command_to_python(command: &str) {
	let mut process_guard = PYTHON_PROCESS.lock().unwrap();
	if let Some(ref mut child) = *process_guard {
		if let Some(stdin) = child.stdin.as_mut() { // Access stdin from the stored Child struct
			if let Err(e) = writeln!(stdin, "{}", command) {
				println!("Failed to write to Python process: {}", e);
			}
		} else {
            println!("Python process stdin is not available."); // Handle case where stdin might be closed.
		}
	} else {
        println!("Python process is not running."); // Handle case where process is not running.
	}
}

// Stop the Python process
pub fn stop_python_process() {
	let mut process_guard = PYTHON_PROCESS.lock().unwrap();
	if let Some(mut child) = process_guard.take() {
		let _ = child.kill();
	}
}

#[tauri::command]
pub async fn index_data() {
	let command = json!({
		"index": true
	}).to_string();
	send_command_to_python(&command);
}

#[tauri::command]
pub async fn search_indexed_data(search_query: String, app: AppHandle) {
    if !search_query.trim().is_empty() {
		cancel_list_files(app.clone()).await.ok();

        // Reset the sent results tracking
        SENT_RESULTS.lock().unwrap().clear();

        let command = json!({
            "search_data": search_query
        }).to_string();
        send_command_to_python(&command);
		send_to_frontend(&app, format!("Searching for: {}", search_query), "status_update");
		send_to_frontend(&app, format!("Searching for: {}", search_query), "remove_all_data");
    }
}
