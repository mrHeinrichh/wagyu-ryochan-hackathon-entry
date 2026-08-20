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
  error?: { code?: string; message?: string; retry_after_seconds?: number };
}

export class ApiRequestError extends Error {
  constructor(
    message: string,
    public readonly status: number,
    public readonly code?: string,
    public readonly retryAfterSeconds?: number,
  ) {
    super(message);
    this.name = "ApiRequestError";
  }
}

async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
  const response = await fetch(path, {
    headers: { "content-type": "application/json" },
    ...options,
  });
  const payload = (await response.json().catch(() => ({}))) as T & ApiErrorBody;
  if (!response.ok) {
    const message = payload.error?.message ?? `${response.status} ${response.statusText}`;
    const retryHeader = Number(response.headers.get("retry-after"));
    const retryAfter = payload.error?.retry_after_seconds ?? (Number.isFinite(retryHeader) ? retryHeader : undefined);
    throw new ApiRequestError(message, response.status, payload.error?.code, retryAfter);
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
