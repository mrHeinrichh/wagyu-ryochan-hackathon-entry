# Global Token News Pulse — RYO-CHAN Hackathon Entry

Global Token News Pulse is our official entry for the **RYO-CHAN Hackathon 2026** (DoraHacks). It is a Rust-backed agent and decision interface that turns market reads, global news, or a user thesis into an inspectable decision — `CONFIRMED`, `WATCHLIST`, or `REJECTED`.

It runs separately from the RYO app, calls RYO MCP/REST from the Rust backend, combines that market evidence with global news, and produces durable, shareable decision receipts. Practice plans are simulation-only and cannot place orders.

The product question is:

```text
Did something happen globally that should materially change my view on this token?
```

## Entry Details

- **Project name:** Global Token News Pulse
- **Event:** RYO-CHAN Hackathon 2026 (DoraHacks)
- **Repository:** `wagyu-ryochan-hackathon-entry`
- **Backend:** Rust (Axum) — chosen for speed and safe, predictable failure handling
- **Frontend:** Static dashboard styled to match the RYO-CHAN UI (Light / System / Dark)
- **Core idea:** Composition over invocation. Chain global news + RYO market intelligence + reasoning into one defensible verdict, and show the argument behind it.
- **Design principle:** Honest degradation — missing data stays `unavailable`, never faked or zeroed.
- **Repeatability:** Receipts and watchlists persist in SQLite and recover after restart.

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
4. A focused second Tavily query looks for independent confirmation of the leading claim.
5. Every story separates search relevance from publisher authority, checks timestamps and primary-source status, and identifies corroborating or conflicting domains.
6. Backend calls RYO market intelligence:
   - `market_overview`
   - `scan_market`
   - `analyze_token`
   - `deep_analysis`
   - `compare_tokens`
   - `monitor_market_sentiment_shift`
7. A Bull Agent and Bear Agent build opposing cases from the same evidence; the AI Judge compares them, may abstain, and returns bullish, bearish, and unclear scenario odds.
8. Stories are scored using available evidence only.
9. The UI displays ranked buckets:
   - Position-changing
   - Watch closely
   - Unverified or conflicting
   - Noise
10. The receipt shows `What changed → Market check → Decision → Next move`, plus the exact invalidation condition.
11. A risk-bounded practice plan stays `NO ENTRY` until live RYO and direction agree.
12. Regional evidence is summarized as converging, diverging, mixed, or insufficient coverage.
13. RYO-CHAN automatically summarizes each receipt in a collapsible, receipt-grounded chat and suggests useful follow-up questions.
14. A spotlight tutorial and integrated FAQ make every major workflow judgeable without prior product knowledge.

Credibility is not presented as guaranteed truth. The score explains provenance,
independent corroboration, conflicts, and evidence completeness. Tavily's result
score is kept separately as search-query relevance. Each uncached live run uses
one advanced discovery search and one basic corroboration search; cached runs do
not repeat those calls.

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
    storage.rs      SQLite receipt/watchlist persistence
    safety.rs       cooldowns, quotas, AI budget and concurrency controls
    error.rs        the shared ApiError type
    util.rs         small dependency-free helpers
    watch.rs        optional background watch loop
    domain/         request/response and receipt data models
    services/       outbound integrations: http, news (Tavily), ryo, openai
    reasoning/      scoring, verdict, decision support, and demo fixture
    handlers/       HTTP handlers and the router
frontend/           Next.js + TypeScript dashboard
  src/app/          App Router entry points
  src/components/   UI panels for controls, feed, receipt, evidence, sidebar
  src/lib/          API client, formatters, receipt/share helpers, shared types
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

Receipts and watchlists are stored at `backend/data/pulse.db` by default. Set
`APP_DB_PATH` to override it.

