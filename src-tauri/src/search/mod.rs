mod duckduckgo;
mod tavily;
mod trigger;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::types::SearchConfig;

pub use trigger::should_search;

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
    match config.provider.as_str() {
        "tavily" => match tavily::search(&config.tavily_api_key, query).await {
            Ok(results) => Ok(results),
            Err(
                SearchError::InvalidApiKey
                | SearchError::RateLimited
                | SearchError::NetworkError(_)
                | SearchError::NoResults,
            ) => {
                log::warn!(
                    "Tavily search failed, falling back to DuckDuckGo: {}",
                    query
                );
                duckduckgo::search(query).await
            }
            Err(e) => Err(e),
        },
        "duckduckgo" => duckduckgo::search(query).await,
        _ => duckduckgo::search(query).await,
    }
}
