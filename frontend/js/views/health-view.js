// Sidebar view: backend/integration health lines.

import { el, escapeHtml } from "../core/dom.js";
import { endpoints } from "../core/api.js";
import { store } from "../core/store.js";

/** Fetch health and render it; show an error line if the backend is down. */
export async function loadHealth() {
  try {
    const health = await endpoints.health();
    store.set({ health });
    renderHealth(health);
  } catch (error) {
    el("#healthStack").innerHTML =
      `<div class="source-line bad"><span>Backend</span><strong>${escapeHtml(error.message)}</strong></div>`;
  }
}

/** Render the integration status lines from a health payload. */
function renderHealth(health) {
  const lines = [
    ["Backend", health.status, "ok"],
    ["RYO MCP", health.ryo_configured ? "configured" : "missing key", health.ryo_configured ? "ok" : "bad"],
    ["Tavily", health.tavily_configured ? "configured" : "missing key", health.tavily_configured ? "ok" : "warn"],
    ["OpenAI", health.openai_configured ? "configured" : "missing key", health.openai_configured ? "ok" : "warn"],
    ["CoinGecko", health.coingecko_configured ? "backup key" : "optional", health.coingecko_configured ? "ok" : "warn"],
    ["DeFiLlama", health.defillama_enabled ? "enabled" : "optional", health.defillama_enabled ? "ok" : "warn"],
    ["DexScreener", health.dexscreener_enabled ? "enabled" : "optional", health.dexscreener_enabled ? "ok" : "warn"],
    ["Reasoning data", "real only", "ok"],
    ["Watch loop", health.watch_loop_enabled ? "on" : "manual", health.watch_loop_enabled ? "ok" : "warn"],
  ];
  el("#healthStack").innerHTML = lines
    .map(
      ([label, value, klass]) =>
        `<div class="source-line ${klass}"><span>${escapeHtml(label)}</span><strong>${escapeHtml(value)}</strong></div>`,
    )
    .join("");
}
