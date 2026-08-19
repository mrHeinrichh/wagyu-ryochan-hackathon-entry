// The decision-receipt detail panel for the selected card.

import { el, escapeHtml } from "../core/dom.js";
import { percentText, nullableScore } from "../core/format.js";
import { verdictBlock } from "./verdict-view.js";
import { detailCell, rawJsonBlock, warningsBlock, sourceLinkBlock } from "./fragments.js";

/** Render the receipt detail panel, either the summary or a card breakdown. */
export function renderReceiptDetail(receipt, card) {
  const title = el("#receiptTitle");
  const target = el("#receiptDetail");
  title.textContent = receipt.symbol
    ? `${receipt.symbol} ${receipt.signal || receipt.verdict?.decision || "receipt"}`
    : "Decision receipt";

  if (!card) {
    target.innerHTML = `
      <div class="receipt-summary">
        ${verdictBlock(receipt)}
        <h3>${escapeHtml(receipt.summary.conclusion)}</h3>
        ${warningsBlock(receipt.warnings)}
        ${rawJsonBlock(receipt)}
      </div>
    `;
    return;
  }

  target.innerHTML = `
    <div class="receipt-summary">
      ${verdictBlock(receipt)}
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
      ${sourceLinkBlock(card)}
      ${warningsBlock(receipt.warnings)}
      ${rawJsonBlock(receipt)}
    </div>
  `;
}
