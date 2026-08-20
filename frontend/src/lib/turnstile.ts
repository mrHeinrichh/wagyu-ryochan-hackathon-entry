const SITE_KEY = process.env.NEXT_PUBLIC_TURNSTILE_SITE_KEY?.trim();
const API_WAIT_MS = 10_000;
const CHALLENGE_TIMEOUT_MS = 30_000;

export type TurnstileAction = "market_analysis" | "assistant_chat";

interface TurnstileApi {
  render(
    container: HTMLElement,
    options: {
      sitekey: string;
      action: TurnstileAction;
      size: "invisible";
      execution: "execute";
      callback: (token: string) => void;
      "error-callback": () => void;
      "expired-callback": () => void;
      "timeout-callback": () => void;
    },
  ): string;
  execute(widgetId: string): void;
  remove(widgetId: string): void;
}

declare global {
  interface Window {
    turnstile?: TurnstileApi;
  }
}

export function turnstileConfigured(): boolean {
  return Boolean(SITE_KEY);
}

/// Produce headers for exactly one protected request. Turnstile tokens are
/// deliberately never cached or reused.
export async function turnstileHeaders(action: TurnstileAction): Promise<HeadersInit> {
  if (!SITE_KEY) return {};
  return { "x-turnstile-token": await executeChallenge(SITE_KEY, action) };
}

async function executeChallenge(siteKey: string, action: TurnstileAction): Promise<string> {
  const api = await waitForApi();
  const container = document.createElement("div");
  container.className = "turnstile-executor";
  container.setAttribute("aria-hidden", "true");
  document.body.appendChild(container);

  return new Promise<string>((resolve, reject) => {
    let widgetId: string | undefined;
    let settled = false;
    const finish = (token?: string) => {
      if (settled) return;
      settled = true;
      window.clearTimeout(timeout);
      if (widgetId) api.remove(widgetId);
      container.remove();
      if (token) {
        resolve(token);
      } else {
        reject(new Error("The security check could not finish. Please try again."));
      }
    };
    const timeout = window.setTimeout(() => finish(), CHALLENGE_TIMEOUT_MS);

    try {
      widgetId = api.render(container, {
        sitekey: siteKey,
        action,
        size: "invisible",
        execution: "execute",
        callback: (token) => finish(token),
        "error-callback": () => finish(),
        "expired-callback": () => finish(),
        "timeout-callback": () => finish(),
      });
      api.execute(widgetId);
    } catch {
      finish();
    }
  });
}

async function waitForApi(): Promise<TurnstileApi> {
  const started = Date.now();
  while (!window.turnstile) {
    if (Date.now() - started >= API_WAIT_MS) {
      throw new Error("The security check is unavailable. Please refresh and try again.");
    }
    await new Promise((resolve) => window.setTimeout(resolve, 50));
  }
  return window.turnstile;
}
