use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub metadata: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub session_id: String,
    pub content: String,
    pub retry: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub ollama_connected: bool,
    pub ollama_model_available: bool,
    pub database_ok: bool,
    pub config_valid: bool,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub llm: LLMConfig,
    pub search: SearchConfig,
    pub ui: UIConfig,
    pub output: OutputConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    pub provider: String,
    pub model: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub temperature: f64,
    pub max_tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    pub enabled: bool,
    pub provider: String,
    pub tavily_api_key: String,
    pub searxng_url: String,
    pub proactive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIConfig {
    pub theme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub include_conversation: bool,
    pub default_save_path: String,
    pub default_target: String,
    #[serde(default = "default_lint_mode")]
    pub lint_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedDocument {
    pub id: String,
    pub session_id: String,
    pub filename: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateDocumentsRequest {
    pub session_id: String,
    pub target: Option<String>,
    pub force: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveToFolderRequest {
    pub session_id: String,
    pub folder_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportCodebaseRequest {
    pub session_id: String,
    pub root_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionFromTemplateRequest {
    pub template_id: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBranchRequest {
    pub session_id: String,
    pub from_message_id: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ForgeTarget {
    Claude,
    Codex,
    Cursor,
    Gemini,
    Generic,
}

impl ForgeTarget {
    pub fn as_str(&self) -> &'static str {
        match self {
            ForgeTarget::Claude => "claude",
            ForgeTarget::Codex => "codex",
            ForgeTarget::Cursor => "cursor",
            ForgeTarget::Gemini => "gemini",
            ForgeTarget::Generic => "generic",
        }
    }
}

impl fmt::Display for ForgeTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for ForgeTarget {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "claude" => Ok(ForgeTarget::Claude),
            "codex" => Ok(ForgeTarget::Codex),
            "cursor" => Ok(ForgeTarget::Cursor),
            "gemini" => Ok(ForgeTarget::Gemini),
            "generic" => Ok(ForgeTarget::Generic),
            other => Err(format!("Unsupported forge target: {}", other)),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GenerateProgress {
    pub current: usize,
    pub total: usize,
    pub filename: String,
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GenerateComplete {
    pub session_id: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityReport {
    pub score: u8,
    pub missing_must_haves: Vec<String>,
    pub missing_should_haves: Vec<String>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CoverageStatus {
    Missing,
    Partial,
    Covered,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageTopic {
    pub topic: String,
    pub status: CoverageStatus,
    pub evidence_message_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    pub must_have: Vec<CoverageTopic>,
    pub should_have: Vec<CoverageTopic>,
    pub missing_must_haves: usize,
    pub missing_should_haves: usize,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceFactor {
    pub name: String,
    pub max_points: u8,
    pub points: u8,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceReport {
    pub score: u8,
    pub factors: Vec<ConfidenceFactor>,
    pub blocking_gaps: Vec<String>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationMetadata {
    pub session_id: String,
    pub target: String,
    pub provider: String,
    pub model: String,
    pub run_id: Option<String>,
    pub quality_json: Option<String>,
    pub confidence_json: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationRunRecord {
    pub run_id: String,
    pub session_id: String,
    pub target: String,
    pub provider: String,
    pub model: String,
    pub input_fingerprint: String,
    pub lint_summary_json: Option<String>,
    pub diff_summary_json: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationRunArtifact {
    pub run_id: String,
    pub filename: String,
    pub bytes: usize,
    pub lines: usize,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub target_stack: String,
    pub version: u8,
    pub recommended_target: Option<String>,
    pub required_sections: Option<Vec<String>>,
    pub verification_focus: Option<Vec<String>>,
    pub seed_prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoCitation {
    pub path: String,
    pub line_start: Option<usize>,
    pub line_end: Option<usize>,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodebaseImportSummary {
    pub root_path: String,
    pub files_scanned: usize,
    pub files_included: usize,
    pub total_bytes_read: u64,
    pub detected_stacks: Vec<String>,
    pub key_files: Vec<String>,
    pub summary_markdown: String,
    #[serde(default)]
    pub architecture_summary_markdown: String,
    #[serde(default)]
    pub risks_gaps_markdown: String,
    #[serde(default)]
    pub phased_plan_markdown: String,
    #[serde(default)]
    pub verification_plan_markdown: String,
    #[serde(default)]
    pub citations: Vec<RepoCitation>,
}

fn default_lint_mode() -> String {
    "fail_on_critical".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            llm: LLMConfig {
                provider: "ollama".to_string(),
                model: "qwen3-coder".to_string(),
                base_url: "http://localhost:11434".to_string(),
                api_key: None,
                temperature: 0.7,
                max_tokens: 65536,
            },
            search: SearchConfig {
                enabled: true,
                provider: "duckduckgo".to_string(),
                tavily_api_key: String::new(),
                searxng_url: String::new(),
                proactive: true,
            },
            ui: UIConfig {
                theme: "dark".to_string(),
            },
            output: OutputConfig {
                include_conversation: true,
                default_save_path: "~/Projects".to_string(),
                default_target: "generic".to_string(),
                lint_mode: "fail_on_critical".to_string(),
            },
        }
    }
}
