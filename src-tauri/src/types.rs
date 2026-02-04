use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    pub metadata: Option<Value>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegenerateDocumentRequest {
    pub session_id: String,
    pub filename: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveToFolderRequest {
    pub session_id: String,
    pub folder_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentVersion {
    pub id: String,
    pub session_id: String,
    pub filename: String,
    pub version: i64,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCapability {
    pub key: String,
    pub supported: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCapabilities {
    pub providers: Vec<ProviderCapability>,
    pub default_provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageItem {
    pub key: String,
    pub label: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningReadiness {
    pub score: u8,
    pub must_haves: Vec<CoverageItem>,
    pub should_haves: Vec<CoverageItem>,
    pub unresolved_tbd: usize,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationBranch {
    pub id: String,
    pub session_id: String,
    pub name: String,
    pub base_message_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBranchRequest {
    pub session_id: String,
    pub name: String,
    pub base_message_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub prompt_seed: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoImportRequest {
    pub path: String,
    pub max_files: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoImportContext {
    pub root: String,
    pub detected_languages: Vec<String>,
    pub key_files: Vec<String>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacklogItem {
    pub title: String,
    pub body: String,
    pub labels: Vec<String>,
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

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            llm: LLMConfig {
                provider: "ollama".to_string(),
                model: "qwen3-coder:30b-a3b-q4_K_M".to_string(),
                base_url: "http://localhost:11434".to_string(),
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
            },
        }
    }
}
