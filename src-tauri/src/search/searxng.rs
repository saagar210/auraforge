use serde::Deserialize;

use super::{SearchError, SearchResult};

#[derive(Debug, Deserialize)]
struct SearxResponse {
    results: Vec<SearxResult>,
}

#[derive(Debug, Deserialize)]
struct SearxResult {
    title: Option<String>,
    url: Option<String>,
    content: Option<String>,
    score: Option<f64>,
}

pub async fn search(
    client: &reqwest::Client,
    base_url: &str,
    query: &str,
) -> Result<Vec<SearchResult>, SearchError> {
    if base_url.trim().is_empty() {
        return Err(SearchError::NetworkError(
            "SearXNG URL is empty".to_string(),
        ));
    }

    let url = format!("{}/search", base_url.trim_end_matches('/'));
    let response = client
        .get(url)
        .query(&[("q", query), ("format", "json")])
        .send()
        .await
        .map_err(|e| SearchError::NetworkError(e.to_string()))?;

    if !response.status().is_success() {
        return Err(SearchError::NetworkError(format!(
            "SearXNG returned status {}",
            response.status()
        )));
    }

    let body: SearxResponse = response
        .json()
        .await
        .map_err(|e| SearchError::ParseError(e.to_string()))?;

    let results = body
        .results
        .into_iter()
        .filter_map(|r| {
            let title = r.title?.trim().to_string();
            let url = r.url?.trim().to_string();
            if title.is_empty() || url.is_empty() {
                return None;
            }
            Some(SearchResult {
                title,
                url,
                snippet: r.content.unwrap_or_default(),
                score: r.score.unwrap_or(0.0),
            })
        })
        .take(5)
        .collect::<Vec<_>>();

    if results.is_empty() {
        return Err(SearchError::NoResults);
    }

    Ok(results)
}
