use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    env,
    net::SocketAddr,
    sync::Arc,
    time::Duration,
};
use tokio::sync::RwLock;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    config: Arc<Config>,
    client: Client,
    receipts: Arc<RwLock<Vec<DecisionReceipt>>>,
    watchlist: Arc<RwLock<HashMap<String, WatchItem>>>,
    news_cache: Arc<RwLock<HashMap<String, CacheEntry<Vec<NewsStory>>>>>,
}

#[derive(Clone)]
struct Config {
    ryo_mcp_url: String,
    ryo_mcp_key: Option<String>,
    tavily_api_key: Option<String>,
    openai_api_key: Option<String>,
    openai_model: String,
    openai_responses_url: String,
    coingecko_demo_api_key: Option<String>,
    defillama_enabled: bool,
    dexscreener_enabled: bool,
    port: u16,
    watch_loop_enabled: bool,
}

#[derive(Debug, Clone)]
struct CacheEntry<T> {
    value: T,
    data_mode: String,
    expires_at: DateTime<Utc>,
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
    detail: Option<Value>,
}

#[derive(Debug, Deserialize, Clone)]
struct PulseRequest {
    symbol: String,
    #[serde(default)]
    timeframe: Option<String>,
    #[serde(default)]
    regions: Option<Vec<String>>,
    #[serde(default)]
    sources: Option<Vec<String>>,
    #[serde(default)]
    thesis: Option<String>,
    #[serde(default)]
    event: Option<String>,
    #[serde(default)]
    news: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WatchRequest {
    symbol: String,
    #[serde(default)]
    interval_minutes: Option<u64>,
}

#[derive(Debug, Serialize, Clone)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
    generated_at: String,
    ryo_configured: bool,
    tavily_configured: bool,
    openai_configured: bool,
    coingecko_configured: bool,
    defillama_enabled: bool,
    dexscreener_enabled: bool,
    watch_loop_enabled: bool,
}

#[derive(Debug, Serialize, Clone)]
struct TokenInfo {
    symbol: &'static str,
    name: &'static str,
    default_peers: Vec<&'static str>,
}

#[derive(Debug, Serialize, Clone)]
struct SourceAvailability {
    source: String,
    status: String,
    data_mode: String,
    detail: String,
    as_of: String,
}

#[derive(Debug, Serialize, Clone)]
struct NewsStory {
    id: String,
    headline: String,
    source: String,
    url: String,
    content: String,
    published_at: Option<String>,
    region: Option<String>,
    language: Option<String>,
    data_mode: String,
}

#[derive(Debug, Serialize, Clone)]
struct RyoToolEvidence {
    tool: String,
    status: String,
    data_mode: String,
    as_of: String,
    request: Value,
    result: Value,
    summary: Option<String>,
    warnings: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
struct StoryScore {
    relevance: Option<u8>,
    credibility: Option<u8>,
    novelty: Option<u8>,
    urgency: Option<u8>,
    market_confirmation: Option<u8>,
    uncertainty: Option<u8>,
    impact: u8,
    formula: String,
}

#[derive(Debug, Serialize, Clone)]
struct StoryCard {
    id: String,
    headline: String,
    source: String,
    url: String,
    region: Option<String>,
    language: Option<String>,
    timestamp: Option<String>,
    related_token: String,
    narrative_cluster: String,
    sentiment: String,
    score: StoryScore,
    ryo_alignment: String,
    missing_data: Vec<String>,
    recommendation: String,
    confidence: String,
    reasoning: Vec<String>,
    category: String,
    data_mode: String,
}

#[derive(Debug, Serialize, Clone)]
struct RankedSection {
    key: String,
    label: String,
    cards: Vec<StoryCard>,
}

#[derive(Debug, Serialize, Clone)]
struct ReceiptSummary {
    headline: String,
    conclusion: String,
    highest_impact: Option<u8>,
    position_changing_count: usize,
    watch_count: usize,
    noise_count: usize,
    unverified_count: usize,
}

#[derive(Debug, Serialize, Clone, Default)]
struct MarketPulse {
    fear_greed_value: Option<u8>,
    fear_greed_label: Option<String>,
    regime: Option<String>,
    btc_dominance: Option<f64>,
    total_market_cap_change: Option<f64>,
    breadth: Option<String>,
    as_of: Option<String>,
    available: bool,
    note: String,
}

#[derive(Debug, Serialize, Clone)]
struct ReasoningVerdict {
    decision: String,
    confidence: u8,
    token_symbol: String,
    top_global_news: Vec<String>,
    why_it_matters: Vec<String>,
    ryo_market_evidence: Vec<String>,
    missing_data: Vec<String>,
    warnings: Vec<String>,
    recommended_next_action: String,
    generated_by: String,
}

#[derive(Debug, Serialize, Clone)]
struct ReasoningLayerOutput {
    signal: String,
    symbol: String,
    confidence: f32,
    reasoning: String,
    ryo_tools_used: Vec<String>,
    unavailable_data: Vec<String>,
    next_action: String,
    affected_tokens: Vec<String>,
    sentiment: String,
    market_confirmation: String,
    timestamp: String,
    run_id: String,
}

#[derive(Debug, Serialize, Clone)]
struct DecisionReceipt {
    id: String,
    run_id: String,
    symbol: String,
    timeframe: String,
    regions: Vec<String>,
    sources: Vec<String>,
    user_thesis: Option<String>,
    created_at: String,
    status: String,
    data_mode: String,
    signal: String,
    confidence: f32,
    reasoning: String,
    ryo_tools_used: Vec<String>,
    unavailable_data: Vec<String>,
    next_action: String,
    reasoning_layer: ReasoningLayerOutput,
    summary: ReceiptSummary,
    verdict: ReasoningVerdict,
    market_pulse: MarketPulse,
    recommendations: Vec<String>,
    editor_note: String,
    sections: Vec<RankedSection>,
    stories: Vec<NewsStory>,
    ryo: Vec<RyoToolEvidence>,
    availability: Vec<SourceAvailability>,
    warnings: Vec<String>,
    scoring_notes: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
struct ReceiptListItem {
    id: String,
    run_id: String,
    symbol: String,
    created_at: String,
    status: String,
    data_mode: String,
    signal: String,
    confidence: f32,
    headline: String,
    highest_impact: Option<u8>,
}

#[derive(Debug, Serialize, Clone)]
struct WatchItem {
    symbol: String,
    interval_minutes: u64,
    enabled: bool,
    created_at: String,
    last_checked_at: Option<String>,
    next_check_at: Option<String>,
    last_receipt_id: Option<String>,
    last_cluster: Option<String>,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "global_token_news_pulse=info,tower_http=warn".into()),
        )
        .init();

