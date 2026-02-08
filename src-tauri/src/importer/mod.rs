use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::error::AppError;
use crate::types::{CodebaseImportSummary, RepoCitation};

const MAX_FILES_SCANNED: usize = 600;
const MAX_FILE_BYTES: u64 = 64 * 1024;
const MAX_TOTAL_BYTES: u64 = 6 * 1024 * 1024;
const MAX_DEPTH: usize = 8;
const MAX_SNIPPETS: usize = 20;
const MAX_SNIPPET_CHARS: usize = 280;

#[derive(Debug, Clone)]
struct SnippetEvidence {
    path: String,
    line_start: Option<usize>,
    line_end: Option<usize>,
    snippet: String,
}

const SKIP_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    "dist",
    "build",
    ".next",
    ".turbo",
    ".venv",
    "venv",
    ".idea",
    ".vscode",
];

const KEY_FILES: &[&str] = &[
    "package.json",
    "Cargo.toml",
    "pyproject.toml",
    "requirements.txt",
    "go.mod",
    "Gemfile",
    "composer.json",
    "Dockerfile",
    "docker-compose.yml",
    "README.md",
];

pub fn summarize_codebase(root_path: &str) -> Result<CodebaseImportSummary, AppError> {
    let root = PathBuf::from(root_path);
    if !root.exists() {
        return Err(AppError::FileSystem {
            path: root_path.to_string(),
            message: "Selected path does not exist.".to_string(),
        });
    }
    if !root.is_dir() {
        return Err(AppError::Validation(
            "Import path must be a directory.".to_string(),
        ));
    }

    let canonical_root = fs::canonicalize(&root).map_err(|err| AppError::FileSystem {
        path: root_path.to_string(),
        message: format!("Failed to access directory: {}", err),
    })?;

    let mut stack = vec![(canonical_root.clone(), 0usize)];
    let mut files_scanned = 0usize;
    let mut files_included = 0usize;
    let mut total_bytes_read = 0u64;
    let mut extension_counts: HashMap<String, usize> = HashMap::new();
    let mut key_files = Vec::new();
    let mut snippets = Vec::<SnippetEvidence>::new();

    while let Some((dir, depth)) = stack.pop() {
        if depth > MAX_DEPTH
            || files_scanned >= MAX_FILES_SCANNED
            || total_bytes_read >= MAX_TOTAL_BYTES
        {
            break;
        }

        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            if files_scanned >= MAX_FILES_SCANNED || total_bytes_read >= MAX_TOTAL_BYTES {
                break;
            }

            // Use DirEntry::file_type() which does NOT follow symlinks
            let ft = match entry.file_type() {
                Ok(ft) => ft,
                Err(_) => continue,
            };

            // Skip symlinks entirely to prevent traversal outside the root
            if ft.is_symlink() {
                continue;
            }

            let path = entry.path();
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            if ft.is_dir() {
                if should_skip_dir(file_name) {
                    continue;
                }
                stack.push((path, depth + 1));
                continue;
            }

            if !ft.is_file() || is_hidden(path.as_path()) {
                continue;
            }

            files_scanned += 1;
            let relative = relative_to_root(&canonical_root, &path);
            let ext = path
                .extension()
                .and_then(|value| value.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            *extension_counts.entry(ext.clone()).or_insert(0) += 1;

            if is_key_file(file_name) {
                key_files.push(relative.clone());
            }

            let metadata = match fs::metadata(&path) {
                Ok(metadata) => metadata,
                Err(_) => continue,
            };
            let capped_size = metadata.len().min(MAX_FILE_BYTES);
            if capped_size == 0 {
                continue;
            }
            if total_bytes_read + capped_size > MAX_TOTAL_BYTES {
                break;
            }

            let bytes = match read_file_prefix(&path, capped_size as usize) {
                Ok(bytes) => bytes,
                Err(_) => continue,
            };
            if bytes.iter().take(2048).any(|b| *b == 0) {
                continue;
            }

            total_bytes_read += bytes.len() as u64;
            files_included += 1;

            if snippets.len() < MAX_SNIPPETS
                && (is_key_file(file_name) || is_source_extension(&ext))
            {
                let text = String::from_utf8_lossy(&bytes);
                let lines = text.lines().take(6).collect::<Vec<_>>();
                let snippet = lines.join(" ");
                let snippet = snippet.chars().take(MAX_SNIPPET_CHARS).collect::<String>();
                if !snippet.trim().is_empty() {
                    snippets.push(SnippetEvidence {
                        path: relative.clone(),
                        line_start: Some(1),
                        line_end: Some(lines.len()),
                        snippet: snippet.trim().to_string(),
                    });
                }
            }
        }
    }

    let detected_stacks = detect_stacks(&key_files, &extension_counts);
    let summary_markdown = build_summary_markdown(
        canonical_root.as_path(),
        files_scanned,
        files_included,
        total_bytes_read,
        &detected_stacks,
        &key_files,
        &snippets,
    );
    let citations = snippets
        .iter()
        .take(10)
        .map(|snippet| RepoCitation {
            path: snippet.path.clone(),
            line_start: snippet.line_start,
            line_end: snippet.line_end,
            snippet: snippet.snippet.clone(),
        })
        .collect::<Vec<_>>();
    let architecture_summary_markdown =
        build_architecture_summary_markdown(&detected_stacks, &key_files, &citations);
    let risks_gaps_markdown = build_risks_gaps_markdown(&detected_stacks, &key_files, &citations);
    let phased_plan_markdown = build_phased_plan_markdown(&detected_stacks, &citations);
    let verification_plan_markdown = build_verification_plan_markdown(&citations);

    Ok(CodebaseImportSummary {
        root_path: canonical_root.to_string_lossy().to_string(),
        files_scanned,
        files_included,
        total_bytes_read,
        detected_stacks,
        key_files,
        summary_markdown,
        architecture_summary_markdown,
        risks_gaps_markdown,
        phased_plan_markdown,
        verification_plan_markdown,
        citations,
    })
}

