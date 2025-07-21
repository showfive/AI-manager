// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use ai_manager_shared::messages::ServiceMessage;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::{mpsc, Mutex};

#[derive(Debug, Serialize, Deserialize)]
struct MessageResponse {
    content: String,
    timestamp: String,
}

struct AppState {
    core_sender: Arc<Mutex<Option<mpsc::UnboundedSender<ServiceMessage>>>>,
}

#[tauri::command]
async fn greet(name: &str) -> Result<String, String> {
    Ok(format!("Hello, {}! You've been greeted from Rust!", name))
}

#[tauri::command]
async fn send_message(message: &str, state: State<'_, AppState>) -> Result<String, String> {
    println!("Received message: {}", message);

    // For now, return a simple response
    // TODO: Connect to the core service
    Ok(format!("Echo: {}", message))
}

#[tokio::main]
async fn main() {
    let app_state = AppState {
        core_sender: Arc::new(Mutex::new(None)),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![greet, send_message])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
