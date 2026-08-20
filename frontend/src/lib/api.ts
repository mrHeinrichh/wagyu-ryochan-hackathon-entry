// Typed fetch client for the Rust backend. Errors are normalized to Error.

import type {
  ChatRequestBody,
  ChatResponse,
  DecisionReceipt,
  HealthResponse,
  PulseRequestBody,
  ReceiptListItem,
  TokenInfo,
  WatchItem,
} from "./types";

interface ApiErrorBody {
  error?: { message?: string };
}

async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
  const response = await fetch(path, {
    headers: { "content-type": "application/json" },
    ...options,
  });
  const payload = (await response.json().catch(() => ({}))) as T & ApiErrorBody;
  if (!response.ok) {
    const message = payload.error?.message ?? `${response.status} ${response.statusText}`;
    throw new Error(message);
  }
  return payload;
}

export interface TokensResponse {
  status: string;
  data_mode: string;
  source: string;
  as_of: string;
  cached: boolean;
  tokens: TokenInfo[];
  warnings: string[];
}

export const api = {
  health: () => request<HealthResponse>("/health"),
  tokens: () => request<TokensResponse>("/api/tokens"),
  receipts: () => request<ReceiptListItem[]>("/api/receipts"),
  receipt: (id: string) => request<DecisionReceipt>(`/api/receipts/${encodeURIComponent(id)}`),
  reason: (body: PulseRequestBody) =>
    request<DecisionReceipt>("/reason", { method: "POST", body: JSON.stringify(body) }),
  demo: (body: PulseRequestBody) =>
    request<DecisionReceipt>("/api/demo", { method: "POST", body: JSON.stringify({ ...body, demo: true }) }),
  chat: (body: ChatRequestBody) =>
    request<ChatResponse>("/api/chat", { method: "POST", body: JSON.stringify(body) }),
  watchlist: () => request<WatchItem[]>("/api/watchlist"),
  addWatch: (body: { symbol: string; interval_minutes: number }) =>
    request<WatchItem>("/api/watchlist", { method: "POST", body: JSON.stringify(body) }),
  removeWatch: (symbol: string) =>
    request<{ status: string; removed: string }>(`/api/watchlist/${encodeURIComponent(symbol)}`, {
      method: "DELETE",
    }),
};