fn should_skip_dir(name: &str) -> bool {
    SKIP_DIRS.contains(&name)
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.starts_with('.'))
        .unwrap_or(false)
}

fn is_key_file(file_name: &str) -> bool {
    KEY_FILES.contains(&file_name)
}

fn is_source_extension(ext: &str) -> bool {
    matches!(
        ext,
        "rs" | "ts"
            | "tsx"
            | "js"
            | "jsx"
            | "py"
            | "go"
            | "java"
            | "kt"
            | "swift"
            | "c"
            | "cpp"
            | "h"
            | "hpp"
            | "cs"
    )
}

fn relative_to_root(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|_| path.to_string_lossy().to_string())
}

fn detect_stacks(key_files: &[String], extension_counts: &HashMap<String, usize>) -> Vec<String> {
    let mut stacks = Vec::new();

    if key_files.iter().any(|path| path.ends_with("package.json")) {
        stacks.push("Node.js / JavaScript ecosystem".to_string());
    }
    if key_files.iter().any(|path| path.ends_with("Cargo.toml"))
        || extension_counts.contains_key("rs")
    {
        stacks.push("Rust".to_string());
    }
    if key_files
        .iter()
        .any(|path| path.ends_with("pyproject.toml") || path.ends_with("requirements.txt"))
        || extension_counts.contains_key("py")
    {
        stacks.push("Python".to_string());
    }
    if key_files.iter().any(|path| path.ends_with("go.mod")) || extension_counts.contains_key("go")
    {
        stacks.push("Go".to_string());
    }
    if key_files
        .iter()
        .any(|path| path.ends_with("Dockerfile") || path.ends_with("docker-compose.yml"))
    {
        stacks.push("Containerized deployment".to_string());
    }

    if stacks.is_empty() {
        stacks.push("General source repository".to_string());
    }

    stacks
}