    let config = Arc::new(Config::from_env());
    let client = Client::builder()
        .timeout(Duration::from_secs(45))
        .build()
        .expect("reqwest client");
    let state = AppState {
        config: config.clone(),
        client,
        receipts: Arc::new(RwLock::new(Vec::new())),
        watchlist: Arc::new(RwLock::new(HashMap::new())),
        news_cache: Arc::new(RwLock::new(HashMap::new())),
    };

    if config.watch_loop_enabled {
        tokio::spawn(watch_loop(state.clone()));
    }

    let static_dir = format!("{}/static", env!("CARGO_MANIFEST_DIR"));
    let app = Router::new()
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
        .with_state(state);

    let addr: SocketAddr = ([127, 0, 0, 1], config.port).into();
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind local server");

    info!(
        "RYO Global Token Reasoning Layer listening on http://{}",
        addr
    );
    axum::serve(listener, app).await.expect("server");
}

impl Config {
    fn from_env() -> Self {
        let ryo_mcp_key = env_optional("RYO_MCP_KEY");
        let tavily_api_key = env_optional("TAVILY_API_KEY");
        let openai_api_key = env_optional("OPENAI_API_KEY");

        Self {
            ryo_mcp_url: env::var("RYO_MCP_URL")
                .unwrap_or_else(|_| "https://app-ryochan.com/api/mcp".to_string())
                .trim_end_matches('/')
                .to_string(),
            ryo_mcp_key,
            tavily_api_key,
            openai_api_key,
            openai_model: env_optional("OPENAI_MODEL")
                .unwrap_or_else(|| "gpt-5.6-luna".to_string()),
            openai_responses_url: env_optional("OPENAI_RESPONSES_URL")
                .unwrap_or_else(|| "https://api.openai.com/v1/responses".to_string()),
            coingecko_demo_api_key: env_optional("COINGECKO_DEMO_API_KEY"),
            defillama_enabled: env_bool("DEFILLAMA_ENABLED", false),
            dexscreener_enabled: env_bool("DEXSCREENER_ENABLED", false),
            port: env::var("APP_PORT")
                .or_else(|_| env::var("PORT"))
                .ok()
                .and_then(|value| value.parse::<u16>().ok())
                .unwrap_or(8788),
            watch_loop_enabled: env_bool("APP_ENABLE_WATCH_LOOP", false),
        }
    }

    fn missing_required_keys(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if self.ryo_mcp_key.is_none() {
            missing.push("RYO_MCP_KEY");
        }
        if self.tavily_api_key.is_none() {
            missing.push("TAVILY_API_KEY");
        }
        if self.openai_api_key.is_none() {
            missing.push("OPENAI_API_KEY");
        }
        missing
    }
}

