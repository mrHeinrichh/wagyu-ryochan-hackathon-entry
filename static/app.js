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
  setDateline();
  initThemeControls();
  bindEvents();
  await Promise.all([loadHealth(), loadTokens(), loadHistory(), loadWatchlist()]);
}

function setDateline() {
  const now = new Date();
  const opts = { weekday: "long", year: "numeric", month: "long", day: "numeric" };
  el("#mastheadDate").textContent = now.toLocaleDateString([], opts).toUpperCase();
}

function initThemeControls() {
  applyTheme(state.theme);
  el("#themeToggle").addEventListener("click", () => {
    const order = ["light", "dark", "system"];
    const next = order[(order.indexOf(state.theme) + 1) % order.length];
    applyTheme(next);
  });
}

function applyTheme(theme) {
  const next = ["light", "system", "dark"].includes(theme) ? theme : "system";
  state.theme = next;
  localStorage.setItem("gtp-theme", next);
  document.documentElement.dataset.theme = next;
  const label = next === "system" ? "Auto Light" : next === "dark" ? "Reading Light: On" : "Reading Light: Off";
  el("#themeToggle").textContent = label;
}

function bindEvents() {
  el("#pulseForm").addEventListener("submit", async (event) => {
    event.preventDefault();
    await runPulse();
  });
  el("#watchButton").addEventListener("click", addWatch);
}

async function loadHealth() {
  try {
    state.health = await api("/health");
    renderHealth(state.health);
  } catch (error) {
    el("#healthStack").innerHTML = markup(
      "span",
      "wire-item bad",
      "Wire down " + escapeHtml(error.message),
    );
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
    /* input still works without the helper list */
  }
}

async function loadHistory() {
  try {
    const receipts = await api("/api/receipts");
    const target = el("#receiptHistory");
    if (!receipts.length) {
      target.innerHTML = '<p class="muted small">No editions filed yet.</p>';
      return;
    }
    target.innerHTML = "";
    for (const item of receipts) {
      const button = document.createElement("button");
      button.className = "archive-item";
      button.type = "button";
      button.innerHTML =
        "<strong>" +
        escapeHtml(item.symbol) +
        " &mdash; " +
        escapeHtml(item.signal || "") +
        "</strong><span>" +
        percentText(item.confidence) +
        " &middot; " +
        escapeHtml(item.data_mode) +
        " &middot; " +
        shortDate(item.created_at) +
        "</span>";
      button.addEventListener("click", async () => {
        const receipt = await api("/api/receipts/" + encodeURIComponent(item.id));
        renderReceipt(receipt);
        window.scrollTo({ top: 0, behavior: "smooth" });
      });
      target.appendChild(button);
    }
  } catch {
    el("#receiptHistory").innerHTML = '<p class="muted small">Archive unavailable.</p>';
  }
}

async function loadWatchlist() {
  try {
    const watchlist = await api("/api/watchlist");
    renderWatchlist(watchlist);
  } catch {
    el("#watchlist").innerHTML = '<p class="muted small">Watch desk unavailable.</p>';
  }
}

async function runPulse() {
  const button = el("#runButton");
  const symbol = el("#symbolInput").value;
  button.disabled = true;
  button.textContent = "Setting type…";
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
    el("#leadStory").innerHTML =
      '<div class="empty-broadsheet"><h2>The presses jammed</h2><p>' +
      escapeHtml(error.message) +
      "</p></div>";
  } finally {
    button.disabled = false;
    button.textContent = "Run the Presses";
  }
}

async function addWatch() {
  const symbol = el("#symbolInput").value;
  const button = el("#watchButton");
  button.disabled = true;
  button.textContent = "Assigning…";
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
    button.textContent = "Assign a Watch";
  }
}

async function removeWatch(symbol) {
  await api("/api/watchlist/" + encodeURIComponent(symbol), { method: "DELETE" });
  await loadWatchlist();
}

