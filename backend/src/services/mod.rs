//! Outbound integrations: the news wire (Tavily), RYO MCP tools, and OpenAI.
//!
//! Each submodule owns one upstream. They share `http::parse_response` for
//! consistent status/JSON handling and return `ApiError` on failure so the
//! caller can decide how to degrade.

pub(crate) mod http;
pub(crate) mod news;
pub(crate) mod openai;
pub(crate) mod ryo;