fn build_summary_markdown(
    root: &Path,
    files_scanned: usize,
    files_included: usize,
    total_bytes_read: u64,
    detected_stacks: &[String],
    key_files: &[String],
    snippets: &[SnippetEvidence],
) -> String {
    let mut summary = String::new();

    summary.push_str("## Imported Codebase Context\n");
    summary.push_str(&format!("- Root path: `{}`\n", root.to_string_lossy()));
    summary.push_str(&format!("- Files scanned: {}\n", files_scanned));
    summary.push_str(&format!(
        "- Files included for context: {}\n",
        files_included
    ));
    summary.push_str(&format!("- Approx bytes read: {}\n", total_bytes_read));

    summary.push_str("\n### Detected stacks\n");
    for stack in detected_stacks {
        summary.push_str(&format!("- {}\n", stack));
    }

    if !key_files.is_empty() {
        summary.push_str("\n### Key files detected\n");
        for file in key_files.iter().take(20) {
            summary.push_str(&format!("- `{}`\n", file));
        }
    }

    if !snippets.is_empty() {
        summary.push_str("\n### Representative snippets\n");
        for snippet in snippets.iter().take(10) {
            summary.push_str(&format!(
                "- `{}` (L{}-L{}): {}\n",
                snippet.path,
                snippet.line_start.unwrap_or(0),
                snippet.line_end.unwrap_or(0),
                snippet.snippet
            ));
        }
    }

    summary.push_str(
        "\nUse this context for refactoring and migration planning. If information is missing, ask for specific files before making assumptions.",
    );

    summary
}

fn build_architecture_summary_markdown(
    detected_stacks: &[String],
    key_files: &[String],
    citations: &[RepoCitation],
) -> String {
    let mut out = String::from("## Architecture Summary (Grounded)\n");
    out.push_str("\n### Detected ecosystem\n");
    for stack in detected_stacks {
        out.push_str(&format!("- {}\n", stack));
    }

    out.push_str("\n### Structural evidence\n");
    if key_files.is_empty() {
        out.push_str("- [TBD] No key files detected. Need a deeper scan target.\n");
    } else {
        for file in key_files.iter().take(12) {
            out.push_str(&format!("- `{}`\n", file));
        }
    }

    out.push_str("\n### Citation samples\n");
    if citations.is_empty() {
        out.push_str("- [TBD] No readable source snippets were captured.\n");
    } else {
        for citation in citations.iter().take(6) {
            out.push_str(&format!(
                "- `{}` (L{}-L{}): {}\n",
                citation.path,
                citation.line_start.unwrap_or(0),
                citation.line_end.unwrap_or(0),
                citation.snippet
            ));
        }
    }

    out
}

fn build_risks_gaps_markdown(
    detected_stacks: &[String],
    key_files: &[String],
    citations: &[RepoCitation],
) -> String {
    let mut out = String::from("## Risks / Gaps Checklist (Grounded)\n");
    out.push_str(
        "\n- [ ] Missing test strategy evidence in repo files (confirm with maintainers).\n",
    );
    out.push_str("- [ ] Verify CI parity with local commands before major refactor.\n");
    if !detected_stacks
        .iter()
        .any(|stack| stack.contains("Containerized"))
    {
        out.push_str("- [ ] [TBD] Deployment topology unclear (no Docker evidence found).\n");
    }
    if !key_files.iter().any(|path| path.ends_with("README.md")) {
        out.push_str("- [ ] [TBD] Repository orientation docs not found at root.\n");
    }
    if citations.is_empty() {
        out.push_str("- [ ] [TBD] Need additional file evidence for risk scoring confidence.\n");
    }
    out
}