function renderHealth(health) {
  const lines = [
    ["Backend", health.status, "ok"],
    ["RYO MCP", health.ryo_configured ? "live" : "missing key", health.ryo_configured ? "ok" : "bad"],
    ["Tavily", health.tavily_configured ? "live" : "missing key", health.tavily_configured ? "ok" : "warn"],
    ["OpenAI", health.openai_configured ? "live" : "missing key", health.openai_configured ? "ok" : "warn"],
    ["CoinGecko", health.coingecko_configured ? "backup" : "optional", health.coingecko_configured ? "ok" : "warn"],
    ["Watch loop", health.watch_loop_enabled ? "on" : "manual", health.watch_loop_enabled ? "ok" : "warn"],
  ];
  el("#healthStack").innerHTML = lines
    .map(
      ([label, value, klass]) =>
        '<span class="wire-item ' +
        klass +
        '">' +
        escapeHtml(label) +
        " <strong>" +
        escapeHtml(value) +
        "</strong></span>",
    )
    .join("");
}

function renderWatchlist(watchlist) {
  const target = el("#watchlist");
  if (!watchlist.length) {
    target.innerHTML = '<p class="muted small">No tokens on watch.</p>';
    return;
  }
  target.innerHTML = "";
  for (const item of watchlist) {
    const row = document.createElement("div");
    row.className = "watch-item";
    row.innerHTML =
      "<strong>" +
      escapeHtml(item.symbol) +
      "</strong><span>every " +
      item.interval_minutes +
      " min &middot; next " +
      shortDate(item.next_check_at) +
      "</span>";
    const remove = document.createElement("button");
    remove.className = "watch-remove";
    remove.type = "button";
    remove.textContent = "Unassign";
    remove.addEventListener("click", () => removeWatch(item.symbol));
    row.appendChild(remove);
    target.appendChild(row);
  }
}

function renderReceipt(receipt) {
  state.currentReceipt = receipt;
  state.selectedCardId = firstCard(receipt)?.id || null;
  renderBanner(receipt);
  renderLeadStory(receipt);
  renderMarketPulse(receipt);
  renderRecommendations(receipt);
  renderDispatch(receipt);
  renderFeed(receipt);
  renderReceiptDetail(receipt, selectedCard(receipt));
}

// Exposed so a demo or an automated check can paint a receipt without live keys.
window.renderReceipt = renderReceipt;

function renderBanner(receipt) {
  const banner = el("#bannerVerdict");
  const signal = receipt.signal || receipt.verdict?.decision || "PENDING";
  banner.hidden = false;
  el("#bannerKicker").innerHTML = escapeHtml(receipt.symbol) + " &mdash; the desk rules";
  const signalNode = el("#bannerSignal");
  signalNode.textContent = signal;
  signalNode.className = "banner-signal " + signal;
  el("#bannerConfidence").textContent = percentText(receipt.confidence) + " confidence";
}

function renderLeadStory(receipt) {
  const verdict = receipt.verdict || {};
  const layer = receipt.reasoning_layer || {};
  const lead = firstCard(receipt);
  const hed = lead ? lead.headline : receipt.summary.headline;
  const dek = receipt.summary.conclusion;
  const bodyParts = [];
  if (layer.reasoning || receipt.reasoning) {
    bodyParts.push(layer.reasoning || receipt.reasoning);
  }
  for (const point of verdict.why_it_matters || []) {
    bodyParts.push(point);
  }
  if (!bodyParts.length) {
    bodyParts.push("The desk reviewed the available evidence and found nothing decision-grade for this token.");
  }
  const meta = [
    ["Signal", receipt.signal],
    ["Confidence", percentText(receipt.confidence)],
    ["Market read", layer.market_confirmation || "-"],
    ["Sentiment", layer.sentiment || "-"],
    ["Filed by", verdict.generated_by || "desk"],
    ["Run", receipt.run_id],
  ];
  el("#leadStory").innerHTML =
    '<h2 class="lead-hed">' +
    escapeHtml(hed) +
    '</h2><p class="lead-dek">' +
    escapeHtml(dek) +
    '</p><div class="lead-body">' +
    bodyParts.map((part) => "<p>" + escapeHtml(part) + "</p>").join("") +
    '</div><div class="lead-meta">' +
    meta
      .map(([label, value]) => "<span>" + escapeHtml(label) + ": <b>" + escapeHtml(value || "-") + "</b></span>")
      .join("") +
    "</div>";
}

