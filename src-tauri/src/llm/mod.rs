use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tauri::Emitter;
use tokio::time::{timeout, Duration};

use crate::error::AppError;
use crate::search::SearchResult;
use crate::types::{AppConfig, LLMConfig};

#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Debug, Deserialize)]
struct OllamaModel {
    name: String,
}

#[derive(Debug, Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
    options: OllamaOptions,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    temperature: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct OpenAiModelsResponse {
    data: Vec<OpenAiModel>,
}

#[derive(Debug, Deserialize)]
struct OpenAiModel {
    id: String,
}

#[derive(Debug, Serialize)]
struct OpenAiChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
    temperature: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct OpenAiChatResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: OpenAiChatMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAiChatMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiStreamResponse {
    choices: Vec<OpenAiStreamChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAiStreamChoice {
    delta: OpenAiStreamDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiStreamDelta {
    content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
struct OllamaStreamResponse {
    message: OllamaStreamMessage,
    done: bool,
}

#[derive(Debug, Deserialize)]
struct OllamaStreamMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    message: OllamaChatResponseMessage,
}

#[derive(Debug, Deserialize)]
struct OllamaChatResponseMessage {
    content: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct StreamChunk {
    pub r#type: String,
    pub content: Option<String>,
    pub error: Option<String>,
    pub search_query: Option<String>,
    pub search_results: Option<Vec<SearchResult>>,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelPullProgress {
    pub status: String,
    pub total: Option<u64>,
    pub completed: Option<u64>,
}

#[derive(Debug, Serialize)]
struct OllamaPullRequest {
    name: String,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct OllamaPullResponse {
    status: Option<String>,
    total: Option<u64>,
    completed: Option<u64>,
    error: Option<String>,
}

pub struct OllamaClient {
    client: Client,
    pull_cancelled: Arc<AtomicBool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProviderKind {
    Ollama,
    OpenAiCompatible,
}

impl ProviderKind {
    fn from_provider(provider: &str) -> Result<Self, AppError> {
        match provider.trim().to_ascii_lowercase().as_str() {
            "ollama" => Ok(Self::Ollama),
            "openai_compatible" | "openai-compatible" | "lmstudio" => Ok(Self::OpenAiCompatible),
            other => Err(AppError::Validation(format!(
                "Unsupported local provider '{}'",
                other
            ))),
        }
    }

    fn from_config(config: &LLMConfig) -> Result<Self, AppError> {
        Self::from_provider(&config.provider)
    }
}

impl OllamaClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .build()
            .unwrap_or_else(|_| Client::new());
        Self {
            client,
            pull_cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    fn endpoint(base_url: &str, path: &str) -> String {
        format!(
            "{}/{}",
            base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    fn with_auth(
        &self,
        request: reqwest::RequestBuilder,
        api_key: Option<&str>,
    ) -> reqwest::RequestBuilder {
        if let Some(key) = api_key.filter(|value| !value.trim().is_empty()) {
            request.bearer_auth(key.trim())
        } else {
            request
        }
    }

    pub async fn list_models(&self, config: &LLMConfig) -> Result<Vec<String>, AppError> {
        match ProviderKind::from_config(config)? {
            ProviderKind::OpenAiCompatible => self.list_models_openai(config).await,
            ProviderKind::Ollama => {
                let base_url = &config.base_url;
                let resp = self
                    .client
                    .get(Self::endpoint(base_url, "/api/tags"))
                    .timeout(std::time::Duration::from_secs(5))
                    .send()
                    .await
                    .map_err(|e| AppError::OllamaConnection {
                        url: base_url.to_string(),
                        message: e.to_string(),
                    })?;

                if !resp.status().is_success() {
                    return Err(AppError::LlmRequest(format!(
                        "Ollama returned {}",
                        resp.status()
                    )));
                }

                let tags: OllamaTagsResponse = resp.json().await.map_err(|e| {
                    AppError::LlmRequest(format!("Failed to parse Ollama response: {}", e))
                })?;

                Ok(tags.models.into_iter().map(|m| m.name).collect())
            }
        }
    }

    async fn list_models_openai(&self, config: &LLMConfig) -> Result<Vec<String>, AppError> {
        let request = self
            .client
            .get(Self::endpoint(&config.base_url, "/v1/models"))
            .timeout(Duration::from_secs(5));
        let resp = self
            .with_auth(request, config.api_key.as_deref())
            .send()
            .await
            .map_err(|e| AppError::OllamaConnection {
                url: config.base_url.to_string(),
                message: e.to_string(),
            })?;

        if !resp.status().is_success() {
            return Err(AppError::LlmRequest(format!(
                "OpenAI-compatible endpoint returned {}",
                resp.status()
            )));
        }

        let body: OpenAiModelsResponse = resp.json().await.map_err(|e| {
            AppError::LlmRequest(format!(
                "Failed to parse OpenAI-compatible models response: {}",
                e
            ))
        })?;

        Ok(body.data.into_iter().map(|model| model.id).collect())
    }

    pub async fn pull_model(
        &self,
        app: &tauri::AppHandle,
        config: &LLMConfig,
        model_name: &str,
    ) -> Result<(), AppError> {
        match ProviderKind::from_config(config)? {
            ProviderKind::OpenAiCompatible => {
                let _ = app.emit(
                    "model:pull_progress",
                    ModelPullProgress {
                        status: "error: model pull not supported for this provider".to_string(),
                        total: None,
                        completed: None,
                    },
                );
                return Err(AppError::Validation(
                    "Model pull is only supported for Ollama. Load models directly in your local runtime."
                        .to_string(),
                ));
            }
            ProviderKind::Ollama => {}
        }

        let base_url = &config.base_url;
        self.pull_cancelled.store(false, Ordering::SeqCst);

        let response = self
            .client
            .post(Self::endpoint(base_url, "/api/pull"))
            .json(&OllamaPullRequest {
                name: model_name.to_string(),
                stream: true,
            })
            .timeout(Duration::from_secs(300))
            .send()
            .await
            .map_err(|e| AppError::OllamaConnection {
                url: base_url.to_string(),
                message: e.to_string(),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            if status == reqwest::StatusCode::NOT_FOUND {
                return Err(AppError::ModelNotFound {
                    model: model_name.to_string(),
                });
            }
            return Err(AppError::LlmRequest(format!(
                "Ollama returned {}: {}",
                status, body
            )));
        }

        let mut stream = response.bytes_stream();
        let mut buffer = String::new();
        let mut completed = false;

        while let Some(chunk) = timeout(Duration::from_secs(120), stream.next())
            .await
            .map_err(|_| AppError::StreamInterrupted)?
        {
            if self.pull_cancelled.load(Ordering::SeqCst) {
                let _ = app.emit(
                    "model:pull_progress",
                    ModelPullProgress {
                        status: "cancelled".to_string(),
                        total: None,
                        completed: None,
                    },
                );
                return Err(AppError::StreamCancelled);
            }

            let chunk = chunk.map_err(|_| AppError::StreamInterrupted)?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            while let Some(newline_pos) = buffer.find('\n') {
                let line = buffer[..newline_pos].trim().to_string();
                buffer = buffer[newline_pos + 1..].to_string();

                if line.is_empty() {
                    continue;
                }

                match serde_json::from_str::<OllamaPullResponse>(&line) {
                    Ok(parsed) => {
                        if let Some(ref err) = parsed.error {
                            let _ = app.emit(
                                "model:pull_progress",
                                ModelPullProgress {
                                    status: format!("error: {}", err),
                                    total: None,
                                    completed: None,
                                },
                            );
                            return Err(AppError::LlmRequest(err.clone()));
                        }

                        let status = parsed.status.unwrap_or_default();
                        let _ = app.emit(
                            "model:pull_progress",
                            ModelPullProgress {
                                status: status.clone(),
                                total: parsed.total,
                                completed: parsed.completed,
                            },
                        );

                        if status == "success" {
                            completed = true;
                            break;
                        }
                    }
                    Err(_) => continue,
                }
            }

            if completed {
                break;
            }
        }

        if !completed {
            let remaining = buffer.trim();
            if !remaining.is_empty() {
                if let Ok(parsed) = serde_json::from_str::<OllamaPullResponse>(remaining) {
                    if let Some(ref err) = parsed.error {
                        let _ = app.emit(
                            "model:pull_progress",
                            ModelPullProgress {
                                status: format!("error: {}", err),
                                total: None,
                                completed: None,
                            },
                        );
                        return Err(AppError::LlmRequest(err.clone()));
                    }
                    if let Some(status) = parsed.status {
                        let _ = app.emit(
                            "model:pull_progress",
                            ModelPullProgress {
                                status: status.clone(),
                                total: parsed.total,
                                completed: parsed.completed,
                            },
                        );
                        if status == "success" {
                            completed = true;
                        }
                    }
                }
            }
        }

        if self.pull_cancelled.load(Ordering::SeqCst) {
            let _ = app.emit(
                "model:pull_progress",
                ModelPullProgress {
                    status: "cancelled".to_string(),
                    total: None,
                    completed: None,
                },
            );
            return Err(AppError::StreamCancelled);
        }

        if completed {
            Ok(())
        } else {
            let _ = app.emit(
                "model:pull_progress",
                ModelPullProgress {
                    status: "error: stream interrupted".to_string(),
                    total: None,
                    completed: None,
                },
            );
            Err(AppError::StreamInterrupted)
        }
    }

    pub fn cancel_pull(&self) {
        self.pull_cancelled.store(true, Ordering::SeqCst);
    }

    pub async fn check_connection(&self, config: &LLMConfig) -> Result<bool, AppError> {
        match ProviderKind::from_config(config)? {
            ProviderKind::OpenAiCompatible => {
                let request = self
                    .client
                    .get(Self::endpoint(&config.base_url, "/v1/models"))
                    .timeout(std::time::Duration::from_secs(5));
                let resp = self
                    .with_auth(request, config.api_key.as_deref())
                    .send()
                    .await
                    .map_err(|e| AppError::OllamaConnection {
                        url: config.base_url.to_string(),
                        message: e.to_string(),
                    })?;
                Ok(resp.status().is_success())
            }
            ProviderKind::Ollama => {
                let resp = self
                    .client
                    .get(Self::endpoint(&config.base_url, "/api/tags"))
                    .timeout(std::time::Duration::from_secs(5))
                    .send()
                    .await
                    .map_err(|e| AppError::OllamaConnection {
                        url: config.base_url.to_string(),
                        message: e.to_string(),
                    })?;
                Ok(resp.status().is_success())
            }
        }
    }

    pub async fn check_model(&self, config: &LLMConfig, model: &str) -> Result<bool, AppError> {
        let models = self.list_models(config).await?;
        match ProviderKind::from_config(config)? {
            ProviderKind::OpenAiCompatible => Ok(models.iter().any(|candidate| candidate == model)),
            ProviderKind::Ollama => {
                let model_base = model.split(':').next().unwrap_or(model);
                Ok(models.iter().any(|candidate| {
                    candidate == model
                        || (!model.contains(':')
                            && candidate.starts_with(&format!("{}:", model_base)))
                }))
            }
        }
    }

    pub async fn health_check(&self, config: &AppConfig) -> (bool, bool) {
        let connected = self.check_connection(&config.llm).await.unwrap_or(false);

        let model_available = if connected {
            self.check_model(&config.llm, &config.llm.model)
                .await
                .unwrap_or(false)
        } else {
            false
        };

        (connected, model_available)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn stream_chat(
        &self,
        app: &tauri::AppHandle,
        config: &LLMConfig,
        messages: Vec<ChatMessage>,
        temperature: f64,
        num_predict: Option<u64>,
        session_id: &str,
        cancel: Option<Arc<AtomicBool>>,
    ) -> Result<String, AppError> {
        if ProviderKind::from_config(config)? == ProviderKind::OpenAiCompatible {
            return self
                .stream_chat_openai(
                    app,
                    config,
                    messages,
                    temperature,
                    num_predict,
                    session_id,
                    cancel,
                )
                .await;
        }

        let base_url = &config.base_url;
        let model = &config.model;
        let url = Self::endpoint(base_url, "/api/chat");

        let response = self
            .client
            .post(&url)
            .json(&OllamaChatRequest {
                model: model.to_string(),
                messages,
                stream: true,
                options: OllamaOptions {
                    temperature,
                    num_predict: num_predict.map(|n| n as i64),
                },
            })
            .timeout(std::time::Duration::from_secs(300))
            .send()
            .await
            .map_err(|e| AppError::OllamaConnection {
                url: base_url.to_string(),
                message: e.to_string(),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            if status == reqwest::StatusCode::NOT_FOUND {
                return Err(AppError::ModelNotFound {
                    model: model.to_string(),
                });
            }
            return Err(AppError::LlmRequest(format!(
                "Ollama returned {}: {}",
                status, body
            )));
        }

        let mut stream = response.bytes_stream();
        let mut full_response = String::new();
        let mut buffer = String::new();

        let mut done = false;
        while let Some(chunk) = timeout(Duration::from_secs(60), stream.next())
            .await
            .map_err(|_| AppError::StreamInterrupted)?
        {
            if let Some(flag) = &cancel {
                if flag.load(Ordering::SeqCst) {
                    let _ = app.emit(
                        "stream:done",
                        StreamChunk {
                            r#type: "done".to_string(),
                            session_id: Some(session_id.to_string()),
                            ..Default::default()
                        },
                    );
                    return Err(AppError::StreamCancelled);
                }
            }
            let chunk = chunk.map_err(|_| AppError::StreamInterrupted)?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            // Process complete lines from the buffer
            while let Some(newline_pos) = buffer.find('\n') {
                let line = buffer[..newline_pos].trim().to_string();
                buffer = buffer[newline_pos + 1..].to_string();

                if line.is_empty() {
                    continue;
                }

                match serde_json::from_str::<OllamaStreamResponse>(&line) {
                    Ok(parsed) => {
                        if !parsed.message.content.is_empty() {
                            full_response.push_str(&parsed.message.content);

                            let _ = app.emit(
                                "stream:chunk",
                                StreamChunk {
                                    r#type: "content".to_string(),
                                    content: Some(parsed.message.content),
                                    session_id: Some(session_id.to_string()),
                                    ..Default::default()
                                },
                            );
                        }

                        if parsed.done {
                            let _ = app.emit(
                                "stream:done",
                                StreamChunk {
                                    r#type: "done".to_string(),
                                    session_id: Some(session_id.to_string()),
                                    ..Default::default()
                                },
                            );
                            done = true;
                            break;
                        }
                    }
                    Err(_) => continue,
                }
            }

            if done {
                break;
            }
        }

        // Process any remaining data in the buffer
        let remaining = buffer.trim();
        if !remaining.is_empty() {
            if let Ok(parsed) = serde_json::from_str::<OllamaStreamResponse>(remaining) {
                if !parsed.message.content.is_empty() {
                    full_response.push_str(&parsed.message.content);
                    let _ = app.emit(
                        "stream:chunk",
                        StreamChunk {
                            r#type: "content".to_string(),
                            content: Some(parsed.message.content),
                            session_id: Some(session_id.to_string()),
                            ..Default::default()
                        },
                    );
                }
                if parsed.done {
                    let _ = app.emit(
                        "stream:done",
                        StreamChunk {
                            r#type: "done".to_string(),
                            session_id: Some(session_id.to_string()),
                            ..Default::default()
                        },
                    );
                    done = true;
                }
            }
        }

        if !done {
            if let Some(flag) = &cancel {
                if flag.load(Ordering::SeqCst) {
                    let _ = app.emit(
                        "stream:done",
                        StreamChunk {
                            r#type: "done".to_string(),
                            session_id: Some(session_id.to_string()),
                            ..Default::default()
                        },
                    );
                    return Err(AppError::StreamCancelled);
                }
            }
            return Err(AppError::StreamInterrupted);
        }

        Ok(full_response)
    }

    /// Non-streaming generation for document creation
    pub async fn generate(
        &self,
        config: &LLMConfig,
        messages: Vec<ChatMessage>,
        temperature: f64,
    ) -> Result<String, AppError> {
        if ProviderKind::from_config(config)? == ProviderKind::OpenAiCompatible {
            return self.generate_openai(config, messages, temperature).await;
        }

        let base_url = &config.base_url;
        let model = &config.model;
        let url = Self::endpoint(base_url, "/api/chat");

        let response = self
            .client
            .post(&url)
            .json(&OllamaChatRequest {
                model: model.to_string(),
                messages,
                stream: false,
                options: OllamaOptions {
                    temperature,
                    num_predict: None, // Use Ollama's default for doc generation
                },
            })
            .timeout(std::time::Duration::from_secs(300))
            .send()
            .await
            .map_err(|e| AppError::OllamaConnection {
                url: base_url.to_string(),
                message: e.to_string(),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            if status == reqwest::StatusCode::NOT_FOUND {
                return Err(AppError::ModelNotFound {
                    model: model.to_string(),
                });
            }
            return Err(AppError::LlmRequest(format!(
                "Ollama returned {}: {}",
                status, body
            )));
        }

        let body: OllamaChatResponse = response
            .json()
            .await
            .map_err(|e| AppError::LlmRequest(format!("Failed to parse Ollama response: {}", e)))?;

        Ok(body.message.content)
    }

    #[allow(clippy::too_many_arguments)]
    async fn stream_chat_openai(
        &self,
        app: &tauri::AppHandle,
        config: &LLMConfig,
        messages: Vec<ChatMessage>,
        temperature: f64,
        max_tokens: Option<u64>,
        session_id: &str,
        cancel: Option<Arc<AtomicBool>>,
    ) -> Result<String, AppError> {
        let request = self
            .client
            .post(Self::endpoint(&config.base_url, "/v1/chat/completions"))
            .json(&OpenAiChatRequest {
                model: config.model.clone(),
                messages,
                stream: true,
                temperature,
                max_tokens,
            })
            .timeout(Duration::from_secs(300));
        let response = self
            .with_auth(request, config.api_key.as_deref())
            .send()
            .await
            .map_err(|e| AppError::OllamaConnection {
                url: config.base_url.to_string(),
                message: e.to_string(),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            if status == reqwest::StatusCode::NOT_FOUND {
                return Err(AppError::ModelNotFound {
                    model: config.model.clone(),
                });
            }
            return Err(AppError::LlmRequest(format!(
                "OpenAI-compatible endpoint returned {}: {}",
                status, body
            )));
        }

        let mut stream = response.bytes_stream();
        let mut full_response = String::new();
        let mut buffer = String::new();
        let mut done = false;

        while let Some(chunk) = timeout(Duration::from_secs(60), stream.next())
            .await
            .map_err(|_| AppError::StreamInterrupted)?
        {
            if let Some(flag) = &cancel {
                if flag.load(Ordering::SeqCst) {
                    let _ = app.emit(
                        "stream:done",
                        StreamChunk {
                            r#type: "done".to_string(),
                            session_id: Some(session_id.to_string()),
                            ..Default::default()
                        },
                    );
                    return Err(AppError::StreamCancelled);
                }
            }

            let chunk = chunk.map_err(|_| AppError::StreamInterrupted)?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            while let Some(newline_pos) = buffer.find('\n') {
                let line = buffer[..newline_pos].trim().to_string();
                buffer = buffer[newline_pos + 1..].to_string();

                if line.is_empty() || line.starts_with(':') {
                    continue;
                }
                if !line.starts_with("data:") {
                    continue;
                }

                let data = line.trim_start_matches("data:").trim();
                if data == "[DONE]" {
                    let _ = app.emit(
                        "stream:done",
                        StreamChunk {
                            r#type: "done".to_string(),
                            session_id: Some(session_id.to_string()),
                            ..Default::default()
                        },
                    );
                    done = true;
                    break;
                }

                match serde_json::from_str::<OpenAiStreamResponse>(data) {
                    Ok(parsed) => {
                        for choice in parsed.choices {
                            if let Some(content) = choice.delta.content {
                                if !content.is_empty() {
                                    full_response.push_str(&content);
                                    let _ = app.emit(
                                        "stream:chunk",
                                        StreamChunk {
                                            r#type: "content".to_string(),
                                            content: Some(content),
                                            session_id: Some(session_id.to_string()),
                                            ..Default::default()
                                        },
                                    );
                                }
                            }
                            if choice.finish_reason.is_some() {
                                let _ = app.emit(
                                    "stream:done",
                                    StreamChunk {
                                        r#type: "done".to_string(),
                                        session_id: Some(session_id.to_string()),
                                        ..Default::default()
                                    },
                                );
                                done = true;
                                break;
                            }
                        }
                    }
                    Err(_) => continue,
                }
            }

            if done {
                break;
            }
        }

        if !done {
            if let Some(flag) = &cancel {
                if flag.load(Ordering::SeqCst) {
                    let _ = app.emit(
                        "stream:done",
                        StreamChunk {
                            r#type: "done".to_string(),
                            session_id: Some(session_id.to_string()),
                            ..Default::default()
                        },
                    );
                    return Err(AppError::StreamCancelled);
                }
            }
            return Err(AppError::StreamInterrupted);
        }

        Ok(full_response)
    }

    async fn generate_openai(
        &self,
        config: &LLMConfig,
        messages: Vec<ChatMessage>,
        temperature: f64,
    ) -> Result<String, AppError> {
        let request = self
            .client
            .post(Self::endpoint(&config.base_url, "/v1/chat/completions"))
            .json(&OpenAiChatRequest {
                model: config.model.clone(),
                messages,
                stream: false,
                temperature,
                max_tokens: None,
            })
            .timeout(Duration::from_secs(300));
        let response = self
            .with_auth(request, config.api_key.as_deref())
            .send()
            .await
            .map_err(|e| AppError::OllamaConnection {
                url: config.base_url.to_string(),
                message: e.to_string(),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            if status == reqwest::StatusCode::NOT_FOUND {
                return Err(AppError::ModelNotFound {
                    model: config.model.clone(),
                });
            }
            return Err(AppError::LlmRequest(format!(
                "OpenAI-compatible endpoint returned {}: {}",
                status, body
            )));
        }

        let body: OpenAiChatResponse = response.json().await.map_err(|e| {
            AppError::LlmRequest(format!("Failed to parse OpenAI-compatible response: {}", e))
        })?;
        let content = body
            .choices
            .into_iter()
            .next()
            .map(|choice| choice.message.content)
            .unwrap_or_default();

        if content.is_empty() {
            return Err(AppError::LlmRequest(
                "OpenAI-compatible endpoint returned an empty response".to_string(),
            ));
        }

        Ok(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_kind_accepts_supported_aliases() {
        assert_eq!(
            ProviderKind::from_provider("ollama").expect("ollama should parse"),
            ProviderKind::Ollama
        );
        assert_eq!(
            ProviderKind::from_provider("openai_compatible")
                .expect("openai_compatible should parse"),
            ProviderKind::OpenAiCompatible
        );
        assert_eq!(
            ProviderKind::from_provider("openai-compatible")
                .expect("openai-compatible should parse"),
            ProviderKind::OpenAiCompatible
        );
        assert_eq!(
            ProviderKind::from_provider("LMStudio").expect("lmstudio alias should parse"),
            ProviderKind::OpenAiCompatible
        );
    }

    #[test]
    fn provider_kind_rejects_unknown_provider() {
        let err = ProviderKind::from_provider("remote_cloud")
            .expect_err("unknown provider should return validation error");
        assert!(matches!(err, AppError::Validation(_)));
    }
}
