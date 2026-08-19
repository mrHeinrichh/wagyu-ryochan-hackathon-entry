// Application entry point.
//
// Wires DOM events to actions and kicks off the initial data loads. All view
// logic lives in ./views and all shared plumbing in ./core.

import { el } from "./core/dom.js";
import { initTheme } from "./core/theme.js";
import { loadHealth } from "./views/health-view.js";
import { loadTokens } from "./views/tokens-view.js";
import { loadHistory } from "./views/history-view.js";
import { loadWatchlist, addWatch } from "./views/watchlist-view.js";
import { runPulse } from "./views/pulse-view.js";

/** Bootstrap: theme, event handlers, then initial data. */
async function init() {
  initTheme();
  bindEvents();
  await Promise.all([loadHealth(), loadTokens(), loadHistory(), loadWatchlist()]);
}

/** Attach the form-submit and watch-button handlers. */
function bindEvents() {
  el("#pulseForm").addEventListener("submit", async (event) => {
    event.preventDefault();
    await runPulse();
  });
  el("#watchButton").addEventListener("click", async () => {
    await addWatch();
  });
}

init();
