use chrono::Local;
use serde::{Deserialize, Serialize};

use crate::types::GeneratedDocument;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LintSeverity {
    Critical,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintFinding {
    pub rule_id: String,
    pub severity: LintSeverity,
    pub filename: String,
    pub title: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LintSummary {
    pub critical: usize,
    pub warning: usize,
    pub info: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintReport {
    pub generated_at: String,
    pub summary: LintSummary,
    pub findings: Vec<LintFinding>,
}

impl LintReport {
    pub fn has_critical(&self) -> bool {
        self.summary.critical > 0
    }
}

pub fn lint_documents(docs: &[GeneratedDocument]) -> LintReport {
    let mut findings = Vec::new();

    findings.extend(rule_tbd_leftovers(docs));
    findings.extend(rule_missing_acceptance_criteria(docs));
    findings.extend(rule_inconsistent_project_naming(docs));
    findings.extend(rule_vague_requirements(docs));
    findings.extend(rule_missing_verification_steps(docs));

    let mut summary = LintSummary::default();
    for finding in &findings {
        match finding.severity {
            LintSeverity::Critical => summary.critical += 1,
            LintSeverity::Warning => summary.warning += 1,
            LintSeverity::Info => summary.info += 1,
        }
    }

    LintReport {
        generated_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        summary,
        findings,
    }
}

pub fn render_lint_report_markdown(report: &LintReport) -> String {
    let mut out = format!(
        "# Lint Report\n\nGenerated: {}\n\n## Summary\n\n- Critical: {}\n- Warning: {}\n- Info: {}\n\n",
        report.generated_at, report.summary.critical, report.summary.warning, report.summary.info
    );

    if report.findings.is_empty() {
        out.push_str("No lint findings.\n");
        return out;
    }

    out.push_str("## Findings\n\n");
    for (idx, finding) in report.findings.iter().enumerate() {
        out.push_str(&format!(
            "{}. **[{:?}] {}** (`{}` / `{}`)\n   - {}\n",
            idx + 1,
            finding.severity,
            finding.title,
            finding.rule_id,
            finding.filename,
            finding.detail
        ));
    }

    out
}

fn rule_tbd_leftovers(docs: &[GeneratedDocument]) -> Vec<LintFinding> {
    let mut findings = Vec::new();

    for doc in docs {
        let tbd_count = doc.content.matches("[TBD").count();
        if tbd_count == 0 {
            continue;
        }

        let severity = if ["SPEC.md", "PROMPTS.md", "MODEL_HANDOFF.md", "START_HERE.md"]
            .contains(&doc.filename.as_str())
        {
            LintSeverity::Critical
        } else {
            LintSeverity::Warning
        };

        findings.push(LintFinding {
            rule_id: "tbd_leftover".to_string(),
            severity,
            filename: doc.filename.clone(),
            title: "Unresolved TBD marker".to_string(),
            detail: format!(
                "Found {} `[TBD ...]` marker(s). Resolve them or explicitly defer with evidence.",
                tbd_count
            ),
        });
    }

    findings
}

fn rule_missing_acceptance_criteria(docs: &[GeneratedDocument]) -> Vec<LintFinding> {
    let mut findings = Vec::new();

    if let Some(spec) = docs.iter().find(|doc| doc.filename == "SPEC.md") {
        let lower = spec.content.to_ascii_lowercase();
        let has_feature_section = lower.contains("## features") || lower.contains("### features");
        let has_acceptance = lower.contains("acceptance criteria");

        if has_feature_section && !has_acceptance {
            findings.push(LintFinding {
                rule_id: "missing_acceptance_criteria".to_string(),
                severity: LintSeverity::Critical,
                filename: spec.filename.clone(),
                title: "Missing acceptance criteria".to_string(),
                detail:
                    "SPEC has feature sections but no explicit acceptance criteria. Add testable outcomes."
                        .to_string(),
            });
        }
    }

    findings
}

fn rule_inconsistent_project_naming(docs: &[GeneratedDocument]) -> Vec<LintFinding> {
    let mut findings = Vec::new();
    let mut names = Vec::<(String, String)>::new();

    for doc in docs {
        if let Some(line) = doc.content.lines().find(|line| line.starts_with("# ")) {
            let heading = line.trim_start_matches("# ").trim().to_string();
            if !heading.is_empty() {
                names.push((doc.filename.clone(), heading));
            }
        }
    }

    if names.len() < 2 {
        return findings;
    }

    let mut canonical = names[0].1.clone();
    if let Some((_, first_non_empty)) = names
        .iter()
        .find(|(_, heading)| !heading.trim().is_empty())
        .cloned()
    {
        canonical = first_non_empty;
    }

    for (filename, heading) in names.into_iter().skip(1) {
        if normalize_name(&heading) != normalize_name(&canonical) {
            findings.push(LintFinding {
                rule_id: "inconsistent_project_naming".to_string(),
                severity: LintSeverity::Warning,
                filename,
                title: "Inconsistent project naming".to_string(),
                detail: format!(
                    "Heading `{}` differs from canonical heading `{}`.",
                    heading, canonical
                ),
            });
        }
    }

    findings
}

fn normalize_name(input: &str) -> String {
    input
        .to_ascii_lowercase()
        .replace(['-', '_'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn rule_vague_requirements(docs: &[GeneratedDocument]) -> Vec<LintFinding> {
    let mut findings = Vec::new();
    let vague_terms = [
        "user-friendly",
        "robust",
        "scalable",
        "fast",
        "intuitive",
        "as needed",
        "etc.",
    ];

    for doc in docs {
        if !["SPEC.md", "PROMPTS.md", "START_HERE.md"].contains(&doc.filename.as_str()) {
            continue;
        }

        let lower = doc.content.to_ascii_lowercase();
        let mut matched = Vec::new();
        for term in vague_terms {
            if lower.contains(term) {
                matched.push(term);
            }
        }

        if !matched.is_empty() {
            findings.push(LintFinding {
                rule_id: "vague_requirements".to_string(),
                severity: LintSeverity::Warning,
                filename: doc.filename.clone(),
                title: "Vague requirement language".to_string(),
                detail: format!(
                    "Found vague term(s): {}. Replace with measurable, verifiable wording.",
                    matched.join(", ")
                ),
            });
        }
    }

    findings
}

fn rule_missing_verification_steps(docs: &[GeneratedDocument]) -> Vec<LintFinding> {
    let mut findings = Vec::new();

    let required = ["PROMPTS.md", "START_HERE.md", "MODEL_HANDOFF.md"];
    for filename in required {
        let Some(doc) = docs.iter().find(|doc| doc.filename == filename) else {
            findings.push(LintFinding {
                rule_id: "missing_verification_steps".to_string(),
                severity: LintSeverity::Critical,
                filename: filename.to_string(),
                title: "Missing document for verification".to_string(),
                detail: "Required execution document was not generated.".to_string(),
            });
            continue;
        };

        let lower = doc.content.to_ascii_lowercase();
        let has_verification = lower.contains("verification") || lower.contains("checklist");
        let has_checkbox = doc.content.contains("- [ ]");
        if !has_verification || !has_checkbox {
            findings.push(LintFinding {
                rule_id: "missing_verification_steps".to_string(),
                severity: LintSeverity::Critical,
                filename: doc.filename.clone(),
                title: "Missing concrete verification steps".to_string(),
                detail:
                    "Document should include explicit verification/checklist steps with checkboxes."
                        .to_string(),
            });
        }
    }

    findings
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
    fn lint_flags_tbd_leftovers() {
        let report = lint_documents(&[doc("SPEC.md", "# Spec\n[TBD - fill later]")]);
        assert!(report.summary.critical > 0);
        assert!(report.findings.iter().any(|f| f.rule_id == "tbd_leftover"));
    }

    #[test]
    fn lint_flags_missing_acceptance_criteria() {
        let report = lint_documents(&[doc(
            "SPEC.md",
            "# Spec\n## Features\n### Login\nDescription only",
        )]);
        assert!(report
            .findings
            .iter()
            .any(|f| f.rule_id == "missing_acceptance_criteria"));
    }

    #[test]
    fn lint_flags_missing_verification_docs() {
        let report = lint_documents(&[doc("SPEC.md", "# Spec")]);
        let missing = report
            .findings
            .iter()
            .filter(|f| f.rule_id == "missing_verification_steps")
            .count();
        assert!(missing >= 3);
    }

    #[test]
    fn lint_passes_core_when_verification_present() {
        let report = lint_documents(&[
            doc("SPEC.md", "# Project\n## Features\nAcceptance Criteria"),
            doc(
                "PROMPTS.md",
                "# Project\n## Verification Checklist\n- [ ] run tests",
            ),
            doc(
                "START_HERE.md",
                "# Project\n## Verification\n- [ ] verify setup",
            ),
            doc(
                "MODEL_HANDOFF.md",
                "# Project\n## Verification\n- [ ] phase checks",
            ),
        ]);
        assert_eq!(report.summary.critical, 0);
    }
}