function renderMarketPulse(receipt) {
  const pulse = receipt.market_pulse || {};
  const value = pulse.fear_greed_value;
  const dial = el("#gaugeDial");
  const valueNode = el("#gaugeValue");
  const labelNode = el("#gaugeLabel");
  const needle = el("#gaugeNeedle");
  if (value === null || value === undefined) {
    valueNode.textContent = "N/A";
    labelNode.textContent = "Unavailable";
    if (needle) needle.style.transform = "translateX(-50%) rotate(0deg)";
  } else {
    valueNode.textContent = String(value);
    labelNode.textContent = pulse.fear_greed_label || "Fear & Greed";
    if (needle) {
      const angle = -90 + (value / 100) * 180;
      needle.style.transform = "translateX(-50%) rotate(" + angle + "deg)";
    }
  }
  const facts = el("#moodFacts");
  facts.innerHTML =
    factRow("Regime", pulse.regime) +
    factRow("BTC Dominance", pulse.btc_dominance != null ? pulse.btc_dominance.toFixed(1) + "%" : null) +
    factRow("Breadth", pulse.breadth) +
    factRow("As of", pulse.as_of ? shortDate(pulse.as_of) : null);
  el("#moodNote").textContent = pulse.note || "";
}

function factRow(label, value) {
  return (
    "<div><dt>" +
    escapeHtml(label) +
    "</dt><dd>" +
    escapeHtml(value != null && value !== "" ? String(value) : "—") +
    "</dd></div>"
  );
}

function renderRecommendations(receipt) {
  const box = el("#recommendationBox");
  const list = el("#recommendationList");
  const items = receipt.recommendations || [];
  if (!items.length) {
    box.hidden = true;
    return;
  }
  box.hidden = false;
  list.innerHTML = items.map((item) => "<li>" + escapeHtml(item) + "</li>").join("");
}

function renderDispatch(receipt) {
  const box = el("#dispatchBox");
  box.hidden = false;
  el("#editorNote").textContent = receipt.editor_note || receipt.summary.conclusion;
  const s = receipt.summary;
  const cells = [
    ["Position-changing", s.position_changing_count],
    ["Watch closely", s.watch_count],
    ["Unverified", s.unverified_count],
    ["Noise", s.noise_count],
  ];
  el("#scoreboard").innerHTML = cells
    .map(
      ([label, count]) =>
        '<div class="score-cell"><span>' + escapeHtml(label) + "</span><strong>" + count + "</strong></div>",
    )
    .join("");
}

function renderFeed(receipt) {
  const target = el("#feedSections");
  target.innerHTML = "";
  const template = el("#sectionTemplate");
  const storyTemplate = el("#storyTemplate");
  let totalCards = 0;

  for (const section of receipt.sections) {
    if (!section.cards.length) continue;
    totalCards += section.cards.length;
    const node = template.content.cloneNode(true);
    node.querySelector("h3").textContent = section.label;
    node.querySelector(".desk-count").textContent =
      section.cards.length + (section.cards.length === 1 ? " dispatch" : " dispatches");
    const list = node.querySelector(".clipping-list");

    for (const card of section.cards) {
      const story = storyTemplate.content.cloneNode(true);
      const article = story.querySelector(".clipping");
      if (card.id === state.selectedCardId) article.classList.add("active");
      story.querySelector(".impact-tag").textContent = "impact " + card.score.impact;
      story.querySelector(".cluster-tag").textContent = card.narrative_cluster;
      const dataTag = story.querySelector(".data-tag");
      dataTag.textContent = card.data_mode;
      dataTag.classList.add(card.data_mode);
      story.querySelector("h4").textContent = card.headline;
      story.querySelector(".clipping-dek").textContent = card.reasoning[0] || card.ryo_alignment;
      story.querySelector(".clipping-byline").textContent =
        card.source + " · " + card.sentiment + " · " + card.recommendation;
      story.querySelector(".clipping-verdict strong").textContent = card.score.impact;
      story.querySelector(".clipping-verdict span").textContent = card.confidence + " conf.";
      story.querySelector(".clipping-hit").addEventListener("click", () => {
        state.selectedCardId = card.id;
        renderFeed(receipt);
        renderReceiptDetail(receipt, card);
      });
      list.appendChild(story);
    }
    target.appendChild(node);
  }

  if (!totalCards) {
    target.innerHTML =
      '<p class="desk-empty">No dispatches cleared the desk. Check the wire status and run again once real keys are live.</p>';
  }
}

