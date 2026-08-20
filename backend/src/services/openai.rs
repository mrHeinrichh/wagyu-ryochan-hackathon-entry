//! The OpenAI reasoning call and the parsing that folds its answer back onto
//! the deterministic baseline verdict.

use axum::http::StatusCode;
use serde_json::{Value, json};

use crate::domain::{
    ChatTurn, DecisionReceipt, NewsStory, RankedSection, ReasoningVerdict, ReceiptSummary,
    RyoToolEvidence,
};
use crate::error::ApiError;
use crate::services::http::parse_response;
use crate::state::AppState;

/// Everything the OpenAI verdict needs: the run context plus the deterministic
/// baseline it refines.
pub(crate) struct OpenAiVerdictInput<'a> {
    pub(crate) symbol: &'a str,
    pub(crate) summary: &'a ReceiptSummary,
    pub(crate) sections: &'a [RankedSection],
    pub(crate) stories: &'a [NewsStory],
    pub(crate) ryo: &'a [RyoToolEvidence],
    pub(crate) warnings: &'a [String],
    pub(crate) base: ReasoningVerdict,
}

/// Ask OpenAI for the final verdict, seeded by the deterministic baseline.
///
/// The model may only refine fields; missing/unavailable data must stay
/// explicit. Any failure propagates as an `ApiError` so the caller can fall
/// back to the deterministic verdict.
pub(crate) async fn openai_verdict(
    state: &AppState,
    openai_key: &str,
    input: OpenAiVerdictInput<'_>,
) -> Result<ReasoningVerdict, ApiError> {
    let context = json!({
        "product": "RYO Global Token News Reasoning Layer",
        "task": "Return the final hackathon decision receipt verdict.",
        "rules": {
            "allowed_decisions": ["CONFIRMED", "WATCHLIST", "REJECTED"],
            "allowed_actions": ["watch", "investigate", "avoid", "wait", "reduce risk", "no action"],
            "must_not": [
                "invent missing data",
                "convert unavailable data to zero",
                "recommend live trading or DEX execution",
                "claim unavailable data is available"
            ]
        },
        "symbol": input.symbol,
        "summary": input.summary,
        "deterministic_baseline": &input.base,
        "top_cards": compact_cards(input.sections),
        "ryo_tools": compact_ryo(input.ryo),
        "news_count": input.stories.len(),
        "source_warnings": input.warnings
    });

    let response = state
        .client
        .post(&state.config.openai_responses_url)
        .bearer_auth(openai_key)
        .json(&json!({
            "model": state.config.openai_model,
            "input": [
                {
                    "role": "system",
                    "content": "You are the reasoning layer for a crypto research dashboard. Return JSON only. Keep missing or unavailable fields explicit. This is not financial advice."
                },
                {
                    "role": "user",
                    "content": context.to_string()
                }
            ],
            "text": {
                "format": {
                    "type": "json_object"
                }
            }
        }))
        .send()
        .await
        .map_err(|error| {
            ApiError::new(
                StatusCode::BAD_GATEWAY,
                "openai_network_error",
                error.to_string(),
            )
        })?;

    let value = parse_response(response, "openai_error").await?;
    let text = extract_openai_text(&value).ok_or_else(|| {
        ApiError::new(
            StatusCode::BAD_GATEWAY,
            "openai_empty_response",
            "OpenAI response did not include output text.",
        )
        .with_detail(value.clone())
    })?;
    let parsed = parse_json_text(&text)?;
    Ok(verdict_from_value(
        input.symbol,
        &state.config.openai_model,
        parsed,
        input.base,
    ))
}

