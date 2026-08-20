//! Building the judged verdict and the compact reasoning-layer output.
//!
//! The deterministic verdict is always computed first. When `OPENAI_API_KEY`
//! is present the model refines it; on any failure we keep the deterministic
//! answer and record why.

use std::collections::BTreeSet;

use crate::domain::{
    NewsConsensus, NewsStory, RankedSection, ReasoningLayerOutput, ReasoningVerdict,
    ReceiptSummary, RyoToolEvidence, StoryCard,
};
use crate::reasoning::debate::build_debate;
use crate::services::openai::{OpenAiVerdictInput, openai_verdict};
use crate::state::AppState;

/// Produce the verdict, preferring OpenAI when configured and reachable.
/// Returns the verdict plus an optional warning describing any fallback.
pub(crate) struct VerdictContext<'a> {
    pub(crate) symbol: &'a str,
    pub(crate) summary: &'a ReceiptSummary,
    pub(crate) sections: &'a [RankedSection],
    pub(crate) stories: &'a [NewsStory],
    pub(crate) ryo: &'a [RyoToolEvidence],
    pub(crate) warnings: &'a [String],
    pub(crate) news_consensus: &'a NewsConsensus,
}

pub(crate) async fn build_reasoning_verdict(
    state: &AppState,
    input: VerdictContext<'_>,
) -> (ReasoningVerdict, Option<String>) {
    let base = deterministic_verdict(
        input.symbol,
        input.summary,
        input.sections,
        input.stories,
        input.ryo,
        input.warnings,
        input.news_consensus,
    );
    let Some(openai_key) = state.config.openai_api_key.as_deref() else {
        let mut fallback = base;
        fallback.generated_by = "deterministic-rust-fallback".to_string();
        return (
            fallback,
            Some(
                "OPENAI_API_KEY is not configured; using deterministic Rust verdict for local demo."
                    .to_string(),
            ),
        );
    };

    match openai_verdict(
        state,
        openai_key,
        OpenAiVerdictInput {
            symbol: input.symbol,
            summary: input.summary,
            sections: input.sections,
            stories: input.stories,
            ryo: input.ryo,
            warnings: input.warnings,
            news_consensus: input.news_consensus,
            base: base.clone(),
        },
    )
    .await
    {
        Ok(verdict) => (verdict, None),
        Err(error) => {
            let mut fallback = base;
            fallback.generated_by = "deterministic-rust-fallback".to_string();
            (
                fallback,
                Some(format!(
                    "OpenAI reasoning unavailable: {}. Deterministic verdict used instead.",
                    error.message
                )),
            )
        }
    }
}

/// The rules-based verdict: decision, confidence, and the supporting lists.
pub(crate) fn deterministic_verdict(
    symbol: &str,
    summary: &ReceiptSummary,
    sections: &[RankedSection],
    stories: &[NewsStory],
    ryo: &[RyoToolEvidence],
    warnings: &[String],
    news_consensus: &NewsConsensus,
) -> ReasoningVerdict {
    let cards = sections
        .iter()
        .flat_map(|section| section.cards.iter())
        .collect::<Vec<_>>();
    let highest = summary
        .highest_impact
        .unwrap_or(if stories.is_empty() { 20 } else { 45 });
    let has_market_confirmation = cards.iter().any(|card| {
        card.score.market_confirmation.unwrap_or(0) >= 70 && card.category != "unverified"
    });
    let decision = if highest >= 75 && has_market_confirmation {
        "CONFIRMED"
    } else if highest >= 55 || summary.unverified_count > 0 {
        "WATCHLIST"
    } else {
        "REJECTED"
    };

    let mut confidence = i16::from(highest);
    if has_market_confirmation {
        confidence += 7;
    }
    if stories.is_empty() {
        confidence = confidence.min(35);
    }
    if ryo.iter().any(|tool| tool.status == "unavailable") {
        confidence -= 15;
    }
    if !warnings.is_empty() {
        confidence -= 5;
    }

    let top_global_news = top_global_news(cards.as_slice(), stories);
    let why_it_matters = why_it_matters(summary, cards.as_slice());
    let missing_data = missing_data_for_verdict(cards.as_slice(), stories, ryo);
    let (scenario_odds, debate) = build_debate(
        symbol,
        sections,
        news_consensus,
        ryo,
        missing_data.as_slice(),
    );

    ReasoningVerdict {
        decision: decision.to_string(),
        confidence: confidence.clamp(5, 95) as u8,
        token_symbol: symbol.to_string(),
        top_global_news,
        why_it_matters,
        ryo_market_evidence: ryo_market_evidence(ryo),
        missing_data,
        warnings: warnings.to_vec(),
        recommended_next_action: recommended_next_action(decision, highest),
        generated_by: "deterministic-rust".to_string(),
        scenario_odds,
        debate,
    }
}

