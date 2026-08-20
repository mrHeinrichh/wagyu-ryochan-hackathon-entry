import type { Metadata } from "next";
import Script from "next/script";
import "@/styles/app.css";

export const metadata: Metadata = {
  title: "RYO Global Token Reasoning Layer",
  description:
    "Turn market reads, global news, or a user thesis into an inspectable CONFIRMED, WATCHLIST, or REJECTED decision.",
  icons: { icon: "/assets/ryo-favicon.svg" },
};

// Apply the persisted theme before paint to avoid a flash.
const themeScript = `(() => {
  try {
    const theme = localStorage.getItem("gtp-theme") || "system";
    document.documentElement.dataset.theme = theme;
  } catch (_) {
    document.documentElement.dataset.theme = "system";
  }
})();`;

export default function RootLayout({ children }: { children: React.ReactNode }) {
  const turnstileEnabled = Boolean(process.env.NEXT_PUBLIC_TURNSTILE_SITE_KEY?.trim());

  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <script dangerouslySetInnerHTML={{ __html: themeScript }} />
      </head>
      <body>
        {children}
        {turnstileEnabled ? (
          <Script
            id="cloudflare-turnstile"
            src="https://challenges.cloudflare.com/turnstile/v0/api.js?render=explicit"
            strategy="afterInteractive"
          />
        ) : null}
      </body>
    </html>
  );
}
