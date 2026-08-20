//! The reasoning pipeline: turn a validated `PulseRequest` into a
//! `DecisionReceipt`.
//!
//! `build_pulse` is the orchestrator. It normalizes input, gathers news and
//! RYO evidence via `crate::services`, scores and ranks it here, and asks
//! `verdict` for the final judgement. The submodules keep each concern small:
//! `scoring` for per-story math, `classify` for text buckets, `sections` for
//! grouping/summary, and `verdict` for the judged output.

mod classify;
mod decision_support;
mod demo;
mod scoring;
mod sections;
mod verdict;

use std::collections::BTreeSet;

use axum::http::StatusCode;
use chrono::Utc;
use uuid::Uuid;

use crate::domain::{DecisionReceipt, NewsStory, PulseRequest, SourceAvailability};
use crate::error::ApiError;
use crate::services::news::{dedupe_stories, fetch_news};
use crate::services::ryo::mock_ryo_evidence;
use crate::services::ryo::{availability_from_ryo, call_ryo_evidence};
use crate::state::AppState;
use crate::util::compact_headline;

use decision_support::{
    build_decision_chain, build_invalidation, build_practice_plan, build_regional_convergence,
};
use demo::demo_news;
use scoring::score_stories;
use sections::{rank_sections, resolve_data_mode, resolve_status, summarize_receipt};
use verdict::{build_reasoning_layer_output, build_reasoning_verdict};

