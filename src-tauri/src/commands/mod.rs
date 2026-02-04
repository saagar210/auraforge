use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tauri::{Emitter, State};

use crate::config::save_config;
use crate::error::{AppError, ErrorResponse};
use crate::llm::ChatMessage;
use crate::search::{self, SearchResult};
use crate::state::AppState;
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

    let (provider_connected, model_available) = state.ollama.health_check(&config).await;

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

    if !provider_connected {
        let msg = match config.llm.provider.as_str() {
            "ollama" => format!(
                "Cannot connect to Ollama at {}. Is it running?",
                config.llm.base_url
            ),
            "openai" => "Cannot reach OpenAI or OPENAI_API_KEY is missing.".to_string(),
            "anthropic" => "Cannot reach Anthropic or ANTHROPIC_API_KEY is missing.".to_string(),
            _ => format!("Unsupported provider '{}'.", config.llm.provider),
        };
        errors.push(msg);
    } else if !model_available {
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
                "Model '{}' is unavailable for {}.",
                config.llm.model, config.llm.provider
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
        ollama_connected: provider_connected,
        ollama_model_available: model_available,
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
pub async fn get_provider_capabilities(
    state: State<'_, AppState>,
) -> Result<ProviderCapabilities, ErrorResponse> {
    let providers = ["ollama", "openai", "anthropic"]
        .iter()
        .map(|provider| match state.ollama.provider_supported(provider) {
            Ok(()) => ProviderCapability {
                key: (*provider).to_string(),
                supported: true,
                reason: None,
            },
            Err(err) => ProviderCapability {
                key: (*provider).to_string(),
                supported: false,
                reason: Some(err.to_string()),
            },
        })
        .collect();
    Ok(ProviderCapabilities {
        providers,
        default_provider: "ollama".to_string(),
    })
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
    let mut models = state
        .ollama
        .list_models_for_provider(&config.llm.provider, &config.llm.base_url)
        .await
        .map_err(to_response)?;
    if models.is_empty() {
        models.push(config.llm.model.clone());
    }
    Ok(models)
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
    if config.llm.provider != "ollama" {
        return Err(to_response(AppError::Config(
            "Model pull is only supported for Ollama provider".to_string(),
        )));
    }
    state
        .ollama
        .pull_model(&app, &config.llm.base_url, &model_name)
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
                let available_bytes = stat.f_bavail as u64 * stat.f_frsize;
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
                let available_kb = stdout
                    .lines()
                    .nth(1)
                    .and_then(|line| line.split_whitespace().nth(3))
                    .and_then(|s| s.parse::<u64>().ok());
                match available_kb {
                    Some(kb) => kb as f64 / 1_048_576.0,
                    None => {
                        log::warn!("Unable to parse df output; assuming sufficient space");
                        100.0
                    }
                }
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
    state
        .db
        .create_session(request.name.as_deref())
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

#[tauri::command(rename_all = "snake_case")]
pub async fn create_branch(
    state: State<'_, AppState>,
    request: CreateBranchRequest,
) -> Result<ConversationBranch, ErrorResponse> {
    state
        .db
        .create_branch(
            &request.session_id,
            &request.name,
            request.base_message_id.as_deref(),
        )
        .map_err(to_response)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn list_branches(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Vec<ConversationBranch>, ErrorResponse> {
    state.db.list_branches(&session_id).map_err(to_response)
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
        let messages = state.db.get_messages(&session_id).map_err(to_response)?;
        let last_user = messages
            .into_iter()
            .rev()
            .find(|m| m.role == "user")
            .ok_or_else(|| {
                to_response(AppError::Unknown(
                    "No user message found to retry".to_string(),
                ))
            })?;
        // Remove the old assistant response to avoid duplicates
        if let Err(e) = state.db.delete_last_assistant_message(&session_id) {
            log::warn!("Failed to delete old assistant message on retry: {}", e);
        }
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
            &config.llm.provider,
            &config.llm.base_url,
            &config.llm.model,
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
                    "search_timestamp": chrono::Utc::now().to_rfc3339(),
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
    crate::docgen::generate_all_documents(&app, &state, &request.session_id)
        .await
        .map_err(to_response)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn regenerate_document(
    state: State<'_, AppState>,
    request: RegenerateDocumentRequest,
) -> Result<GeneratedDocument, ErrorResponse> {
    crate::docgen::regenerate_single_document(&state, &request.session_id, &request.filename)
        .await
        .map_err(to_response)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_documents(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Vec<GeneratedDocument>, ErrorResponse> {
    state.db.get_documents(&session_id).map_err(to_response)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_document_versions(
    state: State<'_, AppState>,
    session_id: String,
    filename: String,
    limit: Option<usize>,
) -> Result<Vec<DocumentVersion>, ErrorResponse> {
    state
        .db
        .get_document_versions(&session_id, &filename, limit.unwrap_or(10))
        .map_err(to_response)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn assess_planning_readiness(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<PlanningReadiness, ErrorResponse> {
    let messages = state.db.get_messages(&session_id).map_err(to_response)?;
    let docs = state.db.get_documents(&session_id).map_err(to_response)?;

    let mut corpus = String::new();
    for msg in &messages {
        if msg.role != "system" {
            corpus.push_str(&msg.content.to_lowercase());
            corpus.push('\n');
        }
    }

    let must_have_topics = vec![
        (
            "problem",
            "Problem statement",
            vec!["problem", "pain", "why", "goal", "outcome"],
        ),
        (
            "flow",
            "Core user flow",
            vec!["flow", "steps", "user journey", "first", "then"],
        ),
        (
            "stack",
            "Tech stack",
            vec!["react", "rust", "tauri", "database", "stack", "framework"],
        ),
        (
            "data",
            "Data model",
            vec!["schema", "table", "entity", "model", "store", "persist"],
        ),
        (
            "scope",
            "Scope boundaries",
            vec!["out of scope", "not include", "v1", "defer", "later"],
        ),
    ];

    let should_have_topics = vec![
        (
            "errors",
            "Error handling",
            vec!["error", "retry", "failure", "fallback"],
        ),
        (
            "tradeoffs",
            "Trade-offs",
            vec!["trade-off", "tradeoff", "pros", "cons", "decision"],
        ),
        (
            "testing",
            "Testing strategy",
            vec!["test", "integration", "unit", "verification"],
        ),
        (
            "security",
            "Security",
            vec!["security", "auth", "permission", "privacy"],
        ),
        (
            "performance",
            "Performance",
            vec!["latency", "performance", "speed", "throughput", "memory"],
        ),
    ];

    let evaluate = |topics: Vec<(&str, &str, Vec<&str>)>| -> Vec<CoverageItem> {
        topics
            .into_iter()
            .map(|(key, label, keywords)| {
                let hit_count = keywords.iter().filter(|k| corpus.contains(**k)).count();
                let status = if hit_count >= 2 {
                    "covered"
                } else if hit_count == 1 {
                    "partial"
                } else {
                    "missing"
                };
                CoverageItem {
                    key: key.to_string(),
                    label: label.to_string(),
                    status: status.to_string(),
                }
            })
            .collect()
    };

    let must_haves = evaluate(must_have_topics);
    let should_haves = evaluate(should_have_topics);
    let unresolved_tbd = docs
        .iter()
        .map(|d| d.content.matches("[TBD").count())
        .sum::<usize>();

    let score_points = must_haves.iter().fold(0u32, |acc, item| {
        acc + match item.status.as_str() {
            "covered" => 20,
            "partial" => 10,
            _ => 0,
        }
    }) + should_haves.iter().fold(0u32, |acc, item| {
        acc + match item.status.as_str() {
            "covered" => 6,
            "partial" => 3,
            _ => 0,
        }
    });
    let score = (score_points.min(100)) as u8;

    let missing_must = must_haves.iter().filter(|i| i.status == "missing").count();
    let recommendation = if missing_must > 0 {
        "Consider filling the missing must-have topics before forging.".to_string()
    } else if unresolved_tbd > 0 {
        "Plan is mostly ready; review unresolved [TBD] items before export.".to_string()
    } else {
        "Planning coverage looks strong.".to_string()
    };

    Ok(PlanningReadiness {
        score,
        must_haves,
        should_haves,
        unresolved_tbd,
        recommendation,
    })
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

// ============ EXPORT ============

#[tauri::command(rename_all = "snake_case")]
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

#[tauri::command(rename_all = "snake_case")]
pub async fn list_plan_templates() -> Result<Vec<PlanTemplate>, ErrorResponse> {
    Ok(vec![
        PlanTemplate {
            id: "saas".to_string(),
            name: "SaaS Web App".to_string(),
            description: "Multi-tenant product with auth, billing, and usage tracking".to_string(),
            tags: vec!["web".to_string(), "saas".to_string(), "product".to_string()],
            prompt_seed: "Plan a SaaS app with user onboarding, core workflows, billing hooks, and analytics.".to_string(),
        },
        PlanTemplate {
            id: "api".to_string(),
            name: "Backend API".to_string(),
            description: "Service-first architecture with contracts, observability, and deployment".to_string(),
            tags: vec!["backend".to_string(), "api".to_string(), "service".to_string()],
            prompt_seed: "Plan an API-first backend with endpoint contracts, data model, tests, and rollout strategy.".to_string(),
        },
        PlanTemplate {
            id: "cli".to_string(),
            name: "Developer CLI".to_string(),
            description: "Command-line tool with subcommands, config, and plugin/extensibility thinking".to_string(),
            tags: vec!["cli".to_string(), "developer-tools".to_string()],
            prompt_seed: "Plan a CLI with clear subcommands, config defaults, error messages, and integration tests.".to_string(),
        },
        PlanTemplate {
            id: "agent".to_string(),
            name: "AI Agent App".to_string(),
            description: "Tool-using assistant with safety gates, memory model, and evaluation plan".to_string(),
            tags: vec!["ai".to_string(), "agent".to_string()],
            prompt_seed: "Plan an AI agent app with tool interfaces, prompt strategy, quality checks, and guardrails.".to_string(),
        },
    ])
}

#[tauri::command(rename_all = "snake_case")]
pub async fn import_repository_context(
    request: RepoImportRequest,
) -> Result<RepoImportContext, ErrorResponse> {
    let root = PathBuf::from(&request.path);
    if !root.exists() || !root.is_dir() {
        return Err(to_response(AppError::FileSystem {
            path: request.path,
            message: "Repository path does not exist or is not a directory".to_string(),
        }));
    }

    let max_files = request.max_files.unwrap_or(400);
    let mut stack = vec![root.clone()];
    let mut scanned = 0usize;
    let mut key_files = Vec::new();
    let mut languages = HashSet::new();

    while let Some(dir) = stack.pop() {
        let entries = fs::read_dir(&dir).map_err(|e| {
            to_response(AppError::FileSystem {
                path: dir.to_string_lossy().to_string(),
                message: e.to_string(),
            })
        })?;
        for entry in entries.flatten() {
            if scanned >= max_files {
                break;
            }
            let path = entry.path();
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();
            if file_name.starts_with('.') && file_name != ".env.example" {
                continue;
            }
            if path.is_dir() {
                if file_name == "node_modules" || file_name == "target" || file_name == "dist" {
                    continue;
                }
                stack.push(path);
                continue;
            }
            scanned += 1;

            if is_key_project_file(&path) {
                key_files.push(path.to_string_lossy().to_string());
            }
            if let Some(lang) = detect_language(&path) {
                languages.insert(lang.to_string());
            }
        }
        if scanned >= max_files {
            break;
        }
    }

    key_files.sort();
    let mut detected_languages = languages.into_iter().collect::<Vec<_>>();
    detected_languages.sort();

    Ok(RepoImportContext {
        root: root.to_string_lossy().to_string(),
        detected_languages: detected_languages.clone(),
        key_files: key_files.iter().take(25).cloned().collect(),
        summary: format!(
            "Scanned {} files. Detected languages: {}. Found {} key config/docs files.",
            scanned,
            if detected_languages.is_empty() {
                "unknown".to_string()
            } else {
                detected_languages.join(", ")
            },
            key_files.len()
        ),
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn build_issue_export_preview(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Vec<BacklogItem>, ErrorResponse> {
    let docs = state.db.get_documents(&session_id).map_err(to_response)?;
    let prompts_doc = docs.iter().find(|d| d.filename == "PROMPTS.md");
    let spec_doc = docs.iter().find(|d| d.filename == "SPEC.md");

    let mut items = Vec::new();
    if let Some(prompts) = prompts_doc {
        for line in prompts
            .content
            .lines()
            .filter(|l| l.starts_with("## Phase "))
        {
            let title = line.trim_start_matches('#').trim().to_string();
            items.push(BacklogItem {
                title,
                body: "Generated from PROMPTS.md phase heading. Expand acceptance criteria before publishing to GitHub/Linear.".to_string(),
                labels: vec!["planning".to_string(), "phase".to_string()],
            });
        }
    }

    if items.is_empty() {
        items.push(BacklogItem {
            title: "Project kickoff".to_string(),
            body: "No phase headings found in PROMPTS.md yet. Generate documents first, then re-run export preview.".to_string(),
            labels: vec!["planning".to_string()],
        });
    }

    if let Some(spec) = spec_doc {
        let tbd_count = spec.content.matches("[TBD").count();
        if tbd_count > 0 {
            items.push(BacklogItem {
                title: "Resolve planning TBDs".to_string(),
                body: format!(
                    "SPEC.md currently has {} unresolved [TBD] markers. Resolve these before implementation starts.",
                    tbd_count
                ),
                labels: vec!["planning".to_string(), "risk".to_string()],
            });
        }
    }

    Ok(items)
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
    if !config.search.enabled || config.search.provider == "none" {
        return Ok(vec![]);
    }
    search::execute_search(&config.search, &query)
        .await
        .map_err(to_response)
}

fn is_key_project_file(path: &Path) -> bool {
    matches!(
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default(),
        "package.json"
            | "package-lock.json"
            | "pnpm-lock.yaml"
            | "Cargo.toml"
            | "go.mod"
            | "pyproject.toml"
            | "requirements.txt"
            | "README.md"
            | "SPEC.md"
            | "CLAUDE.md"
            | "PROMPTS.md"
            | "tauri.conf.json"
    )
}

fn detect_language(path: &Path) -> Option<&'static str> {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
    {
        "ts" | "tsx" | "js" | "jsx" => Some("TypeScript/JavaScript"),
        "rs" => Some("Rust"),
        "py" => Some("Python"),
        "go" => Some("Go"),
        "java" => Some("Java"),
        "kt" => Some("Kotlin"),
        "swift" => Some("Swift"),
        _ => None,
    }
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
