//! Per-story scoring: turning a `NewsStory` and RYO evidence into a `StoryCard`.
//!
//! Every sub-score is `Option<u8>`. A `None` means the input was unavailable
//! and is dropped from the weighted average, so honest degradation is built in:
//! nothing missing is silently scored as zero.

use chrono::{Duration as ChronoDuration, Utc};

use crate::domain::{NewsStory, RyoToolEvidence, StoryCard, StoryScore};
use crate::reasoning::classify::{narrative_cluster, recommendation_for, sentiment};
use crate::reasoning::credibility::verify_story;
use crate::util::{count_hits, parse_datetime};

/// Score every story against the shared RYO evidence.
pub(crate) fn score_stories(
    symbol: &str,
    stories: &[NewsStory],
    ryo: &[RyoToolEvidence],
) -> Vec<StoryCard> {
    stories
        .iter()
        .map(|story| score_story(symbol, story, stories, ryo))
        .collect()
}

/// Score a single story: sub-scores, blended impact, category, and reasoning.
pub(crate) fn score_story(
    symbol: &str,
    story: &NewsStory,
    all_stories: &[NewsStory],
    ryo: &[RyoToolEvidence],
) -> StoryCard {
    let text = format!("{} {}", story.headline, story.content);
    let lower = text.to_lowercase();
    let cluster = narrative_cluster(&lower);
    let sentiment = sentiment(&lower);
    let relevance = relevance_score(symbol, &lower, story.search_relevance);
    let verification = verify_story(story, all_stories);
    let credibility = Some(verification.credibility_score);
    let urgency = urgency_score(&lower, story.published_at.as_deref());
    let novelty = novelty_score(&cluster, all_stories);
    let ryo_available = ryo.iter().any(|item| item.status != "unavailable");
    let market_confirmation = ryo_available.then(|| market_confirmation_score(&lower, ryo));
    let uncertainty = uncertainty_score(&lower, ryo, &verification.claim_status);

    let mut missing_data = Vec::new();
    if story.published_at.is_none() {
        missing_data.push("published timestamp".to_string());
    }
    if story.region.is_none() {
        missing_data.push("region".to_string());
    }
    if story.language.is_none() {
        missing_data.push("language".to_string());
    }
    if !ryo_available {
        missing_data.push("RYO market confirmation".to_string());
    }
    if matches!(
        verification.claim_status.as_str(),
        "single source" | "user claim"
    ) {
        missing_data.push("independent news corroboration".to_string());
    }

    let impact = weighted_available_score(&[
        (relevance, 0.24),
        (credibility, 0.18),
        (novelty, 0.12),
        (urgency, 0.18),
        (market_confirmation, 0.2),
        (uncertainty.map(|value| 100_u8.saturating_sub(value)), 0.08),
    ]);

    let contradicted = lower.contains("rumor")
        || lower.contains("unconfirmed")
        || lower.contains("denies")
        || uncertainty.unwrap_or(0) >= 70;
    let category = if contradicted || verification.credibility_score < 45 {
        "unverified"
    } else if impact >= 75 {
        "position-changing"
    } else if impact >= 55 {
        "watch"
    } else {
        "noise"
    }
    .to_string();
    let recommendation = recommendation_for(&category, &sentiment, &missing_data);
    let confidence = confidence_for(story, ryo_available, impact, missing_data.len());
    let ryo_alignment = ryo_alignment(&lower, ryo_available, market_confirmation);
    let attention_reason = attention_reason(&category, &sentiment, impact, urgency);
    let requires_attention = attention_reason.is_some();
    let reasoning = reasoning_points(
        symbol,
        &cluster,
        &sentiment,
        impact,
        market_confirmation,
        &missing_data,
        &verification.explanation,
    );

    StoryCard {
        id: story.id.clone(),
        headline: story.headline.clone(),
        source: story.source.clone(),
        url: story.url.clone(),
        region: story.region.clone(),
        language: story.language.clone(),
        timestamp: story.published_at.clone(),
        related_token: symbol.to_string(),
        narrative_cluster: cluster,
        sentiment,
        score: StoryScore {
            relevance,
            credibility,
            novelty,
            urgency,
            market_confirmation,
            uncertainty,
            impact,
            formula: "weighted available signals; missing values excluded, not scored as zero"
                .to_string(),
        },
        ryo_alignment,
        missing_data,
        recommendation,
        confidence,
        reasoning,
        category,
        data_mode: story.data_mode.clone(),
        requires_attention,
        attention_reason,
        verification,
    }
}