fn env_optional(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn env_bool(name: &str, default: bool) -> bool {
    env::var(name)
        .ok()
        .map(|value| matches!(value.to_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(default)
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = json!({
            "error": {
                "code": self.code,
                "message": self.message,
                "detail": self.detail,
            }
        });
        (self.status, Json(body)).into_response()
    }
}

impl ApiError {
    fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
            detail: None,
        }
    }

    fn with_detail(mut self, detail: Value) -> Self {
        self.detail = Some(detail);
        self
    }
}

async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "global-token-news-pulse",
        generated_at: Utc::now().to_rfc3339(),
        ryo_configured: state.config.ryo_mcp_key.is_some(),
        tavily_configured: state.config.tavily_api_key.is_some(),
        openai_configured: state.config.openai_api_key.is_some(),
        coingecko_configured: state.config.coingecko_demo_api_key.is_some(),
        defillama_enabled: state.config.defillama_enabled,
        dexscreener_enabled: state.config.dexscreener_enabled,
        watch_loop_enabled: state.config.watch_loop_enabled,
    })
}

async fn tokens() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "data_mode": "static-reference",
        "tokens": default_tokens(),
        "note": "This starter list exists for UI convenience. RYO MCP/REST remains the source for market intelligence."
    }))
}

async fn ryo_tools(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    if state.config.ryo_mcp_key.is_none() {
        return Ok(Json(json!({
            "status": "unavailable",
            "data_mode": "unknown",
            "tools": [],
            "warnings": ["RYO_MCP_KEY is not configured. Add it to .env for the live tool catalog."]
        })));
    }
    ryo_get(&state, "tools").await.map(Json)
}

async fn create_pulse(
    State(state): State<AppState>,
    Json(request): Json<PulseRequest>,
) -> Result<Json<DecisionReceipt>, ApiError> {
    let receipt = build_pulse(&state, request).await?;
    state.receipts.write().await.insert(0, receipt.clone());
    Ok(Json(receipt))
}

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

async fn remove_watchlist(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let symbol = normalize_symbol(&symbol)?;
    state.watchlist.write().await.remove(&symbol);
    Ok(Json(json!({ "status": "ok", "removed": symbol })))
}

async fn build_pulse(state: &AppState, request: PulseRequest) -> Result<DecisionReceipt, ApiError> {
    let missing_keys = state.config.missing_required_keys();
    if !missing_keys.is_empty() {
        return Err(ApiError::new(
            StatusCode::PRECONDITION_REQUIRED,
            "missing_required_keys",
            "Reasoning runs are disabled until real API keys are configured. No mock data will be generated.",
        )
        .with_detail(json!({ "missing_keys": missing_keys })));
    }

    let symbol = normalize_symbol(&request.symbol)?;
    let timeframe = normalize_timeframe(request.timeframe.as_deref());
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
    let mut stories = fetch_news(
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
    });
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

    let ryo = call_ryo_evidence(state, &symbol).await;
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
    let (verdict, ai_warning) = build_reasoning_verdict(
        state,
        &symbol,
        &summary,
        &sections,
        stories.as_slice(),
        ryo.as_slice(),
        receipt_warnings.as_slice(),
    )
    .await;
    if let Some(warning) = ai_warning {
        receipt_warnings.push(warning);
    }
    let reasoning_layer = build_reasoning_layer_output(
        &symbol,
        &run_id,
        &created_at,
        &sections,
        &verdict,
        ryo.as_slice(),
    );

    let market_pulse = build_market_pulse(ryo.as_slice());
    let recommendations =
        build_recommendations(&symbol, &verdict, &summary, &market_pulse, &sections);
    let editor_note = build_editor_note(&symbol, &verdict, &market_pulse, stories.len());

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
        reasoning_layer,
        summary,
        verdict,
        market_pulse,
        recommendations,
        editor_note,
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

fn default_tokens() -> Vec<TokenInfo> {
    vec![
        TokenInfo {
            symbol: "BTC",
            name: "Bitcoin",
            default_peers: vec!["ETH", "BNB", "SOL"],
        },
        TokenInfo {
            symbol: "ETH",
            name: "Ethereum",
            default_peers: vec!["BTC", "BNB", "SOL"],
        },
        TokenInfo {
            symbol: "BNB",
            name: "BNB",
            default_peers: vec!["BTC", "ETH", "SOL"],
        },
        TokenInfo {
            symbol: "SOL",
            name: "Solana",
            default_peers: vec!["BTC", "ETH", "BNB"],
        },
        TokenInfo {
            symbol: "LINK",
            name: "Chainlink",
            default_peers: vec!["ETH", "BTC", "BNB"],
        },
        TokenInfo {
            symbol: "ARB",
            name: "Arbitrum",
            default_peers: vec!["ETH", "OP", "BASE"],
        },
        TokenInfo {
            symbol: "CAKE",
            name: "PancakeSwap",
            default_peers: vec!["BNB", "UNI", "AAVE"],
        },
    ]
}

