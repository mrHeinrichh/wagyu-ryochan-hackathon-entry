// Thin fetch wrapper that centralizes JSON handling and error shaping.
//
// Every backend call goes through `api` so error messages are consistent and
// the endpoints live in one place.

/**
 * Call a backend endpoint and return parsed JSON, throwing on non-2xx.
 * @param {string} path
 * @param {RequestInit} [options]
 * @returns {Promise<any>}
 */
export async function api(path, options = {}) {
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

// Named endpoint helpers keep call sites declarative.
export const endpoints = {
  health: () => api("/health"),
  tokens: () => api("/api/tokens"),
  receipts: () => api("/api/receipts"),
  receipt: (id) => api(`/api/receipts/${encodeURIComponent(id)}`),
  reason: (body) => api("/reason", { method: "POST", body: JSON.stringify(body) }),
  watchlist: () => api("/api/watchlist"),
  addWatch: (body) => api("/api/watchlist", { method: "POST", body: JSON.stringify(body) }),
  removeWatch: (symbol) =>
    api(`/api/watchlist/${encodeURIComponent(symbol)}`, { method: "DELETE" }),
};
