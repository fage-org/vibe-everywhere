#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

mod commands;

struct AppState {
    server_url: Mutex<String>,
}

fn main() {
    tauri::Builder::default()
        .manage(AppState {
            server_url: Mutex::new(String::new()),
        })
        .invoke_handler(tauri::generate_handler![
            commands::ping
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
