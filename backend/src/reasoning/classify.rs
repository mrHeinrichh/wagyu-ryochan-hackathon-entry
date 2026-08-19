//! Text classification: narrative cluster, sentiment, and the per-card
//! recommendation that follows from a category and sentiment.

use crate::util::count_hits;

/// Bucket a story into a narrative cluster by keyword match, defaulting to the
/// general market narrative when nothing specific hits.
pub(crate) fn narrative_cluster(lower: &str) -> String {
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

/// Classify sentiment as bullish, bearish, or mixed by keyword balance.
pub(crate) fn sentiment(lower: &str) -> String {
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

/// Map a category and sentiment to a cautious recommendation, escalating to
/// "investigate" when an unverified item is also missing data.
pub(crate) fn recommendation_for(
    category: &str,
    sentiment: &str,
    missing_data: &[String],
) -> String {
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