function renderReceiptDetail(receipt, card) {
  const target = el("#receiptDetail");
  const verdict = receipt.verdict || {};
  const layer = receipt.reasoning_layer || {};
  const focus = card || firstCard(receipt);

  const head =
    '<h3 class="rec-hed">' +
    escapeHtml(focus ? focus.headline : receipt.summary.headline) +
    "</h3>";

  const grid =
    '<div class="detail-grid">' +
    detailCell("Signal", receipt.signal) +
    detailCell("Confidence", percentText(receipt.confidence)) +
    detailCell("Market read", layer.market_confirmation) +
    detailCell("Sentiment", layer.sentiment) +
    detailCell("Tools used", (layer.ryo_tools_used || []).join(", ")) +
    detailCell("Next action", receipt.next_action) +
    "</div>";

  let cardBlock = "";
  if (focus) {
    cardBlock =
      '<p class="eyebrow">Reasoning trace</p><ol class="detail-list">' +
      focus.reasoning.map((item) => "<li>" + escapeHtml(item) + "</li>").join("") +
      "</ol>" +
      '<p class="eyebrow">Missing data</p><p class="small">' +
      (focus.missing_data.length ? escapeHtml(focus.missing_data.join(", ")) : "None declared for this card.") +
      "</p>" +
      '<p class="eyebrow">RYO alignment</p><p class="small">' +
      escapeHtml(focus.ryo_alignment) +
      "</p>" +
      sourceLink(focus);
  }

  const missing =
    '<p class="eyebrow">Data we could not get</p><p class="small">' +
    ((receipt.unavailable_data || []).length
      ? escapeHtml(receipt.unavailable_data.join(", "))
      : "Nothing withheld this run.") +
    "</p>";

  target.innerHTML =
    head + grid + cardBlock + missing + renderWarnings(receipt.warnings) + renderRawJson(receipt);
}

function sourceLink(card) {
  if (!card.url || card.url.startsWith("about:")) return "";
  return (
    '<p class="eyebrow">Source</p><p class="small"><a href="' +
    escapeHtml(card.url) +
    '" target="_blank" rel="noreferrer">' +
    escapeHtml(card.url) +
    "</a></p>"
  );
}

function renderWarnings(warnings) {
  if (!warnings || !warnings.length) return "";
  return (
    '<div class="warn-block"><p class="eyebrow">Editor\u2019s warnings</p><ol class="detail-list">' +
    warnings.map((item) => "<li>" + escapeHtml(item) + "</li>").join("") +
    "</ol></div>"
  );
}

function renderRawJson(receipt) {
  return (
    '<details class="raw-json"><summary>Raw record for judges</summary><pre>' +
    escapeHtml(JSON.stringify(receipt, null, 2)) +
    "</pre></details>"
  );
}

function detailCell(label, value) {
  return (
    '<div class="detail-cell"><span>' +
    escapeHtml(label) +
    "</span><strong>" +
    escapeHtml(value || "-") +
    "</strong></div>"
  );
}

function markup(tag, className, text) {
  return "<" + tag + ' class="' + className + '">' + escapeHtml(text) + "</" + tag + ">";
}

function percentText(value) {
  if (value === null || value === undefined || value === "") return "-";
  const numeric = Number(value);
  if (Number.isNaN(numeric)) return String(value);
  const pct = numeric <= 1 ? Math.round(numeric * 100) : Math.round(numeric);
  return pct + "%";
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
  return el('input[name="' + name + '"]:checked')?.value || "";
}

function checkedValues(name) {
  return all('input[name="' + name + '"]:checked').map((node) => node.value);
}

async function api(path, options = {}) {
  const response = await fetch(path, {
    headers: { "content-type": "application/json" },
    ...options,
  });
  const payload = await response.json().catch(() => ({}));
  if (!response.ok) {
    const message = payload.error?.message || response.status + " " + response.statusText;
    throw new Error(message);
  }
  return payload;
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
