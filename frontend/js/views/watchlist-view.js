// Sidebar view: the watchlist, with add/remove actions.

import { el, escapeHtml } from "../core/dom.js";
import { endpoints } from "../core/api.js";
import { shortDate } from "../core/format.js";

/** Load and render the current watchlist. */
export async function loadWatchlist() {
  try {
    const watchlist = await endpoints.watchlist();
    renderWatchlist(watchlist);
  } catch {
    el("#watchlist").innerHTML = `<div class="empty-mini">Watchlist unavailable</div>`;
  }
}

/** Add the current symbol to the watchlist at a 15-minute interval. */
export async function addWatch() {
  const symbol = el("#symbolInput").value;
  const button = el("#watchButton");
  button.disabled = true;
  button.textContent = "Saving";
  try {
    await endpoints.addWatch({ symbol, interval_minutes: 15 });
    await loadWatchlist();
  } catch (error) {
    alert(error.message);
  } finally {
    button.disabled = false;
    button.textContent = "Watch";
  }
}

/** Remove a symbol from the watchlist and refresh. */
async function removeWatch(symbol) {
  await endpoints.removeWatch(symbol);
  await loadWatchlist();
}

/** Render the watchlist rows with their remove buttons. */
function renderWatchlist(watchlist) {
  const target = el("#watchlist");
  if (!watchlist.length) {
    target.innerHTML = `<div class="empty-mini">No watched tokens</div>`;
    return;
  }
  target.innerHTML = "";
  for (const item of watchlist) {
    const row = document.createElement("div");
    row.className = "watch-item";
    row.innerHTML = `
      <strong>${escapeHtml(item.symbol)}</strong>
      <span>${item.interval_minutes} min - next ${shortDate(item.next_check_at)}</span>
    `;
    const remove = document.createElement("button");
    remove.className = "secondary-button";
    remove.type = "button";
    remove.textContent = "Remove";
    remove.style.marginTop = "8px";
    remove.addEventListener("click", () => removeWatch(item.symbol));
    row.appendChild(remove);
    target.appendChild(row);
  }
}
