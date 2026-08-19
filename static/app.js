const state = {
  health: null,
  currentReceipt: null,
  selectedCardId: null,
  theme: localStorage.getItem("gtp-theme") || "system",
};

const el = (selector) => document.querySelector(selector);
const all = (selector) => Array.from(document.querySelectorAll(selector));

init();

async function init() {
  initThemeControls();
  bindEvents();
  await Promise.all([loadHealth(), loadTokens(), loadHistory(), loadWatchlist()]);
}

function initThemeControls() {
  setTheme(state.theme);
  for (const button of all("[data-theme-choice]")) {
    button.addEventListener("click", () => setTheme(button.dataset.themeChoice));
  }
}

function setTheme(theme) {
  const nextTheme = ["light", "system", "dark"].includes(theme) ? theme : "system";
  state.theme = nextTheme;
  localStorage.setItem("gtp-theme", nextTheme);
  document.documentElement.dataset.theme = nextTheme;
  renderThemeControls();
}

function renderThemeControls() {
  for (const button of all("[data-theme-choice]")) {
    const active = button.dataset.themeChoice === state.theme;
    button.classList.toggle("active", active);
    button.setAttribute("aria-pressed", active ? "true" : "false");
  }
}

function bindEvents() {
  el("#pulseForm").addEventListener("submit", async (event) => {
    event.preventDefault();
    await runPulse();
  });

  el("#watchButton").addEventListener("click", async () => {
    await addWatch();
  });
}

async function loadHealth() {
  try {
    state.health = await api("/health");
    renderHealth(state.health);
  } catch (error) {
    el("#healthStack").innerHTML = `<div class="source-line bad"><span>Backend</span><strong>${escapeHtml(error.message)}</strong></div>`;
  }
}

async function loadTokens() {
  try {
    const payload = await api("/api/tokens");
    const datalist = el("#tokenList");
    datalist.innerHTML = "";
    for (const token of payload.tokens || []) {
      const option = document.createElement("option");
      option.value = token.symbol;
      option.label = token.name;
      datalist.appendChild(option);
    }
  } catch {
    // Token input remains usable even if the helper list is unavailable.
  }
}

async function loadHistory() {
  try {
    const receipts = await api("/api/receipts");
    const target = el("#receiptHistory");
    if (!receipts.length) {
      target.innerHTML = `<div class="empty-mini">No receipts yet</div>`;
      return;
    }
    target.innerHTML = "";
    for (const item of receipts) {
      const button = document.createElement("button");
      button.className = "history-item";
      button.type = "button";
      button.innerHTML = `
        <strong>${escapeHtml(item.symbol)} ${escapeHtml(item.signal || "")}</strong>
        <span>${percentText(item.confidence)} - ${escapeHtml(item.data_mode)} - ${shortDate(item.created_at)}</span>
      `;
      button.addEventListener("click", async () => {
        const receipt = await api(`/api/receipts/${encodeURIComponent(item.id)}`);
        renderReceipt(receipt);
      });
      target.appendChild(button);
    }
  } catch {
    el("#receiptHistory").innerHTML = `<div class="empty-mini">Receipt history unavailable</div>`;
  }
}

async function loadWatchlist() {
  try {
    const watchlist = await api("/api/watchlist");
    renderWatchlist(watchlist);
  } catch {
    el("#watchlist").innerHTML = `<div class="empty-mini">Watchlist unavailable</div>`;
  }
}

async function runPulse() {
  const button = el("#runButton");
  const symbol = el("#symbolInput").value;
  button.disabled = true;
  button.textContent = "Running";
  setMode("partial", "running");

  try {
    const receipt = await api("/reason", {
      method: "POST",
      body: JSON.stringify({
        symbol,
        timeframe: valueOfRadio("timeframe"),
        regions: checkedValues("regions"),
        sources: checkedValues("sources"),
        thesis: el("#thesisInput").value,
      }),
    });
    renderReceipt(receipt);
    await loadHistory();
  } catch (error) {
    setMode("unavailable", "failed");
    el("#feedSections").innerHTML = `<div class="empty-state">${escapeHtml(error.message)}</div>`;
  } finally {
    button.disabled = false;
    button.textContent = "Run Reasoning";
  }
}

