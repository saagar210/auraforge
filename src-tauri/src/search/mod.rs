mod duckduckgo;
mod searxng;
mod tavily;
mod trigger;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use thiserror::Error;

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

const SEARCH_CACHE_TTL_SECS: u64 = 45;
const SEARCH_CACHE_MAX_ENTRIES: usize = 64;

#[derive(Debug, Clone)]
struct SearchCacheEntry {
    inserted_at: Instant,
    results: Vec<SearchResult>,
}

fn search_cache() -> &'static Mutex<HashMap<String, SearchCacheEntry>> {
    static CACHE: OnceLock<Mutex<HashMap<String, SearchCacheEntry>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn cache_key(provider: &str, query: &str) -> String {
    format!(
        "{}::{}",
        provider.trim().to_ascii_lowercase(),
        query.trim().to_ascii_lowercase()
    )
}

fn get_cached_results(key: &str) -> Option<Vec<SearchResult>> {
    let cache = search_cache();
    let mut guard = cache.lock().ok()?;
    let ttl = Duration::from_secs(SEARCH_CACHE_TTL_SECS);
    guard.retain(|_, entry| entry.inserted_at.elapsed() < ttl);
    guard.get(key).map(|entry| entry.results.clone())
}

fn put_cached_results(key: String, results: Vec<SearchResult>) {
    let cache = search_cache();
    let Ok(mut guard) = cache.lock() else {
        return;
    };
    let ttl = Duration::from_secs(SEARCH_CACHE_TTL_SECS);
    guard.retain(|_, entry| entry.inserted_at.elapsed() < ttl);

    if guard.len() >= SEARCH_CACHE_MAX_ENTRIES {
        if let Some(oldest_key) = guard
            .iter()
            .min_by_key(|(_, entry)| entry.inserted_at)
            .map(|(key, _)| key.clone())
        {
            guard.remove(&oldest_key);
        }
    }

    guard.insert(
        key,
        SearchCacheEntry {
            inserted_at: Instant::now(),
            results,
        },
    );
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
    let query = query.trim();
    if query.is_empty() || !config.enabled || config.provider == "none" {
        return Ok(vec![]);
    }

    let provider = config.provider.trim().to_ascii_lowercase();
    let key = cache_key(&provider, query);
    if let Some(cached) = get_cached_results(&key) {
        return Ok(cached);
    }

    let client = search_client();
    let results = match provider.as_str() {
        "tavily" => match tavily::search(client, &config.tavily_api_key, query).await {
            Ok(results) => results,
            Err(
                SearchError::InvalidApiKey
                | SearchError::RateLimited
                | SearchError::NetworkError(_)
                | SearchError::ParseError(_)
                | SearchError::NoResults,
            ) => {
                log::warn!(
                    "Tavily search failed, falling back to DuckDuckGo for query '{}'",
                    query
                );
                duckduckgo::search(client, query).await?
            }
        },
        "duckduckgo" => duckduckgo::search(client, query).await?,
        "searxng" => match searxng::search(client, &config.searxng_url, query).await {
            Ok(results) => results,
            Err(err) => {
                log::warn!(
                    "SearXNG search failed ({:?}), falling back to DuckDuckGo for query '{}'",
                    err,
                    query
                );
                duckduckgo::search(client, query).await?
            }
        },
        other => {
            log::warn!("Unknown search provider '{}', returning no results", other);
            vec![]
        }
    };

    put_cached_results(key, results.clone());
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_key_normalizes_provider_and_query() {
        let a = cache_key(" Tavily ", "How To Build");
        let b = cache_key("tavily", "how to build");
        assert_eq!(a, b);
        assert_eq!(a, "tavily::how to build");
    }
}