fn build_phased_plan_markdown(detected_stacks: &[String], citations: &[RepoCitation]) -> String {
    let mut out = String::from("## Phased Implementation Plan (Grounded)\n");
    out.push_str("\n1. Foundation: establish baseline checks and architecture invariants from cited files.\n");
    out.push_str(
        "2. Iteration 1: stabilize core runtime + interfaces where citations show highest churn.\n",
    );
    out.push_str(
        "3. Iteration 2: address risk hotspots and test gaps identified in cited evidence.\n",
    );
    out.push_str("4. Polish: run full verification, update docs, and lock release gates.\n");

    if detected_stacks.is_empty() || citations.is_empty() {
        out.push_str(
            "\n[TBD] Evidence coverage is limited. Expand import scope before finalizing effort estimates.\n",
        );
    }
    out
}

fn build_verification_plan_markdown(citations: &[RepoCitation]) -> String {
    let mut out = String::from("## Verification Plan (Grounded)\n");
    out.push_str("\n- [ ] Run repo-defined typecheck, tests, and build gates.\n");
    out.push_str("- [ ] Validate changes against cited files to prevent contract regressions.\n");
    out.push_str("- [ ] Re-run import and compare new summary against prior citations.\n");
    if citations.is_empty() {
        out.push_str("- [ ] [TBD] Add citation evidence before final sign-off.\n");
    }
    out
}

