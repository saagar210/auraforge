mod duckduckgo;
mod searxng;
mod tavily;
mod trigger;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use reqwest::Client;
use std::sync::OnceLock;
use std::time::Duration;

use crate::types::SearchConfig;

pub use trigger::should_search;

fn search_client() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| Client::new())
    })
}

#[derive(Debug, Error)]
pub enum SearchError {
    #[error("Invalid API key")]
    InvalidApiKey,
    #[error("Rate limited")]
    RateLimited,
    #[error("No results found")]
    NoResults,
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub score: f64,
}

pub async fn execute_search(
    config: &SearchConfig,
    query: &str,
) -> Result<Vec<SearchResult>, SearchError> {
    if !config.enabled || config.provider == "none" {
        return Ok(vec![]);
    }
    let client = search_client();
    match config.provider.as_str() {
        "tavily" => match tavily::search(client, &config.tavily_api_key, query).await {
            Ok(results) => Ok(results),
            Err(
                SearchError::InvalidApiKey
                | SearchError::RateLimited
                | SearchError::NetworkError(_)
                | SearchError::ParseError(_)
                | SearchError::NoResults,
            ) => {
                log::warn!(
                    "Tavily search failed, falling back to DuckDuckGo: {}",
                    query
                );
                duckduckgo::search(client, query).await
            }
        },
        "duckduckgo" => duckduckgo::search(client, query).await,
        "searxng" => searxng::search(client, &config.searxng_url, query).await,
        other => {
            log::warn!("Unknown search provider '{}', returning no results", other);
            Ok(vec![])
        }
    }
}
