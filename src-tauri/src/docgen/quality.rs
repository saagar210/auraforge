use std::collections::HashSet;

use crate::types::{CoverageReport, CoverageStatus, CoverageTopic, Message, QualityReport};

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
    let coverage = analyze_planning_coverage(messages);
    let missing_must_haves = coverage
        .must_have
        .iter()
        .filter(|topic| topic.status == CoverageStatus::Missing)
        .map(|topic| topic.topic.clone())
        .collect::<Vec<_>>();
    let missing_should_haves = coverage
        .should_have
        .iter()
        .filter(|topic| topic.status == CoverageStatus::Missing)
        .map(|topic| topic.topic.clone())
        .collect::<Vec<_>>();

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

pub fn analyze_planning_coverage(messages: &[Message]) -> CoverageReport {
    let non_system_messages = messages
        .iter()
        .filter(|message| message.role != "system")
        .collect::<Vec<_>>();

    let must_have = evaluate_topics(MUST_HAVE_TOPICS, &non_system_messages);
    let should_have = evaluate_topics(SHOULD_HAVE_TOPICS, &non_system_messages);
    let missing_must_haves = must_have
        .iter()
        .filter(|topic| topic.status == CoverageStatus::Missing)
        .count();
    let missing_should_haves = should_have
        .iter()
        .filter(|topic| topic.status == CoverageStatus::Missing)
        .count();

    let summary = if missing_must_haves == 0 && missing_should_haves == 0 {
        "Coverage is complete across must-have and should-have planning topics.".to_string()
    } else if missing_must_haves == 0 {
        format!(
            "Must-have coverage is complete. {} should-have topic(s) are still thin.",
            missing_should_haves
        )
    } else {
        format!(
            "{} must-have topic(s) still need clarification before high-confidence forge.",
            missing_must_haves
        )
    };

    CoverageReport {
        must_have,
        should_have,
        missing_must_haves,
        missing_should_haves,
        summary,
    }
}

fn evaluate_topics(topics: &[(&str, &[&str])], messages: &[&Message]) -> Vec<CoverageTopic> {
    topics
        .iter()
        .map(|(topic, keywords)| {
            let mut evidence_message_ids = Vec::new();
            let mut matched_keywords = HashSet::new();

            for message in messages {
                let content = message.content.to_ascii_lowercase();
                let mut matched_this_message = false;

                for keyword in *keywords {
                    if content.contains(keyword) {
                        matched_keywords.insert(*keyword);
                        matched_this_message = true;
                    }
                }

                if matched_this_message && evidence_message_ids.len() < 4 {
                    evidence_message_ids.push(message.id.clone());
                }
            }

            let status = if matched_keywords.is_empty() {
                CoverageStatus::Missing
            } else if matched_keywords.len() >= 2 && evidence_message_ids.len() >= 2 {
                CoverageStatus::Covered
            } else {
                CoverageStatus::Partial
            };

            CoverageTopic {
                topic: (*topic).to_string(),
                status,
                evidence_message_ids,
            }
        })
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

    #[test]
    fn planning_coverage_marks_partial_when_single_mention() {
        let coverage = analyze_planning_coverage(&[message(
            "user",
            "The problem is onboarding friction and our goal is to ship quickly.",
        )]);
        let topic = coverage
            .must_have
            .iter()
            .find(|topic| topic.topic == "Problem statement / why this exists")
            .expect("topic should exist");
        assert_eq!(topic.status, CoverageStatus::Partial);
        assert_eq!(coverage.missing_must_haves, 4);
    }

    #[test]
    fn empty_conversation_has_zero_score_and_all_missing() {
        let report = analyze_plan_readiness(&[]);
        assert_eq!(report.score, 0);
        assert_eq!(report.missing_must_haves.len(), 5);
        assert!(!report.missing_should_haves.is_empty());
    }

    #[test]
    fn planning_coverage_marks_covered_with_multiple_evidence() {
        let coverage = analyze_planning_coverage(&[
            message(
                "user",
                "The core user flow starts with sign in, then workflow setup.",
            ),
            message(
                "assistant",
                "Great, this step-by-step user journey is clear with each screen.",
            ),
        ]);
        let topic = coverage
            .must_have
            .iter()
            .find(|topic| topic.topic == "Core user flow (step-by-step)")
            .expect("topic should exist");
        assert_eq!(topic.status, CoverageStatus::Covered);
        assert!(!topic.evidence_message_ids.is_empty());
    }
}
