// The central receipt view: paints the feed, metrics, detail, and evidence
// whenever a new receipt is selected or a card is clicked.

import { el, all } from "../core/dom.js";
import { store } from "../core/store.js";
import { percentText } from "../core/format.js";
import { firstCard, findCard } from "../core/receipt.js";
import { renderReceiptDetail } from "./detail-view.js";
import { renderEvidence } from "./evidence-view.js";

/** Render a full receipt: title, mode, metrics, feed, detail, evidence. */
export function renderReceipt(receipt) {
  store.set({ currentReceipt: receipt, selectedCardId: firstCard(receipt)?.id || null });

  el("#feedTitle").textContent = receipt.summary.headline;
  setMode(receipt.status, receipt.data_mode);
  renderMetrics(receipt);
  renderFeed(receipt);
  renderReceiptDetail(receipt, findCard(receipt, store.get().selectedCardId));
  renderEvidence(receipt);
}

/** Update the four headline metrics. */
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

/** Render the ranked feed sections and their story cards. */
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
      list.appendChild(buildStoryCard(receipt, card, storyTemplate));
    }
    target.appendChild(node);
  }

  if (!totalCards) {
    target.innerHTML = `<div class="empty-state">No cards were produced. Check the source availability and run again with live keys.</div>`;
  }
}

/** Build a single story card element and wire its selection handler. */
function buildStoryCard(receipt, card, storyTemplate) {
  const story = storyTemplate.content.cloneNode(true);
  const article = story.querySelector(".story-row");
  if (card.id === store.get().selectedCardId) article.classList.add("active");
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
    store.set({ selectedCardId: card.id });
    renderFeed(receipt);
    renderReceiptDetail(receipt, card);
  });
  return story;
}

/** Update the status/data-mode badge. */
export function setMode(status, mode) {
  const badge = el("#modeBadge");
  badge.textContent = `${status} / ${mode}`;
  badge.className = `mode-badge ${status} ${mode}`;
}
