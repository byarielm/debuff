import type { NextConfig } from "next";

const apiBase = process.env.API_URL || "http://localhost:3000";

const nextConfig: NextConfig = {
  reactCompiler: true,
  trailingSlash: true,
  images: { unoptimized: true },
};

if (process.env.NODE_ENV === "production") {
  nextConfig.output = "export";
} else {
  nextConfig.rewrites = async () => ({
    beforeFiles: [
      { source: "/api/:path*", destination: `${apiBase}/api/:path*` },
      { source: "/auth/:path*", destination: `${apiBase}/auth/:path*` },
      { source: "/xrpc/:path*", destination: `${apiBase}/xrpc/:path*` },
      { source: "/health", destination: `${apiBase}/health` },
      { source: "/oauth/:path*", destination: `${apiBase}/oauth/:path*` },
    ],
    afterFiles: [],
    fallback: [],
  });
}

export default nextConfig;