/// Derive the compact top-level reasoning-layer summary from the verdict.
pub(crate) fn build_reasoning_layer_output(
    symbol: &str,
    run_id: &str,
    timestamp: &str,
    sections: &[RankedSection],
    verdict: &ReasoningVerdict,
    ryo: &[RyoToolEvidence],
) -> ReasoningLayerOutput {
    let ryo_tools_used = ryo
        .iter()
        .map(|tool| tool.tool.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let unavailable_data = verdict
        .missing_data
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let reasoning = verdict.why_it_matters.first().cloned().unwrap_or_else(|| {
        format!(
            "{} is classified as {} from available RYO and news evidence.",
            symbol, verdict.decision
        )
    });

    ReasoningLayerOutput {
        signal: verdict.decision.clone(),
        symbol: symbol.to_string(),
        confidence: confidence_ratio(verdict.confidence),
        reasoning,
        ryo_tools_used,
        unavailable_data,
        next_action: verdict.recommended_next_action.clone(),
        affected_tokens: vec![symbol.to_string()],
        sentiment: top_sentiment(sections),
        market_confirmation: market_confirmation_label(sections, ryo),
        timestamp: timestamp.to_string(),
        run_id: run_id.to_string(),
    }
}

/// Convert a 0-100 confidence into a rounded 0-1 ratio.
fn confidence_ratio(confidence: u8) -> f32 {
    ((confidence as f32 / 100.0) * 100.0).round() / 100.0
}

/// Sentiment of the single highest-impact card across all sections.
fn top_sentiment(sections: &[RankedSection]) -> String {
    sections
        .iter()
        .flat_map(|section| section.cards.iter())
        .max_by_key(|card| card.score.impact)
        .map(|card| card.sentiment.clone())
        .unwrap_or_else(|| "uncertain".to_string())
}

/// A coarse label for how the market read landed across the run.
fn market_confirmation_label(sections: &[RankedSection], ryo: &[RyoToolEvidence]) -> String {
    if ryo.iter().any(|tool| tool.data_mode == "simulated") {
        return "simulated_unconfirmed".to_string();
    }
    if ryo.iter().all(|tool| tool.status == "unavailable") {
        return "unavailable".to_string();
    }
    let best_confirmation = sections
        .iter()
        .flat_map(|section| section.cards.iter())
        .filter_map(|card| card.score.market_confirmation)
        .max();
    match best_confirmation {
        Some(score) if score >= 70 => "confirmed".to_string(),
        Some(score) if score <= 45 => "contradicted_or_weak".to_string(),
        Some(_) => "mixed".to_string(),
        None => "missing".to_string(),
    }
}

/// Top three headlines, preferring scored cards and falling back to raw stories.
fn top_global_news(cards: &[&StoryCard], stories: &[NewsStory]) -> Vec<String> {
    let from_cards = cards
        .iter()
        .take(3)
        .map(|card| card.headline.clone())
        .collect::<Vec<_>>();
    if !from_cards.is_empty() {
        return from_cards;
    }
    stories
        .iter()
        .take(3)
        .map(|story| story.headline.clone())
        .collect()
}

/// Assemble the "why it matters" bullets from the summary and top cards.
fn why_it_matters(summary: &ReceiptSummary, cards: &[&StoryCard]) -> Vec<String> {
    let mut points = vec![summary.conclusion.clone()];
    for card in cards.iter().take(2) {
        points.extend(card.reasoning.iter().take(2).cloned());
    }
    points.truncate(5);
    points
}

/// Collect the distinct missing-data items across cards, plus run-level gaps.
fn missing_data_for_verdict(
    cards: &[&StoryCard],
    stories: &[NewsStory],
    ryo: &[RyoToolEvidence],
) -> Vec<String> {
    let mut missing = BTreeSet::new();
    for card in cards {
        for item in &card.missing_data {
            missing.insert(item.clone());
        }
    }
    if stories.is_empty() {
        missing.insert("live news evidence".to_string());
    }
    for tool in ryo.iter().filter(|tool| tool.status == "unavailable") {
        missing.insert(format!("RYO {}", tool.tool));
    }
    missing.into_iter().collect()
}

/// One line per RYO tool describing its status and summary.
fn ryo_market_evidence(ryo: &[RyoToolEvidence]) -> Vec<String> {
    ryo.iter()
        .map(|tool| {
            let detail = tool
                .summary
                .clone()
                .or_else(|| tool.warnings.first().cloned())
                .unwrap_or_else(|| "no summary returned".to_string());
            format!(
                "{}: {} / {} - {}",
                tool.tool, tool.status, tool.data_mode, detail
            )
        })
        .collect()
}

/// The cautious next action implied by the decision and highest impact.
fn recommended_next_action(decision: &str, highest_impact: u8) -> String {
    match decision {
        "CONFIRMED" if highest_impact >= 85 => {
            "reduce risk or update watch assumptions after reviewing the receipt".to_string()
        }
        "CONFIRMED" => "investigate the top narrative before changing exposure".to_string(),
        "WATCHLIST" => "wait for confirmation and keep the token on watch".to_string(),
        _ => "no action; keep the receipt for comparison with the next run".to_string(),
    }
}
