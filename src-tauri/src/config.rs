use std::fs;
use std::path::PathBuf;

use crate::types::AppConfig;

const DEFAULT_CONFIG_YAML: &str = r#"# AuraForge Configuration

# LLM Provider Settings
llm:
  provider: ollama                          # ollama | anthropic | openai
  model: qwen3-coder:30b-a3b-instruct-q4_K_M
  base_url: http://localhost:11434          # Ollama default
  temperature: 0.7
  max_tokens: 65536

# Web Search Settings
search:
  enabled: true
  provider: tavily                          # tavily | duckduckgo | searxng | none
  tavily_api_key: ""                        # Required if using Tavily
  searxng_url: ""                           # Required if using SearXNG
  proactive: true                           # Auto-search during conversation

# UI Preferences
ui:
  theme: dark                               # dark | light (dark is default)

# Output Preferences
output:
  include_conversation: true                # Include CONVERSATION.md
  default_save_path: ~/Projects             # Default folder picker location
"#;

pub fn auraforge_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".auraforge")
}

pub fn config_path() -> PathBuf {
    auraforge_dir().join("config.yaml")
}

pub fn db_path() -> PathBuf {
    auraforge_dir().join("auraforge.db")
}

pub fn load_or_create_config() -> Result<AppConfig, String> {
    let path = config_path();

    if !path.exists() {
        // Create default config
        fs::create_dir_all(auraforge_dir())
            .map_err(|e| format!("Failed to create .auraforge dir: {}", e))?;
        fs::write(&path, DEFAULT_CONFIG_YAML)
            .map_err(|e| format!("Failed to write default config: {}", e))?;
        log::info!("Created default config at {}", path.display());
    }

    let content = fs::read_to_string(&path).map_err(|e| format!("Failed to read config: {}", e))?;

    match serde_yaml::from_str::<AppConfig>(&content) {
        Ok(config) => Ok(config),
        Err(e) => {
            log::warn!(
                "Config file is invalid ({}), backing up and recreating with defaults",
                e
            );
            // Back up the broken config
            let backup = path.with_extension("yaml.bak");
            let _ = fs::rename(&path, &backup);
            // Write fresh defaults
            fs::write(&path, DEFAULT_CONFIG_YAML)
                .map_err(|e| format!("Failed to write default config: {}", e))?;
            serde_yaml::from_str(DEFAULT_CONFIG_YAML)
                .map_err(|e| format!("Default config is invalid: {}", e))
        }
    }
}

pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let path = config_path();
    let yaml =
        serde_yaml::to_string(config).map_err(|e| format!("Failed to serialize config: {}", e))?;
    fs::write(&path, yaml).map_err(|e| format!("Failed to write config: {}", e))?;
    Ok(())
}