/// Ask the model to explain one receipt without introducing outside facts.
pub(crate) async fn openai_chat(
    state: &AppState,
    openai_key: &str,
    question: &str,
    history: &[ChatTurn],
    receipt: Option<&DecisionReceipt>,
) -> Result<(String, Vec<String>), ApiError> {
    let history_start = history.len().saturating_sub(8);
    let conversation = history[history_start..]
        .iter()
        .filter(|turn| matches!(turn.role.as_str(), "user" | "assistant"))
        .map(|turn| {
            json!({
                "role": turn.role,
                "content": turn.content.chars().take(1_200).collect::<String>()
            })
        })
        .collect::<Vec<_>>();
    let receipt_context = receipt.map(|item| {
        json!({
            "id": item.id,
            "symbol": item.symbol,
            "timeframe": item.timeframe,
            "data_mode": item.data_mode,
            "summary": item.summary,
            "verdict": item.verdict,
            "decision_chain": item.decision_chain,
            "invalidation": item.invalidation,
            "practice_plan": item.practice_plan,
            "regional_convergence": item.regional_convergence,
            "top_cards": compact_cards(item.sections.as_slice()),
            "ryo_tools": compact_ryo(item.ryo.as_slice()),
            "warnings": item.warnings
        })
    });
    let context = json!({
        "product": "RYO Global Token News Pulse",
        "task": "Answer the user's question using only the supplied product and receipt context.",
        "question": question,
        "conversation": conversation,
        "receipt": receipt_context,
        "rules": [
            "Keep the answer under 140 words",
            "State when evidence is simulated, partial, missing, or unavailable",
            "Do not add prices, events, or market facts absent from the receipt",
            "Do not recommend live trading or execution",
            "Return a direct plain-language answer and exactly three short follow-up questions"
        ]
    });

    let response = state
        .client
        .post(&state.config.openai_responses_url)
        .bearer_auth(openai_key)
        .json(&json!({
            "model": state.config.openai_model,
            "input": [
                {
                    "role": "system",
                    "content": "You are RYO-CHAN, a concise crypto research guide. Explain supplied evidence and uncertainty. Return JSON only with answer and suggestions. This is not financial advice."
                },
                {
                    "role": "user",
                    "content": context.to_string()
                }
            ],
            "text": {
                "format": {
                    "type": "json_object"
                }
            }
        }))
        .send()
        .await
        .map_err(|error| {
            ApiError::new(
                StatusCode::BAD_GATEWAY,
                "openai_chat_network_error",
                error.to_string(),
            )
        })?;

    let value = parse_response(response, "openai_chat_error").await?;
    let text = extract_openai_text(&value).ok_or_else(|| {
        ApiError::new(
            StatusCode::BAD_GATEWAY,
            "openai_chat_empty_response",
            "OpenAI response did not include assistant text.",
        )
    })?;
    let parsed = parse_json_text(&text)?;
    let root = parsed.get("response").unwrap_or(&parsed);
    let answer = read_string(root, &["answer", "message"]).ok_or_else(|| {
        ApiError::new(
            StatusCode::BAD_GATEWAY,
            "openai_chat_answer_missing",
            "OpenAI did not return an assistant answer.",
        )
    })?;
    let suggestions = read_string_vec(root, &["suggestions", "follow_up_questions"])
        .unwrap_or_else(|| {
            vec![
                "What could invalidate this view?".to_string(),
                "Which headline matters most?".to_string(),
                "What should I monitor next?".to_string(),
            ]
        })
        .into_iter()
        .take(3)
        .collect();

    Ok((answer, suggestions))
}

/// Trim the top cards down to the fields the model needs for context.
fn compact_cards(sections: &[RankedSection]) -> Vec<Value> {
    sections
        .iter()
        .flat_map(|section| section.cards.iter())
        .take(8)
        .map(|card| {
            json!({
                "headline": card.headline,
                "source": card.source,
                "category": card.category,
                "cluster": card.narrative_cluster,
                "impact": card.score.impact,
                "sentiment": card.sentiment,
                "recommendation": card.recommendation,
                "ryo_alignment": card.ryo_alignment,
                "missing_data": card.missing_data,
                "reasoning": card.reasoning
            })
        })
        .collect()
}

/// Trim RYO evidence down to status and summary for the model context.
fn compact_ryo(ryo: &[RyoToolEvidence]) -> Vec<Value> {
    ryo.iter()
        .map(|tool| {
            json!({
                "tool": tool.tool,
                "status": tool.status,
                "data_mode": tool.data_mode,
                "summary": tool.summary,
                "warnings": tool.warnings
            })
        })
        .collect()
}

/// Pull the assistant text out of the Responses API envelope.
fn extract_openai_text(value: &Value) -> Option<String> {
    if let Some(text) = value.get("output_text").and_then(Value::as_str) {
        return Some(text.to_string());
    }
    value
        .get("output")
        .and_then(Value::as_array)?
        .iter()
        .find_map(|item| {
            item.get("content")
                .and_then(Value::as_array)
                .and_then(|content| extract_text_from_content(content.as_slice()))
                .or_else(|| item.get("text").and_then(Value::as_str).map(str::to_string))
        })
}