fn normalize_symbol(symbol: &str) -> Result<String, ApiError> {
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

fn compact_headline(value: &str, max_chars: usize) -> String {
    let mut compact = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.chars().count() > max_chars {
        compact = compact.chars().take(max_chars.saturating_sub(1)).collect();
        compact.push_str("...");
    }
    compact
}

async fn fetch_news(
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
            data_mode: cached.data_mode.clone(),
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
                data_mode: "live".to_string(),
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

fn news_cache_key(symbol: &str, timeframe: &str, regions: &[String], sources: &[String]) -> String {
    format!(
        "{}:{}:{}:{}",
        symbol,
        timeframe,
        regions.join(",").to_lowercase(),
        sources.join(",").to_lowercase()
    )
}

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
        .map(|(index, item)| NewsStory {
            id: format!("news-{}", index + 1),
            headline: item
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or("Untitled source")
                .to_string(),
            source: host_from_url(item.get("url").and_then(Value::as_str).unwrap_or("")),
            url: item
                .get("url")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            content: item
                .get("content")
                .or_else(|| item.get("snippet"))
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            published_at: item
                .get("published_date")
                .or_else(|| item.get("published_at"))
                .and_then(Value::as_str)
                .map(str::to_string),
            region: infer_region(item.get("url").and_then(Value::as_str).unwrap_or("")),
            language: None,
            data_mode: "live".to_string(),
        })
        .collect();
    Ok(stories)
}

fn dedupe_stories(stories: Vec<NewsStory>) -> Vec<NewsStory> {
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

async fn call_ryo_evidence(state: &AppState, symbol: &str) -> Vec<RyoToolEvidence> {
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

fn peer_symbols(symbol: &str) -> Vec<String> {
    let mut peers = vec![symbol.to_string()];
    for candidate in ["BTC", "ETH", "BNB", "SOL"] {
        if candidate != symbol && peers.len() < 4 {
            peers.push(candidate.to_string());
        }
    }
    peers
}

async fn ryo_get(state: &AppState, path: &str) -> Result<Value, ApiError> {
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

async fn parse_response(
    response: reqwest::Response,
    code: &'static str,
) -> Result<Value, ApiError> {
    let status = response.status();
    let text = response.text().await.map_err(|error| {
        ApiError::new(
            StatusCode::BAD_GATEWAY,
            "response_read_error",
            error.to_string(),
        )
    })?;
    let value = serde_json::from_str::<Value>(&text).unwrap_or_else(|_| json!({ "raw": text }));
    if !status.is_success() {
        return Err(ApiError::new(status, code, "Upstream request failed.").with_detail(value));
    }
    Ok(value)
}

fn unavailable_ryo(tool: &str, request: Value, reason: String) -> RyoToolEvidence {
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

fn extract_string(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_string)
}

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

fn availability_from_ryo(tool: &RyoToolEvidence) -> SourceAvailability {
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

fn score_stories(symbol: &str, stories: &[NewsStory], ryo: &[RyoToolEvidence]) -> Vec<StoryCard> {
    stories
        .iter()
        .map(|story| score_story(symbol, story, stories, ryo))
        .collect()
}

fn score_story(
    symbol: &str,
    story: &NewsStory,
    all_stories: &[NewsStory],
    ryo: &[RyoToolEvidence],
) -> StoryCard {
    let text = format!("{} {}", story.headline, story.content);
    let lower = text.to_lowercase();
    let cluster = narrative_cluster(&lower);
    let sentiment = sentiment(&lower);
    let relevance = relevance_score(symbol, &lower);
    let credibility = credibility_score(&story.source);
    let urgency = urgency_score(&lower, story.published_at.as_deref());
    let novelty = novelty_score(&cluster, all_stories);
    let ryo_available = ryo.iter().any(|item| item.status != "unavailable");
    let market_confirmation = ryo_available.then(|| market_confirmation_score(&lower, ryo));
    let uncertainty = uncertainty_score(&lower, ryo);

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
    let category = if contradicted || credibility.unwrap_or(0) < 45 {
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
    let reasoning = reasoning_points(
        symbol,
        &cluster,
        &sentiment,
        impact,
        market_confirmation,
        &missing_data,
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
    }
}

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

fn relevance_score(symbol: &str, lower: &str) -> Option<u8> {
    let symbol_lower = symbol.to_lowercase();
    if lower.contains(&symbol_lower) {
        Some(92)
    } else if token_alias(symbol)
        .iter()
        .any(|alias| lower.contains(alias))
    {
        Some(82)
    } else if lower.contains("crypto") || lower.contains("token") || lower.contains("market") {
        Some(50)
    } else {
        Some(25)
    }
}

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

fn credibility_score(source: &str) -> Option<u8> {
    let source = source.to_lowercase();
    let score = if source.contains("reuters")
        || source.contains("bloomberg")
        || source.contains("sec.gov")
        || source.contains("federalreserve.gov")
    {
        92
    } else if source.contains("coindesk")
        || source.contains("cointelegraph")
        || source.contains("theblock")
        || source.contains("decrypt")
        || source.contains("blockworks")
    {
        82
    } else if source.contains("blog") || source.contains("medium") || source.contains("substack") {
        58
    } else if source.is_empty() {
        45
    } else {
        66
    };
    Some(score)
}

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

fn uncertainty_score(lower: &str, ryo: &[RyoToolEvidence]) -> Option<u8> {
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
    Some(score.min(100))
}

fn count_hits(text: &str, words: &[&str]) -> usize {
    words.iter().filter(|word| text.contains(**word)).count()
}

fn narrative_cluster(lower: &str) -> String {
    let lower = lower.to_lowercase();
    let clusters = [
        (
            "security / exploit risk",
            ["exploit", "hack", "rug", "scam", "phishing", "bridge"].as_slice(),
        ),
        (
            "regulation / policy",
            ["sec", "regulation", "law", "lawsuit", "policy", "court"].as_slice(),
        ),
        (
            "exchange / liquidity",
            ["listing", "delist", "exchange", "liquidity", "volume"].as_slice(),
        ),
        (
            "macro / risk regime",
            ["rates", "cpi", "fomc", "inflation", "macro", "risk-off"].as_slice(),
        ),
        (
            "ecosystem / adoption",
            [
                "partnership",
                "integration",
                "upgrade",
                "developer",
                "mainnet",
            ]
            .as_slice(),
        ),
        (
            "derivatives / leverage",
            [
                "perp",
                "funding",
                "liquidation",
                "open interest",
                "short squeeze",
            ]
            .as_slice(),
        ),
        (
            "tokenomics / unlock",
            ["unlock", "supply", "emission", "burn", "staking", "airdrop"].as_slice(),
        ),
    ];
    for (label, words) in clusters {
        if words.iter().any(|word| lower.contains(word)) {
            return label.to_string();
        }
    }
    "general market narrative".to_string()
}

fn sentiment(lower: &str) -> String {
    let bullish = count_hits(
        lower,
        &[
            "adoption",
            "approved",
            "breakout",
            "bull",
            "bullish",
            "buy",
            "listing",
            "momentum",
            "partnership",
            "rally",
            "support",
            "upgrade",
            "upside",
        ],
    );
    let bearish = count_hits(
        lower,
        &[
            "bear", "bearish", "delist", "downside", "exploit", "hack", "lawsuit", "risk", "scam",
            "sell", "weak",
        ],
    );
    if bullish > bearish + 1 {
        "bullish".to_string()
    } else if bearish > bullish + 1 {
        "bearish".to_string()
    } else {
        "mixed".to_string()
    }
}

fn recommendation_for(category: &str, sentiment: &str, missing_data: &[String]) -> String {
    if !missing_data.is_empty() && category == "unverified" {
        return "investigate before acting".to_string();
    }
    match (category, sentiment) {
        ("position-changing", "bullish") => "watch for confirmed setup".to_string(),
        ("position-changing", "bearish") => "reduce risk or avoid new exposure".to_string(),
        ("position-changing", _) => "review position assumptions".to_string(),
        ("watch", _) => "watch closely".to_string(),
        ("unverified", _) => "wait for verification".to_string(),
        _ => "no action".to_string(),
    }
}

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

fn reasoning_points(
    symbol: &str,
    cluster: &str,
    sentiment: &str,
    impact: u8,
    market_confirmation: Option<u8>,
    missing_data: &[String],
) -> Vec<String> {
    let mut points = vec![
        format!(
            "The story maps to {cluster}, which can affect {symbol} if it changes liquidity, risk appetite or token-specific demand."
        ),
        format!(
            "Narrative sentiment is {sentiment}; impact score is {impact}/100 after weighting available evidence."
        ),
    ];
    match market_confirmation {
        Some(score) => points.push(format!("RYO market confirmation contributes {score}/100 to the score.")),
        None => points.push("RYO market confirmation was unavailable and was excluded from scoring, not treated as zero.".to_string()),
    }
    if !missing_data.is_empty() {
        points.push(format!("Missing fields: {}.", missing_data.join(", ")));
    }
    points
}

fn rank_sections(cards: Vec<StoryCard>) -> Vec<RankedSection> {
    let mut buckets: BTreeMap<String, Vec<StoryCard>> = BTreeMap::new();
    for card in cards {
        buckets.entry(card.category.clone()).or_default().push(card);
    }
    for cards in buckets.values_mut() {
        cards.sort_by_key(|card| std::cmp::Reverse(card.score.impact));
    }
    let order = [
        ("position-changing", "Position-changing"),
        ("watch", "Watch closely"),
        ("unverified", "Unverified or conflicting"),
        ("noise", "Noise"),
    ];
    order
        .iter()
        .map(|(key, label)| RankedSection {
            key: (*key).to_string(),
            label: (*label).to_string(),
            cards: buckets.remove(*key).unwrap_or_default(),
        })
        .collect()
}

fn summarize_receipt(
    symbol: &str,
    sections: &[RankedSection],
    story_count: usize,
    ryo: &[RyoToolEvidence],
) -> ReceiptSummary {
    let position_changing_count = section_count(sections, "position-changing");
    let watch_count = section_count(sections, "watch");
    let noise_count = section_count(sections, "noise");
    let unverified_count = section_count(sections, "unverified");
    let highest_impact = sections
        .iter()
        .flat_map(|section| section.cards.iter().map(|card| card.score.impact))
        .max();
    let ryo_available = ryo.iter().any(|item| item.status != "unavailable");

    let conclusion = if story_count == 0 {
        format!("No live news evidence was available for {symbol}; do not infer a changed view.")
    } else if position_changing_count > 0 {
        format!(
            "{symbol} has at least one story that may change a position view; inspect the receipt before acting."
        )
    } else if watch_count > 0 {
        format!(
            "{symbol} has watch-level narratives, but the evidence does not yet force a position change."
        )
    } else {
        format!("{symbol} news looks low-impact or unverified in this run.")
    };

    ReceiptSummary {
        headline: if ryo_available {
            format!("{symbol} global news ranked against RYO market evidence")
        } else {
            format!("{symbol} global news ranked; RYO market evidence unavailable")
        },
        conclusion,
        highest_impact,
        position_changing_count,
        watch_count,
        noise_count,
        unverified_count,
    }
}

fn find_number_deep(value: &Value, keys: &[&str]) -> Option<f64> {
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                let lower = k.to_lowercase();
                if keys.iter().any(|target| lower.contains(target)) {
                    if let Some(number) = v.as_f64() {
                        return Some(number);
                    }
                    if let Some(text) = v.as_str() {
                        if let Ok(parsed) = text.trim().trim_end_matches('%').parse::<f64>() {
                            return Some(parsed);
                        }
                    }
                }
                if let Some(found) = find_number_deep(v, keys) {
                    return Some(found);
                }
            }
            None
        }
        Value::Array(items) => items.iter().find_map(|item| find_number_deep(item, keys)),
        _ => None,
    }
}

fn find_string_deep(value: &Value, keys: &[&str]) -> Option<String> {
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                let lower = k.to_lowercase();
                if keys.iter().any(|target| lower.contains(target)) {
                    if let Some(text) = v.as_str() {
                        if !text.trim().is_empty() {
                            return Some(text.trim().to_string());
                        }
                    }
                }
                if let Some(found) = find_string_deep(v, keys) {
                    return Some(found);
                }
            }
            None
        }
        Value::Array(items) => items.iter().find_map(|item| find_string_deep(item, keys)),
        _ => None,
    }
}

