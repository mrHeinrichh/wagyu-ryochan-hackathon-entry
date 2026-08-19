// Pure formatting helpers. No DOM, no side effects.

/**
 * Render a 0-1 ratio or 0-100 number as a percent string, or "-" when absent.
 * @param {unknown} value
 * @returns {string}
 */
export function percentText(value) {
  if (value === null || value === undefined || value === "") return "-";
  const numeric = Number(value);
  if (Number.isNaN(numeric)) return String(value);
  return `${percentNumber(numeric)}%`;
}

/**
 * Render a 0-1 ratio or 0-100 number as a rounded integer, or "-" when absent.
 * @param {unknown} value
 * @returns {number|string}
 */
export function percentNumber(value) {
  if (value === null || value === undefined || value === "") return "-";
  const numeric = Number(value);
  if (Number.isNaN(numeric)) return String(value);
  return numeric <= 1 ? Math.round(numeric * 100) : Math.round(numeric);
}

/** Render a nullable numeric sub-score, using "n/a" when it was unavailable. */
export function nullableScore(value) {
  return value === null || value === undefined ? "n/a" : String(value);
}

/** Format an ISO timestamp compactly, falling back to the raw value. */
export function shortDate(value) {
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
