//! RYO MCP tool calls and the helpers that read their responses.

use axum::http::StatusCode;
use chrono::Utc;
use serde_json::{Value, json};

use crate::domain::{RyoToolEvidence, SourceAvailability};
use crate::error::ApiError;
use crate::services::http::parse_response;
use crate::state::AppState;

/// Call the six market-intelligence tools RYO exposes and collect their
/// evidence. A failed call becomes an `unavailable` record, never a gap.
pub(crate) async fn call_ryo_evidence(state: &AppState, symbol: &str) -> Vec<RyoToolEvidence> {
    if state.config.ryo_mock_enabled && state.config.ryo_mcp_key.is_none() {
        return mock_ryo_evidence(symbol);
    }

    let mut evidence = Vec::new();
    evidence.push(
        call_ryo_tool(state, "market_overview", json!({}))
            .await
            .unwrap_or_else(|error| unavailable_ryo("market_overview", json!({}), error.message)),
    );
    evidence.push(
        call_ryo_tool(
            state,
            "scan_market",
            json!({ "theme": format!("{symbol} global news impact"), "limit": 12 }),
        )
        .await
        .unwrap_or_else(|error| {
            unavailable_ryo(
                "scan_market",
                json!({ "theme": format!("{symbol} global news impact"), "limit": 12 }),
                error.message,
            )
        }),
    );
    evidence.push(
        call_ryo_tool(state, "analyze_token", json!({ "symbol": symbol }))
            .await
            .unwrap_or_else(|error| {
                unavailable_ryo("analyze_token", json!({ "symbol": symbol }), error.message)
            }),
    );
    evidence.push(
        call_ryo_tool(
            state,
            "deep_analysis",
            json!({ "symbol": symbol, "include_perp": true }),
        )
        .await
        .unwrap_or_else(|error| {
            unavailable_ryo(
                "deep_analysis",
                json!({ "symbol": symbol, "include_perp": true }),
                error.message,
            )
        }),
    );
    evidence.push(
        call_ryo_tool(
            state,
            "compare_tokens",
            json!({ "symbols": peer_symbols(symbol), "intent": "swing" }),
        )
        .await
        .unwrap_or_else(|error| {
            unavailable_ryo(
                "compare_tokens",
                json!({ "symbols": peer_symbols(symbol), "intent": "swing" }),
                error.message,
            )
        }),
    );
    evidence.push(
        call_ryo_tool(state, "monitor_market_sentiment_shift", json!({}))
            .await
            .unwrap_or_else(|error| {
                unavailable_ryo("monitor_market_sentiment_shift", json!({}), error.message)
            }),
    );
    evidence
}