fn fear_greed_label(value: u8) -> String {
    match value {
        0..=24 => "Extreme Fear",
        25..=44 => "Fear",
        45..=54 => "Neutral",
        55..=74 => "Greed",
        _ => "Extreme Greed",
    }
    .to_string()
}

fn build_market_pulse(ryo: &[RyoToolEvidence]) -> MarketPulse {
    let overview = ryo.iter().find(|tool| tool.tool == "market_overview");
    let Some(overview) = overview else {
        return MarketPulse {
            note: "market_overview was not called this run.".to_string(),
            ..Default::default()
        };
    };
    if overview.status == "unavailable" || overview.result.is_null() {
        return MarketPulse {
            as_of: Some(overview.as_of.clone()),
            available: false,
            note: "RYO market_overview was unavailable; Fear & Greed left blank rather than faked."
                .to_string(),
            ..Default::default()
        };
    }

    let result = &overview.result;
    let fear_greed_value = find_number_deep(result, &["fear_greed", "fear_and_greed", "feargreed"])
        .or_else(|| find_number_deep(result, &["sentiment_score"]))
        .map(|value| value.clamp(0.0, 100.0).round() as u8);
    let fear_greed_label_value = fear_greed_value
        .map(fear_greed_label)
        .or_else(|| find_string_deep(result, &["fear_greed_label", "sentiment_label", "classification"]));
    let regime = find_string_deep(result, &["regime", "market_phase", "phase"]);
    let btc_dominance = find_number_deep(result, &["btc_dominance", "dominance", "btc_dom"]);
    let total_market_cap_change =
        find_number_deep(result, &["total_market_cap_change", "mcap_change", "market_cap_change"]);
    let breadth = find_string_deep(result, &["breadth"])
        .or_else(|| find_number_deep(result, &["breadth"]).map(|value| format!("{value:+.0}")));

    let available = fear_greed_value.is_some() || regime.is_some() || btc_dominance.is_some();
    let note = if available {
        "Live market regime pulled from RYO market_overview.".to_string()
    } else {
        "market_overview responded, but no Fear & Greed field was present; left blank, not faked."
            .to_string()
    };

    MarketPulse {
        fear_greed_value,
        fear_greed_label: fear_greed_label_value,
        regime,
        btc_dominance,
        total_market_cap_change,
        breadth,
        as_of: Some(overview.as_of.clone()),
        available,
        note,
    }
}