async function addWatch() {
  const symbol = el("#symbolInput").value;
  const button = el("#watchButton");
  button.disabled = true;
  button.textContent = "Saving";
  try {
    await api("/api/watchlist", {
      method: "POST",
      body: JSON.stringify({ symbol, interval_minutes: 15 }),
    });
    await loadWatchlist();
  } catch (error) {
    alert(error.message);
  } finally {
    button.disabled = false;
    button.textContent = "Watch";
  }
}

async function removeWatch(symbol) {
  await api(`/api/watchlist/${encodeURIComponent(symbol)}`, { method: "DELETE" });
  await loadWatchlist();
}

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

function renderReceipt(receipt) {
  state.currentReceipt = receipt;
  state.selectedCardId = firstCard(receipt)?.id || null;

  el("#feedTitle").textContent = receipt.summary.headline;
  setMode(receipt.status, receipt.data_mode);
  renderMetrics(receipt);
  renderFeed(receipt);
  renderReceiptDetail(receipt, selectedCard(receipt));
  renderEvidence(receipt);
}

function renderMetrics(receipt) {
  const values = [
    receipt.signal || receipt.verdict?.decision || "-",
    percentText(receipt.confidence),
    receipt.unavailable_data?.length ?? receipt.verdict?.missing_data?.length ?? 0,
    receipt.ryo_tools_used?.length ?? receipt.ryo?.length ?? 0,
  ];
  all("#metricGrid .metric strong").forEach((node, index) => {
    node.textContent = values[index];
  });
}

function renderFeed(receipt) {
  const target = el("#feedSections");
  target.innerHTML = "";
  const template = el("#sectionTemplate");
  const storyTemplate = el("#storyTemplate");
  let totalCards = 0;

  for (const section of receipt.sections) {
    totalCards += section.cards.length;
    const node = template.content.cloneNode(true);
    node.querySelector("h3").textContent = section.label;
    node.querySelector(".section-header span").textContent = `${section.cards.length} cards`;
    const list = node.querySelector(".story-list");

    if (!section.cards.length) {
      list.innerHTML = `<div class="empty-mini">No cards in this bucket</div>`;
    }

    for (const card of section.cards) {
      const story = storyTemplate.content.cloneNode(true);
      const article = story.querySelector(".story-row");
      if (card.id === state.selectedCardId) article.classList.add("active");
      story.querySelector(".impact-pill").textContent = `impact ${card.score.impact}`;
      story.querySelector(".cluster-pill").textContent = card.narrative_cluster;
      const dataPill = story.querySelector(".data-pill");
      dataPill.textContent = card.data_mode;
      dataPill.classList.add(card.data_mode);
      story.querySelector("h4").textContent = card.headline;
      story.querySelector("p").textContent = card.reasoning[0] || card.ryo_alignment;
      story.querySelector(".story-side strong").textContent = card.score.impact;
      story.querySelector(".story-side span").textContent = card.recommendation;
      story.querySelector(".story-hitbox").addEventListener("click", () => {
        state.selectedCardId = card.id;
        renderFeed(receipt);
        renderReceiptDetail(receipt, card);
      });
      list.appendChild(story);
    }
    target.appendChild(node);
  }

  if (!totalCards) {
    target.innerHTML = `<div class="empty-state">No cards were produced. Check the source availability and run again with live keys.</div>`;
  }
}

