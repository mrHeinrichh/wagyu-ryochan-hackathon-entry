//! Small, dependency-free helpers shared across modules.

use chrono::{DateTime, Utc};

/// Collapse whitespace and truncate a headline to `max_chars`, adding an
/// ellipsis when it was cut.
pub(crate) fn compact_headline(value: &str, max_chars: usize) -> String {
    let mut compact = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.chars().count() > max_chars {
        compact = compact.chars().take(max_chars.saturating_sub(1)).collect();
        compact.push_str("...");
    }
    compact
}

/// Extract a bare host from a URL, dropping the scheme, path, and `www.`.
pub(crate) fn host_from_url(url: &str) -> String {
    let host = url
        .split("://")
        .nth(1)
        .unwrap_or(url)
        .split('/')
        .next()
        .unwrap_or("")
        .trim_start_matches("www.");
    if host.is_empty() {
        "unknown-source".to_string()
    } else {
        host.to_string()
    }
}

/// Best-effort region guess from a URL's top-level domain.
pub(crate) fn infer_region(url: &str) -> Option<String> {
    let host = host_from_url(url).to_lowercase();
    if host.ends_with(".jp") || host.ends_with(".kr") || host.ends_with(".sg") {
        Some("Asia".to_string())
    } else if host.ends_with(".uk") || host.ends_with(".eu") || host.ends_with(".de") {
        Some("Europe".to_string())
    } else if host.ends_with(".com") || host.ends_with(".org") {
        Some("Global".to_string())
    } else {
        None
    }
}

/// Parse an RFC 3339 timestamp into UTC, returning `None` on any failure.
pub(crate) fn parse_datetime(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .ok()
}

/// Count how many of `words` appear as substrings of `text`.
pub(crate) fn count_hits(text: &str, words: &[&str]) -> usize {
    words.iter().filter(|word| text.contains(**word)).count()
}
