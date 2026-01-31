use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tauri::Emitter;

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
}

pub struct OllamaClient {
    client: Client,
}

impl OllamaClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn check_connection(&self, base_url: &str) -> Result<bool, String> {
        match self
            .client
            .get(format!("{}/api/tags", base_url))
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
        {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    pub async fn check_model(&self, base_url: &str, model: &str) -> Result<bool, String> {
        let resp = self
            .client
            .get(format!("{}/api/tags", base_url))
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| format!("Failed to connect to Ollama: {}", e))?;

        if !resp.status().is_success() {
            return Ok(false);
        }

        let tags: OllamaTagsResponse = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

        Ok(tags.models.iter().any(|m| {
            m.name == model
                || m.name
                    .starts_with(&format!("{}:", model.split(':').next().unwrap_or(model)))
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

    pub async fn stream_chat(
        &self,
        app: &tauri::AppHandle,
        base_url: &str,
        model: &str,
        messages: Vec<ChatMessage>,
        temperature: f64,
    ) -> Result<String, String> {
        let url = format!("{}/api/chat", base_url);

        let response = self
            .client
            .post(&url)
            .json(&OllamaChatRequest {
                model: model.to_string(),
                messages,
                stream: true,
                options: OllamaOptions { temperature },
            })
            .timeout(std::time::Duration::from_secs(300))
            .send()
            .await
            .map_err(|e| format!("Failed to connect to Ollama: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Ollama returned {}: {}", status, body));
        }

        let mut stream = response.bytes_stream();
        let mut full_response = String::new();
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| format!("Stream error: {}", e))?;
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
                                    ..Default::default()
                                },
                            );
                        }

                        if parsed.done {
                            let _ = app.emit("stream:done", ());
                        }
                    }
                    Err(_) => continue,
                }
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
    ) -> Result<String, String> {
        let url = format!("{}/api/chat", base_url);

        let response = self
            .client
            .post(&url)
            .json(&OllamaChatRequest {
                model: model.to_string(),
                messages,
                stream: false,
                options: OllamaOptions { temperature },
            })
            .timeout(std::time::Duration::from_secs(300))
            .send()
            .await
            .map_err(|e| format!("Failed to connect to Ollama: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Ollama returned {}: {}", status, body));
        }

        let body: OllamaChatResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

        Ok(body.message.content)
    }
}