fn read_file_prefix(path: &Path, max_bytes: usize) -> std::io::Result<Vec<u8>> {
    let file = fs::File::open(path)?;
    let mut buffer = Vec::with_capacity(max_bytes.min(8192));
    let mut handle = file.take(max_bytes as u64);
    handle.read_to_end(&mut buffer)?;
    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn read_file_prefix_respects_max_bytes() {
        let dir = tempdir().expect("temp dir should be created");
        let file_path = dir.path().join("large.txt");
        fs::write(&file_path, vec![b'a'; 4096]).expect("test file should be written");

        let bytes = read_file_prefix(&file_path, 128).expect("prefix read should succeed");
        assert_eq!(bytes.len(), 128);
        assert!(bytes.iter().all(|value| *value == b'a'));
    }

    #[test]
    fn read_file_prefix_reads_full_small_file() {
        let dir = tempdir().expect("temp dir should be created");
        let file_path = dir.path().join("small.txt");
        fs::write(&file_path, b"hello").expect("test file should be written");

        let bytes = read_file_prefix(&file_path, 128).expect("prefix read should succeed");
        assert_eq!(bytes, b"hello");
    }

    #[cfg(unix)]
    #[test]
    fn summarize_codebase_skips_symlinks() {
        let dir = tempdir().expect("temp dir should be created");
        let root = dir.path();

        // Create a real file
        fs::write(root.join("real.rs"), "fn main() {}").unwrap();

        // Create a symlink pointing outside the root
        let outside = tempdir().expect("outside dir");
        let secret = outside.path().join("secret.txt");
        fs::write(&secret, "TOP SECRET DATA").unwrap();
        std::os::unix::fs::symlink(&secret, root.join("link.txt")).unwrap();

        let summary = summarize_codebase(root.to_str().unwrap()).unwrap();

        // The real file should be included but the symlink target should not
        assert!(
            summary.files_scanned >= 1,
            "should scan at least the real file"
        );
        assert!(
            !summary.summary_markdown.contains("TOP SECRET DATA"),
            "symlink target content should not appear in summary"
        );
    }

    #[test]
    fn summarize_codebase_emits_grounded_sections_with_citations() {
        let dir = tempdir().expect("temp dir should be created");
        let root = dir.path();

        fs::write(
            root.join("package.json"),
            r#"{"name":"fixture","version":"1.0.0"}"#,
        )
        .expect("package file should be written");
        fs::create_dir_all(root.join("src")).expect("src directory should be created");
        fs::write(
            root.join("src").join("main.ts"),
            "export function run() { return 'ok'; }",
        )
        .expect("source file should be written");

        let summary = summarize_codebase(root.to_str().expect("path should be valid utf-8"))
            .expect("summary should succeed");
        assert!(
            !summary.citations.is_empty(),
            "citations should be present for grounded summaries"
        );
        assert!(
            summary
                .architecture_summary_markdown
                .contains("## Architecture Summary (Grounded)"),
            "architecture section should be generated"
        );
        assert!(
            summary
                .architecture_summary_markdown
                .contains("package.json"),
            "key file evidence should be included"
        );
        assert!(
            summary
                .risks_gaps_markdown
                .contains("Risks / Gaps Checklist (Grounded)"),
            "risk checklist should be generated"
        );
        assert!(
            summary.phased_plan_markdown.contains("1. Foundation"),
            "phased plan should include foundation step"
        );
        assert!(
            summary
                .verification_plan_markdown
                .contains("Run repo-defined typecheck, tests, and build gates"),
            "verification section should include canonical validation guidance"
        );
    }

    #[test]
    fn summarize_codebase_marks_tbd_when_evidence_is_sparse() {
        let dir = tempdir().expect("temp dir should be created");
        let summary = summarize_codebase(dir.path().to_str().expect("path should be valid utf-8"))
            .expect("summary should succeed");
        assert!(
            summary.architecture_summary_markdown.contains("[TBD]"),
            "architecture section should mark missing evidence"
        );
        assert!(
            summary.risks_gaps_markdown.contains("[TBD]"),
            "risk section should mark uncertain findings"
        );
        assert!(
            summary.phased_plan_markdown.contains("[TBD]"),
            "phase plan should mark limited evidence"
        );
        assert!(
            summary.verification_plan_markdown.contains("[TBD]"),
            "verification plan should mark missing evidence"
        );
    }

    #[test]
    #[ignore = "manual smoke test (set AURAFORGE_INGEST_SMOKE_REPO to run)"]
    fn smoke_import_real_repo_from_env() {
        let repo_path = std::env::var("AURAFORGE_INGEST_SMOKE_REPO")
            .expect("AURAFORGE_INGEST_SMOKE_REPO must be set for smoke tests");
        let summary = summarize_codebase(&repo_path).expect("smoke import should succeed");

        assert!(summary.files_scanned > 0, "smoke import should scan files");
        assert!(
            summary.files_included > 0,
            "smoke import should include files"
        );
        assert!(
            !summary.architecture_summary_markdown.trim().is_empty(),
            "architecture summary should be present"
        );
        assert!(
            !summary.risks_gaps_markdown.trim().is_empty(),
            "risk checklist should be present"
        );
        assert!(
            !summary.phased_plan_markdown.trim().is_empty(),
            "phased plan should be present"
        );
        assert!(
            !summary.verification_plan_markdown.trim().is_empty(),
            "verification plan should be present"
        );

        let sample_citations = summary
            .citations
            .iter()
            .take(3)
            .map(|citation| {
                format!(
                    "{}:{}-{}",
                    citation.path,
                    citation.line_start.unwrap_or(0),
                    citation.line_end.unwrap_or(0)
                )
            })
            .collect::<Vec<_>>();

        println!("SMOKE_REPO={}", summary.root_path);
        println!("SMOKE_FILES_SCANNED={}", summary.files_scanned);
        println!("SMOKE_FILES_INCLUDED={}", summary.files_included);
        println!("SMOKE_BYTES_READ={}", summary.total_bytes_read);
        println!("SMOKE_STACKS={}", summary.detected_stacks.join(" | "));
        println!("SMOKE_KEY_FILES={}", summary.key_files.len());
        println!("SMOKE_CITATIONS={}", summary.citations.len());
        println!(
            "SMOKE_CITATION_SAMPLE={}",
            if sample_citations.is_empty() {
                "none".to_string()
            } else {
                sample_citations.join(", ")
            }
        );
    }
}
