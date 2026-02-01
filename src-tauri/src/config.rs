use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::error::ConfigError;
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
  provider: duckduckgo                      # tavily | duckduckgo | searxng | none
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
    if let Some(home) = dirs::home_dir() {
        home.join(".auraforge")
    } else {
        log::warn!("Home directory not found; using temp directory for AuraForge");
        std::env::temp_dir().join("auraforge")
    }
}

pub fn config_path() -> PathBuf {
    auraforge_dir().join("config.yaml")
}

pub fn db_path() -> PathBuf {
    auraforge_dir().join("auraforge.db")
}

pub fn load_or_create_config() -> (AppConfig, Option<String>) {
    let path = config_path();

    if !path.exists() {
        // Create default config
        if let Err(e) = fs::create_dir_all(auraforge_dir()) {
            return (
                AppConfig::default(),
                Some(format!("Failed to create config dir: {}", e)),
            );
        }
        if let Err(e) = fs::write(&path, DEFAULT_CONFIG_YAML) {
            return (
                AppConfig::default(),
                Some(format!("Failed to write default config: {}", e)),
            );
        }
        log::info!("Created default config at {}", path.display());
    }

    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            return (
                AppConfig::default(),
                Some(format!("Failed to read config: {}", e)),
            );
        }
    };

    match serde_yaml::from_str::<AppConfig>(&content) {
        Ok(config) => {
            if let Err(e) = validate_config(&config) {
                return (AppConfig::default(), Some(e.to_string()));
            }
            (config, None)
        }
        Err(e) => {
            log::warn!(
                "Config file is invalid ({}), backing up and recreating with defaults",
                e
            );
            // Back up the broken config
            let backup = path.with_extension("yaml.bak");
            let _ = fs::rename(&path, &backup);
            // Write fresh defaults
            if let Err(e) = fs::write(&path, DEFAULT_CONFIG_YAML) {
                return (
                    AppConfig::default(),
                    Some(format!("Failed to write default config: {}", e)),
                );
            }
            match serde_yaml::from_str(DEFAULT_CONFIG_YAML) {
                Ok(config) => (config, Some(format!("Config parse error: {}", e))),
                Err(e) => (
                    AppConfig::default(),
                    Some(format!("Default config is invalid: {}", e)),
                ),
            }
        }
    }
}

pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let path = config_path();
    validate_config(config).map_err(|e| e.to_string())?;
    let yaml =
        serde_yaml::to_string(config).map_err(|e| format!("Failed to serialize config: {}", e))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create config dir: {}", e))?;
    }
    let tmp_path = path.with_extension("yaml.tmp");
    let mut file =
        fs::File::create(&tmp_path).map_err(|e| format!("Failed to write config: {}", e))?;
    file.write_all(yaml.as_bytes())
        .map_err(|e| format!("Failed to write config: {}", e))?;
    file.sync_all()
        .map_err(|e| format!("Failed to sync config: {}", e))?;
    fs::rename(&tmp_path, &path).map_err(|e| format!("Failed to write config: {}", e))?;
    Ok(())
}

fn validate_config(config: &AppConfig) -> Result<(), ConfigError> {
    let llm_provider = config.llm.provider.as_str();
    if !["ollama", "anthropic", "openai"].contains(&llm_provider) {
        return Err(ConfigError::InvalidValue(format!(
            "llm.provider={}",
            config.llm.provider
        )));
    }

    if config.llm.model.trim().is_empty() {
        return Err(ConfigError::MissingField("llm.model".to_string()));
    }

    if !(0.0..=2.0).contains(&config.llm.temperature) {
        return Err(ConfigError::InvalidValue(format!(
            "llm.temperature={} (must be 0.0-2.0)",
            config.llm.temperature
        )));
    }

    if config.llm.base_url.trim().is_empty() {
        return Err(ConfigError::MissingField("llm.base_url".to_string()));
    }
    if let Err(e) = url::Url::parse(&config.llm.base_url) {
        return Err(ConfigError::InvalidValue(format!(
            "llm.base_url: {}",
            e
        )));
    }

    let search_provider = config.search.provider.as_str();
    if !["tavily", "duckduckgo", "searxng", "none"].contains(&search_provider) {
        return Err(ConfigError::InvalidValue(format!(
            "search.provider={}",
            config.search.provider
        )));
    }

    if config.search.enabled && search_provider == "tavily" && config.search.tavily_api_key.is_empty()
    {
        return Err(ConfigError::MissingField(
            "search.tavily_api_key".to_string(),
        ));
    }

    if config.search.enabled && search_provider == "searxng" && config.search.searxng_url.is_empty()
    {
        return Err(ConfigError::MissingField(
            "search.searxng_url".to_string(),
        ));
    }
    if config.search.enabled
        && search_provider == "searxng"
        && !config.search.searxng_url.is_empty()
    {
        if let Err(e) = url::Url::parse(&config.search.searxng_url) {
            return Err(ConfigError::InvalidValue(format!(
                "search.searxng_url: {}",
                e
            )));
        }
    }

    if config.output.default_save_path.trim().is_empty() {
        return Err(ConfigError::MissingField(
            "output.default_save_path".to_string(),
        ));
    }

    Ok(())
}
