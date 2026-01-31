use reqwest::Client;
use scraper::{Html, Selector};

use super::{SearchError, SearchResult};

pub async fn search(query: &str) -> Result<Vec<SearchResult>, SearchError> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| SearchError::NetworkError(e.to_string()))?;

    let response = client
        .post("https://html.duckduckgo.com/html/")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header(
            "User-Agent",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36",
        )
        .body(format!("q={}", urlencoding(query)))
        .send()
        .await
        .map_err(|e| SearchError::NetworkError(e.to_string()))?;

    if !response.status().is_success() {
        return Err(SearchError::NetworkError(format!(
            "DuckDuckGo returned status {}",
            response.status()
        )));
    }

    let html = response
        .text()
        .await
        .map_err(|e| SearchError::ParseError(e.to_string()))?;

    parse_results(&html)
}

fn urlencoding(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}

fn parse_results(html: &str) -> Result<Vec<SearchResult>, SearchError> {
    let document = Html::parse_document(html);

    let result_sel =
        Selector::parse(".result").map_err(|e| SearchError::ParseError(format!("{:?}", e)))?;
    let link_sel =
        Selector::parse(".result__a").map_err(|e| SearchError::ParseError(format!("{:?}", e)))?;
    let snippet_sel = Selector::parse(".result__snippet")
        .map_err(|e| SearchError::ParseError(format!("{:?}", e)))?;

    let mut results = Vec::new();

    for (i, result) in document.select(&result_sel).enumerate() {
        if i >= 5 {
            break;
        }

        let title = result
            .select(&link_sel)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default()
            .trim()
            .to_string();

        let raw_url = result
            .select(&link_sel)
            .next()
            .and_then(|el| el.value().attr("href"))
            .unwrap_or_default()
            .to_string();

        let url = extract_ddg_url(&raw_url).unwrap_or(raw_url);

        let snippet = result
            .select(&snippet_sel)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default()
            .trim()
            .to_string();

        if title.is_empty() && snippet.is_empty() {
            continue;
        }

        // Position-based scoring: top results get higher scores
        let score = 1.0 - (i as f64 * 0.15);

        results.push(SearchResult {
            title,
            url,
            snippet,
            score,
        });
    }

    if results.is_empty() {
        return Err(SearchError::NoResults);
    }

    Ok(results)
}

fn extract_ddg_url(href: &str) -> Option<String> {
    // DDG wraps URLs like //duckduckgo.com/l/?uddg=https%3A%2F%2Fexample.com&...
    let parsed = url::Url::parse(&format!("https:{}", href.trim_start_matches("//"))).ok()?;
    parsed
        .query_pairs()
        .find(|(k, _)| k == "uddg")
        .map(|(_, v)| v.into_owned())
}
