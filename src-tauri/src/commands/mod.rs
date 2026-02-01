use serde::Serialize;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tauri::{Emitter, State};

use crate::config::save_config;
use crate::error::{AppError, ErrorResponse};
use crate::llm::ChatMessage;
use crate::search::{self, SearchResult};
use crate::state::AppState;
use crate::types::*;

const SYSTEM_PROMPT: &str = r#"You are AuraForge, a senior engineering assistant specialized in project planning for AI-assisted development workflows. Your role is to help users transform project ideas into comprehensive, actionable plans.

## Your Personality
- Friendly and supportive: Encourage good ideas, celebrate clarity
- Challenging when needed: Push back on weak assumptions, ask probing questions
- Alternatives-focused: When you see a better approach, suggest it
- Practical: Focus on what actually works, not theoretical perfection
- Honest: If something is too complex or a bad idea, say so kindly

## Your Approach
1. Understand first: Ask clarifying questions before suggesting solutions
2. Challenge assumptions: "Are you sure you need X?" / "What if you just..."
3. Offer trade-offs: Present options with pros/cons, let user decide
4. Stay grounded: Reference current best practices
5. Track progress: Mentally note what's decided vs. what's open

## What You Help With
- Clarifying project scope and goals
- Choosing appropriate tech stacks
- Designing system architecture
- Breaking down features into phases
- Identifying potential challenges early
- Creating actionable implementation plans

## What You Don't Do
- Write actual code (that's Claude Code's job)
- Make decisions for the user without discussion
- Over-engineer simple projects
- Recommend technologies without justification
- Rush to solutions before understanding the problem

## Output
When the user is ready, you help generate: README.md, SPEC.md, CLAUDE.md, PROMPTS.md, and CONVERSATION.md."#;

fn to_response<E: Into<AppError>>(err: E) -> ErrorResponse {
    err.into().to_response()
}

// ============ HEALTH & CONFIG ============

#[tauri::command]
pub async fn check_health(state: State<'_, AppState>) -> Result<HealthStatus, ErrorResponse> {
    let config = state
        .config
        .lock()
        .map_err(|_| to_response(AppError::Config("Config lock poisoned".to_string())))?
        .clone();

    let (ollama_connected, ollama_model_available) = state.ollama.health_check(&config).await;

    let config_error = state
        .config_error
        .lock()
        .map_err(|_| to_response(AppError::Config("Config lock poisoned".to_string())))?
        .clone();
    let db_error = state
        .db_error
        .lock()
        .map_err(|_| to_response(AppError::Config("Config lock poisoned".to_string())))?
        .clone();
    let database_ok = state.db.is_ok() && db_error.is_none();
    let config_valid = config_error.is_none();

    let mut errors = Vec::new();

    if !ollama_connected {
        errors.push(format!(
            "Cannot connect to Ollama at {}. Is it running?",
            config.llm.base_url
        ));
    } else if !ollama_model_available {
        errors.push(format!(
            "Model '{}' not found. Run: ollama pull {}",
            config.llm.model,
            config
                .llm
                .model
                .split(':')
                .next()
                .unwrap_or(&config.llm.model)
        ));
    }

    if !database_ok || db_error.is_some() {
        errors.push("Database connection failed.".to_string());
    }
    if let Some(err) = config_error {
        errors.push(format!("Configuration error: {}", err));
    }
    if let Some(err) = db_error {
        errors.push(format!("Database error: {}", err));
    }

    Ok(HealthStatus {
        ollama_connected,
        ollama_model_available,
        database_ok,
        config_valid,
        errors,
    })
}

#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<AppConfig, ErrorResponse> {
    Ok(state
        .config
        .lock()
        .map_err(|_| to_response(AppError::Config("Config lock poisoned".to_string())))?
        .clone())
}

