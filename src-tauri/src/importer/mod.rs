use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::AppError;
use crate::types::CodebaseImportSummary;

const MAX_FILES_SCANNED: usize = 600;
const MAX_FILE_BYTES: u64 = 64 * 1024;
const MAX_TOTAL_BYTES: u64 = 6 * 1024 * 1024;
const MAX_DEPTH: usize = 8;
const MAX_SNIPPETS: usize = 20;
const MAX_SNIPPET_CHARS: usize = 280;

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
    let mut snippets = Vec::new();

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

            let path = entry.path();
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            if path.is_dir() {
                if should_skip_dir(file_name) {
                    continue;
                }
                stack.push((path, depth + 1));
                continue;
            }

            if !path.is_file() || is_hidden(path.as_path()) {
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

            let bytes = match fs::read(&path) {
                Ok(bytes) => bytes,
                Err(_) => continue,
            };
            if bytes.iter().take(2048).any(|b| *b == 0) {
                continue;
            }

            total_bytes_read += capped_size;
            files_included += 1;

            if snippets.len() < MAX_SNIPPETS
                && (is_key_file(file_name) || is_source_extension(&ext))
            {
                let snippet = String::from_utf8_lossy(&bytes)
                    .lines()
                    .take(6)
                    .collect::<Vec<_>>()
                    .join(" ");
                let snippet = snippet.chars().take(MAX_SNIPPET_CHARS).collect::<String>();
                if !snippet.trim().is_empty() {
                    snippets.push(format!("{}: {}", relative, snippet.trim()));
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

    Ok(CodebaseImportSummary {
        root_path: canonical_root.to_string_lossy().to_string(),
        files_scanned,
        files_included,
        total_bytes_read,
        detected_stacks,
        key_files,
        summary_markdown,
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
    snippets: &[String],
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
            summary.push_str(&format!("- {}\n", snippet));
        }
    }

    summary.push_str(
        "\nUse this context for refactoring and migration planning. If information is missing, ask for specific files before making assumptions.",
    );

    summary
}
