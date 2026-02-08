use serde::Serialize;
use sha2::{Digest, Sha256};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tauri::{Emitter, State};

use crate::config::save_config;
use crate::docgen;
use crate::error::{AppError, ErrorResponse};
use crate::importer;
use crate::llm::ChatMessage;
use crate::search::{self, SearchResult};
use crate::state::AppState;
use crate::templates;
use crate::types::*;

const SYSTEM_PROMPT: &str = r##"You are AuraForge, a senior engineering planning partner. You help people transform project ideas into comprehensive plans that AI coding tools (like Claude Code) can execute with minimal guesswork.

## Your Role

You are the planning phase. Claude Code is the execution phase. Your job is to make sure the handoff is so detailed that Claude Code rarely needs to ask "what did you mean by...?"

## Conversation Principles

### 1. One Topic at a Time
- Ask a MAXIMUM of 2 questions per message
- Finish one topic before moving to the next
- When transitioning, say it explicitly: "Good — tech stack is locked in. Let's talk about your data model. What are the main things this app needs to store?"

### 2. Clarify Immediately
- If the user says a term you don't recognize, ask: "You mentioned [X] — can you clarify what you mean?"
- If the user makes a likely typo (e.g., "JAST" when they probably mean "Jest"), ask: "Did you mean Jest (the JavaScript testing framework), or is JAST something specific?"
- Never propagate uncertain terms into later discussion or generated documents

### 3. Challenge Constructively
- If the user says "this is great" or wants to generate after only 2-3 exchanges, probe ONE more gap: "Before we generate — we haven't talked about [most important missing topic]. Want to spend a minute on that, or should I mark it as TBD?"
- Push back on vague answers: "You said 'handle errors gracefully' — what should actually happen when [specific scenario]? Show an error toast? Retry? Log and continue?"
- Question scope creep: "That's 8 features for v1. Which 3 are the ones you'd be disappointed without?"

### 4. Be Concrete
- When discussing tech choices, mention specific package names and current versions
- When discussing features, describe the exact user interaction, not abstract capabilities
- When discussing data, name the entities and their relationships

### 5. Use Web Search Proactively
You have access to web search. Use it when:
- User discusses technology choices (verify current best practices, latest versions)
- You need to confirm a library is maintained and compatible
- User asks about a specific tool or framework
- Current best practices would strengthen a recommendation

When you search, mention it briefly: "[Searching: best Rust HTTP client for Tauri 2.0...]"

## Conversation Flow

**Early (turns 1-3): Discovery**
- Understand what they want to build and why
- Ask about platform, users, core feature
- One question at a time — let them talk

**Mid (turns 4-8): Decisions**
- Tech stack with specific choices
- Data model and persistence
- Core user flows step-by-step
- Push back on vague answers
- Summarize decisions as you go: "So far we've decided: [list]. Next up: [topic]."

**Late (turns 8+): Convergence**
- Fill remaining gaps
- Challenge anything that seems contradictory
- Summarize all decisions before generation
- Flag what's missing

## Readiness Tracking

Internally track which planning topics have been covered:

**Must-haves (warn if missing before generation):**
- Problem statement / why this exists
- Core user flow (step-by-step)
- Tech stack with rationale
- Data model / persistence strategy
- Scope boundaries (what's NOT included)

**Should-haves (note if missing):**
- Error handling approach
- Hard design decisions / trade-offs
- Testing strategy
- Security considerations
- Performance requirements (only if app type warrants it)

When the user triggers generation (says "generate", "I'm ready", "forge it", etc.):

1. Assess coverage
2. If must-haves are missing, respond with:
   "Ready to forge. Before I do — a few gaps worth noting:
   **Not yet discussed:** [list]
   **Partially covered:** [list with what's missing]
   I can generate now with [TBD] sections, or we can fill the gaps first. Your call."
3. If everything is covered: "Looking solid — we've covered [summary]. Generating your documents now."
4. Never block generation. The user can always say "generate anyway."

## Conversation Steering

If the user seems stuck or unsure what to discuss next, suggest the next uncovered topic naturally:
- "Want to talk about what data this app needs to store?"
- "We should figure out what happens when things go wrong — errors, network failures, invalid input."
- "Let's walk through the user experience step by step — what does someone see when they first open the app?"

## What You Don't Do

- Write code (that's Claude Code's job)
- Make decisions without discussion
- Over-engineer simple projects
- Ask more than 2 questions at once
- Accept vague answers without pushing for specifics
- Propagate typos or unclear terms without clarifying
- Rush to architecture before understanding the problem"##;

const EXPORT_FILE_ORDER: &[&str] = &[
    "START_HERE.md",
    "README.md",
    "SPEC.md",
    "CLAUDE.md",
    "PROMPTS.md",
    "MODEL_HANDOFF.md",
    "CONVERSATION.md",
];

fn to_response<E: Into<AppError>>(err: E) -> ErrorResponse {
    err.into().to_response()
}

// ============ HEALTH & CONFIG ============

#[tauri::command(rename_all = "snake_case")]
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
    let provider_label = if config.llm.provider == "ollama" {
        "Ollama"
    } else {
        "local OpenAI-compatible runtime"
    };

    if !ollama_connected {
        errors.push(format!(
            "Cannot connect to {} at {}.",
            provider_label, config.llm.base_url
        ));
    } else if !ollama_model_available {
        if config.llm.provider == "ollama" {
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
        } else {
            errors.push(format!(
                "Model '{}' is not available from the configured runtime. Load the model in your runtime and retry.",
                config.llm.model
            ));
        }
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

#[tauri::command(rename_all = "snake_case")]
pub async fn get_config(state: State<'_, AppState>) -> Result<AppConfig, ErrorResponse> {
    Ok(state
        .config
        .lock()
        .map_err(|_| to_response(AppError::Config("Config lock poisoned".to_string())))?
        .clone())
}

#[tauri::command(rename_all = "snake_case")]
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

#[tauri::command(rename_all = "snake_case")]
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

#[tauri::command(rename_all = "snake_case")]
pub async fn get_preference(
    state: State<'_, AppState>,
    key: String,
) -> Result<Option<String>, ErrorResponse> {
    state.db.get_preference(&key).map_err(to_response)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn set_preference(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> Result<(), ErrorResponse> {
    state.db.set_preference(&key, &value).map_err(to_response)
}

// ============ MODELS ============

#[tauri::command(rename_all = "snake_case")]
pub async fn list_models(state: State<'_, AppState>) -> Result<Vec<String>, ErrorResponse> {
    let config = state
        .config
        .lock()
        .map_err(|_| to_response(AppError::Config("Config lock poisoned".to_string())))?
        .clone();
    state
        .ollama
        .list_models(&config.llm)
        .await
        .map_err(to_response)
}

#[tauri::command(rename_all = "snake_case")]
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
        .pull_model(&app, &config.llm, &model_name)
        .await
        .map_err(to_response)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn cancel_pull_model(state: State<'_, AppState>) -> Result<(), ErrorResponse> {
    state.ollama.cancel_pull();
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
pub struct DiskSpace {
    pub available_gb: f64,
    pub sufficient: bool,
}

#[tauri::command(rename_all = "snake_case")]
pub async fn check_disk_space() -> Result<DiskSpace, ErrorResponse> {
    let result = tauri::async_runtime::spawn_blocking(|| -> Result<DiskSpace, AppError> {
        #[cfg(unix)]
        {
            // Use statvfs for accurate cross-platform Unix disk space check
            use std::ffi::CString;
            use std::mem::MaybeUninit;

            let path = CString::new("/").unwrap();
            let mut stat = MaybeUninit::<libc::statvfs>::uninit();
            let ret = unsafe { libc::statvfs(path.as_ptr(), stat.as_mut_ptr()) };
            if ret == 0 {
                let stat = unsafe { stat.assume_init() };
                let available_bytes_u128 =
                    u128::from(stat.f_bavail).saturating_mul(u128::from(stat.f_frsize));
                let available_bytes = available_bytes_u128.min(u128::from(u64::MAX)) as u64;
                let available_gb = available_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
                return Ok(DiskSpace {
                    available_gb,
                    sufficient: available_gb > 20.0,
                });
            }
        }

        // Fallback: try `df` command (works on macOS/Linux, fails gracefully elsewhere)
        let output = std::process::Command::new("df").args(["-k", "/"]).output();

        let available_gb = match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let available_kb: u64 = stdout
                    .lines()
                    .nth(1)
                    .and_then(|line| line.split_whitespace().nth(3))
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                available_kb as f64 / 1_048_576.0
            }
            Err(_) => {
                // Cannot determine disk space (e.g., Windows without df)
                log::warn!("Cannot determine disk space; assuming sufficient");
                100.0
            }
        };

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

#[tauri::command(rename_all = "snake_case")]
pub async fn create_session(
    state: State<'_, AppState>,
    request: CreateSessionRequest,
) -> Result<Session, ErrorResponse> {
    if let Some(ref name) = request.name {
        if name.len() > 200 {
            return Err(to_response(AppError::Validation(
                "Session name too long (max 200 chars).".to_string(),
            )));
        }
    }
    state
        .db
        .create_session(request.name.as_deref())
        .map_err(to_response)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn list_templates() -> Result<Vec<PlanningTemplate>, ErrorResponse> {
    templates::list_templates().map_err(to_response)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn create_session_from_template(
    state: State<'_, AppState>,
    request: CreateSessionFromTemplateRequest,
) -> Result<Session, ErrorResponse> {
    let template = templates::get_template(&request.template_id).map_err(to_response)?;
    let session_name = request.name.as_deref().unwrap_or(template.name.as_str());
    let session = state
        .db
        .create_session(Some(session_name))
        .map_err(to_response)?;

    let metadata = serde_json::json!({
        "template_id": template.id,
        "template_version": template.version,
    })
    .to_string();
    state
        .db
        .save_message(
            &session.id,
            "assistant",
            &template.seed_prompt,
            Some(metadata.as_str()),
        )
        .map_err(to_response)?;

    state.db.get_session(&session.id).map_err(to_response)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn create_branch_from_message(
    state: State<'_, AppState>,
    request: CreateBranchRequest,
) -> Result<Session, ErrorResponse> {
    let source_session = state
        .db
        .get_session(&request.session_id)
        .map_err(to_response)?;
    let source_messages = state
        .db
        .get_messages(&request.session_id)
        .map_err(to_response)?;
    if source_messages.is_empty() {
        return Err(to_response(AppError::Validation(
            "Cannot branch an empty conversation.".to_string(),
        )));
    }

    let cutoff_index = match request.from_message_id.as_ref() {
        Some(message_id) => source_messages
            .iter()
            .position(|message| &message.id == message_id)
            .ok_or_else(|| {
                to_response(AppError::Validation(format!(
                    "Message '{}' was not found in this session.",
                    message_id
                )))
            })?,
        None => source_messages.len() - 1,
    };
    let copied_messages = &source_messages[..=cutoff_index];

    let default_name = request
        .name
        .unwrap_or_else(|| format!("{} (branch)", source_session.name));
    let branch_session = state
        .db
        .create_session(Some(default_name.as_str()))
        .map_err(to_response)?;
    let root_session_id = state
        .db
        .get_branch_root_session_id(&request.session_id)
        .map_err(to_response)?;
    state
        .db
        .register_branch(
            &branch_session.id,
            &root_session_id,
            &request.session_id,
            request.from_message_id.as_deref(),
        )
        .map_err(to_response)?;

    for message in copied_messages {
        if message.role == "system" {
            continue;
        }
        state
            .db
            .save_message(
                &branch_session.id,
                &message.role,
                &message.content,
                message.metadata.as_deref(),
            )
            .map_err(to_response)?;
    }

    let note_metadata = serde_json::json!({
        "branch_root_session_id": root_session_id,
        "branch_source_session_id": request.session_id,
        "branch_source_message_id": request.from_message_id,
    })
    .to_string();
    let branch_note = "Branch created. Continue this path with alternate decisions while preserving the original session.";
    state
        .db
        .save_message(
            &branch_session.id,
            "assistant",
            branch_note,
            Some(note_metadata.as_str()),
        )
        .map_err(to_response)?;

    state
        .db
        .get_session(&branch_session.id)
        .map_err(to_response)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_sessions(state: State<'_, AppState>) -> Result<Vec<Session>, ErrorResponse> {
    state.db.get_sessions().map_err(to_response)
}

#[tauri::command(rename_all = "snake_case")]
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

#[tauri::command(rename_all = "snake_case")]
pub async fn update_session(
    state: State<'_, AppState>,
    session_id: String,
    name: Option<String>,
    status: Option<String>,
) -> Result<Session, ErrorResponse> {
    if let Some(ref n) = name {
        if n.len() > 200 {
            return Err(to_response(AppError::Validation(
                "Session name too long (max 200 chars).".to_string(),
            )));
        }
    }
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

#[tauri::command(rename_all = "snake_case")]
pub async fn delete_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<(), ErrorResponse> {
    state.db.delete_session(&session_id).map_err(to_response)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn delete_sessions(
    state: State<'_, AppState>,
    session_ids: Vec<String>,
) -> Result<usize, ErrorResponse> {
    state.db.delete_sessions(&session_ids).map_err(to_response)
}

// ============ MESSAGES ============

#[tauri::command(rename_all = "snake_case")]
pub async fn get_messages(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Vec<Message>, ErrorResponse> {
    state.db.get_messages(&session_id).map_err(to_response)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn import_codebase_context(
    state: State<'_, AppState>,
    request: ImportCodebaseRequest,
) -> Result<CodebaseImportSummary, ErrorResponse> {
    let root_path = request.root_path.clone();
    let summary =
        tauri::async_runtime::spawn_blocking(move || importer::summarize_codebase(&root_path))
            .await
            .map_err(|e| {
                to_response(AppError::FileSystem {
                    path: request.root_path.clone(),
                    message: format!("Failed to import codebase: {}", e),
                })
            })?
            .map_err(to_response)?;

    let metadata = serde_json::json!({
        "import_summary": &summary,
    })
    .to_string();
    let content = format!(
        "{}\n\nImported automatically from `{}`.",
        summary.summary_markdown, summary.root_path
    );

    state
        .db
        .save_message(
            &request.session_id,
            "assistant",
            &content,
            Some(metadata.as_str()),
        )
        .map_err(to_response)?;

    Ok(summary)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn send_message(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    request: SendMessageRequest,
) -> Result<Message, ErrorResponse> {
    let session_id = request.session_id;
    let content = request.content;
    let is_retry = request.retry.unwrap_or(false);

    if content.len() > 102_400 {
        return Err(to_response(AppError::Validation(
            "Message too long (max 100 KB).".to_string(),
        )));
    }

    // Save user message (skip on retry — message already exists in DB)
    let user_msg = if is_retry {
        // Find the last user message from DB
        let messages = state.db.get_messages(&session_id).map_err(to_response)?;
        let last_user = messages
            .into_iter()
            .rev()
            .find(|m| m.role == "user")
            .ok_or_else(|| {
                to_response(AppError::Validation(
                    "No prior user message exists for retry in this session.".to_string(),
                ))
            })?;
        // Remove the old assistant response to avoid duplicates
        state
            .db
            .delete_last_assistant_message(&session_id)
            .map_err(to_response)?;
        last_user
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
        let auto_name = if content.chars().count() > 60 {
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
    let db_messages = state.db.get_messages(&session_id).map_err(to_response)?;

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
            &config.llm,
            chat_messages,
            config.llm.temperature,
            Some(config.llm.max_tokens),
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

            if let Err(e) = state.db.save_message(
                &session_id,
                "assistant",
                &response_text,
                metadata.as_deref(),
            ) {
                log::error!("Failed to save assistant message: {}", e);
            }
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

#[tauri::command(rename_all = "snake_case")]
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

#[tauri::command(rename_all = "snake_case")]
pub async fn generate_documents(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    request: GenerateDocumentsRequest,
) -> Result<Vec<GeneratedDocument>, ErrorResponse> {
    let config = state
        .config
        .lock()
        .map_err(|_| to_response(AppError::Config("Config lock poisoned".to_string())))?
        .clone();
    let target = resolve_forge_target(request.target.as_deref(), &config)?;
    let quality = analyze_plan_readiness_internal(&state, &request.session_id)?;

    if !request.force.unwrap_or(false) && !quality.missing_must_haves.is_empty() {
        return Err(to_response(AppError::Validation(format!(
            "Readiness check has missing must-haves: {}. Continue with force=true to forge anyway.",
            quality.missing_must_haves.join(", ")
        ))));
    }

    let docs = docgen::generate_all_documents(&app, &state, &request.session_id, &target)
        .await
        .map_err(to_response)?;

    let confidence = docgen::analyze_generation_confidence(&docs, Some(&quality));
    let quality_json = serde_json::to_string(&quality).ok();
    let confidence_json = serde_json::to_string(&confidence).ok();
    state
        .db
        .upsert_generation_metadata(
            &request.session_id,
            target.as_str(),
            &config.llm.provider,
            &config.llm.model,
            quality_json.as_deref(),
            confidence_json.as_deref(),
        )
        .map_err(to_response)?;

    Ok(docs)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_documents(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Vec<GeneratedDocument>, ErrorResponse> {
    state.db.get_documents(&session_id).map_err(to_response)
}

#[tauri::command(rename_all = "snake_case")]
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

#[tauri::command(rename_all = "snake_case")]
pub async fn analyze_plan_readiness(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<QualityReport, ErrorResponse> {
    analyze_plan_readiness_internal(&state, &session_id)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_planning_coverage(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<CoverageReport, ErrorResponse> {
    analyze_planning_coverage_internal(&state, &session_id)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_generation_metadata(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Option<GenerationMetadata>, ErrorResponse> {
    state
        .db
        .get_generation_metadata(&session_id)
        .map_err(to_response)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_generation_confidence(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Option<ConfidenceReport>, ErrorResponse> {
    let docs = state.db.get_documents(&session_id).map_err(to_response)?;
    if docs.is_empty() {
        return Ok(None);
    }

    let metadata = state
        .db
        .get_generation_metadata(&session_id)
        .map_err(to_response)?;

    if let Some(meta) = metadata.as_ref() {
        if let Some(conf_json) = meta.confidence_json.as_ref() {
            if let Ok(conf) = serde_json::from_str::<ConfidenceReport>(conf_json) {
                return Ok(Some(conf));
            }
        }
    }

    let quality = metadata
        .as_ref()
        .and_then(|m| m.quality_json.as_ref())
        .and_then(|q| serde_json::from_str::<QualityReport>(q).ok());

    Ok(Some(docgen::analyze_generation_confidence(
        &docs,
        quality.as_ref(),
    )))
}

// ============ EXPORT ============

#[tauri::command(rename_all = "snake_case")]
pub async fn save_to_folder(
    state: State<'_, AppState>,
    request: SaveToFolderRequest,
) -> Result<String, ErrorResponse> {
    let requested_root = std::path::PathBuf::from(&request.folder_path);
    let root_metadata = std::fs::metadata(&requested_root).map_err(|e| {
        to_response(AppError::FileSystem {
            path: request.folder_path.clone(),
            message: format!("Cannot access destination folder: {}", e),
        })
    })?;
    if !root_metadata.is_dir() {
        return Err(to_response(AppError::FileSystem {
            path: request.folder_path.clone(),
            message: "Destination must be a folder.".to_string(),
        }));
    }
    if root_metadata.permissions().readonly() {
        return Err(to_response(AppError::FileSystem {
            path: request.folder_path.clone(),
            message: "Destination folder is read-only.".to_string(),
        }));
    }

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
    let export_documents = prepare_export_documents(&documents).map_err(to_response)?;

    let session = state
        .db
        .get_session(&request.session_id)
        .map_err(to_response)?;
    let generation_meta = state
        .db
        .get_generation_metadata(&request.session_id)
        .map_err(to_response)?;
    let import_context = state
        .db
        .get_messages(&request.session_id)
        .map_err(to_response)?
        .into_iter()
        .rev()
        .find_map(|message| {
            message
                .metadata
                .as_deref()
                .and_then(extract_import_summary_from_metadata)
        });

    // Sanitize session name for folder name
    let sanitized_name = sanitize_folder_name(&session.name);
    let output_dir = requested_root.join(format!("{}-plan", sanitized_name));

    let output_path = output_dir.to_string_lossy().to_string();
    let output_path_for_thread = output_path.clone();
    let docs_for_thread = export_documents.clone();
    let output_dir_for_thread = output_dir.clone();
    let meta_for_thread = generation_meta.clone();
    let import_context_for_thread = import_context.clone();
    let session_name_for_thread = session.name.clone();
    let session_id_for_thread = request.session_id.clone();

    let write_result = tauri::async_runtime::spawn_blocking(move || -> Result<(), AppError> {
        if output_dir_for_thread.exists() {
            return Err(AppError::FolderExists(output_path_for_thread));
        }

        let staging_dir = output_dir_for_thread
            .with_extension(format!("plan_tmp_{}", uuid::Uuid::new_v4().simple()));

        std::fs::create_dir(&staging_dir).map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                AppError::FileSystem {
                    path: staging_dir.to_string_lossy().to_string(),
                    message: "Can't write to this location. Choose another folder.".to_string(),
                }
            } else {
                AppError::FileSystem {
                    path: staging_dir.to_string_lossy().to_string(),
                    message: format!("Failed to create folder: {}", e),
                }
            }
        })?;

        let write_docs_result = (|| -> Result<(), AppError> {
            for doc in &docs_for_thread {
                let staging_file_path = staging_dir.join(&doc.filename);
                let final_file_path = output_dir_for_thread.join(&doc.filename);
                std::fs::write(&staging_file_path, &doc.content).map_err(|e| {
                    if e.raw_os_error() == Some(28) {
                        AppError::FileSystem {
                            path: final_file_path.to_string_lossy().to_string(),
                            message: "Not enough disk space. Free up space and try again."
                                .to_string(),
                        }
                    } else if e.kind() == std::io::ErrorKind::PermissionDenied {
                        AppError::FileSystem {
                            path: final_file_path.to_string_lossy().to_string(),
                            message: format!(
                                "Permission denied writing {}. Choose another folder.",
                                doc.filename
                            ),
                        }
                    } else {
                        AppError::FileSystem {
                            path: final_file_path.to_string_lossy().to_string(),
                            message: format!("Failed to write {}: {}", doc.filename, e),
                        }
                    }
                })?;
            }
            Ok(())
        })();

        if let Err(err) = write_docs_result {
            let _ = std::fs::remove_dir_all(&staging_dir);
            return Err(err);
        }

        let manifest = ExportManifest {
            schema_version: 2,
            session_id: session_id_for_thread.clone(),
            session_name: session_name_for_thread.clone(),
            target: meta_for_thread
                .as_ref()
                .map(|m| m.target.clone())
                .unwrap_or_else(|| "generic".to_string()),
            provider: meta_for_thread
                .as_ref()
                .map(|m| m.provider.clone())
                .unwrap_or_else(|| "ollama".to_string()),
            model: meta_for_thread
                .as_ref()
                .map(|m| m.model.clone())
                .unwrap_or_else(|| "unknown".to_string()),
            created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            quality: meta_for_thread
                .as_ref()
                .and_then(|m| m.quality_json.as_ref())
                .and_then(|q| serde_json::from_str::<QualityReport>(q).ok()),
            confidence: meta_for_thread
                .as_ref()
                .and_then(|m| m.confidence_json.as_ref())
                .and_then(|q| serde_json::from_str::<ConfidenceReport>(q).ok()),
            import_context: import_context_for_thread.clone(),
            files: build_export_manifest_files(&docs_for_thread),
        };
        let manifest_json =
            serde_json::to_string_pretty(&manifest).map_err(|e| AppError::FileSystem {
                path: staging_dir.to_string_lossy().to_string(),
                message: format!("Failed to serialize export manifest: {}", e),
            })?;
        std::fs::write(staging_dir.join("manifest.json"), manifest_json).map_err(|e| {
            AppError::FileSystem {
                path: staging_dir
                    .join("manifest.json")
                    .to_string_lossy()
                    .to_string(),
                message: format!("Failed to write export manifest: {}", e),
            }
        })?;

        std::fs::rename(&staging_dir, &output_dir_for_thread).map_err(|e| {
            let _ = std::fs::remove_dir_all(&staging_dir);
            if e.kind() == std::io::ErrorKind::AlreadyExists || output_dir_for_thread.exists() {
                AppError::FolderExists(output_path_for_thread.clone())
            } else if e.kind() == std::io::ErrorKind::PermissionDenied {
                AppError::FileSystem {
                    path: output_dir_for_thread.to_string_lossy().to_string(),
                    message: "Can't finalize export in this location. Choose another folder."
                        .to_string(),
                }
            } else {
                AppError::FileSystem {
                    path: output_dir_for_thread.to_string_lossy().to_string(),
                    message: format!("Failed to finalize export: {}", e),
                }
            }
        })?;

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
    log::info!(
        "Saved {} documents to {}",
        export_documents.len(),
        output_path
    );

    Ok(output_path)
}

// ============ SEARCH ============

#[tauri::command(rename_all = "snake_case")]
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

fn analyze_plan_readiness_internal(
    state: &State<'_, AppState>,
    session_id: &str,
) -> Result<QualityReport, ErrorResponse> {
    let messages = state.db.get_messages(session_id).map_err(to_response)?;
    Ok(docgen::analyze_plan_readiness(&messages))
}

fn analyze_planning_coverage_internal(
    state: &State<'_, AppState>,
    session_id: &str,
) -> Result<CoverageReport, ErrorResponse> {
    let messages = state.db.get_messages(session_id).map_err(to_response)?;
    Ok(docgen::analyze_planning_coverage(&messages))
}

fn resolve_forge_target(
    target: Option<&str>,
    config: &AppConfig,
) -> Result<ForgeTarget, ErrorResponse> {
    let candidate = target.unwrap_or(config.output.default_target.as_str());
    candidate.parse::<ForgeTarget>().map_err(|e| {
        to_response(AppError::Validation(format!(
            "Invalid forge target '{}': {}",
            candidate, e
        )))
    })
}

#[derive(Debug, Clone, Serialize)]
struct ExportManifest {
    schema_version: u32,
    session_id: String,
    session_name: String,
    target: String,
    provider: String,
    model: String,
    created_at: String,
    quality: Option<QualityReport>,
    confidence: Option<ConfidenceReport>,
    import_context: Option<CodebaseImportSummary>,
    files: Vec<ExportManifestFile>,
}

#[derive(Debug, Clone, Serialize)]
struct ExportManifestFile {
    filename: String,
    bytes: usize,
    lines: usize,
    sha256: String,
}

#[derive(Debug, Clone)]
struct ExportDocument {
    filename: String,
    content: String,
}

// ============ HELPERS ============

fn prepare_export_documents(docs: &[GeneratedDocument]) -> Result<Vec<ExportDocument>, AppError> {
    docs.iter()
        .map(|doc| {
            validate_export_filename(&doc.filename)?;
            Ok(ExportDocument {
                filename: doc.filename.clone(),
                content: doc.content.clone(),
            })
        })
        .collect()
}

fn validate_export_filename(filename: &str) -> Result<(), AppError> {
    let trimmed = filename.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation(
            "Cannot export document with an empty filename.".to_string(),
        ));
    }
    let path = std::path::Path::new(trimmed);
    if path.is_absolute() || path.components().count() != 1 {
        return Err(AppError::Validation(format!(
            "Unsafe export filename '{}'. Nested or absolute paths are not allowed.",
            filename
        )));
    }
    let is_same_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value == trimmed);
    if !is_same_name {
        return Err(AppError::Validation(format!(
            "Unsafe export filename '{}'.",
            filename
        )));
    }

    Ok(())
}

fn build_export_manifest_files(docs: &[ExportDocument]) -> Vec<ExportManifestFile> {
    let mut files: Vec<ExportManifestFile> = docs
        .iter()
        .map(|doc| ExportManifestFile {
            filename: doc.filename.clone(),
            bytes: doc.content.len(),
            lines: if doc.content.is_empty() {
                0
            } else {
                doc.content.lines().count()
            },
            sha256: sha256_hex(doc.content.as_bytes()),
        })
        .collect();

    files.sort_by(|a, b| {
        let rank_a = export_file_rank(&a.filename);
        let rank_b = export_file_rank(&b.filename);
        rank_a
            .cmp(&rank_b)
            .then_with(|| a.filename.cmp(&b.filename))
    });

    files
}

fn export_file_rank(filename: &str) -> usize {
    EXPORT_FILE_ORDER
        .iter()
        .position(|known| known == &filename)
        .unwrap_or(EXPORT_FILE_ORDER.len())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    digest.iter().map(|b| format!("{:02x}", b)).collect()
}

fn extract_import_summary_from_metadata(metadata: &str) -> Option<CodebaseImportSummary> {
    let value = serde_json::from_str::<serde_json::Value>(metadata).ok()?;
    serde_json::from_value::<CodebaseImportSummary>(value.get("import_summary")?.clone()).ok()
}

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

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(filename: &str, content: &str) -> GeneratedDocument {
        GeneratedDocument {
            id: "doc-id".to_string(),
            session_id: "session-id".to_string(),
            filename: filename.to_string(),
            content: content.to_string(),
            created_at: "2026-01-01 00:00:00".to_string(),
        }
    }

    #[test]
    fn build_export_manifest_files_orders_known_documents_first() {
        let export_docs = prepare_export_documents(&[
            doc("Z_NOTES.md", "notes"),
            doc("README.md", "read me"),
            doc("START_HERE.md", "start here"),
            doc("A_CUSTOM.md", "custom"),
        ])
        .expect("export docs should validate");
        let files = build_export_manifest_files(&export_docs);

        let ordered_names: Vec<String> = files.into_iter().map(|f| f.filename).collect();
        assert_eq!(
            ordered_names,
            vec![
                "START_HERE.md".to_string(),
                "README.md".to_string(),
                "A_CUSTOM.md".to_string(),
                "Z_NOTES.md".to_string(),
            ]
        );
    }

    #[test]
    fn build_export_manifest_files_includes_hash_bytes_and_lines() {
        let export_docs = prepare_export_documents(&[doc("SPEC.md", "abc"), doc("EMPTY.md", "")])
            .expect("export docs should validate");
        let files = build_export_manifest_files(&export_docs);
        let spec = files
            .iter()
            .find(|f| f.filename == "SPEC.md")
            .expect("SPEC.md entry missing");
        assert_eq!(spec.bytes, 3);
        assert_eq!(spec.lines, 1);
        assert_eq!(
            spec.sha256,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );

        let empty = files
            .iter()
            .find(|f| f.filename == "EMPTY.md")
            .expect("EMPTY.md entry missing");
        assert_eq!(empty.bytes, 0);
        assert_eq!(empty.lines, 0);
        assert_eq!(
            empty.sha256,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn prepare_export_documents_rejects_nested_or_absolute_paths() {
        let nested = prepare_export_documents(&[doc("../escape.md", "bad")]);
        assert!(nested.is_err(), "parent traversal should be rejected");

        let absolute = prepare_export_documents(&[doc("/tmp/evil.md", "bad")]);
        assert!(absolute.is_err(), "absolute paths should be rejected");
    }

    #[test]
    fn prepare_export_documents_rejects_empty_filename() {
        let result = prepare_export_documents(&[doc("   ", "bad")]);
        assert!(result.is_err(), "blank filenames should be rejected");
    }
}
