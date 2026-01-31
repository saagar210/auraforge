const TECH_KEYWORDS: &[&str] = &[
    "react",
    "vue",
    "angular",
    "svelte",
    "next.js",
    "nextjs",
    "nuxt",
    "typescript",
    "javascript",
    "python",
    "rust",
    "go",
    "golang",
    "java",
    "kotlin",
    "swift",
    "node",
    "deno",
    "bun",
    "postgres",
    "postgresql",
    "mysql",
    "mongodb",
    "redis",
    "sqlite",
    "docker",
    "kubernetes",
    "k8s",
    "aws",
    "gcp",
    "azure",
    "terraform",
    "graphql",
    "rest api",
    "grpc",
    "webpack",
    "vite",
    "tailwind",
    "prisma",
    "drizzle",
    "supabase",
    "firebase",
];

const TRIGGER_PATTERNS: &[&str] = &[
    " vs ",
    " versus ",
    "should i use",
    "best practice",
    "best way to",
    "how to implement",
    "latest version",
    "what is the difference",
    "compare ",
    "comparison",
    "recommend",
    "alternative to",
    "pros and cons",
    "trade-off",
    "tradeoff",
    "which is better",
];

pub fn should_search(message: &str) -> Option<String> {
    let lower = message.to_lowercase();

    // Check for trigger patterns first
    let has_trigger = TRIGGER_PATTERNS.iter().any(|p| lower.contains(p));

    if !has_trigger {
        return None;
    }

    // Must also mention at least one tech keyword
    let has_tech = TECH_KEYWORDS.iter().any(|k| lower.contains(k));

    if !has_tech {
        return None;
    }

    Some(build_search_query(message))
}

fn build_search_query(message: &str) -> String {
    let lower = message.to_lowercase();

    // For comparison queries, extract a focused query
    if let Some(q) = extract_comparison_query(&lower) {
        return q;
    }

    // Otherwise, take the core of the message (strip filler words)
    let cleaned = message
        .trim_start_matches(|c: char| !c.is_alphanumeric())
        .trim_end_matches(|c: char| !c.is_alphanumeric() && c != '?');

    // Cap at ~80 chars for a reasonable search query (safe for multi-byte UTF-8)
    if cleaned.chars().count() > 80 {
        let boundary = cleaned
            .char_indices()
            .take(80)
            .last()
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(cleaned.len());
        let truncated = &cleaned[..boundary];
        // Try to break at a word boundary
        truncated
            .rfind(' ')
            .map(|i| &truncated[..i])
            .unwrap_or(truncated)
            .to_string()
    } else {
        cleaned.to_string()
    }
}

fn extract_comparison_query(lower: &str) -> Option<String> {
    // "react vs vue" → "react vs vue comparison"
    if lower.contains(" vs ") || lower.contains(" versus ") {
        let parts: Vec<&str> = lower
            .splitn(2, " vs ")
            .flat_map(|s| s.splitn(2, " versus "))
            .collect();

        if parts.len() >= 2 {
            let a = parts[0].split_whitespace().last().unwrap_or("").trim();
            let b = parts[1].split_whitespace().next().unwrap_or("").trim();

            if !a.is_empty() && !b.is_empty() {
                return Some(format!("{} vs {} comparison", a, b));
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Trigger Detection ----

    #[test]
    fn triggers_on_comparison_with_tech() {
        let result = should_search("Should I use React vs Vue for my dashboard?");
        assert!(result.is_some());
    }

    #[test]
    fn triggers_on_best_practice() {
        let result = should_search("What are best practices for using PostgreSQL?");
        assert!(result.is_some());
    }

    #[test]
    fn triggers_on_recommendation() {
        let result = should_search("Can you recommend an alternative to Firebase?");
        assert!(result.is_some());
    }

    #[test]
    fn triggers_on_how_to_implement() {
        let result = should_search("How to implement authentication with Next.js?");
        assert!(result.is_some());
    }

    #[test]
    fn no_trigger_without_tech_keyword() {
        let result = should_search("What are best practices for cooking pasta?");
        assert!(result.is_none());
    }

    #[test]
    fn no_trigger_without_pattern() {
        let result = should_search("I like using React for my projects");
        assert!(result.is_none());
    }

    #[test]
    fn no_trigger_on_empty() {
        assert!(should_search("").is_none());
    }

    #[test]
    fn case_insensitive() {
        let result = should_search("SHOULD I USE REACT VS VUE?");
        assert!(result.is_some());
    }

    // ---- Query Building ----

    #[test]
    fn comparison_query_extracted() {
        let result = should_search("Should I use React vs Vue?").unwrap();
        assert!(result.contains("vs"));
        assert!(result.contains("comparison"));
    }

    #[test]
    fn non_comparison_query_uses_message() {
        let result = should_search("What are best practices for Docker?").unwrap();
        assert!(result.contains("Docker"));
    }

    #[test]
    fn long_query_truncated() {
        let long_msg = format!(
            "What are the best practices for using {} in a large-scale enterprise production environment with complex microservices architecture?",
            "Kubernetes"
        );
        let result = should_search(&long_msg).unwrap();
        assert!(result.len() <= 80);
    }

    // ---- Keyword Coverage ----

    #[test]
    fn triggers_for_various_tech() {
        let techs = [
            "postgres", "docker", "tailwind", "supabase", "prisma", "vite",
        ];
        for tech in techs {
            let msg = format!("What are best practices for {}?", tech);
            assert!(should_search(&msg).is_some(), "Failed for: {}", tech);
        }
    }

    #[test]
    fn utf8_multibyte_no_panic() {
        // This previously panicked by slicing mid-character
        let msg = "What are the best practices for using Kubernetes в крупномасштабной корпоративной production среде с микросервисной архитектурой?";
        let result = should_search(msg);
        assert!(result.is_some());
        // Should not exceed 80 chars
        assert!(result.unwrap().chars().count() <= 80);
    }

    #[test]
    fn triggers_for_various_patterns() {
        let patterns = [
            "React vs Vue",
            "Should I use Rust",
            "best practice for Python",
            "latest version of Next.js",
            "pros and cons of MongoDB",
            "which is better React or Angular",
        ];
        for p in patterns {
            assert!(should_search(p).is_some(), "Failed for: {}", p);
        }
    }
}
