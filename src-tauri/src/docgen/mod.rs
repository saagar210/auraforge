mod confidence;
mod prompts;
mod quality;

use tauri::Emitter;

use crate::error::AppError;
use crate::llm::ChatMessage;
use crate::state::AppState;
use crate::types::{
    ForgeTarget, GenerateComplete, GenerateProgress, GeneratedDocument, Message, QualityReport,
    Session,
};

pub use confidence::analyze_generation_confidence;
use prompts::*;
pub use quality::{analyze_plan_readiness, analyze_planning_coverage};

pub async fn generate_all_documents(
    app: &tauri::AppHandle,
    state: &AppState,
    session_id: &str,
    target: &ForgeTarget,
) -> Result<Vec<GeneratedDocument>, AppError> {
    let messages = state.db.get_messages(session_id).map_err(AppError::from)?;

    let user_msgs = messages.iter().any(|m| m.role == "user");
    if !user_msgs {
        return Err(AppError::Validation(
            "Cannot generate documents from an empty conversation".to_string(),
        ));
    }

    let session = state.db.get_session(session_id).map_err(AppError::from)?;

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

    let total = doc_configs.len() + if include_conversation { 2 } else { 1 };

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
            .generate(&config.llm, llm_messages, 0.4) // Lower temperature for structured output
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
                .generate(&config.llm, retry_messages, 0.3)
                .await?;
        }

        drafts.push((filename.to_string(), content));
    }

    // CONVERSATION.md — generated from data, not LLM (optional)
    if include_conversation {
        let conversation_step = total - 1;
        let _ = app.emit(
            "generate:progress",
            GenerateProgress {
                current: conversation_step,
                total,
                filename: "CONVERSATION.md".to_string(),
                session_id: session_id.to_string(),
            },
        );

        let conversation_md = generate_conversation_md(&session, &messages);
        drafts.push(("CONVERSATION.md".to_string(), conversation_md));
    }

    // MODEL_HANDOFF.md — target-aware handoff instructions.
    let handoff_step = total;
    let _ = app.emit(
        "generate:progress",
        GenerateProgress {
            current: handoff_step,
            total,
            filename: "MODEL_HANDOFF.md".to_string(),
            session_id: session_id.to_string(),
        },
    );
    let quality = analyze_plan_readiness(&messages);
    drafts.push((
        "MODEL_HANDOFF.md".to_string(),
        generate_model_handoff_doc(&session, target, &quality),
    ));

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

fn generate_model_handoff_doc(
    session: &Session,
    target: &ForgeTarget,
    quality: &QualityReport,
) -> String {
    let target_name = match target {
        ForgeTarget::Claude => "Claude Code",
        ForgeTarget::Codex => "OpenAI Codex",
        ForgeTarget::Cursor => "Cursor Agent",
        ForgeTarget::Gemini => "Gemini CLI/Agent",
        ForgeTarget::Generic => "Any Coding Model",
    };

    let mut output = format!(
        "# Model Handoff ({})\n\n\
         This execution pack was forged for **{}** and can be adapted for other coding agents.\n\n\
         ## Session\n\n\
         - Project: **{}**\n\
         - Created: {}\n\
         - Planning score: **{}/100**\n\n\
         ## Use This Order\n\n\
         1. Read `START_HERE.md`\n\
         2. Read `SPEC.md`\n\
         3. Read `PROMPTS.md`\n\
         4. Read `CLAUDE.md` for repo conventions (applies broadly even for non-Claude targets)\n\n",
        target.as_str(),
        target_name,
        session.name,
        session.updated_at,
        quality.score
    );

    if !quality.missing_must_haves.is_empty() {
        output.push_str("## Missing Must-Haves\n\n");
        for item in &quality.missing_must_haves {
            output.push_str(&format!("- {}\n", item));
        }
        output.push('\n');
    }

    if !quality.missing_should_haves.is_empty() {
        output.push_str("## Missing Should-Haves\n\n");
        for item in &quality.missing_should_haves {
            output.push_str(&format!("- {}\n", item));
        }
        output.push('\n');
    }

    output.push_str("## Target-Specific Prompt Header\n\n");
    output.push_str(match target {
        ForgeTarget::Claude => {
            "Use `PROMPTS.md` phases directly in Claude Code, keeping checks after each phase.\n"
        }
        ForgeTarget::Codex => {
            "Ask Codex to execute one phase at a time from `PROMPTS.md`, always running verification commands before moving to the next phase.\n"
        }
        ForgeTarget::Cursor => {
            "Use Cursor Agent with one phase at a time, then apply and verify before continuing.\n"
        }
        ForgeTarget::Gemini => {
            "Use Gemini with explicit phase boundaries and require command output summaries after each phase.\n"
        }
        ForgeTarget::Generic => {
            "Use any coding model by enforcing phase-by-phase execution from `PROMPTS.md` with validation gates between phases.\n"
        }
    });

    output.push_str(
        "\n## Reliability Rules\n\n\
         - Do not skip tests/checks listed in this plan.\n\
         - Do not rewrite architecture unless required by a failing constraint.\n\
         - Keep commits small and scoped to one logical fix/change.\n",
    );

    output
}
