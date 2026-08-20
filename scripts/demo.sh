#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${BASE_URL:-http://127.0.0.1:8788}"

echo "Health"
curl -s "$BASE_URL/health"
echo
echo

echo "Run deterministic, clearly simulated BNB judge sample"
curl -s "$BASE_URL/api/demo" \
  -H 'content-type: application/json' \
  -d '{
    "symbol": "BNB",
    "timeframe": "24h",
    "regions": ["Global", "Asia"],
    "sources": ["Tavily", "Crypto-native"],
    "thesis": "",
    "risk_budget_pct": 0.5
  }' | jq '{signal, symbol, confidence, decision_chain, invalidation, practice_plan, regional_convergence, run_id, status, data_mode}'
echo
echo

echo "Runs"
curl -s "$BASE_URL/runs"
echo
