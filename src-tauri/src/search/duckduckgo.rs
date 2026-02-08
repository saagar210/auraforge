use scraper::{Html, Selector};

use super::{SearchError, SearchResult};

pub async fn search(
    client: &reqwest::Client,
    query: &str,
) -> Result<Vec<SearchResult>, SearchError> {
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

/// Selector sets to try in order. DDG has changed its HTML structure before,
/// so we attempt multiple known layouts.
const SELECTOR_SETS: &[(&str, &str, &str)] = &[
    // Current (2024-2026) layout
    (".result", ".result__a", ".result__snippet"),
    // Alternative class names seen in some DDG responses
    (".web-result", ".result__a", ".result__snippet"),
    (".result", "a.result__url", ".result__snippet"),
];

fn parse_results(html: &str) -> Result<Vec<SearchResult>, SearchError> {
    let document = Html::parse_document(html);

    // Try each selector set until one produces results
    for (container, link, snippet) in SELECTOR_SETS {
        if let Ok(results) = try_parse_with_selectors(&document, container, link, snippet) {
            if !results.is_empty() {
                return Ok(results);
            }
        }
    }

    // Fallback: extract DDG redirect links directly from the entire page
    let fallback = extract_links_fallback(&document);
    if !fallback.is_empty() {
        log::warn!(
            "DuckDuckGo primary selectors failed; used link-extraction fallback ({} results)",
            fallback.len()
        );
        return Ok(fallback);
    }

    // Nothing worked — log diagnostic info
    let body_len = html.len();
    let has_noscript = html.contains("noscript");
    log::warn!(
        "DuckDuckGo returned HTML ({} bytes, noscript={}) but no results could be parsed. \
         Selectors may be outdated.",
        body_len,
        has_noscript,
    );

    Err(SearchError::NoResults)
}

fn try_parse_with_selectors(
    document: &Html,
    container_sel: &str,
    link_sel: &str,
    snippet_sel: &str,
) -> Result<Vec<SearchResult>, SearchError> {
    let container =
        Selector::parse(container_sel).map_err(|e| SearchError::ParseError(format!("{:?}", e)))?;
    let link =
        Selector::parse(link_sel).map_err(|e| SearchError::ParseError(format!("{:?}", e)))?;
    let snippet =
        Selector::parse(snippet_sel).map_err(|e| SearchError::ParseError(format!("{:?}", e)))?;

    let mut results = Vec::new();

    for (i, result) in document.select(&container).enumerate() {
        if i >= 5 {
            break;
        }

        let title = result
            .select(&link)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default()
            .trim()
            .to_string();

        let raw_url = result
            .select(&link)
            .next()
            .and_then(|el| el.value().attr("href"))
            .unwrap_or_default()
            .to_string();

        let url = extract_ddg_url(&raw_url).unwrap_or(raw_url);

        let snippet_text = result
            .select(&snippet)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default()
            .trim()
            .to_string();

        if title.is_empty() && snippet_text.is_empty() {
            continue;
        }

        // Position-based scoring: top results get higher scores
        let score = 1.0 - (i as f64 * 0.15);

        results.push(SearchResult {
            title,
            url,
            snippet: snippet_text,
            score,
        });
    }

    Ok(results)
}

/// Last-resort fallback: find all `<a>` tags with DDG redirect hrefs and extract
/// the target URLs. This works even if DDG changes container/class names, as long
/// as the redirect URL structure (`uddg=`) remains.
fn extract_links_fallback(document: &Html) -> Vec<SearchResult> {
    let a_sel = match Selector::parse("a[href]") {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let mut results = Vec::new();
    let mut seen_urls = std::collections::HashSet::new();

    for el in document.select(&a_sel) {
        if results.len() >= 5 {
            break;
        }

        let href = el.value().attr("href").unwrap_or_default();
        if let Some(url) = extract_ddg_url(href) {
            // Skip DDG internal links and duplicates
            if url.contains("duckduckgo.com") || !seen_urls.insert(url.clone()) {
                continue;
            }

            let title = el.text().collect::<String>().trim().to_string();
            if title.is_empty() {
                continue;
            }

            let score = 1.0 - (results.len() as f64 * 0.15);
            results.push(SearchResult {
                title,
                url,
                snippet: String::new(),
                score,
            });
        }
    }

    results
}

fn extract_ddg_url(href: &str) -> Option<String> {
    // DDG wraps URLs like //duckduckgo.com/l/?uddg=https%3A%2F%2Fexample.com&...
    let parsed = url::Url::parse(&format!("https:{}", href.trim_start_matches("//"))).ok()?;
    parsed
        .query_pairs()
        .find(|(k, _)| k == "uddg")
        .map(|(_, v)| v.into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_results_with_result_class() {
        let html = r#"
        <html><body>
        <div class="result">
            <a class="result__a" href="//duckduckgo.com/l/?uddg=https%3A%2F%2Fexample.com%2Fpage">Example Title</a>
            <span class="result__snippet">This is a snippet about the result.</span>
        </div>
        <div class="result">
            <a class="result__a" href="//duckduckgo.com/l/?uddg=https%3A%2F%2Fother.com">Other Page</a>
            <span class="result__snippet">Another snippet here.</span>
        </div>
        </body></html>
        "#;
        let results = parse_results(html).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "Example Title");
        assert_eq!(results[0].url, "https://example.com/page");
        assert_eq!(results[0].snippet, "This is a snippet about the result.");
        assert_eq!(results[1].title, "Other Page");
        assert_eq!(results[1].url, "https://other.com");
    }

    #[test]
    fn parse_results_fallback_with_uddg_links() {
        // No .result containers, but there are <a> tags with uddg params
        let html = r#"
        <html><body>
        <a href="//duckduckgo.com/l/?uddg=https%3A%2F%2Ffallback.com%2Fpath&rut=abc">Fallback Link</a>
        <a href="//duckduckgo.com/l/?uddg=https%3A%2F%2Fother-fallback.com&rut=def">Another Link</a>
        </body></html>
        "#;
        let results = parse_results(html).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "Fallback Link");
        assert_eq!(results[0].url, "https://fallback.com/path");
        assert!(results[0].snippet.is_empty());
    }

    #[test]
    fn parse_results_empty_html_returns_no_results() {
        let html = "<html><body><p>No search results here.</p></body></html>";
        let err = parse_results(html).unwrap_err();
        assert!(matches!(err, SearchError::NoResults));
    }

    #[test]
    fn extract_ddg_url_decodes_correctly() {
        let href = "//duckduckgo.com/l/?uddg=https%3A%2F%2Fexample.com%2Ffoo%3Fbar%3Dbaz&rut=abc";
        let url = extract_ddg_url(href).unwrap();
        assert_eq!(url, "https://example.com/foo?bar=baz");
    }

    #[test]
    fn extract_ddg_url_returns_none_for_non_ddg_links() {
        let href = "https://example.com/plain-link";
        assert!(extract_ddg_url(href).is_none());
    }
}