function renderReceiptDetail(receipt, card) {
  const title = el("#receiptTitle");
  const target = el("#receiptDetail");
  title.textContent = receipt.symbol
    ? `${receipt.symbol} ${receipt.signal || receipt.verdict?.decision || "receipt"}`
    : "Decision receipt";

  if (!card) {
    target.innerHTML = `
      <div class="receipt-summary">
        ${renderVerdictBlock(receipt)}
        <h3>${escapeHtml(receipt.summary.conclusion)}</h3>
        ${renderWarnings(receipt.warnings)}
        ${renderRawJson(receipt)}
      </div>
    `;
    return;
  }

  target.innerHTML = `
    <div class="receipt-summary">
      ${renderVerdictBlock(receipt)}
      <h3>${escapeHtml(card.headline)}</h3>
      <p>${escapeHtml(receipt.summary.conclusion)}</p>
      <div class="detail-grid">
        ${detailCell("Signal", receipt.signal)}
        ${detailCell("Confidence", percentText(receipt.confidence))}
        ${detailCell("Reasoning", receipt.reasoning)}
        ${detailCell("Next action", receipt.next_action)}
        ${detailCell("Recommendation", card.recommendation)}
        ${detailCell("Card confidence", card.confidence)}
        ${detailCell("Sentiment", card.sentiment)}
        ${detailCell("Cluster", card.narrative_cluster)}
        ${detailCell("Source", card.source)}
        ${detailCell("Run ID", receipt.run_id)}
      </div>
      <div>
        <p class="eyebrow">Reasoning</p>
        <ol class="detail-list">
          ${card.reasoning.map((item) => `<li>${escapeHtml(item)}</li>`).join("")}
        </ol>
      </div>
      <div>
        <p class="eyebrow">Missing data</p>
        <p>${card.missing_data.length ? escapeHtml(card.missing_data.join(", ")) : "None declared for this card."}</p>
      </div>
      <div>
        <p class="eyebrow">RYO alignment</p>
        <p>${escapeHtml(card.ryo_alignment)}</p>
      </div>
      <div>
        <p class="eyebrow">Scoring</p>
        <div class="detail-grid">
          ${detailCell("Impact", String(card.score.impact))}
          ${detailCell("Relevance", nullableScore(card.score.relevance))}
          ${detailCell("Credibility", nullableScore(card.score.credibility))}
          ${detailCell("Urgency", nullableScore(card.score.urgency))}
          ${detailCell("Market confirm", nullableScore(card.score.market_confirmation))}
          ${detailCell("Uncertainty", nullableScore(card.score.uncertainty))}
        </div>
      </div>
      ${renderSourceLink(card)}
      ${renderWarnings(receipt.warnings)}
      ${renderRawJson(receipt)}
    </div>
  `;
}

function renderEvidence(receipt) {
  const target = el("#evidenceMatrix");
  const verdict = receipt.verdict || {};
  const layer = receipt.reasoning_layer || {};
  const availabilityCards = receipt.availability.map((item) => ({
    title: item.source,
    status: item.status,
    text: `${item.data_mode} - ${item.detail}`,
  }));
  const ryoCards = receipt.ryo.map((item) => ({
    title: item.tool,
    status: item.status,
    text: item.summary || item.warnings.join(", ") || "No summary returned.",
  }));
  const cards = [
    {
      title: "Reasoning layer signal",
      status: String(verdict.generated_by || "").startsWith("openai") ? "ok" : "partial",
      text: `${layer.signal || verdict.decision || "PENDING"} - ${percentText(
        layer.confidence ?? receipt.confidence,
      )} confidence - ${layer.next_action || verdict.recommended_next_action || "No action yet."}`,
    },
    ...availabilityCards,
    ...ryoCards,
  ];
  if (!cards.length) {
    target.innerHTML = `<div class="empty-state">No evidence was returned.</div>`;
    return;
  }
  target.innerHTML = cards
    .map(
      (card) => `
        <article class="evidence-card">
          <h3>
            <span>${escapeHtml(card.title)}</span>
            <span class="mini-badge ${escapeHtml(card.status)}">${escapeHtml(card.status)}</span>
          </h3>
          <p>${escapeHtml(card.text)}</p>
        </article>
      `,
    )
    .join("");
}

