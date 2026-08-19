// The pulse action: submit the form, render the resulting receipt, refresh
// the history list. Errors surface in the feed area rather than throwing.

import { el, escapeHtml } from "../core/dom.js";
import { endpoints } from "../core/api.js";
import { readPulseForm } from "../core/forms.js";
import { renderReceipt, setMode } from "./receipt-view.js";
import { loadHistory } from "./history-view.js";

/** Run a reasoning pass from the current form and paint the result. */
export async function runPulse() {
  const button = el("#runButton");
  button.disabled = true;
  button.textContent = "Running";
  setMode("partial", "running");

  try {
    const receipt = await endpoints.reason(readPulseForm());
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
