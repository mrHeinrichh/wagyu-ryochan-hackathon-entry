// DOM helpers shared across views.

/** Query a single element. */
export const el = (selector) => document.querySelector(selector);

/** Query all matching elements as an array. */
export const all = (selector) => Array.from(document.querySelectorAll(selector));

/**
 * Escape a value for safe interpolation into innerHTML.
 * @param {unknown} value
 * @returns {string}
 */
export function escapeHtml(value) {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}