/// Reserve the red attention treatment for position-changing, highly urgent,
/// or materially bearish evidence. High impact alone is not enough.
fn attention_reason(
    category: &str,
    sentiment: &str,
    impact: u8,
    urgency: Option<u8>,
) -> Option<String> {
    let urgency = urgency.unwrap_or(0);

    if category == "position-changing" {
        Some(format!(
            "This may change a position view: its available-evidence impact is {impact}/100."
        ))
    } else if urgency >= 80 {
        Some(format!(
            "This needs prompt review because event urgency reached {urgency}/100."
        ))
    } else if sentiment == "bearish" && impact >= 65 {
        Some(format!(
            "This needs risk attention because bearish evidence reached {impact}/100 impact."
        ))
    } else {
        None
    }
}

/// Weighted average over only the present sub-scores. Absent inputs drop out of
/// both the numerator and the denominator.
fn weighted_available_score(parts: &[(Option<u8>, f32)]) -> u8 {
    let (weighted, total_weight) =
        parts
            .iter()
            .fold((0.0_f32, 0.0_f32), |(weighted, total), (score, weight)| {
                if let Some(score) = score {
                    (weighted + (*score as f32 * *weight), total + *weight)
                } else {
                    (weighted, total)
                }
            });
    if total_weight == 0.0 {
        0
    } else {
        (weighted / total_weight).round().clamp(0.0, 100.0) as u8
    }
}

/// How directly the story is about this token.
fn relevance_score(symbol: &str, lower: &str, search_relevance: Option<u8>) -> Option<u8> {
    let symbol_lower = symbol.to_lowercase();
    let textual = if lower.contains(&symbol_lower) {
        92
    } else if token_alias(symbol)
        .iter()
        .any(|alias| lower.contains(alias))
    {
        82
    } else if lower.contains("crypto") || lower.contains("token") || lower.contains("market") {
        50
    } else {
        25
    };
    Some(search_relevance.map_or(textual, |retrieval| {
        ((f32::from(textual) * 0.65) + (f32::from(retrieval) * 0.35))
            .round()
            .clamp(0.0, 100.0) as u8
    }))
}

/// Common long-form aliases for a ticker, used to boost relevance.
fn token_alias(symbol: &str) -> Vec<&'static str> {
    match symbol {
        "BTC" => vec!["bitcoin"],
        "ETH" => vec!["ethereum", "ether"],
        "BNB" => vec!["bnb chain", "binance coin"],
        "SOL" => vec!["solana"],
        "LINK" => vec!["chainlink"],
        "ARB" => vec!["arbitrum"],
        "CAKE" => vec!["pancakeswap"],
        _ => vec![],
    }
}

/// Urgency from action words and recency of the published timestamp.
fn urgency_score(lower: &str, published_at: Option<&str>) -> Option<u8> {
    let mut score = 35;
    let urgent_words = [
        "breaking", "urgent", "exploit", "hack", "outage", "delist", "listing", "etf", "lawsuit",
        "approved", "rejected", "halted", "resumes",
    ];
    for word in urgent_words {
        if lower.contains(word) {
            score += 9;
        }
    }
    if let Some(published_at) = published_at.and_then(parse_datetime) {
        let age = Utc::now() - published_at;
        if age <= ChronoDuration::hours(6) {
            score += 20;
        } else if age <= ChronoDuration::hours(24) {
            score += 12;
        } else if age <= ChronoDuration::days(7) {
            score += 4;
        }
    }
    Some(score.min(100))
}

/// Novelty falls as more stories share the same narrative cluster.
fn novelty_score(cluster: &str, stories: &[NewsStory]) -> Option<u8> {
    let same_cluster = stories
        .iter()
        .filter(|story| {
            narrative_cluster(&format!("{} {}", story.headline, story.content)) == cluster
        })
        .count();
    Some(match same_cluster {
        0 | 1 => 78,
        2 => 65,
        3 => 54,
        _ => 42,
    })
}

