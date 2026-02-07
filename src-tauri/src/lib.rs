mod commands;
mod config;
mod db;
mod docgen;
mod error;
mod importer;
mod llm;
mod search;
mod state;
mod templates;
mod types;

use std::sync::Mutex;

use config::{db_path, load_or_create_config};
use db::Database;
use llm::OllamaClient;
use state::AppState;
use tauri::menu::{MenuBuilder, PredefinedMenuItem, SubmenuBuilder};
use tauri::Emitter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (config, config_error) = load_or_create_config();

    let db_file = db_path();
    let mut db_error = None;
    let db = match Database::new(&db_file) {
        Ok(db) => db,
        Err(e) => {
            log::warn!("Database corrupted ({}), recreating", e);
            let backup = db_file.with_extension("db.bak");
            let _ = std::fs::rename(&db_file, &backup);
            match Database::new(&db_file) {
                Ok(db) => db,
                Err(e) => {
                    db_error = Some(format!("Failed to open database: {}", e));
                    match Database::new_in_memory() {
                        Ok(db) => db,
                        Err(err) => {
                            log::error!("Failed to create fallback DB: {}", err);
                            std::process::exit(1);
                        }
                    }
                }
            }
        }
    };

    let ollama = OllamaClient::new();

    let app_state = AppState {
        db,
        ollama,
        config: Mutex::new(config),
        config_error: Mutex::new(config_error),
        db_error: Mutex::new(db_error),
        stream_cancel: Mutex::new(std::collections::HashMap::new()),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            {
                let level = if cfg!(debug_assertions) {
                    log::LevelFilter::Debug
                } else {
                    log::LevelFilter::Warn
                };
                app.handle()
                    .plugin(tauri_plugin_log::Builder::default().level(level).build())?;
            }

            // Build menu bar
            let file_menu = SubmenuBuilder::new(app, "File")
                .item(&tauri::menu::MenuItem::with_id(
                    app,
                    "new_session",
                    "New Session",
                    true,
                    Some("CmdOrCtrl+N"),
                )?)
                .item(&tauri::menu::MenuItem::with_id(
                    app,
                    "save_to_folder",
                    "Save to Folder",
                    true,
                    Some("CmdOrCtrl+S"),
                )?)
                .separator()
                .item(&PredefinedMenuItem::close_window(app, None)?)
                .build()?;

            let edit_menu = SubmenuBuilder::new(app, "Edit")
                .item(&PredefinedMenuItem::undo(app, None)?)
                .item(&PredefinedMenuItem::redo(app, None)?)
                .separator()
                .item(&PredefinedMenuItem::cut(app, None)?)
                .item(&PredefinedMenuItem::copy(app, None)?)
                .item(&PredefinedMenuItem::paste(app, None)?)
                .item(&PredefinedMenuItem::select_all(app, None)?)
                .build()?;

            let view_menu = SubmenuBuilder::new(app, "View")
                .item(&tauri::menu::MenuItem::with_id(
                    app,
                    "toggle_sidebar",
                    "Toggle Sidebar",
                    true,
                    Some("CmdOrCtrl+\\"),
                )?)
                .item(&tauri::menu::MenuItem::with_id(
                    app,
                    "toggle_preview",
                    "Toggle Preview",
                    true,
                    Some("CmdOrCtrl+Shift+P"),
                )?)
                .build()?;

            let session_menu = SubmenuBuilder::new(app, "Session")
                .item(&tauri::menu::MenuItem::with_id(
                    app,
                    "rename_session",
                    "Rename Session",
                    true,
                    None::<&str>,
                )?)
                .item(&tauri::menu::MenuItem::with_id(
                    app,
                    "delete_session",
                    "Delete Session",
                    true,
                    None::<&str>,
                )?)
                .build()?;

            let help_menu = SubmenuBuilder::new(app, "Help")
                .item(&tauri::menu::MenuItem::with_id(
                    app,
                    "help_panel",
                    "AuraForge Help",
                    true,
                    Some("CmdOrCtrl+/"),
                )?)
                .separator()
                .item(&PredefinedMenuItem::about(app, None, None)?)
                .build()?;

            let menu = MenuBuilder::new(app)
                .item(&file_menu)
                .item(&edit_menu)
                .item(&view_menu)
                .item(&session_menu)
                .item(&help_menu)
                .build()?;

            app.set_menu(menu)?;

            let handle = app.handle().clone();
            app.on_menu_event(move |_app, event| {
                let _ = handle.emit("menu:action", event.id().0.as_str());
            });

            Ok(())
        })
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::check_health,
            commands::get_preference,
            commands::set_preference,
            commands::list_models,
            commands::pull_model,
            commands::cancel_pull_model,
            commands::check_disk_space,
            commands::create_session,
            commands::create_session_from_template,
            commands::create_branch_from_message,
            commands::get_sessions,
            commands::get_session,
            commands::update_session,
            commands::delete_session,
            commands::delete_sessions,
            commands::list_templates,
            commands::get_messages,
            commands::send_message,
            commands::cancel_response,
            commands::import_codebase_context,
            commands::get_config,
            commands::update_search_config,
            commands::update_config,
            commands::generate_documents,
            commands::get_documents,
            commands::check_documents_stale,
            commands::analyze_plan_readiness,
            commands::get_planning_coverage,
            commands::get_generation_metadata,
            commands::get_generation_confidence,
            commands::save_to_folder,
            commands::web_search,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            log::error!("error while running tauri application: {}", e);
        });
}
