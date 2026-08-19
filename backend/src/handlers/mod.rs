//! HTTP handlers and the router that wires them to paths.
//!
//! Handlers stay thin: they validate input, call into `crate::reasoning` or
//! `crate::services`, and shape the response. The route table lives in
//! `router` so `main` only has to mount it.

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
};
use chrono::{Duration as ChronoDuration, Utc};
use serde_json::{Value, json};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};

use crate::domain::{
    DecisionReceipt, HealthResponse, PulseRequest, ReceiptListItem, WatchItem, WatchRequest,
    default_tokens,
};
use crate::error::ApiError;
use crate::reasoning::{build_pulse, normalize_symbol};
use crate::services::ryo::ryo_get;
use crate::state::AppState;

/// Build the application router with all API routes, the static file fallback,
/// and the shared middleware (tracing + permissive CORS for local use).
pub(crate) fn router(state: AppState, static_dir: String) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/health", get(health))
        .route("/api/tokens", get(tokens))
        .route("/api/ryo/tools", get(ryo_tools))
        .route("/api/pulse", post(create_pulse))
        .route("/api/reason", post(create_pulse))
        .route("/api/receipts", get(list_receipts))
        .route("/api/receipts/{id}", get(get_receipt))
        .route("/reason", post(create_pulse))
        .route("/runs", get(list_receipts))
        .route("/runs/{id}", get(get_receipt))
        .route("/api/watchlist", get(list_watchlist).post(add_watchlist))
        .route("/api/watchlist/{symbol}", delete(remove_watchlist))
        .fallback_service(ServeDir::new(static_dir).append_index_html_on_directories(true))
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state)
}

/// Report which integrations are configured. Always 200.
async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "global-token-news-pulse",
        generated_at: Utc::now().to_rfc3339(),
        ryo_configured: state.config.ryo_mcp_key.is_some(),
        ryo_mock_enabled: state.config.ryo_mock_enabled,
        tavily_configured: state.config.tavily_api_key.is_some(),
        openai_configured: state.config.openai_api_key.is_some(),
        coingecko_configured: state.config.coingecko_demo_api_key.is_some(),
        defillama_enabled: state.config.defillama_enabled,
        dexscreener_enabled: state.config.dexscreener_enabled,
        watch_loop_enabled: state.config.watch_loop_enabled,
    })
}

/// The static UI token list. RYO remains the source of market intelligence.
async fn tokens() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "data_mode": "static-reference",
        "tokens": default_tokens(),
        "note": "This starter list exists for UI convenience. RYO MCP/REST remains the source for market intelligence."
    }))
}

/// Proxy the live RYO `/tools` catalog, or an empty list if the key is unset.
async fn ryo_tools(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    if state.config.ryo_mcp_key.is_none() {
        if state.config.ryo_mock_enabled {
            return Ok(Json(json!({
                "status": "partial",
                "data_mode": "simulated",
                "tools": [
                    { "name": "market_overview", "description": "Mock market regime, totals, sentiment, breadth and movers." },
                    { "name": "scan_market", "description": "Mock ranked market shortlist." },
                    { "name": "analyze_token", "description": "Mock token market and technical analysis." },
                    { "name": "deep_analysis", "description": "Mock comprehensive token evidence pack." },
                    { "name": "compare_tokens", "description": "Mock comparison for two to four assets." },
                    { "name": "monitor_market_sentiment_shift", "description": "Mock seven-day sentiment shift." }
                ],
                "warnings": [
                    "APP_MOCK_RYO=true and RYO_MCP_KEY is not configured.",
                    "This catalog is simulated for local demo only."
                ]
            })));
        }
        return Ok(Json(json!({
            "status": "unavailable",
            "data_mode": "unknown",
            "tools": [],
            "warnings": ["RYO_MCP_KEY is not configured. Add it to .env for the live tool catalog."]
        })));
    }
    ryo_get(&state, "tools").await.map(Json)
}

/// Run a reasoning pass and store the receipt at the front of history.
async fn create_pulse(
    State(state): State<AppState>,
    Json(request): Json<PulseRequest>,
) -> Result<Json<DecisionReceipt>, ApiError> {
    let receipt = build_pulse(&state, request).await?;
    state.receipts.write().await.insert(0, receipt.clone());
    Ok(Json(receipt))
}

/// List up to the 30 most recent receipts as compact entries.
async fn list_receipts(State(state): State<AppState>) -> Json<Vec<ReceiptListItem>> {
    let receipts = state.receipts.read().await;
    let items = receipts
        .iter()
        .take(30)
        .map(|receipt| ReceiptListItem {
            id: receipt.id.clone(),
            run_id: receipt.run_id.clone(),
            symbol: receipt.symbol.clone(),
            created_at: receipt.created_at.clone(),
            status: receipt.status.clone(),
            data_mode: receipt.data_mode.clone(),
            signal: receipt.signal.clone(),
            confidence: receipt.confidence,
            headline: receipt.summary.headline.clone(),
            highest_impact: receipt.summary.highest_impact,
        })
        .collect();
    Json(items)
}

/// Fetch one full receipt by `id` or `run_id`.
async fn get_receipt(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DecisionReceipt>, ApiError> {
    let receipts = state.receipts.read().await;
    let receipt = receipts
        .iter()
        .find(|receipt| receipt.id == id || receipt.run_id == id)
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::NOT_FOUND,
                "receipt_not_found",
                "No decision receipt exists for that id in this running process.",
            )
        })?;
    Ok(Json(receipt.clone()))
}

/// List watched tokens, sorted by symbol.
async fn list_watchlist(State(state): State<AppState>) -> Json<Vec<WatchItem>> {
    let mut items = state
        .watchlist
        .read()
        .await
        .values()
        .cloned()
        .collect::<Vec<_>>();
    items.sort_by(|a, b| a.symbol.cmp(&b.symbol));
    Json(items)
}

/// Add or replace a watched token, clamping the interval to 5-360 minutes.
async fn add_watchlist(
    State(state): State<AppState>,
    Json(request): Json<WatchRequest>,
) -> Result<Json<WatchItem>, ApiError> {
    let symbol = normalize_symbol(&request.symbol)?;
    let interval_minutes = request.interval_minutes.unwrap_or(15).clamp(5, 360);
    let now = Utc::now();
    let next = now + ChronoDuration::minutes(interval_minutes as i64);
    let item = WatchItem {
        symbol: symbol.clone(),
        interval_minutes,
        enabled: true,
        created_at: now.to_rfc3339(),
        last_checked_at: None,
        next_check_at: Some(next.to_rfc3339()),
        last_receipt_id: None,
        last_cluster: None,
    };
    state.watchlist.write().await.insert(symbol, item.clone());
    Ok(Json(item))
}

/// Remove a watched token by symbol.
async fn remove_watchlist(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let symbol = normalize_symbol(&symbol)?;
    state.watchlist.write().await.remove(&symbol);
    Ok(Json(json!({ "status": "ok", "removed": symbol })))
}