fn build_recommendations(
    symbol: &str,
    verdict: &ReasoningVerdict,
    summary: &ReceiptSummary,
    pulse: &MarketPulse,
    sections: &[RankedSection],
) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();

    match verdict.decision.as_str() {
        "CONFIRMED" => out.push(format!(
            "Signal is CONFIRMED at {}% confidence: treat the top {symbol} narrative as decision-grade and review exposure against your own risk limits.",
            verdict.confidence
        )),
        "WATCHLIST" => out.push(format!(
            "Signal is WATCHLIST at {}% confidence: keep {symbol} on watch and wait for a second confirming source before acting.",
            verdict.confidence
        )),
        _ => out.push(format!(
            "Signal is REJECTED at {}% confidence: no defensible reason to change your {symbol} view from this run.",
            verdict.confidence
        )),
    }

    if let Some(value) = pulse.fear_greed_value {
        let label = pulse
            .fear_greed_label
            .clone()
            .unwrap_or_else(|| fear_greed_label(value));
        let steer = match value {
            0..=24 => "Extreme Fear historically rewards patience over panic; size in slowly and avoid forced exits.",
            25..=44 => "Fear means crowd conviction is low; demand stronger confirmation before adding risk.",
            45..=54 => "Neutral sentiment gives no crowd edge; let the token-specific evidence decide.",
            55..=74 => "Greed can extend trends but thins the margin of safety; tighten stops and take partials into strength.",
            _ => "Extreme Greed is where late entries get punished; protect gains and resist chasing.",
        };
        out.push(format!("Market mood is {label} ({value}/100). {steer}"));
    } else {
        out.push(
            "Fear & Greed was unavailable this run, so no crowd-sentiment adjustment was applied (left blank, not assumed neutral).".to_string(),
        );
    }

    if summary.position_changing_count > 0 {
        out.push(format!(
            "{} position-changing item(s) surfaced. Read those receipts first; they are the only stories worth acting on today.",
            summary.position_changing_count
        ));
    } else if summary.watch_count > 0 {
        out.push(format!(
            "Nothing is position-changing yet, but {} watch-level item(s) deserve a second look before the next session.",
            summary.watch_count
        ));
    }

    if !verdict.missing_data.is_empty() {
        out.push(format!(
            "Missing data ({}) is reported honestly and was not counted as zero; weight the verdict accordingly.",
            verdict.missing_data.join(", ")
        ));
    }

    let top_headline = sections
        .iter()
        .flat_map(|section| section.cards.iter())
        .max_by_key(|card| card.score.impact)
        .map(|card| card.headline.clone());
    if let Some(headline) = top_headline {
        out.push(format!("Lead story to brief your team on: \"{}\".", compact_headline(&headline, 90)));
    }

    out.push(verdict.recommended_next_action.clone());
    out
}