#[tauri::command]
pub async fn update_search_config(
    state: State<'_, AppState>,
    search_config: SearchConfig,
) -> Result<(), ErrorResponse> {
    let mut config = state
        .config
        .lock()
        .map_err(|_| to_response(AppError::Config("Config lock poisoned".to_string())))?;
    config.search = search_config;
    save_config(&config).map_err(|e| to_response(AppError::Config(e)))?;
    if let Ok(mut err) = state.config_error.lock() {
        *err = None;
    }
    Ok(())
}

#[tauri::command]
pub async fn update_config(
    state: State<'_, AppState>,
    config: AppConfig,
) -> Result<AppConfig, ErrorResponse> {
    let mut state_config = state
        .config
        .lock()
        .map_err(|_| to_response(AppError::Config("Config lock poisoned".to_string())))?;
    *state_config = config;
    save_config(&state_config).map_err(|e| to_response(AppError::Config(e)))?;
    if let Ok(mut err) = state.config_error.lock() {
        *err = None;
    }
    Ok(state_config.clone())
}

// ============ PREFERENCES ============

#[tauri::command]
pub async fn get_preference(
    state: State<'_, AppState>,
    key: String,
) -> Result<Option<String>, ErrorResponse> {
    state
        .db
        .get_preference(&key)
        .map_err(to_response)
}

