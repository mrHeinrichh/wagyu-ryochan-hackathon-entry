// Reusable HTML fragment builders shared by the receipt views.
//
// Each returns an escaped HTML string. Keeping them here avoids duplicating
// markup across the verdict, detail, and evidence views.

import { escapeHtml } from "../core/dom.js";

/** A labelled key/value cell. */
export function detailCell(label, value) {
  return `
    <div class="detail-cell">
      <span>${escapeHtml(label)}</span>
      <strong>${escapeHtml(value || "-")}</strong>
    </div>
  `;
}

/** A labelled ordered list, or a "None declared." note when empty. */
export function listBlock(label, items) {
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

/** The collapsible raw-JSON block for judges. */
export function rawJsonBlock(receipt) {
  return `
    <details class="raw-json">
      <summary>Raw JSON for judges</summary>
      <pre>${escapeHtml(JSON.stringify(receipt, null, 2))}</pre>
    </details>
  `;
}

/** A warnings list, or empty string when there are none. */
export function warningsBlock(warnings) {
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

/** A source link block, hidden for synthetic about: URLs. */
export function sourceLinkBlock(card) {
  if (!card.url || card.url.startsWith("about:")) return "";
  return `
    <div>
      <p class="eyebrow">Source link</p>
      <p><a href="${escapeHtml(card.url)}" target="_blank" rel="noreferrer">${escapeHtml(card.url)}</a></p>
    </div>
  `;
}