fn build_editor_note(
    symbol: &str,
    verdict: &ReasoningVerdict,
    pulse: &MarketPulse,
    story_count: usize,
) -> String {
    let mood = pulse
        .fear_greed_label
        .clone()
        .map(|label| format!("with the market in {label}"))
        .unwrap_or_else(|| "with market mood unavailable".to_string());
    format!(
        "Filed on {symbol}: {} sources reviewed {mood}. The desk rules this a {} read - {}",
        story_count,
        verdict.decision,
        verdict.recommended_next_action
    )
}

async fn build_reasoning_verdict(
    state: &AppState,
    symbol: &str,
    summary: &ReceiptSummary,
    sections: &[RankedSection],
    stories: &[NewsStory],
    ryo: &[RyoToolEvidence],
    warnings: &[String],
) -> (ReasoningVerdict, Option<String>) {
    let base = deterministic_verdict(symbol, summary, sections, stories, ryo, warnings);
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
            symbol,
            summary,
            sections,
            stories,
            ryo,
            warnings,
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

fn deterministic_verdict(
    symbol: &str,
    summary: &ReceiptSummary,
    sections: &[RankedSection],
    stories: &[NewsStory],
    ryo: &[RyoToolEvidence],
    warnings: &[String],
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
    }
}

fn build_reasoning_layer_output(
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

fn confidence_ratio(confidence: u8) -> f32 {
    ((confidence as f32 / 100.0) * 100.0).round() / 100.0
}

fn top_sentiment(sections: &[RankedSection]) -> String {
    sections
        .iter()
        .flat_map(|section| section.cards.iter())
        .max_by_key(|card| card.score.impact)
        .map(|card| card.sentiment.clone())
        .unwrap_or_else(|| "uncertain".to_string())
}

fn market_confirmation_label(sections: &[RankedSection], ryo: &[RyoToolEvidence]) -> String {
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

struct OpenAiVerdictInput<'a> {
    symbol: &'a str,
    summary: &'a ReceiptSummary,
    sections: &'a [RankedSection],
    stories: &'a [NewsStory],
    ryo: &'a [RyoToolEvidence],
    warnings: &'a [String],
    base: ReasoningVerdict,
}

async fn openai_verdict(
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

fn why_it_matters(summary: &ReceiptSummary, cards: &[&StoryCard]) -> Vec<String> {
    let mut points = vec![summary.conclusion.clone()];
    for card in cards.iter().take(2) {
        points.extend(card.reasoning.iter().take(2).cloned());
    }
    points.truncate(5);
    points
}

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

fn read_string(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_str))
        .map(str::to_string)
}

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

