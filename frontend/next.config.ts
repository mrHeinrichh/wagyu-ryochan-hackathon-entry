import type { NextConfig } from "next";

// Serve as a static export the Rust backend can host from `frontend/out`.
// Set BUILD_STATIC=1 for the production export; plain `next dev` keeps the
// dev server with API rewrites to the local backend.
const isStatic = process.env.BUILD_STATIC === "1";

// Where the Rust backend listens during development.
const BACKEND_ORIGIN = process.env.BACKEND_ORIGIN ?? "http://127.0.0.1:8788";

const nextConfig: NextConfig = {
  reactStrictMode: true,
  ...(isStatic
    ? { output: "export", distDir: "out", images: { unoptimized: true } }
    : {}),
  async rewrites() {
    // Rewrites only apply to `next dev`/`next start`, not static export.
    if (isStatic) return [];
    return [
      { source: "/health", destination: `${BACKEND_ORIGIN}/health` },
      { source: "/reason", destination: `${BACKEND_ORIGIN}/reason` },
      { source: "/runs/:path*", destination: `${BACKEND_ORIGIN}/runs/:path*` },
      { source: "/api/:path*", destination: `${BACKEND_ORIGIN}/api/:path*` },
    ];
  },
};

export default nextConfig;
