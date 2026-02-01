use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;

use crate::db::Database;
use crate::llm::OllamaClient;
use crate::types::AppConfig;

pub struct AppState {
    pub db: Database,
    pub ollama: OllamaClient,
    pub config: Mutex<AppConfig>,
    pub config_error: Mutex<Option<String>>,
    pub db_error: Mutex<Option<String>>,
    pub stream_cancel: Mutex<HashMap<String, Arc<AtomicBool>>>,
}
