# Three-Minute Demo Script

## 0:00 - 0:20

This is the RYO Global Token Reasoning Layer, a separate RYO-CHAN hackathon app. It does not trade, connect to a wallet, or rebuild the RYO chatbot. It answers one decision question: should this market read change my view on a specific token?

## 0:20 - 0:50

First, show health: RYO, Tavily and OpenAI are configured. Then select a token, timeframe, regions, source mix, and a news/event thesis. I will run `BNB` with: market momentum is positive, but sentiment shift is incomplete and derivatives data is unavailable.

## 0:50 - 1:30

The output starts with a signal: `CONFIRMED`, `WATCHLIST`, or `REJECTED`. Under it, the evidence buckets are ranked by decision impact, not chronology. This avoids the hackathon anti-pattern: a wall of charts or headlines with no argument.

## 1:30 - 2:10

Open the receipt. The first card is the reasoning-layer verdict: signal, confidence, next action, why it matters, RYO evidence and missing data. Then open the top evidence card to show the narrative cluster, sentiment and scoring.

## 2:10 - 2:35

The evidence matrix shows every source and every RYO tool call. The raw JSON drawer is there for judges. If Tavily, RYO, OpenAI or a dependency fails, the receipt is marked partial or unavailable. Missing data stays missing; it is never converted to zero.

## 2:35 - 3:00

Watch mode lets the app monitor selected tokens and create a new receipt only when the top narrative materially changes. That gives Track 1 agent behavior while the dashboard satisfies Track 2 readability.