function renderVerdictBlock(receipt) {
  const verdict = receipt.verdict || {};
  const layer = receipt.reasoning_layer || {};
  return `
    <article class="verdict-card">
      <div class="verdict-top">
        <div>
          <p class="eyebrow">RYO-CHAN reasoning layer</p>
          <h3>${escapeHtml(layer.signal || receipt.signal || verdict.decision || "PENDING")}</h3>
        </div>
        <div class="confidence-dial">
          <strong>${escapeHtml(percentNumber(layer.confidence ?? receipt.confidence ?? verdict.confidence))}</strong>
          <span>confidence</span>
        </div>
      </div>
      <div class="detail-grid">
        ${detailCell("Token", layer.symbol || verdict.token_symbol || receipt.symbol)}
        ${detailCell("Market confirm", layer.market_confirmation || "-")}
        ${detailCell("Sentiment", layer.sentiment || "-")}
        ${detailCell("Tools used", (layer.ryo_tools_used || receipt.ryo_tools_used || []).join(", "))}
        ${detailCell("Generated by", verdict.generated_by || "pending")}
        ${detailCell("Next action", layer.next_action || verdict.recommended_next_action || "-")}
        ${detailCell("Replay ID", layer.run_id || receipt.run_id)}
      </div>
      <div>
        <p class="eyebrow">Reasoning</p>
        <p>${escapeHtml(layer.reasoning || receipt.reasoning || "No reasoning returned yet.")}</p>
      </div>
      ${listBlock("Why it matters", verdict.why_it_matters)}
      ${listBlock("Top global news", verdict.top_global_news)}
      ${listBlock("RYO market evidence", verdict.ryo_market_evidence)}
      ${listBlock("Missing data", layer.unavailable_data || verdict.missing_data)}
    </article>
  `;
}

function listBlock(label, items) {
  if (!items || !items.length) {
    return `
      <div>
        <p class="eyebrow">${escapeHtml(label)}</p>
        <p class="muted">None declared.</p>
      </div>
    `;
  }
  return `
    <div>
      <p class="eyebrow">${escapeHtml(label)}</p>
      <ol class="detail-list">
        ${items.map((item) => `<li>${escapeHtml(item)}</li>`).join("")}
      </ol>
    </div>
  `;
}

function renderRawJson(receipt) {
  return `
    <details class="raw-json">
      <summary>Raw JSON for judges</summary>
      <pre>${escapeHtml(JSON.stringify(receipt, null, 2))}</pre>
    </details>
  `;
}

function renderWarnings(warnings) {
  if (!warnings || !warnings.length) return "";
  return `
    <div>
      <p class="eyebrow">Warnings</p>
      <ol class="detail-list">
        ${warnings.map((item) => `<li>${escapeHtml(item)}</li>`).join("")}
      </ol>
    </div>
  `;
}

function renderSourceLink(card) {
  if (!card.url || card.url.startsWith("about:")) return "";
  return `
    <div>
      <p class="eyebrow">Source link</p>
      <p><a href="${escapeHtml(card.url)}" target="_blank" rel="noreferrer">${escapeHtml(card.url)}</a></p>
    </div>
  `;
}

function detailCell(label, value) {
  return `
    <div class="detail-cell">
      <span>${escapeHtml(label)}</span>
      <strong>${escapeHtml(value || "-")}</strong>
    </div>
  `;
}

function setMode(status, mode) {
  const badge = el("#modeBadge");
  badge.textContent = `${status} / ${mode}`;
  badge.className = `mode-badge ${status} ${mode}`;
}

function percentText(value) {
  if (value === null || value === undefined || value === "") return "-";
  const numeric = Number(value);
  if (Number.isNaN(numeric)) return String(value);
  return `${percentNumber(numeric)}%`;
}

function percentNumber(value) {
  if (value === null || value === undefined || value === "") return "-";
  const numeric = Number(value);
  if (Number.isNaN(numeric)) return String(value);
  return numeric <= 1 ? Math.round(numeric * 100) : Math.round(numeric);
}

function firstCard(receipt) {
  return receipt.sections.flatMap((section) => section.cards)[0];
}

function selectedCard(receipt) {
  return receipt.sections
    .flatMap((section) => section.cards)
    .find((card) => card.id === state.selectedCardId);
}

function valueOfRadio(name) {
  return el(`input[name="${name}"]:checked`)?.value || "";
}

function checkedValues(name) {
  return all(`input[name="${name}"]:checked`).map((node) => node.value);
}

async function api(path, options = {}) {
  const response = await fetch(path, {
    headers: { "content-type": "application/json" },
    ...options,
  });
  const payload = await response.json().catch(() => ({}));
  if (!response.ok) {
    const message = payload.error?.message || `${response.status} ${response.statusText}`;
    throw new Error(message);
  }
  return payload;
}

function nullableScore(value) {
  return value === null || value === undefined ? "n/a" : String(value);
}

function scoreText(value) {
  return value === null || value === undefined ? "" : `impact ${value}`;
}

function shortDate(value) {
  if (!value) return "-";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleString([], {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function escapeHtml(value) {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}
