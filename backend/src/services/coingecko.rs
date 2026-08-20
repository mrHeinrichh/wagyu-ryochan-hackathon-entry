//! Live token catalog backed by CoinGecko with a short-lived in-memory cache.

use std::collections::BTreeSet;

use axum::http::StatusCode;
use chrono::{Duration as ChronoDuration, Utc};
use serde::Deserialize;

use crate::domain::TokenInfo;
use crate::error::ApiError;
use crate::services::http::parse_response;
use crate::state::{AppState, CacheEntry};

const MARKETS_URL: &str = "https://api.coingecko.com/api/v3/coins/markets";
const CATALOG_TTL_MINUTES: i64 = 5;

#[derive(Debug, Deserialize)]
struct CoinGeckoMarketToken {
    symbol: String,
    name: String,
    market_cap_rank: Option<u32>,
    image: Option<String>,
}

/// Fetch the latest ranked market catalog, reusing a five-minute cache.
pub(crate) async fn latest_tokens(
    state: &AppState,
) -> Result<(Vec<TokenInfo>, bool, String), ApiError> {
    if let Some(cached) = state.token_cache.read().await.as_ref().cloned()
        && cached.expires_at > Utc::now()
    {
        return Ok((cached.value, true, cached.fetched_at.to_rfc3339()));
    }

    let key = state
        .config
        .coingecko_demo_api_key
        .as_deref()
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::PRECONDITION_REQUIRED,
                "coingecko_key_missing",
                "COINGECKO_DEMO_API_KEY is not configured.",
            )
        })?;

    let response = state
        .client
        .get(MARKETS_URL)
        .header("x-cg-demo-api-key", key)
        .query(&[
            ("vs_currency", "usd"),
            ("order", "market_cap_desc"),
            ("per_page", "100"),
            ("page", "1"),
            ("sparkline", "false"),
        ])
        .send()
        .await
        .map_err(|error| {
            ApiError::new(
                StatusCode::BAD_GATEWAY,
                "coingecko_network_error",
                error.to_string(),
            )
        })?;

    let value = parse_response(response, "coingecko_error").await?;
    let market_tokens =
        serde_json::from_value::<Vec<CoinGeckoMarketToken>>(value).map_err(|error| {
            ApiError::new(
                StatusCode::BAD_GATEWAY,
                "coingecko_response_error",
                error.to_string(),
            )
        })?;
    let tokens = normalize_market_tokens(market_tokens);
    if tokens.is_empty() {
        return Err(ApiError::new(
            StatusCode::BAD_GATEWAY,
            "coingecko_empty_catalog",
            "CoinGecko returned no usable market tokens.",
        ));
    }

    let fetched_at = Utc::now();
    state.token_cache.write().await.replace(CacheEntry {
        value: tokens.clone(),
        fetched_at,
        expires_at: fetched_at + ChronoDuration::minutes(CATALOG_TTL_MINUTES),
    });

    Ok((tokens, false, fetched_at.to_rfc3339()))
}

fn normalize_market_tokens(mut market_tokens: Vec<CoinGeckoMarketToken>) -> Vec<TokenInfo> {
    market_tokens.sort_by_key(|token| token.market_cap_rank.unwrap_or(u32::MAX));

    let mut seen = BTreeSet::new();
    let mut ranked = market_tokens
        .into_iter()
        .filter_map(|token| {
            let symbol = token.symbol.trim().to_uppercase();
            let name = token.name.trim().to_string();
            let valid_symbol = !symbol.is_empty()
                && symbol.len() <= 16
                && symbol
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric());
            if !valid_symbol || name.is_empty() || !seen.insert(symbol.clone()) {
                return None;
            }
            Some(TokenInfo {
                symbol,
                name,
                default_peers: Vec::new(),
                market_cap_rank: token.market_cap_rank,
                image_url: token.image,
            })
        })
        .take(100)
        .collect::<Vec<_>>();

    let leading_symbols = ranked
        .iter()
        .map(|token| token.symbol.clone())
        .collect::<Vec<_>>();
    for token in &mut ranked {
        token.default_peers = leading_symbols
            .iter()
            .filter(|symbol| **symbol != token.symbol)
            .take(3)
            .cloned()
            .collect();
    }

    ranked
}

#[cfg(test)]
mod tests {
    use super::{CoinGeckoMarketToken, normalize_market_tokens};

    #[test]
    fn normalizes_sorts_and_deduplicates_market_tokens() {
        let tokens = normalize_market_tokens(vec![
            CoinGeckoMarketToken {
                symbol: " eth ".to_string(),
                name: "Ethereum".to_string(),
                market_cap_rank: Some(2),
                image: Some("https://assets.example/eth.png".to_string()),
            },
            CoinGeckoMarketToken {
                symbol: "btc".to_string(),
                name: "Bitcoin".to_string(),
                market_cap_rank: Some(1),
                image: Some("https://assets.example/btc.png".to_string()),
            },
            CoinGeckoMarketToken {
                symbol: "BTC".to_string(),
                name: "Duplicate Bitcoin".to_string(),
                market_cap_rank: Some(30),
                image: None,
            },
            CoinGeckoMarketToken {
                symbol: "bad-token".to_string(),
                name: "Invalid".to_string(),
                market_cap_rank: Some(3),
                image: None,
            },
        ]);

        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].symbol, "BTC");
        assert_eq!(tokens[0].market_cap_rank, Some(1));
        assert_eq!(tokens[0].default_peers, vec!["ETH"]);
        assert_eq!(
            tokens[0].image_url.as_deref(),
            Some("https://assets.example/btc.png")
        );
        assert_eq!(tokens[1].symbol, "ETH");
    }
}
