use crate::types::{Message, QualityReport};

const MUST_HAVE_TOPICS: &[(&str, &[&str])] = &[
    (
        "Problem statement / why this exists",
        &["problem", "goal", "why", "build", "need", "pain point"],
    ),
    (
        "Core user flow (step-by-step)",
        &["flow", "workflow", "step", "screen", "journey", "user does"],
    ),
    (
        "Tech stack with rationale",
        &[
            "stack",
            "react",
            "rust",
            "database",
            "framework",
            "tauri",
            "why this",
        ],
    ),
    (
        "Data model / persistence strategy",
        &["data", "schema", "entity", "table", "persist", "storage"],
    ),
    (
        "Scope boundaries (what is out for v1)",
        &[
            "scope",
            "mvp",
            "v1",
            "out of scope",
            "not included",
            "later",
        ],
    ),
];

const SHOULD_HAVE_TOPICS: &[(&str, &[&str])] = &[
    (
        "Error handling approach",
        &["error", "failure", "retry", "fallback", "recover"],
    ),
    (
        "Design trade-offs / decisions",
        &["trade-off", "tradeoff", "decision", "chose", "alternative"],
    ),
    (
        "Testing strategy",
        &[
            "test",
            "verification",
            "qa",
            "integration test",
            "unit test",
        ],
    ),
    (
        "Security considerations",
        &["security", "auth", "permissions", "privacy", "threat"],
    ),
    (
        "Performance requirements",
        &["performance", "latency", "throughput", "memory", "optimize"],
    ),
];

pub fn analyze_plan_readiness(messages: &[Message]) -> QualityReport {
    let corpus = messages
        .iter()
        .filter(|m| m.role != "system")
        .map(|m| m.content.to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join("\n");

    let missing_must_haves = missing_topics(MUST_HAVE_TOPICS, &corpus);
    let missing_should_haves = missing_topics(SHOULD_HAVE_TOPICS, &corpus);

    let mut score = 100i32;
    score -= (missing_must_haves.len() as i32) * 14;
    score -= (missing_should_haves.len() as i32) * 6;
    score = score.clamp(0, 100);

    let summary = if missing_must_haves.is_empty() && missing_should_haves.is_empty() {
        "Planning coverage looks strong. You can forge with high confidence.".to_string()
    } else if missing_must_haves.is_empty() {
        format!(
            "Core planning coverage is good. {} optional topic(s) are still thin.",
            missing_should_haves.len()
        )
    } else {
        format!(
            "{} must-have topic(s) are missing. You can still forge, but expect [TBD] sections.",
            missing_must_haves.len()
        )
    };

    QualityReport {
        score: score as u8,
        missing_must_haves,
        missing_should_haves,
        summary,
    }
}

fn missing_topics(topics: &[(&str, &[&str])], corpus: &str) -> Vec<String> {
    topics
        .iter()
        .filter(|(_, keywords)| !keywords.iter().any(|keyword| corpus.contains(keyword)))
        .map(|(topic, _)| (*topic).to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message(role: &str, content: &str) -> Message {
        Message {
            id: "m1".to_string(),
            session_id: "s1".to_string(),
            role: role.to_string(),
            content: content.to_string(),
            metadata: None,
            created_at: "2026-02-07 00:00:00".to_string(),
        }
    }

    #[test]
    fn reports_missing_must_haves_for_short_conversations() {
        let report = analyze_plan_readiness(&[
            message("user", "I want to build an app"),
            message("assistant", "Tell me more"),
        ]);
        assert!(report.score < 90);
        assert!(!report.missing_must_haves.is_empty());
    }

    #[test]
    fn scores_higher_for_complete_coverage() {
        let report = analyze_plan_readiness(&[message(
            "user",
            "Our problem is onboarding friction. For v1 scope, out of scope is billing. \
                 Core user flow: user signs up, creates project, exports plan. \
                 Tech stack is React + Rust Tauri because of local-first needs. \
                 Data schema stores sessions/messages/documents in sqlite. \
                 Testing strategy includes unit and integration test coverage. \
                 Security and performance constraints are documented with trade-off decisions.",
        )]);
        assert!(report.score >= 90);
        assert!(report.missing_must_haves.is_empty());
    }
}
