use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tauri::Emitter;
use tokio::time::{timeout, Duration};

use crate::error::AppError;
use crate::search::SearchResult;
use crate::types::AppConfig;

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

    pub async fn list_models(&self, base_url: &str) -> Result<Vec<String>, AppError> {
        let resp = self
            .client
            .get(format!("{}/api/tags", base_url))
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

        let tags: OllamaTagsResponse = resp
            .json()
            .await
            .map_err(|e| AppError::LlmRequest(format!("Failed to parse Ollama response: {}", e)))?;

        Ok(tags.models.into_iter().map(|m| m.name).collect())
    }

    pub async fn pull_model(
        &self,
        app: &tauri::AppHandle,
        base_url: &str,
        model_name: &str,
    ) -> Result<(), AppError> {
        self.pull_cancelled.store(false, Ordering::SeqCst);

        let response = self
            .client
            .post(format!("{}/api/pull", base_url))
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
                            return Ok(());
                        }
                    }
                    Err(_) => continue,
                }
            }
        }

        Ok(())
    }

    pub fn cancel_pull(&self) {
        self.pull_cancelled.store(true, Ordering::SeqCst);
    }

    pub async fn check_connection(&self, base_url: &str) -> Result<bool, AppError> {
        let resp = self
            .client
            .get(format!("{}/api/tags", base_url))
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| AppError::OllamaConnection {
                url: base_url.to_string(),
                message: e.to_string(),
            })?;
        Ok(resp.status().is_success())
    }

    pub async fn check_model(&self, base_url: &str, model: &str) -> Result<bool, AppError> {
        let resp = self
            .client
            .get(format!("{}/api/tags", base_url))
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| AppError::OllamaConnection {
                url: base_url.to_string(),
                message: e.to_string(),
            })?;

        if !resp.status().is_success() {
            return Ok(false);
        }

        let tags: OllamaTagsResponse = resp
            .json()
            .await
            .map_err(|e| AppError::LlmRequest(format!("Failed to parse Ollama response: {}", e)))?;

        let model_base = model.split(':').next().unwrap_or(model);
        Ok(tags.models.iter().any(|m| {
            m.name == model
                || (!model.contains(':')
                    && m.name.starts_with(&format!("{}:", model_base)))
        }))
    }

    pub async fn health_check(&self, config: &AppConfig) -> (bool, bool) {
        let connected = self
            .check_connection(&config.llm.base_url)
            .await
            .unwrap_or(false);

        let model_available = if connected {
            self.check_model(&config.llm.base_url, &config.llm.model)
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
        base_url: &str,
        model: &str,
        messages: Vec<ChatMessage>,
        temperature: f64,
        num_predict: Option<u64>,
        session_id: &str,
        cancel: Option<Arc<AtomicBool>>,
    ) -> Result<String, AppError> {
        let url = format!("{}/api/chat", base_url);

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
            }
        }

        Ok(full_response)
    }

    /// Non-streaming generation for document creation
    pub async fn generate(
        &self,
        base_url: &str,
        model: &str,
        messages: Vec<ChatMessage>,
        temperature: f64,
    ) -> Result<String, AppError> {
        let url = format!("{}/api/chat", base_url);

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
}
