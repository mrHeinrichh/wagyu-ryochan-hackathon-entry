//! The optional background watch loop.
//!
//! When `APP_ENABLE_WATCH_LOOP` is set, this ticks once a minute, runs a
//! pulse for any due watched token, and only stores a new receipt when the top
//! narrative changed and impact is high enough to be worth surfacing.

use std::time::Duration;

use chrono::{Duration as ChronoDuration, Utc};
use tracing::warn;

use crate::domain::{DecisionReceipt, PulseRequest};
use crate::reasoning::build_pulse;
use crate::state::AppState;
use crate::util::parse_datetime;

/// Run forever, checking due watched tokens each minute.
pub(crate) async fn watch_loop(state: AppState) {
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

/// The narrative cluster of the highest-impact card, used to detect change.
fn top_cluster(receipt: &DecisionReceipt) -> Option<String> {
    receipt
        .sections
        .iter()
        .flat_map(|section| section.cards.iter())
        .max_by_key(|card| card.score.impact)
        .map(|card| card.narrative_cluster.clone())
}
