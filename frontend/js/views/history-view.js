// Sidebar view: recent receipt history, clickable to reopen a run.

import { el, escapeHtml } from "../core/dom.js";
import { endpoints } from "../core/api.js";
import { percentText, shortDate } from "../core/format.js";
import { renderReceipt } from "./receipt-view.js";

/** Load the receipt history list and wire each entry to reopen its receipt. */
export async function loadHistory() {
  try {
    const receipts = await endpoints.receipts();
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
        const receipt = await endpoints.receipt(item.id);
        renderReceipt(receipt);
      });
      target.appendChild(button);
    }
  } catch {
    el("#receiptHistory").innerHTML = `<div class="empty-mini">Receipt history unavailable</div>`;
  }
}
