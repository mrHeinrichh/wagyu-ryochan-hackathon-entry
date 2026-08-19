import type { Metadata } from "next";
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
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <script dangerouslySetInnerHTML={{ __html: themeScript }} />
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link rel="preconnect" href="https://fonts.gstatic.com" crossOrigin="anonymous" />
        <link
          href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800;900&display=swap"
          rel="stylesheet"
        />
      </head>
      <body>{children}</body>
    </html>
  );
}
