// The evidence matrix: reasoning-layer signal plus source and RYO tool status.

import { el, escapeHtml } from "../core/dom.js";
import { percentText } from "../core/format.js";

/** Render the evidence cards from availability and RYO tool records. */
export function renderEvidence(receipt) {
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
