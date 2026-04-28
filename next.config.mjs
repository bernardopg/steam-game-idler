import path from 'node:path'
import { fileURLToPath } from 'node:url'

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)

const nextConfig = {
  output: 'export',
  devIndicators: false,
  transpilePackages: ['geist'],
  turbopack: {
    root: __dirname,
  },
  allowedDevOrigins: ['http://127.0.0.1:3000', 'http://localhost:3000'],
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