fn section_count(sections: &[RankedSection], key: &str) -> usize {
    sections
        .iter()
        .find(|section| section.key == key)
        .map(|section| section.cards.len())
        .unwrap_or(0)
}

fn resolve_data_mode(stories: &[NewsStory], ryo: &[RyoToolEvidence]) -> String {
    let mut modes = BTreeSet::new();
    for story in stories {
        modes.insert(story.data_mode.as_str());
    }
    for tool in ryo {
        modes.insert(tool.data_mode.as_str());
    }
    let has_live = modes.contains("live");
    let has_user_provided = modes.contains("user-provided");
    if has_live && has_user_provided {
        "mixed".to_string()
    } else if has_live {
        "live".to_string()
    } else if has_user_provided {
        "user-provided".to_string()
    } else {
        "unknown".to_string()
    }
}

fn resolve_status(stories: &[NewsStory], ryo: &[RyoToolEvidence]) -> String {
    if stories.is_empty() && ryo.iter().all(|tool| tool.status == "unavailable") {
        "unavailable".to_string()
    } else if stories.is_empty() || ryo.iter().any(|tool| tool.status == "unavailable") {
        "partial".to_string()
    } else {
        "ok".to_string()
    }
}

fn host_from_url(url: &str) -> String {
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

fn infer_region(url: &str) -> Option<String> {
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

fn parse_datetime(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .ok()
}

async fn watch_loop(state: AppState) {
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        let due_items = {
            let watchlist = state.watchlist.read().await;
            let now = Utc::now();
            watchlist
                .values()
                .filter(|item| item.enabled)
                .filter(|item| {
                    item.next_check_at
                        .as_deref()
                        .and_then(parse_datetime)
                        .map(|next| next <= now)
                        .unwrap_or(true)
                })
                .cloned()
                .collect::<Vec<_>>()
        };

        for item in due_items {
            let request = PulseRequest {
                symbol: item.symbol.clone(),
                timeframe: Some("24h".to_string()),
                regions: Some(vec!["Global".to_string()]),
                sources: Some(vec!["Tavily".to_string()]),
                thesis: None,
                event: None,
                news: None,
            };
            match build_pulse(&state, request).await {
                Ok(receipt) => {
                    let top_cluster = top_cluster(&receipt);
                    let material = receipt.summary.highest_impact.unwrap_or(0) >= 65
                        && top_cluster != item.last_cluster;
                    let now = Utc::now();
                    let next = now + ChronoDuration::minutes(item.interval_minutes as i64);
                    let mut watchlist = state.watchlist.write().await;
                    if let Some(current) = watchlist.get_mut(&item.symbol) {
                        current.last_checked_at = Some(now.to_rfc3339());
                        current.next_check_at = Some(next.to_rfc3339());
                        if material {
                            current.last_receipt_id = Some(receipt.id.clone());
                            current.last_cluster = top_cluster;
                            state.receipts.write().await.insert(0, receipt);
                        }
                    }
                }
                Err(error) => {
                    warn!("watch check failed for {}: {}", item.symbol, error.message);
                }
            }
        }
    }
}

fn top_cluster(receipt: &DecisionReceipt) -> Option<String> {
    receipt
        .sections
        .iter()
        .flat_map(|section| section.cards.iter())
        .max_by_key(|card| card.score.impact)
        .map(|card| card.narrative_cluster.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

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
