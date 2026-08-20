//! Deterministic, clearly simulated news used by the one-click judge demo.

use crate::domain::NewsStory;

pub(crate) fn demo_news(symbol: &str, created_at: &str) -> Vec<NewsStory> {
    vec![
        story(
            "demo-asia",
            format!("{symbol} regional integration lifts trading volume across Asia"),
            "demo.nikkei.example",
            "Asia",
            format!(
                "{symbol} regional integration improved exchange volume and adoption momentum."
            ),
            created_at,
        ),
        story(
            "demo-us",
            format!("Trading desks watch {symbol} as regional integration lifts volume"),
            "demo.marketwire.example",
            "US",
            format!(
                "Institutional commentary remains cautious while {symbol} market momentum and liquidity improve."
            ),
            created_at,
        ),
        story(
            "demo-europe",
            format!("Breaking European policy lawsuit raises {symbol} downside risk"),
            "demo.reuters.example",
            "Europe",
            format!(
                "A newly filed lawsuit adds urgent policy risk, weak demand signals, and downside pressure for {symbol} despite broader momentum."
            ),
            created_at,
        ),
        story(
            "demo-global",
            format!("{symbol} momentum improves, but confirmation remains incomplete"),
            "demo.coindesk.example",
            "Global",
            format!(
                "Global crypto market attention is rising for {symbol}; live confirmation is still required before acting."
            ),
            created_at,
        ),
    ]
}

fn story(
    id: &str,
    headline: String,
    source: &str,
    region: &str,
    content: String,
    created_at: &str,
) -> NewsStory {
    NewsStory {
        id: id.to_string(),
        headline,
        source: source.to_string(),
        url: format!("https://{source}/{id}"),
        content,
        published_at: Some(created_at.to_string()),
        region: Some(region.to_string()),
        language: Some("en".to_string()),
        data_mode: "simulated".to_string(),
        search_relevance: Some(90),
    }
}
