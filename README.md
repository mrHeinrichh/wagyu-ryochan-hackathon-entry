# Wagyu — RYO-CHAN Hackathon Entry

Wagyu is our official entry for the **RYO-CHAN Hackathon 2026** (DoraHacks). It is a Global Token News & Reasoning Layer: a Rust-backed agent and dashboard that turns market reads, global news, or a user thesis into an inspectable decision — `CONFIRMED`, `WATCHLIST`, or `REJECTED`.

Wagyu is not another RYO chatbot, wallet, DEX connector, or paper-trading app. It runs separately from the RYO app, calls RYO MCP/REST from the Rust backend, combines that market evidence with a news source or user-supplied thesis, and produces auditable decision receipts.

The product question is:

```text
Did something happen globally that should materially change my view on this token?
```

## Entry Details

- **Project name:** Wagyu
- **Event:** RYO-CHAN Hackathon 2026 (DoraHacks)
- **Repository:** `wagyu-ryochan-hackathon-entry`
- **Backend:** Rust (Axum) — chosen for speed and safe, predictable failure handling
- **Frontend:** Static dashboard styled to match the RYO-CHAN UI (Light / System / Dark)
- **Core idea:** Composition over invocation. Chain global news + RYO market intelligence + reasoning into one defensible verdict, and show the argument behind it.
- **Design principle:** Honest degradation — missing data stays `unavailable`, never faked or zeroed.

Example output:

```text
{
  "signal": "WATCHLIST",
  "symbol": "BNB",
  "confidence": 0.72,
  "reasoning": "Market momentum is positive, but sentiment shift is incomplete and derivatives data is unavailable.",
  "ryo_tools_used": ["market_overview", "deep_analysis", "compare_tokens"],
  "unavailable_data": ["derivatives"],
  "next_action": "Wait for confirmation before any downstream action"
}
```

## Track Fit

**Track 1 — Autonomous Agents (headline track).** Watch mode runs a background reasoning loop over selected tokens, emitting a fresh decision receipt only when evidence materially shifts. Every signal is traced from headline to RYO technical gate, so the cause-and-effect chain is inspectable.

**Track 2 — Dashboards & Interfaces.** The main screen is a 30-second triage layer that ranks events by importance (Position-changing / Watch closely / Unverified or conflicting / Noise) and explains why each one matters. Styled to match the RYO-CHAN UI across Light, System, and Dark.

**Special — Honest Degradation.** Missing derivatives, timestamps, or RYO evidence are surfaced as `unavailable` in each receipt rather than laundered into a zero or a plausible-looking guess.

## What It Does

1. User selects a token such as `SOL`, `BTC`, `ETH`, or `BNB`.
2. Backend searches global news through Tavily.
3. Backend deduplicates and clusters stories into narratives.
4. Backend calls RYO market intelligence:
   - `market_overview`
   - `scan_market`
   - `analyze_token`
   - `deep_analysis`
   - `compare_tokens`
   - `monitor_market_sentiment_shift`
5. Backend optionally asks OpenAI for the final reasoning verdict.
6. Stories are scored using available evidence only.
7. The UI displays ranked buckets:
   - Position-changing
   - Watch closely
   - Unverified or conflicting
   - Noise
8. Each card opens a decision receipt with sources, RYO calls, reasoning, missing data, confidence, and warnings.

## Subscriptions

Required:

- RYO MCP/REST key from the organizers.
- Tavily Free for global news search.
- OpenAI API for the final reasoning verdict. The default hackathon model is `gpt-5.6-terra`; use `gpt-5.6-sol` only for final polish runs when quality matters more than cost.

Optional:

- CoinGecko Demo API key as a backup market-data sanity check.
- DeFiLlama for protocol and TVL context.
- DexScreener for liquidity context.

Not required:

- TradingView API.
- DEX execution API.
- Wallet or live money infrastructure.

## Why This Is Not a Wall of Charts

Each card must answer:

- What happened?
- Why could it matter for this token?
- Does RYO market intelligence confirm or contradict it?
- What is missing?
- What action should a cautious user consider: watch, investigate, wait, reduce risk, or no action?

The app ranks evidence and shows an argument. Raw headline volume does not increase the score by itself.

## Project Structure

The repository separates the Rust backend from the Next.js frontend, and each is
split into small, single-purpose modules.

```text
backend/            Rust crate (Axum API + reasoning pipeline)
  src/
    main.rs         process bootstrap only: env, state, router, serve
    config.rs       environment-driven configuration
    state.rs        shared AppState and the news cache entry
    error.rs        the shared ApiError type
    util.rs         small dependency-free helpers
    watch.rs        optional background watch loop
    domain/         request/response and receipt data models
    services/       outbound integrations: http, news (Tavily), ryo, openai
    reasoning/      the scoring/verdict pipeline (build_pulse + submodules)
    handlers/       HTTP handlers and the router
frontend/           Next.js + TypeScript dashboard
  src/app/          App Router entry points
  src/components/   UI panels for controls, feed, receipt, evidence, sidebar
  src/lib/          typed API client, formatters, receipt helpers, shared types
  src/store/        Zustand UI state store
  src/styles/       application stylesheet
  public/assets/    RYO-CHAN mascot and favicon
  out/              static export generated by `npm run build:static`
```

