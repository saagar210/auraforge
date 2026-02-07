use serde::Serialize;
use thiserror::Error;

use crate::search::SearchError;

#[derive(Error, Debug, Clone)]
pub enum ConfigError {
    #[allow(dead_code)]
    #[error("Config file not found. Creating default at {0}")]
    NotFound(String),
    #[allow(dead_code)]
    #[error("Invalid YAML syntax: {0}")]
    ParseError(String),
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Invalid value: {0}")]
    InvalidValue(String),
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Cannot connect to Ollama at {url}: {message}")]
    OllamaConnection { url: String, message: String },
    #[error("Model '{model}' not found. Run: ollama pull {model}")]
    ModelNotFound { model: String },
    #[error("LLM request failed: {0}")]
    LlmRequest(String),
    #[error("Response stream interrupted")]
    StreamInterrupted,
    #[error("Response cancelled")]
    StreamCancelled,
    #[error("Tavily API error: {0}")]
    TavilyError(String),
    #[error("Search rate limited. Daily limit reached.")]
    SearchRateLimit,
    #[error("Web search unavailable")]
    SearchUnavailable,
    #[error("Database error: {0}")]
    Database(String),
    #[error("Session not found: {0}")]
    SessionNotFound(String),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Cannot write to {path}: {message}")]
    FileSystem { path: String, message: String },
    #[error("Folder already exists: {0}")]
    FolderExists(String),
    #[error("Invalid request: {0}")]
    Validation(String),
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
    pub recoverable: bool,
    pub action: Option<String>,
}

impl AppError {
    pub fn to_response(&self) -> ErrorResponse {
        ErrorResponse {
            code: self.code().to_string(),
            message: self.to_string(),
            recoverable: self.is_recoverable(),
            action: self.suggested_action(),
        }
    }

    fn code(&self) -> &'static str {
        match self {
            AppError::OllamaConnection { .. } => "ollama_connection",
            AppError::ModelNotFound { .. } => "ollama_model_missing",
            AppError::LlmRequest(_) => "llm_request_failed",
            AppError::StreamInterrupted => "stream_interrupted",
            AppError::StreamCancelled => "stream_cancelled",
            AppError::TavilyError(_) => "tavily_error",
            AppError::SearchRateLimit => "search_rate_limited",
            AppError::SearchUnavailable => "search_unavailable",
            AppError::Database(_) => "database_error",
            AppError::SessionNotFound(_) => "session_not_found",
            AppError::Config(_) => "config_error",
            AppError::FileSystem { .. } => "filesystem_error",
            AppError::FolderExists(_) => "folder_exists",
            AppError::Validation(_) => "validation_error",
        }
    }

    fn is_recoverable(&self) -> bool {
        match self {
            AppError::Database(_)
            | AppError::FileSystem { .. }
            | AppError::SearchRateLimit
            | AppError::SearchUnavailable
            | AppError::LlmRequest(_)
            | AppError::StreamInterrupted
            | AppError::StreamCancelled => true,
            AppError::Config(_)
            | AppError::OllamaConnection { .. }
            | AppError::ModelNotFound { .. }
            | AppError::SessionNotFound(_)
            | AppError::FolderExists(_)
            | AppError::TavilyError(_)
            | AppError::Validation(_) => false,
        }
    }

    fn suggested_action(&self) -> Option<String> {
        match self {
            AppError::OllamaConnection { .. } => Some("Start Ollama and retry".to_string()),
            AppError::ModelNotFound { model } => Some(format!("ollama pull {}", model)),
            AppError::SearchRateLimit => Some("Switch to DuckDuckGo or try later".to_string()),
            AppError::FileSystem { .. } => Some("Choose another folder".to_string()),
            AppError::FolderExists(_) => Some("Choose a different folder name".to_string()),
            AppError::Validation(_) => Some("Review the request and try again".to_string()),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        AppError::Database(err.to_string())
    }
}

impl From<ConfigError> for AppError {
    fn from(err: ConfigError) -> Self {
        AppError::Config(err.to_string())
    }
}

impl From<SearchError> for AppError {
    fn from(err: SearchError) -> Self {
        match err {
            SearchError::InvalidApiKey => AppError::TavilyError("Invalid API key".to_string()),
            SearchError::RateLimited => AppError::SearchRateLimit,
            SearchError::NoResults => AppError::SearchUnavailable,
            SearchError::NetworkError(_) | SearchError::ParseError(_) => {
                AppError::SearchUnavailable
            }
        }
    }
}
