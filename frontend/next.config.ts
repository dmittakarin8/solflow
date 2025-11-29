import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  // Disable Cache Components for now to avoid prerendering conflicts
  // We'll use dynamic routes instead
  // cacheComponents: false,

  // React Compiler is built-in to React 19.2
  // No configuration needed - automatically enabled

  // Turbopack enabled by default in Next.js 16
  // No config needed - use `next dev --turbopack`
};

export default nextConfig;
