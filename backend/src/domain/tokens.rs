//! Token catalog entries offered to the UI.

use serde::Serialize;

/// A market token the UI can offer in its selector. RYO MCP/REST remains the
/// source of truth for the research evidence generated after selection.
#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub(crate) struct TokenInfo {
    pub(crate) symbol: String,
    pub(crate) name: String,
    pub(crate) default_peers: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) market_cap_rank: Option<u32>,
}

/// The built-in starter tokens shown in the UI selector.
pub(crate) fn default_tokens() -> Vec<TokenInfo> {
    vec![
        TokenInfo {
            symbol: "BTC".to_string(),
            name: "Bitcoin".to_string(),
            default_peers: peers(["ETH", "BNB", "SOL"]),
            market_cap_rank: None,
        },
        TokenInfo {
            symbol: "ETH".to_string(),
            name: "Ethereum".to_string(),
            default_peers: peers(["BTC", "BNB", "SOL"]),
            market_cap_rank: None,
        },
        TokenInfo {
            symbol: "BNB".to_string(),
            name: "BNB".to_string(),
            default_peers: peers(["BTC", "ETH", "SOL"]),
            market_cap_rank: None,
        },
        TokenInfo {
            symbol: "SOL".to_string(),
            name: "Solana".to_string(),
            default_peers: peers(["BTC", "ETH", "BNB"]),
            market_cap_rank: None,
        },
        TokenInfo {
            symbol: "LINK".to_string(),
            name: "Chainlink".to_string(),
            default_peers: peers(["ETH", "BTC", "BNB"]),
            market_cap_rank: None,
        },
        TokenInfo {
            symbol: "ARB".to_string(),
            name: "Arbitrum".to_string(),
            default_peers: peers(["ETH", "OP", "BASE"]),
            market_cap_rank: None,
        },
        TokenInfo {
            symbol: "CAKE".to_string(),
            name: "PancakeSwap".to_string(),
            default_peers: peers(["BNB", "UNI", "AAVE"]),
            market_cap_rank: None,
        },
    ]
}

fn peers<const N: usize>(symbols: [&str; N]) -> Vec<String> {
    symbols.into_iter().map(str::to_string).collect()
}
