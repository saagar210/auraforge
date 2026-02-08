use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::types::GeneratedDocument;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactDiffStatus {
    Added,
    Removed,
    Changed,
    Unchanged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactDiffEntry {
    pub filename: String,
    pub status: ArtifactDiffStatus,
    pub lines_added: usize,
    pub lines_removed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactDiffReport {
    pub added: usize,
    pub removed: usize,
    pub changed: usize,
    pub unchanged: usize,
    pub entries: Vec<ArtifactDiffEntry>,
}

pub fn build_diff_report(
    previous: &[GeneratedDocument],
    current: &[GeneratedDocument],
) -> ArtifactDiffReport {
    let mut prev_map = BTreeMap::new();
    let mut curr_map = BTreeMap::new();

    for doc in previous {
        prev_map.insert(doc.filename.clone(), doc.content.clone());
    }
    for doc in current {
        curr_map.insert(doc.filename.clone(), doc.content.clone());
    }

    let mut filenames = prev_map
        .keys()
        .cloned()
        .chain(curr_map.keys().cloned())
        .collect::<Vec<_>>();
    filenames.sort();
    filenames.dedup();

    let mut entries = Vec::new();

    for filename in filenames {
        match (prev_map.get(&filename), curr_map.get(&filename)) {
            (None, Some(new_content)) => {
                entries.push(ArtifactDiffEntry {
                    filename,
                    status: ArtifactDiffStatus::Added,
                    lines_added: new_content.lines().count(),
                    lines_removed: 0,
                });
            }
            (Some(old_content), None) => {
                entries.push(ArtifactDiffEntry {
                    filename,
                    status: ArtifactDiffStatus::Removed,
                    lines_added: 0,
                    lines_removed: old_content.lines().count(),
                });
            }
            (Some(old_content), Some(new_content)) => {
                if old_content == new_content {
                    entries.push(ArtifactDiffEntry {
                        filename,
                        status: ArtifactDiffStatus::Unchanged,
                        lines_added: 0,
                        lines_removed: 0,
                    });
                } else {
                    let (added, removed) = line_delta(old_content, new_content);
                    entries.push(ArtifactDiffEntry {
                        filename,
                        status: ArtifactDiffStatus::Changed,
                        lines_added: added,
                        lines_removed: removed,
                    });
                }
            }
            (None, None) => {}
        }
    }

    let added = entries
        .iter()
        .filter(|entry| entry.status == ArtifactDiffStatus::Added)
        .count();
    let removed = entries
        .iter()
        .filter(|entry| entry.status == ArtifactDiffStatus::Removed)
        .count();
    let changed = entries
        .iter()
        .filter(|entry| entry.status == ArtifactDiffStatus::Changed)
        .count();
    let unchanged = entries
        .iter()
        .filter(|entry| entry.status == ArtifactDiffStatus::Unchanged)
        .count();

    ArtifactDiffReport {
        added,
        removed,
        changed,
        unchanged,
        entries,
    }
}

pub fn render_changelog_markdown(report: &ArtifactDiffReport) -> String {
    let mut out = format!(
        "# Artifact Changelog\n\n## Summary\n\n- Added files: {}\n- Removed files: {}\n- Changed files: {}\n- Unchanged files: {}\n\n",
        report.added, report.removed, report.changed, report.unchanged
    );

    if report.entries.is_empty() {
        out.push_str("No prior run available for diffing.\n");
        return out;
    }

    out.push_str("## File-level changes\n\n");
    for entry in &report.entries {
        out.push_str(&format!(
            "- `{}`: `{:?}` (+{} / -{})\n",
            entry.filename, entry.status, entry.lines_added, entry.lines_removed
        ));
    }

    out
}

fn line_delta(old_content: &str, new_content: &str) -> (usize, usize) {
    let old_lines = old_content.lines().collect::<Vec<_>>();
    let new_lines = new_content.lines().collect::<Vec<_>>();

    let mut added = 0usize;
    for line in &new_lines {
        if !old_lines.contains(line) {
            added += 1;
        }
    }

    let mut removed = 0usize;
    for line in &old_lines {
        if !new_lines.contains(line) {
            removed += 1;
        }
    }

    (added, removed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(name: &str, content: &str) -> GeneratedDocument {
        GeneratedDocument {
            id: "id".to_string(),
            session_id: "session".to_string(),
            filename: name.to_string(),
            content: content.to_string(),
            created_at: "2026-01-01 00:00:00".to_string(),
        }
    }

    #[test]
    fn diff_detects_added_removed_changed_unchanged() {
        let prev = vec![doc("SPEC.md", "line1\nline2"), doc("README.md", "same")];
        let curr = vec![
            doc("SPEC.md", "line1\nline3"),
            doc("README.md", "same"),
            doc("PROMPTS.md", "new"),
        ];

        let report = build_diff_report(&prev, &curr);
        assert_eq!(report.added, 1);
        assert_eq!(report.removed, 0);
        assert_eq!(report.changed, 1);
        assert_eq!(report.unchanged, 1);
    }
}