The frontend uses Next.js with TypeScript and Zustand. In development, `next dev`
proxies API calls to the Rust backend. For the integrated demo, `npm run
build:static` writes `frontend/out/`, and the Rust backend serves that export
same-origin.

## Setup

Configuration lives next to the crate, in `backend/`:

```bash
cd backend
cp .env.example .env
```

Then fill in:

```bash
RYO_MCP_KEY=your_builder_key
TAVILY_API_KEY=your_tavily_key
OPENAI_API_KEY=your_openai_key
```

If the RYO builder key has not arrived yet, local demo mode can run with
simulated RYO evidence:

```bash
APP_MOCK_RYO=true
```

Mock mode marks every RYO tool as `partial` with `data_mode=simulated`, adds
receipt warnings, and prevents a live `CONFIRMED` verdict. Do not use mock mode
for final judging or present it as real RYO confirmation.

Install frontend dependencies once:

```bash
cd ../frontend
npm install
```

For local development, run both processes:

```bash
# terminal 1
cd backend
cargo run

# terminal 2
cd frontend
npm run dev
```

Open:

```text
http://127.0.0.1:3000
```

For the integrated Rust-served demo, build the static frontend first:

```bash
cd frontend
npm run build:static
```

Then run from the `backend/` directory:

```bash
cargo run
```

Open:

```text
http://127.0.0.1:8788
```

## API

### `GET /health`

Returns backend, RYO, Tavily, OpenAI, optional source, and watch-loop status.

### `GET /api/tokens`

Returns a small static token list for the UI. RYO remains the source of market intelligence.

### `POST /api/pulse`

Alias: `POST /reason`

Body:

```json
{
  "symbol": "BNB",
  "timeframe": "24h",
  "regions": ["Global", "Asia"],
  "sources": ["Tavily", "Crypto-native"],
  "thesis": "Market momentum is positive, but sentiment shift is incomplete and derivatives data is unavailable."
}
```

Requires `TAVILY_API_KEY` and `OPENAI_API_KEY`. It also requires `RYO_MCP_KEY`
unless `APP_MOCK_RYO=true` is enabled for a local simulated demo. Once those
keys or mock mode are configured, it creates a decision receipt and stores it in
memory for the running process. The response includes both the compact
reasoning-layer output and the full receipt:

```json
{
  "signal": "WATCHLIST",
  "symbol": "BNB",
  "confidence": 0.32,
  "reasoning": "BNB has watch-level narratives, but the evidence does not yet force a position change.",
  "ryo_tools_used": ["market_overview", "scan_market", "analyze_token"],
  "unavailable_data": ["RYO market confirmation"],
  "next_action": "wait for confirmation and keep the token on watch",
  "reasoning_layer": {
    "signal": "WATCHLIST",
    "market_confirmation": "unavailable",
    "run_id": "..."
  }
}
```

When required keys are missing, the endpoint returns `428 missing_required_keys`
and creates no receipt.

### `GET /api/receipts`

Alias: `GET /runs`

Lists recent receipts.

### `GET /api/receipts/:id`

Alias: `GET /runs/:id`

Returns one full receipt.

### `GET /api/ryo/tools`

Fetches the organizer-provided live RYO tool catalog. Use this before final integration:

```bash
curl -s "$RYO_MCP_URL/tools" \
  -H "Authorization: Bearer $RYO_MCP_KEY" | jq
```

### `POST /api/watchlist`

Body:

```json
{
  "symbol": "SOL",
  "interval_minutes": 15
}
```

Adds a token to watch mode. Background checks run only when `APP_ENABLE_WATCH_LOOP=true`.

### `DELETE /api/watchlist/:symbol`

Removes a watched token.

## Failure Handling

This is directly aligned with the hackathon scoring.

- Missing `RYO_MCP_KEY`: `/reason` returns `428 missing_required_keys` unless `APP_MOCK_RYO=true`. Mock mode creates clearly labelled simulated RYO evidence for local demos only.
- Missing `TAVILY_API_KEY` or `OPENAI_API_KEY`: `/reason` returns `428 missing_required_keys` and creates no receipt.
- Upstream 429 or failed calls: the receipt becomes `partial` and the failing source is recorded under source availability.
- Missing published time, region, language, or RYO evidence: the card lists those fields under `missing_data`.

Never present simulated or placeholder data as live evidence. Never commit `.env`.

## Demo Flow

1. Open the dashboard.
2. Show backend health and source status.
3. Run a pulse for `SOL`.
4. Open the top card and show the reasoning.
5. Show source availability and RYO tool statuses.
6. Add `SOL` to the watchlist.
7. Explain that watch mode creates receipts only for material changes when enabled.

## Submission Checklist

- Public or judge-accessible repository.
- Completed `.env.example` with no live secrets.
- README section explaining failure handling.
- Working app URL or one-command local start.
- Demo video under 3 minutes.
- Run artifacts from real-key runs, such as replayable receipt JSON copied from `/runs/:id`.

## Organizer Questions

- Please provide `RYO_MCP_KEY` and confirm the live `RYO_MCP_URL`.
- Does the live API expose six tools or a larger tool catalog?
- Can you share the exact `/tools` schema output for each tool?
- What are the exact rate limits and expected retry behavior?
- Should final reasoning output be JSON, UI, or both?
- Can this reasoning layer hand off a verdict to the existing DEX/execution layer later, without us building execution in the hackathon?