/// Run the full pipeline for one request and return a stored-ready receipt.
///
/// Refuses to run until required keys are present, except explicit RYO mock
/// mode. Failures in a single source degrade to warnings and missing-data notes
/// rather than aborting the run.
pub(crate) async fn build_pulse(
    state: &AppState,
    request: PulseRequest,
) -> Result<DecisionReceipt, ApiError> {
    let demo_mode = request.demo;
    let missing_keys = state.config.missing_required_keys();
    if !demo_mode && !missing_keys.is_empty() {
        return Err(ApiError::new(
            StatusCode::PRECONDITION_REQUIRED,
            "missing_required_keys",
            "Reasoning runs are disabled until required API keys are configured. Enable APP_MOCK_RYO=true only for labelled local RYO simulation.",
        )
        .with_detail(serde_json::json!({ "missing_keys": missing_keys })));
    }

    let symbol = normalize_symbol(&request.symbol)?;
    let timeframe = normalize_timeframe(request.timeframe.as_deref());
    let risk_budget_pct = request.risk_budget_pct.unwrap_or(0.5).clamp(0.1, 2.0);
    let user_thesis = normalize_thesis(&request);
    let regions = normalize_list(
        request.regions.unwrap_or_default(),
        vec!["Global".to_string()],
    );
    let sources = normalize_list(
        request.sources.unwrap_or_default(),
        vec!["Tavily".to_string()],
    );
    let run_id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();

    let mut warnings = Vec::new();
    let mut availability = Vec::new();
    let mut stories = if demo_mode {
        availability.push(SourceAvailability {
            source: "Judge demo fixture".to_string(),
            status: "partial".to_string(),
            data_mode: "simulated".to_string(),
            detail: "Four deterministic sample stories; never presented as live evidence."
                .to_string(),
            as_of: created_at.clone(),
        });
        demo_news(&symbol, &created_at)
    } else {
        fetch_news(
            state,
            &symbol,
            &timeframe,
            &regions,
            &sources,
            &mut availability,
        )
        .await
        .unwrap_or_else(|error| {
            warnings.push(error.message);
            Vec::new()
        })
    };
    if let Some(story) = user_story(&symbol, user_thesis.as_deref(), stories.len()) {
        availability.push(SourceAvailability {
            source: "User thesis".to_string(),
            status: "ok".to_string(),
            data_mode: "user-provided".to_string(),
            detail: "User-supplied event/thesis included as evidence and clearly labelled."
                .to_string(),
            as_of: created_at.clone(),
        });
        stories.push(story);
    }
    stories = dedupe_stories(stories);

    let ryo = if demo_mode {
        mock_ryo_evidence(&symbol)
    } else {
        call_ryo_evidence(state, &symbol).await
    };
    availability.extend(ryo.iter().map(availability_from_ryo));

    let cards = score_stories(&symbol, stories.as_slice(), ryo.as_slice());
    let sections = rank_sections(cards);
    let summary = summarize_receipt(&symbol, &sections, stories.len(), &ryo);
    let data_mode = resolve_data_mode(stories.as_slice(), ryo.as_slice());
    let status = resolve_status(stories.as_slice(), ryo.as_slice());

    let mut receipt_warnings = warnings;
    if stories.is_empty() {
        receipt_warnings.push("No news stories were available for this run. The receipt remains valid but cannot infer narrative impact from news.".to_string());
    }
    if ryo.iter().any(|tool| tool.status == "unavailable") {
        receipt_warnings.push("At least one RYO tool was unavailable. Market confirmation is excluded or marked missing where appropriate.".to_string());
    }
    if ryo.iter().any(|tool| tool.data_mode == "simulated") {
        receipt_warnings.push("RYO evidence is simulated because APP_MOCK_RYO=true. Use this for local demos only; live judging requires RYO_MCP_KEY.".to_string());
    }
    let (mut verdict, ai_warning) = if demo_mode {
        let mut verdict = verdict::deterministic_verdict(
            &symbol,
            &summary,
            &sections,
            stories.as_slice(),
            ryo.as_slice(),
            receipt_warnings.as_slice(),
        );
        verdict.generated_by = "deterministic-rust-demo".to_string();
        (verdict, None)
    } else {
        build_reasoning_verdict(
            state,
            &symbol,
            &summary,
            &sections,
            stories.as_slice(),
            ryo.as_slice(),
            receipt_warnings.as_slice(),
        )
        .await
    };
    if let Some(warning) = ai_warning {
        receipt_warnings.push(warning);
    }
    if ryo.iter().any(|tool| tool.data_mode == "simulated") {
        if verdict.decision == "CONFIRMED" {
            verdict.decision = "WATCHLIST".to_string();
        }
        verdict.confidence = verdict.confidence.min(64);
        if !verdict
            .missing_data
            .iter()
            .any(|item| item == "live RYO market confirmation")
        {
            verdict
                .missing_data
                .push("live RYO market confirmation".to_string());
        }
        verdict
            .warnings
            .push("Mock RYO mode prevents a live CONFIRMED verdict.".to_string());
        verdict.recommended_next_action =
            "treat as pre-demo watchlist until live RYO confirms or rejects the thesis".to_string();
    }
    if demo_mode {
        receipt_warnings.push(
            "Judge demo uses deterministic simulated news and RYO evidence. No item in this receipt is live market data."
                .to_string(),
        );
    }
    let reasoning_layer = build_reasoning_layer_output(
        &symbol,
        &run_id,
        &created_at,
        &sections,
        &verdict,
        ryo.as_slice(),
    );
    let invalidation = build_invalidation(&verdict);
    let decision_chain =
        build_decision_chain(&sections, &verdict, &reasoning_layer, ryo.as_slice());
    let practice_plan = build_practice_plan(
        &timeframe,
        risk_budget_pct,
        &verdict,
        &reasoning_layer,
        ryo.as_slice(),
        &invalidation,
    );
    let regional_convergence = build_regional_convergence(&regions, &sections);

    Ok(DecisionReceipt {
        id: Uuid::new_v4().to_string(),
        run_id,
        symbol,
        timeframe,
        regions,
        sources,
        user_thesis,
        created_at,
        status,
        data_mode,
        signal: reasoning_layer.signal.clone(),
        confidence: reasoning_layer.confidence,
        reasoning: reasoning_layer.reasoning.clone(),
        ryo_tools_used: reasoning_layer.ryo_tools_used.clone(),
        unavailable_data: reasoning_layer.unavailable_data.clone(),
        next_action: reasoning_layer.next_action.clone(),
        decision_chain,
        invalidation,
        practice_plan,
        regional_convergence,
        reasoning_layer,
        summary,
        verdict,
        sections,
        stories,
        ryo,
        availability,
        warnings: receipt_warnings,
        scoring_notes: vec![
            "Impact uses weighted available evidence only; unavailable RYO or news data is not converted to zero.".to_string(),
            "Cards are grouped by narrative cluster and sorted by impact, urgency, credibility and market confirmation.".to_string(),
            "A card needs both evidence and an argument. Raw headline volume does not increase score by itself.".to_string(),
        ],
    })
}

