use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{SearchError, SearchResult};

#[derive(Serialize)]
struct TavilyRequest<'a> {
    api_key: &'a str,
    query: &'a str,
    search_depth: &'a str,
    max_results: u32,
}

#[derive(Deserialize)]
struct TavilyResponse {
    results: Vec<TavilyResult>,
}

#[derive(Deserialize)]
struct TavilyResult {
    title: String,
    url: String,
    content: String,
    score: f64,
}

pub async fn search(api_key: &str, query: &str) -> Result<Vec<SearchResult>, SearchError> {
    if api_key.is_empty() {
        return Err(SearchError::InvalidApiKey);
    }

    let client = Client::new();

    let response = client
        .post("https://api.tavily.com/search")
        .json(&TavilyRequest {
            api_key,
            query,
            search_depth: "basic",
            max_results: 5,
        })
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| SearchError::NetworkError(e.to_string()))?;

    match response.status().as_u16() {
        200 => {}
        401 => return Err(SearchError::InvalidApiKey),
        429 => return Err(SearchError::RateLimited),
        status => {
            return Err(SearchError::NetworkError(format!(
                "Tavily returned status {}",
                status
            )));
        }
    }

    let body: TavilyResponse = response
        .json()
        .await
        .map_err(|e| SearchError::ParseError(e.to_string()))?;

    if body.results.is_empty() {
        return Err(SearchError::NoResults);
    }

    Ok(body
        .results
        .into_iter()
        .map(|r| SearchResult {
            title: r.title,
            url: r.url,
            snippet: r.content,
            score: r.score,
        })
        .collect())
}