Cost and abuse controls are enabled by default. Analysis is limited to 6
requests per client per minute with a 12-second cooldown; chat is limited to 12
requests per client per minute with a 2-second cooldown. OpenAI is capped at 30
calls per minute and 2 concurrent calls across the service. All values can be
tuned with the `APP_*_RATE_LIMIT_*`, `APP_*_COOLDOWN_SECONDS`, and
`APP_AI_MAX_CONCURRENCY` variables documented in `backend/.env.example`.
Rate-limited responses use HTTP `429`, include `Retry-After`, and never start an
upstream AI call.

Cloudflare Turnstile can additionally screen live analysis and assistant chat
before those requests reach a paid provider. Configure the public widget key as
`NEXT_PUBLIC_TURNSTILE_SITE_KEY`, keep `TURNSTILE_SECRET_KEY` on the backend,
set `TURNSTILE_EXPECTED_HOSTNAME`, then enable `APP_TURNSTILE_REQUIRED=true`.
The labelled sample route remains available without a challenge so the core
product can still be evaluated during an upstream outage.

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

Returns backend, RYO, Tavily, OpenAI, optional source, watch-loop, and active
guardrail configuration status.

### `GET /api/tokens`

Returns the latest ranked CoinGecko market tokens with a built-in fallback list.
RYO remains the source of market intelligence.

### `POST /api/pulse`

Alias: `POST /reason`

Body:

```json
{
  "symbol": "BNB",
  "timeframe": "24h",
  "regions": ["Global", "Asia"],
  "sources": ["Tavily", "Crypto-native"],
  "thesis": "Market momentum is positive, but sentiment shift is incomplete.",
  "risk_budget_pct": 0.5
}
```

Requires `TAVILY_API_KEY` and `OPENAI_API_KEY`. It also requires `RYO_MCP_KEY`
unless `APP_MOCK_RYO=true` is enabled for a local simulated demo. Once those
keys or mock mode are configured, it creates a decision receipt and stores it in
SQLite for replay after restart. The response includes both the compact
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

### `POST /api/demo`

Accepts the same request body as `/reason`, but uses a deterministic four-region
news fixture and simulated RYO evidence. It needs no external API keys and every
sample field is labelled `simulated`. The dashboard's **Try sample** button uses
this route so judges can evaluate the full workflow immediately.

### `POST /api/chat`

Answers a product or receipt question using the configured OpenAI model and a
compact copy of the selected receipt. The assistant is instructed not to add
market facts that are absent from the receipt. If OpenAI is unavailable, the
endpoint returns a deterministic receipt summary with a warning instead of
breaking the chat. AI inputs treat headlines and user text as untrusted data,
model output is length-bounded, and execution-style directions are rejected in
favor of the deterministic research-only fallback.

```json
{
  "receipt_id": "receipt-or-run-id",
  "question": "What is the biggest risk in this receipt?",
  "history": []
}
```

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
- Warm news and token caches provide a last-known response during short upstream interruptions.
- SQLite keeps receipts and watchlists available after process restart; write failures are surfaced in the receipt.
- The deterministic Rust verdict remains available when OpenAI fails during a configured live run.
- The assistant falls back to the stored receipt when its OpenAI call fails; it never fabricates a conversational answer from unavailable evidence.

Never present simulated or placeholder data as live evidence. Never commit `.env`.

## Demo Flow

1. Open the dashboard and click **Try sample**; no setup is required.
2. Read the verdict, confidence, four-step decision path, and invalidation condition.
3. Show that the practice plan remains `NO ENTRY` because RYO is simulated.
4. Compare the regional strip and the three highest-impact stories.
5. Open **Research details** to show source health, charts, and raw evidence.
6. Open RYO-CHAN to show the automatic typed summary, ask a receipt question, and open the FAQ.
7. Start **Tour** to demonstrate the spotlight walkthrough and mobile-friendly controls.
8. Copy the deep link or export the social image and JSON receipt.
9. Restart the backend and reopen the same receipt to demonstrate repeatability.

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
