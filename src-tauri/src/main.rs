// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod utils;
mod initialise;
mod image_processor;
mod search_processor;
mod file_processor;

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            initialise::initialize_environment,
            initialise::fetch_config,
            initialise::save_config,
            utils::select_folder,
            utils::read_image,
            utils::reset_all,
            utils::relaunch,
            search_processor::search_indexed_data,
            file_processor::list_files,
            file_processor::cancel_list_files
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}