/// Validate and canonicalize a token symbol (uppercased, no `$`/spaces).
pub(crate) fn normalize_symbol(symbol: &str) -> Result<String, ApiError> {
    let cleaned = symbol
        .trim()
        .trim_start_matches('$')
        .replace(['-', ' '], "")
        .to_uppercase();
    if cleaned.is_empty()
        || cleaned.len() > 16
        || !cleaned.chars().all(|ch| ch.is_ascii_alphanumeric())
    {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "invalid_symbol",
            "Provide a token symbol such as SOL, BTC, ETH, or BNB.",
        ));
    }
    Ok(cleaned)
}

/// Clamp the timeframe to one of the supported windows, defaulting to 24h.
fn normalize_timeframe(timeframe: Option<&str>) -> String {
    match timeframe.unwrap_or("24h").trim().to_lowercase().as_str() {
        "6h" => "6h",
        "24h" => "24h",
        "7d" => "7d",
        "30d" => "30d",
        _ => "24h",
    }
    .to_string()
}

/// Trim, dedupe, and default a list of regions or sources.
fn normalize_list(values: Vec<String>, default: Vec<String>) -> Vec<String> {
    let cleaned = values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if cleaned.is_empty() { default } else { cleaned }
}

/// Pull the user narrative from `thesis`/`event`/`news`, capped at 1,200 chars.
fn normalize_thesis(request: &PulseRequest) -> Option<String> {
    request
        .thesis
        .as_deref()
        .or(request.event.as_deref())
        .or(request.news.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.chars().take(1_200).collect::<String>())
}

/// Wrap a user thesis as a clearly-labelled `NewsStory` so it enters scoring.
fn user_story(symbol: &str, thesis: Option<&str>, existing_count: usize) -> Option<NewsStory> {
    let thesis = thesis?;
    Some(NewsStory {
        id: format!("user-thesis-{}", existing_count + 1),
        headline: format!("User thesis for {symbol}: {}", compact_headline(thesis, 90)),
        source: "user-input".to_string(),
        url: "about:user-thesis".to_string(),
        content: thesis.to_string(),
        published_at: Some(Utc::now().to_rfc3339()),
        region: Some("User-supplied".to_string()),
        language: None,
        data_mode: "user-provided".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::ryo::unavailable_ryo;
    use scoring::score_story;
    use serde_json::json;

    #[test]
    fn normalizes_symbols() {
        assert_eq!(normalize_symbol("$sol ").unwrap(), "SOL");
        assert!(normalize_symbol("bad symbol with spaces").is_err());
    }

    #[test]
    fn unavailable_market_confirmation_is_not_zeroed() {
        let story = NewsStory {
            id: "1".to_string(),
            headline: "SOL upgrade momentum".to_string(),
            source: "coindesk.com".to_string(),
            url: "https://coindesk.com/example".to_string(),
            content: "SOL upgrade momentum and listing discussion".to_string(),
            published_at: Some(Utc::now().to_rfc3339()),
            region: Some("Global".to_string()),
            language: Some("en".to_string()),
            data_mode: "live".to_string(),
        };
        let ryo = vec![unavailable_ryo(
            "analyze_token",
            json!({ "symbol": "SOL" }),
            "missing key".to_string(),
        )];
        let card = score_story("SOL", &story, std::slice::from_ref(&story), &ryo);
        assert_eq!(card.score.market_confirmation, None);
        assert!(
            card.reasoning
                .iter()
                .any(|point| point.contains("excluded from scoring"))
        );
    }

    #[test]
    fn deduplicates_by_source_and_normalized_headline() {
        let stories = vec![
            NewsStory {
                id: "1".to_string(),
                headline: "BTC breaks out today!".to_string(),
                source: "example.com".to_string(),
                url: "https://example.com/a".to_string(),
                content: String::new(),
                published_at: None,
                region: None,
                language: None,
                data_mode: "live".to_string(),
            },
            NewsStory {
                id: "2".to_string(),
                headline: "BTC breaks out today".to_string(),
                source: "example.com".to_string(),
                url: "https://example.com/b".to_string(),
                content: String::new(),
                published_at: None,
                region: None,
                language: None,
                data_mode: "live".to_string(),
            },
        ];
        assert_eq!(dedupe_stories(stories).len(), 1);
    }
}
