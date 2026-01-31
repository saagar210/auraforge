use std::sync::Mutex;

use crate::db::Database;
use crate::llm::OllamaClient;
use crate::types::AppConfig;

pub struct AppState {
    pub db: Database,
    pub ollama: OllamaClient,
    pub config: Mutex<AppConfig>,
}
