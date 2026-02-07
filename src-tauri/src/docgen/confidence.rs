use std::collections::HashMap;

use crate::types::{ConfidenceFactor, ConfidenceReport, GeneratedDocument, QualityReport};

const REQUIRED_DOCS: &[&str] = &[
    "START_HERE.md",
    "SPEC.md",
    "CLAUDE.md",
    "PROMPTS.md",
    "README.md",
    "MODEL_HANDOFF.md",
];

pub fn analyze_generation_confidence(
    docs: &[GeneratedDocument],
    readiness: Option<&QualityReport>,
) -> ConfidenceReport {
    let by_name: HashMap<&str, &GeneratedDocument> = docs
        .iter()
        .map(|doc| (doc.filename.as_str(), doc))
        .collect();

    let mut factors = Vec::new();
    let mut blocking_gaps = Vec::new();
    let mut total_points = 0u16;
    let mut max_points = 0u16;

    // Factor 1: required document set presence.
    let mut present = 0u8;
    for name in REQUIRED_DOCS {
        if by_name.contains_key(name) {
            present += 1;
        } else {
            blocking_gaps.push(format!("Missing required document: {}", name));
        }
    }
    let required_factor = factor_linear(
        "Required document coverage",
        30,
        present as u16,
        REQUIRED_DOCS.len() as u16,
        format!(
            "{} of {} required docs generated",
            present,
            REQUIRED_DOCS.len()
        ),
    );
    add_factor(
        &mut factors,
        &mut total_points,
        &mut max_points,
        required_factor,
    );

    // Factor 2: heading sanity in key files.
    let heading_checks = [
        ("SPEC.md", vec!["# ", "## "]),
        ("PROMPTS.md", vec!["## Phase", "### Verification Checklist"]),
        ("CLAUDE.md", vec!["# ", "## Commands"]),
        ("START_HERE.md", vec!["# ", "## Step-by-Step Setup"]),
    ];
    let mut passed = 0u16;
    let mut total_checks = 0u16;
    for (name, checks) in heading_checks {
        if let Some(doc) = by_name.get(name) {
            for marker in checks {
                total_checks += 1;
                if doc.content.contains(marker) {
                    passed += 1;
                } else {
                    blocking_gaps.push(format!(
                        "{} missing expected section marker '{}'",
                        name, marker
                    ));
                }
            }
        }
    }
    let heading_factor = factor_linear(
        "Document structure sanity",
        25,
        passed,
        total_checks.max(1),
        format!(
            "{} of {} heading/marker checks passed",
            passed, total_checks
        ),
    );
    add_factor(
        &mut factors,
        &mut total_points,
        &mut max_points,
        heading_factor,
    );

    // Factor 3: unresolved TBD density in core docs.
    let mut tbd_count = 0usize;
    let mut total_chars = 0usize;
    for name in ["SPEC.md", "PROMPTS.md", "README.md"] {
        if let Some(doc) = by_name.get(name) {
            tbd_count += doc.content.matches("[TBD").count();
            total_chars += doc.content.len();
        }
    }
    let tbd_density = if total_chars == 0 {
        1.0
    } else {
        tbd_count as f64 / total_chars as f64
    };
    let tbd_points = if tbd_density <= 0.0005 {
        20
    } else if tbd_density <= 0.001 {
        15
    } else if tbd_density <= 0.002 {
        10
    } else if tbd_density <= 0.004 {
        5
    } else {
        0
    };
    add_factor(
        &mut factors,
        &mut total_points,
        &mut max_points,
        ConfidenceFactor {
            name: "Unresolved TBD density".to_string(),
            max_points: 20,
            points: tbd_points,
            detail: format!("{} TBD markers across core docs", tbd_count),
        },
    );

    // Factor 4: readiness carry-over.
    let readiness_points = readiness
        .map(|report| ((report.score as f64 / 100.0) * 25.0).round() as u8)
        .unwrap_or(10);
    let readiness_detail = readiness
        .map(|report| format!("Readiness score {} carried into confidence", report.score))
        .unwrap_or_else(|| "Readiness unavailable; partial default applied".to_string());
    add_factor(
        &mut factors,
        &mut total_points,
        &mut max_points,
        ConfidenceFactor {
            name: "Planning readiness carry-over".to_string(),
            max_points: 25,
            points: readiness_points,
            detail: readiness_detail,
        },
    );

    let mut score = if max_points == 0 {
        0
    } else {
        ((total_points as f64 / max_points as f64) * 100.0).round() as u8
    };
    if !blocking_gaps.is_empty() && score > 89 {
        score = 89;
    }

    let summary = if blocking_gaps.is_empty() {
        if score >= 85 {
            "High confidence: execution pack looks complete and internally consistent.".to_string()
        } else if score >= 70 {
            "Medium confidence: pack is usable, but some structure/detail gaps remain.".to_string()
        } else {
            "Low confidence: pack likely needs more clarification before implementation."
                .to_string()
        }
    } else {
        format!(
            "Confidence limited by {} blocking gap(s) in required output.",
            blocking_gaps.len()
        )
    };

    ConfidenceReport {
        score,
        factors,
        blocking_gaps,
        summary,
    }
}

fn factor_linear(
    name: &str,
    max_points: u8,
    passed: u16,
    total: u16,
    detail: String,
) -> ConfidenceFactor {
    let points = if total == 0 {
        0
    } else {
        ((passed as f64 / total as f64) * max_points as f64).round() as u8
    };
    ConfidenceFactor {
        name: name.to_string(),
        max_points,
        points,
        detail,
    }
}

fn add_factor(
    factors: &mut Vec<ConfidenceFactor>,
    total_points: &mut u16,
    max_points: &mut u16,
    factor: ConfidenceFactor,
) {
    *total_points += factor.points as u16;
    *max_points += factor.max_points as u16;
    factors.push(factor);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::QualityReport;

    fn doc(name: &str, content: &str) -> GeneratedDocument {
        GeneratedDocument {
            id: "d1".to_string(),
            session_id: "s1".to_string(),
            filename: name.to_string(),
            content: content.to_string(),
            created_at: "2026-01-01 00:00:00".to_string(),
        }
    }

    #[test]
    fn confidence_drops_when_required_docs_missing() {
        let report = analyze_generation_confidence(
            &[
                doc("SPEC.md", "# Spec"),
                doc("PROMPTS.md", "## Phase 1"),
                doc("README.md", "# Readme"),
            ],
            None,
        );
        assert!(report.score < 90);
        assert!(!report.blocking_gaps.is_empty());
    }

    #[test]
    fn complete_pack_scores_higher() {
        let readiness = QualityReport {
            score: 92,
            missing_must_haves: vec![],
            missing_should_haves: vec![],
            summary: "good".to_string(),
        };
        let report = analyze_generation_confidence(
            &[
                doc(
                    "START_HERE.md",
                    "# Start Here\n## Step-by-Step Setup\nno tbd here",
                ),
                doc("SPEC.md", "# Spec\n## Design"),
                doc("CLAUDE.md", "# Claude\n## Commands"),
                doc(
                    "PROMPTS.md",
                    "# Prompts\n## Phase 1\n### Verification Checklist",
                ),
                doc("README.md", "# Readme"),
                doc("MODEL_HANDOFF.md", "# Handoff"),
            ],
            Some(&readiness),
        );
        assert!(report.blocking_gaps.is_empty());
        assert!(report.score >= 80);
    }
}
