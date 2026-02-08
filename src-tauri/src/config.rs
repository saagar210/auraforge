use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::error::ConfigError;
use crate::types::AppConfig;

const DEFAULT_CONFIG_YAML: &str = r#"# AuraForge Configuration

# LLM Provider Settings
llm:
  provider: ollama                          # ollama | openai_compatible
  model: qwen3-coder
  base_url: http://localhost:11434          # Ollama default (LM Studio commonly uses :1234)
  api_key: ""                               # optional for openai_compatible runtimes
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
  default_target: generic                   # claude | codex | cursor | gemini | generic
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
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            let _ = fs::set_permissions(&path, perms);
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
        Ok(mut config) => {
            let normalized = normalize_local_model_config(&mut config);
            if let Err(e) = validate_config(&config) {
                return (AppConfig::default(), Some(e.to_string()));
            }
            if normalized {
                if let Err(err) = save_config(&config) {
                    log::warn!(
                        "Failed to persist normalized local-model config defaults: {}",
                        err
                    );
                }
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
    write_config_atomically(&path, yaml.as_bytes())
}

fn write_config_atomically(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("Config path has no parent: {}", path.display()))?;
    fs::create_dir_all(parent).map_err(|e| format!("Failed to create config dir: {}", e))?;

    let tmp_path = path.with_extension("yaml.tmp");
    let mut file =
        fs::File::create(&tmp_path).map_err(|e| format!("Failed to write config: {}", e))?;
    file.write_all(bytes)
        .map_err(|e| format!("Failed to write config: {}", e))?;
    file.sync_all()
        .map_err(|e| format!("Failed to sync config: {}", e))?;
    drop(file);

    if let Err(e) = fs::rename(&tmp_path, path) {
        let _ = fs::remove_file(&tmp_path);
        return Err(format!("Failed to write config: {}", e));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        if let Err(e) = fs::set_permissions(path, perms) {
            log::warn!("Failed to set config file permissions: {}", e);
        }
    }

    sync_directory(parent)?;
    Ok(())
}

fn sync_directory(path: &Path) -> Result<(), String> {
    let dir = fs::File::open(path).map_err(|e| {
        format!(
            "Failed to open config dir for sync ({}): {}",
            path.display(),
            e
        )
    })?;
    dir.sync_all()
        .map_err(|e| format!("Failed to sync config dir ({}): {}", path.display(), e))
}

fn validate_config(config: &AppConfig) -> Result<(), ConfigError> {
    let llm_provider = config.llm.provider.as_str();
    if !["ollama", "openai_compatible"].contains(&llm_provider) {
        return Err(ConfigError::InvalidValue(format!(
            "llm.provider={} (expected 'ollama' or 'openai_compatible')",
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
    match url::Url::parse(&config.llm.base_url) {
        Ok(parsed) => {
            if parsed.scheme() != "http" && parsed.scheme() != "https" {
                return Err(ConfigError::InvalidValue(format!(
                    "llm.base_url: scheme '{}' is not allowed (only http/https)",
                    parsed.scheme()
                )));
            }
        }
        Err(e) => {
            return Err(ConfigError::InvalidValue(format!("llm.base_url: {}", e)));
        }
    }

    let search_provider = config.search.provider.as_str();
    if !["tavily", "duckduckgo", "searxng", "none"].contains(&search_provider) {
        return Err(ConfigError::InvalidValue(format!(
            "search.provider={}",
            config.search.provider
        )));
    }

    if config.search.enabled
        && search_provider == "tavily"
        && config.search.tavily_api_key.trim().is_empty()
    {
        return Err(ConfigError::MissingField(
            "search.tavily_api_key".to_string(),
        ));
    }

    if config.search.enabled && search_provider == "searxng" && config.search.searxng_url.is_empty()
    {
        return Err(ConfigError::MissingField("search.searxng_url".to_string()));
    }
    if config.search.enabled
        && search_provider == "searxng"
        && !config.search.searxng_url.is_empty()
    {
        match url::Url::parse(&config.search.searxng_url) {
            Ok(parsed) => {
                if parsed.scheme() != "http" && parsed.scheme() != "https" {
                    return Err(ConfigError::InvalidValue(format!(
                        "search.searxng_url: scheme '{}' is not allowed (only http/https)",
                        parsed.scheme()
                    )));
                }
            }
            Err(e) => {
                return Err(ConfigError::InvalidValue(format!(
                    "search.searxng_url: {}",
                    e
                )));
            }
        }
    }

    if config.output.default_save_path.trim().is_empty() {
        return Err(ConfigError::MissingField(
            "output.default_save_path".to_string(),
        ));
    }
    let target = config.output.default_target.as_str();
    if !["claude", "codex", "cursor", "gemini", "generic"].contains(&target) {
        return Err(ConfigError::InvalidValue(format!(
            "output.default_target={}",
            config.output.default_target
        )));
    }

    Ok(())
}

fn normalize_local_model_config(config: &mut AppConfig) -> bool {
    let mut changed = false;

    let provider = config.llm.provider.trim().to_ascii_lowercase();
    let normalized_provider = match provider.as_str() {
        "ollama" => "ollama",
        "openai_compatible" | "openai-compatible" | "lmstudio" => "openai_compatible",
        _ => "ollama",
    };
    if config.llm.provider != normalized_provider {
        config.llm.provider = normalized_provider.to_string();
        changed = true;
    }

    if config
        .llm
        .api_key
        .as_deref()
        .is_some_and(|key| key.trim().is_empty())
    {
        config.llm.api_key = None;
        changed = true;
    }

    changed
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn default_config() -> AppConfig {
        serde_yaml::from_str(DEFAULT_CONFIG_YAML).expect("default config should parse")
    }

    #[test]
    fn validate_config_accepts_http_base_url() {
        let config = default_config();
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn validate_config_rejects_file_scheme_base_url() {
        let mut config = default_config();
        config.llm.base_url = "file:///etc/passwd".to_string();
        let err = validate_config(&config);
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("not allowed"));
    }

    #[cfg(unix)]
    #[test]
    fn write_config_atomically_sets_0600_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempdir().expect("temp dir should be created");
        let path = dir.path().join("config.yaml");

        write_config_atomically(&path, b"key: value").expect("write should succeed");
        let perms = fs::metadata(&path)
            .expect("file should exist")
            .permissions();
        assert_eq!(perms.mode() & 0o777, 0o600);
    }

    #[test]
    fn write_config_atomically_creates_and_replaces_file() {
        let dir = tempdir().expect("temp dir should be created");
        let path = dir.path().join("config.yaml");

        write_config_atomically(&path, b"first: value").expect("initial write should succeed");
        let first = fs::read_to_string(&path).expect("file should be readable");
        assert_eq!(first, "first: value");

        write_config_atomically(&path, b"second: value").expect("replace write should succeed");
        let second = fs::read_to_string(&path).expect("file should be readable");
        assert_eq!(second, "second: value");

        assert!(
            !path.with_extension("yaml.tmp").exists(),
            "temporary file should not remain after successful write"
        );
    }
}