/// Simulated RYO evidence for local demos before the builder key arrives.
///
/// The public RYO contract allows `data_mode=simulated`; we use that explicitly
/// and mark every mocked tool `partial` so the receipt cannot be mistaken for a
/// live RYO-confirmed decision.
fn mock_ryo_evidence(symbol: &str) -> Vec<RyoToolEvidence> {
    vec![
        mock_ryo_tool(
            "market_overview",
            json!({}),
            "Simulated market regime is mixed: breadth is cautious, Fear & Greed is neutral, and top movers are uneven.",
            json!({
                "regime": "mixed",
                "fear_greed": { "value": 52, "label": "neutral", "simulated": true },
                "breadth": { "advancing": 46, "declining": 54 },
                "top_movers": ["BTC", "ETH", symbol]
            }),
        ),
        mock_ryo_tool(
            "scan_market",
            json!({ "theme": format!("{symbol} global news impact"), "limit": 12 }),
            &format!(
                "Simulated scan ranks {symbol} as a watch candidate, not a confirmed market leader."
            ),
            json!({
                "theme": format!("{symbol} global news impact"),
                "candidates": [
                    { "symbol": symbol, "rank": 2, "reason": "news-linked momentum requires live RYO confirmation" },
                    { "symbol": "BTC", "rank": 1, "reason": "broad-market anchor" },
                    { "symbol": "ETH", "rank": 3, "reason": "market beta comparison" }
                ]
            }),
        ),
        mock_ryo_tool(
            "analyze_token",
            json!({ "symbol": symbol }),
            &format!(
                "Simulated {symbol} analysis shows positive short-window momentum but incomplete confirmation."
            ),
            json!({
                "symbol": symbol,
                "performance": { "24h": 2.4, "7d": 5.8 },
                "technicals": { "rsi_14": 58, "atr_14": "moderate" },
                "verdict": "watchlist"
            }),
        ),
        mock_ryo_tool(
            "deep_analysis",
            json!({ "symbol": symbol, "include_perp": true }),
            &format!(
                "Simulated deep analysis for {symbol} finds a watchable setup with derivatives evidence unavailable."
            ),
            json!({
                "symbol": symbol,
                "confluence": "moderate",
                "catalysts": ["global news attention", "market momentum"],
                "risks": ["live RYO key missing", "derivatives evidence unavailable"],
                "preview_plan": { "stance": "watch", "risk": "medium" }
            }),
        ),
        mock_ryo_tool(
            "compare_tokens",
            json!({ "symbols": peer_symbols(symbol), "intent": "swing" }),
            &format!(
                "Simulated comparison keeps {symbol} on watch against BTC, ETH and BNB/SOL peers."
            ),
            json!({
                "symbols": peer_symbols(symbol),
                "intent": "swing",
                "leader": "BTC",
                "watched": symbol,
                "coverage": "simulated common factors only"
            }),
        ),
        mock_ryo_tool(
            "monitor_market_sentiment_shift",
            json!({}),
            "Simulated seven-day sentiment shift is slightly improving but not strong enough for live confirmation.",
            json!({
                "time_window": "7d",
                "fear_greed_change": 6,
                "market_phase": "recovering",
                "combined_sentiment": "cautious-positive"
            }),
        ),
    ]
}

fn mock_ryo_tool(tool: &str, request: Value, summary: &str, data: Value) -> RyoToolEvidence {
    let result = json!({
        "schema_version": "mock-2026-08-13",
        "tool": tool,
        "status": "partial",
        "data_mode": "simulated",
        "as_of": Utc::now().to_rfc3339(),
        "request": request,
        "data": data,
        "summary": { "headline": summary },
        "availability": [
            {
                "source": "RYO mock",
                "status": "partial",
                "data_mode": "simulated",
                "detail": "APP_MOCK_RYO=true and RYO_MCP_KEY is not configured."
            }
        ],
        "warnings": [
            "Simulated RYO evidence for local demo only. Do not present as live RYO confirmation.",
            "Replace with the authenticated RYO MCP catalog before final judging."
        ]
    });
    RyoToolEvidence {
        tool: tool.to_string(),
        status: "partial".to_string(),
        data_mode: "simulated".to_string(),
        as_of: Utc::now().to_rfc3339(),
        request,
        summary: Some(summary.to_string()),
        warnings: vec![
            "Simulated RYO evidence for local demo only. Do not present as live RYO confirmation."
                .to_string(),
            "Replace with the authenticated RYO MCP catalog before final judging.".to_string(),
        ],
        result,
    }
}

/// Build a small peer set for `compare_tokens`, starting with the symbol.
fn peer_symbols(symbol: &str) -> Vec<String> {
    let mut peers = vec![symbol.to_string()];
    for candidate in ["BTC", "ETH", "BNB", "SOL"] {
        if candidate != symbol && peers.len() < 4 {
            peers.push(candidate.to_string());
        }
    }
    peers
}

