// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod utils;
mod initialise;

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            initialise::initialize_environment,
            initialise::fetch_config,
            initialise::save_config,
            utils::select_folder,
            utils::list_files,
            utils::read_image
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}