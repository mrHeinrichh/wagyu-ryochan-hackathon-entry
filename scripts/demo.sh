#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${BASE_URL:-http://127.0.0.1:8788}"

echo "Health"
curl -s "$BASE_URL/health"
echo
echo

echo "This reasoning pass requires real RYO_MCP_KEY, TAVILY_API_KEY, and OPENAI_API_KEY in the running backend."
echo "Run BNB reasoning pass"
curl -s "$BASE_URL/reason" \
  -H 'content-type: application/json' \
  -d '{
    "symbol": "BNB",
    "timeframe": "24h",
    "regions": ["Global", "Asia"],
    "sources": ["Tavily", "Crypto-native"],
    "thesis": "Market momentum is positive, but sentiment shift is incomplete and derivatives data is unavailable."
  }' | jq '{signal, symbol, confidence, reasoning, ryo_tools_used, unavailable_data, next_action, run_id, status, data_mode}'
echo
echo

echo "Runs"
curl -s "$BASE_URL/runs"
echo
