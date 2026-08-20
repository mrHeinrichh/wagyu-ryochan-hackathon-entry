//! The news wire: Tavily search, a short-lived cache, and deduplication.

use std::collections::BTreeSet;

use axum::http::StatusCode;
use chrono::{Duration as ChronoDuration, Utc};
use serde_json::{Value, json};

use crate::domain::{NewsStory, SourceAvailability};
use crate::error::ApiError;
use crate::services::http::parse_response;
use crate::state::{AppState, CacheEntry};
use crate::util::{host_from_url, infer_region_from_context};

/// Fetch news for a symbol, using the 5-minute cache when it is warm.
///
/// When `TAVILY_API_KEY` is unset, this records an `unavailable` line and
/// returns no stories rather than inventing any.
pub(crate) async fn fetch_news(
    state: &AppState,
    symbol: &str,
    timeframe: &str,
    regions: &[String],
    sources: &[String],
    availability: &mut Vec<SourceAvailability>,
) -> Result<Vec<NewsStory>, ApiError> {
    let cache_key = news_cache_key(symbol, timeframe, regions, sources);
    if let Some(cached) = state.news_cache.read().await.get(&cache_key).cloned()
        && cached.expires_at > Utc::now()
    {
        availability.push(SourceAvailability {
            source: "Tavily cache".to_string(),
            status: "ok".to_string(),
            data_mode: "cached".to_string(),
            detail: format!(
                "Using cached news for {symbol}; expires at {}.",
                cached.expires_at.to_rfc3339()
            ),
            as_of: Utc::now().to_rfc3339(),
        });
        return Ok(cached.value);
    }

    if let Some(key) = state.config.tavily_api_key.as_deref() {
        let stories = search_tavily(state, key, symbol, timeframe, regions, sources).await?;
        state.news_cache.write().await.insert(
            cache_key,
            CacheEntry {
                value: stories.clone(),
                fetched_at: Utc::now(),
                expires_at: Utc::now() + ChronoDuration::minutes(5),
            },
        );
        availability.push(SourceAvailability {
            source: "Tavily".to_string(),
            status: "ok".to_string(),
            data_mode: "live".to_string(),
            detail: format!("{} stories returned for {symbol}.", stories.len()),
            as_of: Utc::now().to_rfc3339(),
        });
        return Ok(stories);
    }

    availability.push(SourceAvailability {
        source: "Tavily".to_string(),
        status: "unavailable".to_string(),
        data_mode: "unknown".to_string(),
        detail: "TAVILY_API_KEY is not configured. Waiting for real API keys; no mock news is generated.".to_string(),
        as_of: Utc::now().to_rfc3339(),
    });
    Ok(Vec::new())
}

/// Stable cache key from the request shape (symbol, window, regions, sources).
fn news_cache_key(symbol: &str, timeframe: &str, regions: &[String], sources: &[String]) -> String {
    format!(
        "{}:{}:{}:{}",
        symbol,
        timeframe,
        regions.join(",").to_lowercase(),
        sources.join(",").to_lowercase()
    )
}

/// Call the Tavily search API and normalize the results into `NewsStory` items.
async fn search_tavily(
    state: &AppState,
    key: &str,
    symbol: &str,
    timeframe: &str,
    regions: &[String],
    sources: &[String],
) -> Result<Vec<NewsStory>, ApiError> {
    let regions_text = regions.join(", ");
    let sources_text = sources.join(", ");
    let query = format!(
        "{symbol} crypto token news market impact {timeframe} regions: {regions_text} sources: {sources_text}"
    );
    let response = state
        .client
        .post("https://api.tavily.com/search")
        .bearer_auth(key)
        .json(&json!({
            "api_key": key,
            "query": query,
            "topic": "news",
            "search_depth": "advanced",
            "max_results": 12,
            "include_answer": false,
            "include_raw_content": false
        }))
        .send()
        .await
        .map_err(|error| {
            ApiError::new(
                StatusCode::BAD_GATEWAY,
                "tavily_network_error",
                error.to_string(),
            )
        })?;

    let value = parse_response(response, "tavily_error").await?;
    let stories = value
        .get("results")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
        .map(|(index, item)| {
            let url = item.get("url").and_then(Value::as_str).unwrap_or("");
            let headline = item
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or("Untitled source");
            let content = item
                .get("content")
                .or_else(|| item.get("snippet"))
                .and_then(Value::as_str)
                .unwrap_or("");
            NewsStory {
                id: format!("news-{}", index + 1),
                headline: headline.to_string(),
                source: host_from_url(url),
                url: url.to_string(),
                content: content.to_string(),
                published_at: item
                    .get("published_date")
                    .or_else(|| item.get("published_at"))
                    .and_then(Value::as_str)
                    .map(str::to_string),
                region: infer_region_from_context(url, &format!("{headline} {content}")),
                language: None,
                data_mode: "live".to_string(),
            }
        })
        .collect();
    Ok(stories)
}

/// Drop near-duplicate stories that share a source and a normalized headline.
pub(crate) fn dedupe_stories(stories: Vec<NewsStory>) -> Vec<NewsStory> {
    let mut seen = BTreeSet::new();
    let mut deduped = Vec::new();
    for story in stories {
        let key = format!("{}:{}", story.source, normalize_headline(&story.headline));
        if seen.insert(key) {
            deduped.push(story);
        }
    }
    deduped
}

/// Lowercase, strip punctuation, and keep the first ten words of a headline.
fn normalize_headline(headline: &str) -> String {
    headline
        .to_lowercase()
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || ch.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .take(10)
        .collect::<Vec<_>>()
        .join(" ")
}
