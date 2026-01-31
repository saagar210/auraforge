mod commands;
mod config;
mod db;
mod docgen;
mod llm;
mod search;
mod state;
mod types;

use std::sync::Mutex;

use config::{db_path, load_or_create_config};
use db::Database;
use llm::OllamaClient;
use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = load_or_create_config().expect("Failed to load configuration");

    let db_file = db_path();
    let db = match Database::new(&db_file) {
        Ok(db) => db,
        Err(e) => {
            log::warn!("Database corrupted ({}), recreating", e);
            // Rename the corrupt DB and create a fresh one
            let backup = db_file.with_extension("db.bak");
            let _ = std::fs::rename(&db_file, &backup);
            Database::new(&db_file).expect("Failed to create fresh database")
        }
    };

    let ollama = OllamaClient::new();

    let app_state = AppState {
        db,
        ollama,
        config: Mutex::new(config),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::check_health,
            commands::create_session,
            commands::get_sessions,
            commands::get_session,
            commands::update_session,
            commands::delete_session,
            commands::get_messages,
            commands::send_message,
            commands::get_config,
            commands::update_search_config,
            commands::update_config,
            commands::generate_documents,
            commands::get_documents,
            commands::check_documents_stale,
            commands::save_to_folder,
            commands::open_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
