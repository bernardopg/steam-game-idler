// next.config.mjs takes precedence at runtime when both files exist.
// This file is kept in sync so either one works as the canonical config.
import type { NextConfig } from 'next'

const nextConfig: NextConfig = {
  output: 'export',
  devIndicators: false,
  transpilePackages: ['geist'],
  images: {
    unoptimized: true,
    remotePatterns: [
      {
        protocol: 'https',
        hostname: 'avatars.steamstatic.com',
      },
      {
        protocol: 'https',
        hostname: 'cdn.cloudflare.steamstatic.com',
      },
    ],
  },
}

export default nextConfig