/// How well the RYO market read agrees with the story's direction.
fn market_confirmation_score(lower: &str, ryo: &[RyoToolEvidence]) -> u8 {
    let ryo_text = ryo
        .iter()
        .filter_map(|item| item.summary.as_deref())
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    if ryo_text.is_empty() {
        return 50;
    }

    let story_positive = count_hits(lower, &["bull", "rally", "breakout", "upside", "support"]);
    let story_negative = count_hits(
        lower,
        &["bear", "risk", "exploit", "sell", "downside", "weak"],
    );
    let ryo_positive = count_hits(
        &ryo_text,
        &["confirmed", "bull", "positive", "momentum", "watchlist"],
    );
    let ryo_negative = count_hits(
        &ryo_text,
        &["unconfirmed", "bear", "risk", "negative", "unavailable"],
    );

    let positive_alignment = story_positive > story_negative && ryo_positive >= ryo_negative;
    let negative_alignment = story_negative > story_positive && ryo_negative >= ryo_positive;
    if positive_alignment || negative_alignment {
        78
    } else if (story_positive + story_negative) == 0 {
        56
    } else {
        42
    }
}

/// Uncertainty from hedging language and any partial/unavailable RYO tools.
fn uncertainty_score(lower: &str, ryo: &[RyoToolEvidence], claim_status: &str) -> Option<u8> {
    let mut score = 15;
    score += (count_hits(
        lower,
        &[
            "rumor",
            "unconfirmed",
            "alleged",
            "unknown",
            "may",
            "could",
            "missing",
            "unavailable",
        ],
    ) * 12) as u8;
    if ryo.iter().any(|item| item.status == "partial") {
        score += 12;
    }
    if ryo.iter().any(|item| item.status == "unavailable") {
        score += 18;
    }
    if matches!(claim_status, "single source" | "user claim") {
        score += 18;
    } else if claim_status == "conflicting" {
        score += 30;
    }
    Some(score.min(100))
}

/// Coarse confidence label from data mode, RYO availability, impact, and gaps.
fn confidence_for(
    story: &NewsStory,
    ryo_available: bool,
    impact: u8,
    missing_count: usize,
) -> String {
    let mut points = 0;
    if story.data_mode == "live" {
        points += 1;
    }
    if ryo_available {
        points += 1;
    }
    if impact >= 70 {
        points += 1;
    }
    if missing_count == 0 {
        points += 1;
    }
    match points {
        0 | 1 => "low",
        2 | 3 => "medium",
        _ => "high",
    }
    .to_string()
}

/// A one-line read on whether RYO confirms the story.
fn ryo_alignment(lower: &str, ryo_available: bool, market_confirmation: Option<u8>) -> String {
    if !ryo_available {
        return "RYO evidence unavailable for this run".to_string();
    }
    match market_confirmation {
        Some(score) if score >= 70 => {
            "RYO market read broadly confirms the story direction".to_string()
        }
        Some(score) if score <= 45 => {
            "RYO market read does not clearly confirm the story direction".to_string()
        }
        Some(_) if lower.contains("macro") || lower.contains("risk-off") => {
            "RYO market context should decide whether this is token-specific or broad risk"
                .to_string()
        }
        Some(_) => "RYO market read is neutral or mixed".to_string(),
        None => "RYO evidence missing".to_string(),
    }
}

/// Build the human-readable reasoning trace for a card.
fn reasoning_points(
    symbol: &str,
    cluster: &str,
    sentiment: &str,
    impact: u8,
    market_confirmation: Option<u8>,
    missing_data: &[String],
    verification: &str,
) -> Vec<String> {
    let mut points = vec![
        format!(
            "The story maps to {cluster}, which can affect {symbol} if it changes liquidity, risk appetite or token-specific demand."
        ),
        format!(
            "Narrative sentiment is {sentiment}; impact score is {impact}/100 after weighting available evidence."
        ),
    ];
    points.push(format!("News verification: {verification}"));
    match market_confirmation {
        Some(score) => points.push(format!("RYO market confirmation contributes {score}/100 to the score.")),
        None => points.push("RYO market confirmation was unavailable and was excluded from scoring, not treated as zero.".to_string()),
    }
    if !missing_data.is_empty() {
        points.push(format!("Missing fields: {}.", missing_data.join(", ")));
    }
    points
}

#[cfg(test)]
mod attention_tests {
    use super::attention_reason;

    #[test]
    fn urgent_evidence_requires_attention() {
        let reason = attention_reason("watch", "mixed", 62, Some(85));

        assert!(reason.is_some());
    }

    #[test]
    fn impact_alone_does_not_trigger_attention() {
        let reason = attention_reason("watch", "bullish", 92, Some(40));

        assert!(reason.is_none());
    }
}
