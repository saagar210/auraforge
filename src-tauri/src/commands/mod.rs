use tauri::{Emitter, State};

use crate::config::save_config;
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

// ============ HEALTH & CONFIG ============

#[tauri::command]
pub async fn check_health(state: State<'_, AppState>) -> Result<HealthStatus, String> {
    let config = state.config.lock().unwrap().clone();

    let (ollama_connected, ollama_model_available) = state.ollama.health_check(&config).await;

    let database_ok = state.db.is_ok();
    let config_valid = true;

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

    if !database_ok {
        errors.push("Database connection failed.".to_string());
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
pub async fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    Ok(state.config.lock().unwrap().clone())
}

#[tauri::command]
pub async fn update_search_config(
    state: State<'_, AppState>,
    search_config: SearchConfig,
) -> Result<(), String> {
    let mut config = state.config.lock().unwrap();
    config.search = search_config;
    save_config(&config)?;
    Ok(())
}

#[tauri::command]
pub async fn update_config(
    state: State<'_, AppState>,
    llm: Option<LLMConfig>,
    search: Option<SearchConfig>,
    ui: Option<UIConfig>,
    output: Option<OutputConfig>,
) -> Result<AppConfig, String> {
    let mut config = state.config.lock().unwrap();
    if let Some(l) = llm {
        config.llm = l;
    }
    if let Some(s) = search {
        config.search = s;
    }
    if let Some(u) = ui {
        config.ui = u;
    }
    if let Some(o) = output {
        config.output = o;
    }
    save_config(&config)?;
    Ok(config.clone())
}

// ============ SESSIONS ============

#[tauri::command]
pub async fn create_session(
    state: State<'_, AppState>,
    name: Option<String>,
) -> Result<Session, String> {
    state
        .db
        .create_session(name.as_deref())
        .map_err(|e| format!("Failed to create session: {}", e))
}

#[tauri::command]
pub async fn get_sessions(state: State<'_, AppState>) -> Result<Vec<Session>, String> {
    state
        .db
        .get_sessions()
        .map_err(|e| format!("Failed to get sessions: {}", e))
}

#[tauri::command]
pub async fn get_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Session, String> {
    state
        .db
        .get_session(&session_id)
        .map_err(|e| format!("Session not found: {}", e))
}

#[tauri::command]
pub async fn update_session(
    state: State<'_, AppState>,
    session_id: String,
    name: Option<String>,
    status: Option<String>,
) -> Result<Session, String> {
    state
        .db
        .update_session(&session_id, name.as_deref(), status.as_deref())
        .map_err(|e| format!("Failed to update session: {}", e))
}

#[tauri::command]
pub async fn delete_session(state: State<'_, AppState>, session_id: String) -> Result<(), String> {
    state
        .db
        .delete_session(&session_id)
        .map_err(|e| format!("Failed to delete session: {}", e))
}

// ============ MESSAGES ============

#[tauri::command]
pub async fn get_messages(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Vec<Message>, String> {
    state
        .db
        .get_messages(&session_id)
        .map_err(|e| format!("Failed to get messages: {}", e))
}

#[tauri::command]
pub async fn send_message(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    session_id: String,
    content: String,
    retry: Option<bool>,
) -> Result<Message, String> {
    let is_retry = retry.unwrap_or(false);

    // Save user message (skip on retry — message already exists in DB)
    let user_msg = if is_retry {
        // Find the last user message from DB
        let messages = state
            .db
            .get_messages(&session_id)
            .map_err(|e| format!("Failed to get messages: {}", e))?;
        messages
            .into_iter()
            .rev()
            .find(|m| m.role == "user")
            .ok_or_else(|| "No user message found to retry".to_string())?
    } else {
        state
            .db
            .save_message(&session_id, "user", &content, None)
            .map_err(|e| format!("Failed to save message: {}", e))?
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
    let config = state.config.lock().unwrap().clone();

    // === Web Search Integration ===
    let mut search_query: Option<String> = None;
    let mut search_results: Option<Vec<SearchResult>> = None;

    if config.search.enabled && config.search.proactive {
        if let Some(query) = search::should_search(&content) {
            search_query = Some(query.clone());

            // Emit search_start event
            let _ = app.emit(
                "stream:chunk",
                crate::llm::StreamChunk {
                    r#type: "search_start".to_string(),
                    search_query: Some(query.clone()),
                    ..Default::default()
                },
            );

            // Execute search
            match search::execute_search(&config.search, &query).await {
                Ok(results) => {
                    // Emit search_result event
                    let _ = app.emit(
                        "stream:chunk",
                        crate::llm::StreamChunk {
                            r#type: "search_result".to_string(),
                            search_results: Some(results.clone()),
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
        .map_err(|e| format!("Failed to load history: {}", e))?;

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
    let full_response = state
        .ollama
        .stream_chat(
            &app,
            &config.llm.base_url,
            &config.llm.model,
            chat_messages,
            config.llm.temperature,
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
        Err(e) => {
            let _ = app.emit(
                "stream:error",
                crate::llm::StreamChunk {
                    r#type: "error".to_string(),
                    error: Some(e.clone()),
                    ..Default::default()
                },
            );
            return Err(e);
        }
    }

    Ok(user_msg)
}

// ============ DOCUMENTS ============

#[tauri::command]
pub async fn generate_documents(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Vec<GeneratedDocument>, String> {
    crate::docgen::generate_all_documents(&app, &state, &session_id).await
}

#[tauri::command]
pub async fn get_documents(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Vec<GeneratedDocument>, String> {
    state
        .db
        .get_documents(&session_id)
        .map_err(|e| format!("Failed to get documents: {}", e))
}

#[tauri::command]
pub async fn check_documents_stale(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<bool, String> {
    let doc_time = state
        .db
        .latest_document_time(&session_id)
        .map_err(|e| format!("Failed to check document time: {}", e))?;

    let msg_time = state
        .db
        .latest_message_time(&session_id)
        .map_err(|e| format!("Failed to check message time: {}", e))?;

    match (doc_time, msg_time) {
        (Some(dt), Some(mt)) => Ok(mt > dt),
        (None, _) => Ok(false), // No docs yet, not "stale"
        _ => Ok(false),
    }
}

// ============ EXPORT ============

#[tauri::command]
pub async fn save_to_folder(
    state: State<'_, AppState>,
    session_id: String,
    folder_path: String,
) -> Result<String, String> {
    let documents = state
        .db
        .get_documents(&session_id)
        .map_err(|e| format!("Failed to load documents: {}", e))?;

    if documents.is_empty() {
        return Err("No documents to save. Generate documents first.".to_string());
    }

    let session = state
        .db
        .get_session(&session_id)
        .map_err(|e| format!("Failed to load session: {}", e))?;

    // Sanitize session name for folder name
    let sanitized_name = sanitize_folder_name(&session.name);
    let output_dir =
        std::path::PathBuf::from(&folder_path).join(format!("{}-plan", sanitized_name));

    // Create the output directory
    std::fs::create_dir_all(&output_dir).map_err(|e| {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            "Can't write to this location. Choose another folder.".to_string()
        } else {
            format!("Failed to create folder: {}", e)
        }
    })?;

    // Write each document
    for doc in &documents {
        let file_path = output_dir.join(&doc.filename);
        std::fs::write(&file_path, &doc.content).map_err(|e| {
            if e.raw_os_error() == Some(28) {
                "Not enough disk space. Free up space and try again.".to_string()
            } else if e.kind() == std::io::ErrorKind::PermissionDenied {
                format!(
                    "Permission denied writing {}. Choose another folder.",
                    doc.filename
                )
            } else {
                format!("Failed to write {}: {}", doc.filename, e)
            }
        })?;
    }

    let output_path = output_dir.to_string_lossy().to_string();
    log::info!("Saved {} documents to {}", documents.len(), output_path);

    Ok(output_path)
}

#[tauri::command]
pub async fn open_folder(path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    Ok(())
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