#[tauri::command]
pub async fn set_preference(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> Result<(), ErrorResponse> {
    state
        .db
        .set_preference(&key, &value)
        .map_err(to_response)
}

// ============ MODELS ============

#[tauri::command]
pub async fn list_models(state: State<'_, AppState>) -> Result<Vec<String>, ErrorResponse> {
    let config = state
        .config
        .lock()
        .map_err(|_| to_response(AppError::Config("Config lock poisoned".to_string())))?
        .clone();
    state
        .ollama
        .list_models(&config.llm.base_url)
        .await
        .map_err(to_response)
}

#[tauri::command]
pub async fn pull_model(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    model_name: String,
) -> Result<(), ErrorResponse> {
    let config = state
        .config
        .lock()
        .map_err(|_| to_response(AppError::Config("Config lock poisoned".to_string())))?
        .clone();
    state
        .ollama
        .pull_model(&app, &config.llm.base_url, &model_name)
        .await
        .map_err(to_response)
}

#[tauri::command]
pub async fn cancel_pull_model(state: State<'_, AppState>) -> Result<(), ErrorResponse> {
    state.ollama.cancel_pull();
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
pub struct DiskSpace {
    pub available_gb: f64,
    pub sufficient: bool,
}

#[tauri::command]
pub async fn check_disk_space() -> Result<DiskSpace, ErrorResponse> {
    let result = tauri::async_runtime::spawn_blocking(|| -> Result<DiskSpace, AppError> {
        let output = std::process::Command::new("df")
            .args(["-k", "/"])
            .output()
            .map_err(|e| AppError::FileSystem {
                path: "/".to_string(),
                message: e.to_string(),
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let available_kb: u64 = stdout
            .lines()
            .nth(1)
            .and_then(|line| line.split_whitespace().nth(3))
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        let available_gb = available_kb as f64 / 1_048_576.0;

        Ok(DiskSpace {
            available_gb,
            sufficient: available_gb > 20.0,
        })
    })
    .await
    .map_err(|e| {
        to_response(AppError::FileSystem {
            path: "/".to_string(),
            message: format!("Failed to check disk space: {}", e),
        })
    })?;

    result.map_err(to_response)
}

// ============ SESSIONS ============

#[tauri::command]
pub async fn create_session(
    state: State<'_, AppState>,
    request: CreateSessionRequest,
) -> Result<Session, ErrorResponse> {
    state
        .db
        .create_session(request.name.as_deref())
        .map_err(to_response)
}

#[tauri::command]
pub async fn get_sessions(state: State<'_, AppState>) -> Result<Vec<Session>, ErrorResponse> {
    state
        .db
        .get_sessions()
        .map_err(to_response)
}

#[tauri::command]
pub async fn get_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Session, ErrorResponse> {
    match state.db.get_session(&session_id) {
        Ok(session) => Ok(session),
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            Err(to_response(AppError::SessionNotFound(session_id)))
        }
        Err(e) => Err(to_response(e)),
    }
}

#[tauri::command]
pub async fn update_session(
    state: State<'_, AppState>,
    session_id: String,
    name: Option<String>,
    status: Option<String>,
) -> Result<Session, ErrorResponse> {
    match state
        .db
        .update_session(&session_id, name.as_deref(), status.as_deref())
    {
        Ok(session) => Ok(session),
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            Err(to_response(AppError::SessionNotFound(session_id)))
        }
        Err(e) => Err(to_response(e)),
    }
}

#[tauri::command]
pub async fn delete_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<(), ErrorResponse> {
    state.db.delete_session(&session_id).map_err(to_response)
}

// ============ MESSAGES ============

#[tauri::command]
pub async fn get_messages(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Vec<Message>, ErrorResponse> {
    state.db.get_messages(&session_id).map_err(to_response)
}

#[tauri::command]
pub async fn send_message(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    request: SendMessageRequest,
) -> Result<Message, ErrorResponse> {
    let session_id = request.session_id;
    let content = request.content;
    let is_retry = request.retry.unwrap_or(false);

    // Save user message (skip on retry — message already exists in DB)
    let user_msg = if is_retry {
        // Find the last user message from DB
        let messages = state
            .db
            .get_messages(&session_id)
            .map_err(to_response)?;
        messages
            .into_iter()
            .rev()
            .find(|m| m.role == "user")
            .ok_or_else(|| to_response(AppError::Unknown("No user message found to retry".to_string())))?
    } else {
        state
            .db
            .save_message(&session_id, "user", &content, None)
            .map_err(to_response)?
    };

    // Auto-name session on first user message
    let user_count = state.db.message_count(&session_id).unwrap_or(0);
    if user_count == 1 && !is_retry {
        let auto_name: String = content.chars().take(60).collect();
        let auto_name = auto_name.trim().to_string();
        let auto_name = if auto_name.len() < content.len() {
            format!("{}...", auto_name.trim_end())
        } else {
            auto_name
        };
        let _ = state.db.update_session(&session_id, Some(&auto_name), None);
    }

    // Get config
    let config = state
        .config
        .lock()
        .map_err(|_| to_response(AppError::Config("Config lock poisoned".to_string())))?
        .clone();

    // === Web Search Integration ===
    let mut search_query: Option<String> = None;
    let mut search_results: Option<Vec<SearchResult>> = None;

    if config.search.enabled && config.search.proactive {
        if let Some(query) = search::should_search(&content) {
            search_query = Some(query.clone());

            // Emit search_start event
            let _ = app.emit(
                "stream:search",
                crate::llm::StreamChunk {
                    r#type: "search_start".to_string(),
                    search_query: Some(query.clone()),
                    session_id: Some(session_id.clone()),
                    ..Default::default()
                },
            );

            // Execute search
            match search::execute_search(&config.search, &query).await {
                Ok(results) => {
                    // Emit search_result event
                    let _ = app.emit(
                        "stream:search",
                        crate::llm::StreamChunk {
                            r#type: "search_result".to_string(),
                            search_results: Some(results.clone()),
                            session_id: Some(session_id.clone()),
                            ..Default::default()
                        },
                    );
                    search_results = Some(results);
                }
                Err(e) => {
                    log::warn!("Search failed (continuing without): {}", e);
                }
            }
        }
    }

    // Build conversation history for LLM
    let db_messages = state
        .db
        .get_messages(&session_id)
        .map_err(to_response)?;

    let mut chat_messages = vec![ChatMessage {
        role: "system".to_string(),
        content: SYSTEM_PROMPT.to_string(),
    }];

    // Inject search context as a system message if we have results
    if let Some(ref results) = search_results {
        chat_messages.push(ChatMessage {
            role: "system".to_string(),
            content: build_search_context(search_query.as_deref().unwrap_or(""), results),
        });
    }

    for msg in &db_messages {
        if msg.role == "system" {
            continue;
        }
        chat_messages.push(ChatMessage {
            role: msg.role.clone(),
            content: msg.content.clone(),
        });
    }

    // Stream the LLM response
    let cancel_flag = Arc::new(AtomicBool::new(false));
    if let Ok(mut map) = state.stream_cancel.lock() {
        map.insert(session_id.clone(), cancel_flag.clone());
    }

    let full_response = state
        .ollama
        .stream_chat(
            &app,
            &config.llm.base_url,
            &config.llm.model,
            chat_messages,
            config.llm.temperature,
            &session_id,
            Some(cancel_flag.clone()),
        )
        .await;

    match full_response {
        Ok(response_text) => {
            // Build metadata with search info
            let metadata = if search_query.is_some() || search_results.is_some() {
                let meta = serde_json::json!({
                    "search_query": search_query,
                    "search_results": search_results,
                });
                Some(meta.to_string())
            } else {
                None
            };

            let _ = state.db.save_message(
                &session_id,
                "assistant",
                &response_text,
                metadata.as_deref(),
            );
        }
        Err(AppError::StreamCancelled) => {
            if let Ok(mut map) = state.stream_cancel.lock() {
                map.remove(&session_id);
            }
            return Ok(user_msg);
        }
        Err(e) => {
            let _ = app.emit(
                "stream:error",
                crate::llm::StreamChunk {
                    r#type: "error".to_string(),
                    error: Some(e.to_string()),
                    session_id: Some(session_id.clone()),
                    ..Default::default()
                },
            );
            if let Ok(mut map) = state.stream_cancel.lock() {
                map.remove(&session_id);
            }
            return Err(to_response(e));
        }
    }

    if let Ok(mut map) = state.stream_cancel.lock() {
        map.remove(&session_id);
    }

    Ok(user_msg)
}

#[tauri::command]
pub async fn cancel_response(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<(), ErrorResponse> {
    if let Ok(map) = state.stream_cancel.lock() {
        if let Some(flag) = map.get(&session_id) {
            flag.store(true, std::sync::atomic::Ordering::SeqCst);
        }
    }
    Ok(())
}

// ============ DOCUMENTS ============

#[tauri::command]
pub async fn generate_documents(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    request: GenerateDocumentsRequest,
) -> Result<Vec<GeneratedDocument>, ErrorResponse> {
    crate::docgen::generate_all_documents(&app, &state, &request.session_id)
        .await
        .map_err(to_response)
}

#[tauri::command]
pub async fn get_documents(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Vec<GeneratedDocument>, ErrorResponse> {
    state.db.get_documents(&session_id).map_err(to_response)
}

#[tauri::command]
pub async fn check_documents_stale(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<bool, ErrorResponse> {
    let doc_time = state
        .db
        .latest_document_time(&session_id)
        .map_err(to_response)?;

    let msg_time = state
        .db
        .latest_message_time(&session_id)
        .map_err(to_response)?;

    match (doc_time, msg_time) {
        (Some(dt), Some(mt)) => {
            let parse = |value: &str| {
                chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S").ok()
            };
            match (parse(&dt), parse(&mt)) {
                (Some(doc_dt), Some(msg_dt)) => Ok(msg_dt > doc_dt),
                _ => Ok(true),
            }
        }
        (None, _) => Ok(false), // No docs yet, not "stale"
        _ => Ok(false),
    }
}

// ============ EXPORT ============

#[tauri::command]
pub async fn save_to_folder(
    state: State<'_, AppState>,
    request: SaveToFolderRequest,
) -> Result<String, ErrorResponse> {
    let documents = state
        .db
        .get_documents(&request.session_id)
        .map_err(to_response)?;

    if documents.is_empty() {
        return Err(to_response(AppError::FileSystem {
            path: request.folder_path.clone(),
            message: "No documents to save. Generate documents first.".to_string(),
        }));
    }

    let session = state
        .db
        .get_session(&request.session_id)
        .map_err(to_response)?;

    // Sanitize session name for folder name
    let sanitized_name = sanitize_folder_name(&session.name);
    let output_dir =
        std::path::PathBuf::from(&request.folder_path).join(format!("{}-plan", sanitized_name));

    let output_path = output_dir.to_string_lossy().to_string();
    let output_path_for_thread = output_path.clone();
    let docs_for_thread = documents.clone();
    let output_dir_for_thread = output_dir.clone();

    let write_result = tauri::async_runtime::spawn_blocking(move || -> Result<(), AppError> {
        if output_dir_for_thread.exists() {
            return Err(AppError::FolderExists(output_path_for_thread));
        }

        std::fs::create_dir(&output_dir_for_thread).map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                AppError::FileSystem {
                    path: output_dir_for_thread.to_string_lossy().to_string(),
                    message: "Can't write to this location. Choose another folder.".to_string(),
                }
            } else {
                AppError::FileSystem {
                    path: output_dir_for_thread.to_string_lossy().to_string(),
                    message: format!("Failed to create folder: {}", e),
                }
            }
        })?;

        for doc in &docs_for_thread {
            let file_path = output_dir_for_thread.join(&doc.filename);
            std::fs::write(&file_path, &doc.content).map_err(|e| {
                if e.raw_os_error() == Some(28) {
                    AppError::FileSystem {
                        path: file_path.to_string_lossy().to_string(),
                        message: "Not enough disk space. Free up space and try again.".to_string(),
                    }
                } else if e.kind() == std::io::ErrorKind::PermissionDenied {
                    AppError::FileSystem {
                        path: file_path.to_string_lossy().to_string(),
                        message: format!(
                            "Permission denied writing {}. Choose another folder.",
                            doc.filename
                        ),
                    }
                } else {
                    AppError::FileSystem {
                        path: file_path.to_string_lossy().to_string(),
                        message: format!("Failed to write {}: {}", doc.filename, e),
                    }
                }
            })?;
        }

        Ok(())
    })
    .await
    .map_err(|e| {
        to_response(AppError::FileSystem {
            path: output_path.clone(),
            message: format!("Failed to write files: {}", e),
        })
    })?;

    write_result.map_err(to_response)?;
    log::info!("Saved {} documents to {}", documents.len(), output_path);

    Ok(output_path)
}

// ============ SEARCH ============

#[tauri::command]
pub async fn web_search(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<SearchResult>, ErrorResponse> {
    let config = state
        .config
        .lock()
        .map_err(|_| to_response(AppError::Config("Config lock poisoned".to_string())))?
        .clone();
    let mut search_config = config.search.clone();
    search_config.enabled = true;
    if search_config.provider == "none" {
        search_config.provider = "duckduckgo".to_string();
    }
    search::execute_search(&search_config, &query)
        .await
        .map_err(to_response)
}

fn sanitize_folder_name(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
        .to_lowercase()
        .chars()
        .take(60)
        .collect();

    if sanitized.is_empty() || sanitized.chars().all(|c| c == '_' || c == '-') {
        "untitled".to_string()
    } else {
        sanitized
    }
}

// ============ HELPERS ============

fn build_search_context(query: &str, results: &[SearchResult]) -> String {
    let mut context = format!(
        "## Web Search Results\nThe following search results were found for \"{}\":\n\n",
        query
    );

    for (i, result) in results.iter().enumerate() {
        context.push_str(&format!(
            "{}. **{}**\n   URL: {}\n   {}\n\n",
            i + 1,
            result.title,
            result.url,
            result.snippet
        ));
    }

    context.push_str(
        "Use these search results to inform your response where relevant. \
         Cite sources when referencing specific information.",
    );

    context
}