/// Find the first text-bearing content block in a Responses `content` array.
fn extract_text_from_content(content: &[Value]) -> Option<String> {
    content.iter().find_map(|item| {
        item.get("text")
            .and_then(Value::as_str)
            .map(str::to_string)
            .or_else(|| {
                item.get("content")
                    .and_then(Value::as_str)
                    .map(str::to_string)
            })
    })
}

/// Parse JSON from model text, tolerating a wrapper by slicing the outer braces.
fn parse_json_text(text: &str) -> Result<Value, ApiError> {
    let trimmed = text.trim();
    if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
        return Ok(value);
    }
    if let (Some(start), Some(end)) = (trimmed.find('{'), trimmed.rfind('}'))
        && start < end
    {
        return serde_json::from_str::<Value>(&trimmed[start..=end]).map_err(|error| {
            ApiError::new(
                StatusCode::BAD_GATEWAY,
                "openai_json_error",
                format!("OpenAI returned non-JSON verdict: {error}"),
            )
        });
    }
    Err(ApiError::new(
        StatusCode::BAD_GATEWAY,
        "openai_json_error",
        "OpenAI returned no parseable JSON verdict.",
    ))
}

/// Overlay the model's JSON answer onto the deterministic baseline verdict,
/// only replacing fields the model actually returned.
fn verdict_from_value(
    symbol: &str,
    model: &str,
    value: Value,
    mut base: ReasoningVerdict,
) -> ReasoningVerdict {
    let root = value.get("verdict").unwrap_or(&value);
    if let Some(decision) = read_string(root, &["decision"])
        && let Some(normalized) = normalize_decision(&decision)
    {
        base.decision = normalized;
    }
    if let Some(confidence) = root
        .get("confidence")
        .or_else(|| root.get("confidence_score"))
        .and_then(Value::as_u64)
    {
        base.confidence = confidence.clamp(1, 100) as u8;
    }
    if let Some(top_global_news) = read_string_vec(root, &["top_global_news", "top_news"]) {
        base.top_global_news = top_global_news;
    }
    if let Some(reason) = read_string(root, &["reason"]) {
        base.why_it_matters = vec![reason];
    }
    if let Some(why_it_matters) = read_string_vec(root, &["why_it_matters", "reasoning", "why"]) {
        base.why_it_matters = why_it_matters;
    }
    if let Some(ryo_market_evidence) =
        read_string_vec(root, &["ryo_market_evidence", "ryo_evidence"])
    {
        base.ryo_market_evidence = ryo_market_evidence;
    }
    if let Some(missing_data) = read_string_vec(root, &["missing_data"]) {
        base.missing_data = missing_data;
    }
    if let Some(warnings) = read_string_vec(root, &["warnings"]) {
        base.warnings = warnings;
    }
    if let Some(next_action) = read_string(root, &["recommended_next_action", "next_action"]) {
        base.recommended_next_action = next_action;
    }
    base.token_symbol = symbol.to_string();
    base.generated_by = format!("openai:{model}");
    base
}

/// Map a free-form decision string onto one of the three allowed verdicts.
fn normalize_decision(value: &str) -> Option<String> {
    let upper = value.trim().replace([' ', '-'], "_").to_uppercase();
    if upper.contains("CONFIRM") {
        Some("CONFIRMED".to_string())
    } else if upper.contains("WATCH") {
        Some("WATCHLIST".to_string())
    } else if upper.contains("REJECT") || upper.contains("NO_ACTION") || upper.contains("WAIT_ONLY")
    {
        Some("REJECTED".to_string())
    } else {
        None
    }
}

/// First string among `keys` present on the JSON object.
fn read_string(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_str))
        .map(str::to_string)
}

/// First non-empty string list among `keys`, accepting a bare string as a
/// single-element list.
fn read_string_vec(value: &Value, keys: &[&str]) -> Option<Vec<String>> {
    for key in keys {
        if let Some(array) = value.get(*key).and_then(Value::as_array) {
            let items = array
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>();
            if !items.is_empty() {
                return Some(items);
            }
        }
        if let Some(text) = value.get(*key).and_then(Value::as_str) {
            return Some(vec![text.to_string()]);
        }
    }
    None
}
