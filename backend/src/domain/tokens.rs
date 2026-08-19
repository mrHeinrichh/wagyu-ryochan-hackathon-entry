//! The small static reference list of tokens offered to the UI.

use serde::Serialize;

/// A token the UI can suggest. RYO MCP/REST remains the source of truth for
/// market intelligence; this list only seeds the input datalist.
#[derive(Debug, Serialize, Clone)]
pub(crate) struct TokenInfo {
    pub(crate) symbol: &'static str,
    pub(crate) name: &'static str,
    pub(crate) default_peers: Vec<&'static str>,
}

/// The built-in starter tokens shown in the UI selector.
pub(crate) fn default_tokens() -> Vec<TokenInfo> {
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
