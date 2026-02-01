mod prompts;

use tauri::Emitter;

use crate::error::AppError;
use crate::llm::ChatMessage;
use crate::state::AppState;
use crate::types::{GenerateComplete, GenerateProgress, GeneratedDocument, Message, Session};

use prompts::*;

pub async fn generate_all_documents(
    app: &tauri::AppHandle,
    state: &AppState,
    session_id: &str,
) -> Result<Vec<GeneratedDocument>, AppError> {
    let messages = state
        .db
        .get_messages(session_id)
        .map_err(AppError::from)?;

    let user_msgs = messages.iter().any(|m| m.role == "user");
    if !user_msgs {
        return Err(AppError::Unknown(
            "Cannot generate documents from an empty conversation".to_string(),
        ));
    }

    let session = state
        .db
        .get_session(session_id)
        .map_err(AppError::from)?;

    let conversation = format_conversation_for_prompt(&messages);
    let config = state
        .config
        .lock()
        .map_err(|_| AppError::Config("Config lock poisoned".to_string()))?
        .clone();

    let mut drafts: Vec<(String, String)> = Vec::new();
    let include_conversation = config.output.include_conversation;

    // Order: SPEC → CLAUDE → PROMPTS → README → START_HERE (cross-referencing order)
    let doc_configs = [
        ("SPEC.md", SPEC_PROMPT),
        ("CLAUDE.md", CLAUDE_PROMPT),
        ("PROMPTS.md", PROMPTS_PROMPT),
        ("README.md", README_PROMPT),
        ("START_HERE.md", START_HERE_PROMPT),
    ];

    let total = doc_configs.len() + if include_conversation { 1 } else { 0 };

    for (i, (filename, prompt_template)) in doc_configs.iter().enumerate() {
        // Emit progress
        let _ = app.emit(
            "generate:progress",
            GenerateProgress {
                current: i + 1,
                total,
                filename: filename.to_string(),
                session_id: session_id.to_string(),
            },
        );

        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let previously_generated = if drafts.is_empty() {
            "No documents generated yet.".to_string()
        } else {
            drafts
                .iter()
                .map(|(name, content)| format!("## {}\n\n{}", name, content))
                .collect::<Vec<_>>()
                .join("\n\n---\n\n")
        };

        let prompt = prompt_template
            .replace("{conversation_history}", &conversation)
            .replace("{current_date}", &today)
            .replace("{previously_generated_docs}", &previously_generated);

        let system_prompt = DOCGEN_SYSTEM_PROMPT.replace("{current_date}", &today);

        let llm_messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt.clone(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: prompt.clone(),
            },
        ];

        let mut content = state
            .ollama
            .generate(
                &config.llm.base_url,
                &config.llm.model,
                llm_messages,
                0.4, // Lower temperature for structured output
            )
            .await?;

        // Validate output starts with # heading — retry once if not
        if !content.trim_start().starts_with('#') {
            let retry_messages = vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt.clone(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: format!(
                        "{}\n\nIMPORTANT: Start with a # heading. Output only valid Markdown.",
                        prompt
                    ),
                },
            ];

            content = state
                .ollama
                .generate(&config.llm.base_url, &config.llm.model, retry_messages, 0.3)
                .await?;
        }

        drafts.push((filename.to_string(), content));
    }

    // CONVERSATION.md — generated from data, not LLM (optional)
    if include_conversation {
        let _ = app.emit(
            "generate:progress",
            GenerateProgress {
                current: total,
                total,
                filename: "CONVERSATION.md".to_string(),
                session_id: session_id.to_string(),
            },
        );

        let conversation_md = generate_conversation_md(&session, &messages);
        drafts.push(("CONVERSATION.md".to_string(), conversation_md));
    }

    let documents = state
        .db
        .replace_documents(session_id, &drafts)
        .map_err(AppError::from)?;

    let _ = app.emit(
        "generate:complete",
        GenerateComplete {
            session_id: session_id.to_string(),
            count: documents.len(),
        },
    );

    Ok(documents)
}

fn format_conversation_for_prompt(messages: &[Message]) -> String {
    let mut output = String::new();

    for msg in messages {
        if msg.role == "system" {
            continue;
        }

        let label = match msg.role.as_str() {
            "user" => "User",
            "assistant" => "AuraForge",
            _ => "Unknown",
        };

        output.push_str(&format!("{}: {}\n\n", label, msg.content));
    }

    output
}

fn generate_conversation_md(session: &Session, messages: &[Message]) -> String {
    let mut output = format!(
        "# {} - Planning Conversation\n\n\
         This is the complete planning conversation that generated these documents.\n\
         Kept for reference—you can revisit to understand why decisions were made.\n\n\
         ---\n\n\
         **Session started**: {}\n\n\
         ---\n\n",
        session.name, session.created_at
    );

    for message in messages {
        let role_label = match message.role.as_str() {
            "user" => "**User**",
            "assistant" => "**AuraForge**",
            "system" => continue,
            _ => "**Unknown**",
        };

        output.push_str(&format!("{}: {}\n\n", role_label, message.content));

        // Include search context if present in metadata
        if let Some(ref metadata_str) = message.metadata {
            if let Ok(meta) = serde_json::from_str::<serde_json::Value>(metadata_str) {
                if let Some(query) = meta.get("search_query").and_then(|v| v.as_str()) {
                    output.push_str(&format!("*[Searched: {}]*\n\n", query));
                }
            }
        }
    }

    output.push_str(&format!(
        "---\n\n\
         **Session ended**: {}\n",
        session.updated_at
    ));

    output
}