/// GET a RYO MCP endpoint (used for the live `/tools` catalog).
pub(crate) async fn ryo_get(state: &AppState, path: &str) -> Result<Value, ApiError> {
    let key = state.config.ryo_mcp_key.as_deref().ok_or_else(|| {
        ApiError::new(
            StatusCode::PRECONDITION_REQUIRED,
            "missing_ryo_key",
            "RYO_MCP_KEY is not configured.",
        )
    })?;
    let url = format!(
        "{}/{}",
        state.config.ryo_mcp_url,
        path.trim_start_matches('/')
    );
    let response = state
        .client
        .get(url)
        .bearer_auth(key)
        .send()
        .await
        .map_err(|error| {
            ApiError::new(
                StatusCode::BAD_GATEWAY,
                "ryo_network_error",
                error.to_string(),
            )
        })?;
    parse_response(response, "ryo_error").await
}

/// POST to a single RYO tool and capture its result as `RyoToolEvidence`.
async fn call_ryo_tool(
    state: &AppState,
    tool: &str,
    arguments: Value,
) -> Result<RyoToolEvidence, ApiError> {
    let key = state.config.ryo_mcp_key.as_deref().ok_or_else(|| {
        ApiError::new(
            StatusCode::PRECONDITION_REQUIRED,
            "missing_ryo_key",
            "RYO_MCP_KEY is not configured.",
        )
    })?;
    let url = format!("{}/tools/{tool}/call", state.config.ryo_mcp_url);
    let response = state
        .client
        .post(url)
        .bearer_auth(key)
        .json(&arguments)
        .send()
        .await
        .map_err(|error| {
            ApiError::new(
                StatusCode::BAD_GATEWAY,
                "ryo_network_error",
                error.to_string(),
            )
        })?;
    let body = parse_response(response, "ryo_error").await?;
    let result = body.get("result").cloned().unwrap_or(body);
    Ok(RyoToolEvidence {
        tool: tool.to_string(),
        status: extract_string(&result, "status").unwrap_or_else(|| "ok".to_string()),
        data_mode: extract_string(&result, "data_mode").unwrap_or_else(|| "live".to_string()),
        as_of: extract_string(&result, "as_of").unwrap_or_else(|| Utc::now().to_rfc3339()),
        request: arguments,
        summary: extract_summary(&result),
        warnings: extract_warnings(&result),
        result,
    })
}

/// Build an `unavailable` evidence record so a failed call is explicit.
pub(crate) fn unavailable_ryo(tool: &str, request: Value, reason: String) -> RyoToolEvidence {
    RyoToolEvidence {
        tool: tool.to_string(),
        status: "unavailable".to_string(),
        data_mode: "unknown".to_string(),
        as_of: Utc::now().to_rfc3339(),
        request,
        result: Value::Null,
        summary: Some(format!("{tool} unavailable")),
        warnings: vec![reason],
    }
}

/// Read a string field from a JSON object.
fn extract_string(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_string)
}

/// Pull a human summary out of a RYO result, however it is shaped.
fn extract_summary(value: &Value) -> Option<String> {
    value
        .get("summary")
        .and_then(|summary| {
            summary
                .as_str()
                .map(str::to_string)
                .or_else(|| {
                    summary
                        .get("headline")
                        .and_then(Value::as_str)
                        .map(str::to_string)
                })
                .or_else(|| serde_json::to_string(summary).ok())
        })
        .or_else(|| {
            value
                .get("message")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
}

/// Collect any `warnings` array from a RYO result.
fn extract_warnings(value: &Value) -> Vec<String> {
    value
        .get("warnings")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect()
}

/// Convert a RYO tool result into an availability line for the receipt.
pub(crate) fn availability_from_ryo(tool: &RyoToolEvidence) -> SourceAvailability {
    SourceAvailability {
        source: format!("RYO {}", tool.tool),
        status: tool.status.clone(),
        data_mode: tool.data_mode.clone(),
        detail: tool
            .summary
            .clone()
            .unwrap_or_else(|| format!("{} returned {}", tool.tool, tool.status)),
        as_of: tool.as_of.clone(),
    }
}
