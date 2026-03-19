import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { execSync } from "child_process";
import { get as httpsGet } from "https";
import { get as httpGet } from "http";
import type { IncomingMessage, ServerResponse } from "http";

// Cache yt-dlp URLs for 30 min (YouTube URLs expire after ~6h)
const urlCache = new Map<string, { url: string; ts: number }>();
const CACHE_TTL = 30 * 60 * 1000;

// YouTube video IDs per camera slug
const YOUTUBE_IDS: Record<string, string> = {
  'shibuya-crossing': 'dfVK7ld38Ys',
  'shibuya-station': 'DjdUEyjx8GM',
  'hachiko-square': 'ehJbjfH1dIo',
  'center-gai': '6dp-bvQ7RWo',
};

function resolveYoutubeHls(slug: string): string | null {
  const videoId = YOUTUBE_IDS[slug];
  if (!videoId) return null;

  const cached = urlCache.get(slug);
  if (cached && Date.now() - cached.ts < CACHE_TTL) return cached.url;

  try {
    const url = execSync(
      `yt-dlp -f 'best[vcodec^=avc1]' --no-warnings -g 'https://www.youtube.com/watch?v=${videoId}'`,
      { timeout: 15000, encoding: 'utf-8' }
    ).trim();
    urlCache.set(slug, { url, ts: Date.now() });
    return url;
  } catch {
    return null;
  }
}

/** Proxy fetch: downloads remote URL and pipes to response */
function proxyFetch(remoteUrl: string, res: ServerResponse) {
  const getter = remoteUrl.startsWith('https') ? httpsGet : httpGet;
  getter(remoteUrl, (upstream: IncomingMessage) => {
    const ct = upstream.headers['content-type'] || 'application/octet-stream';
    res.writeHead(upstream.statusCode || 502, {
      'Content-Type': ct,
      'Access-Control-Allow-Origin': '*',
      'Cache-Control': 'no-cache',
    });
    upstream.pipe(res);
  }).on('error', () => {
    res.writeHead(502);
    res.end('upstream error');
  });
}

/** Rewrite m3u8 so relative URLs go through our proxy */
function rewriteM3u8(body: string, baseUrl: string, proxyBase: string, slug: string): string {
  return body.replace(/^(?!#)(https?:\/\/\S+|[^\s#]\S+)/gm, (match) => {
    const absolute = match.startsWith('http') ? match : new URL(match, baseUrl).href;
    return `${proxyBase}?src=${slug}&seg=${encodeURIComponent(absolute)}`;
  });
}

export default defineConfig(async () => ({
  plugins: [
    react(),
    {
      name: 'hls-proxy',
      configureServer(server) {
        // HLS proxy: /hls-proxy?src=shibuya-crossing → proxied m3u8
        // Segment proxy: /hls-proxy?src=shibuya-crossing&seg=https://...ts → proxied segment
        server.middlewares.use((req: IncomingMessage, res: ServerResponse, next: () => void) => {
          if (!req.url?.startsWith('/hls-proxy')) return next();

          const params = new URL(req.url, 'http://localhost').searchParams;
          const slug = params.get('src');
          const seg = params.get('seg');

          if (!slug) {
            res.writeHead(400);
            res.end('missing src');
            return;
          }

          // Segment proxy – just pipe through
          if (seg) {
            proxyFetch(seg, res);
            return;
          }

          // Master playlist – resolve via yt-dlp, fetch, rewrite URLs
          const hlsUrl = resolveYoutubeHls(slug);
          if (!hlsUrl) {
            res.writeHead(404);
            res.end('stream not found');
            return;
          }

          const getter = hlsUrl.startsWith('https') ? httpsGet : httpGet;
          getter(hlsUrl, (upstream) => {
            let body = '';
            upstream.on('data', (chunk: Buffer) => { body += chunk.toString(); });
            upstream.on('end', () => {
              const rewritten = rewriteM3u8(body, hlsUrl, '/hls-proxy', slug);
              res.writeHead(200, {
                'Content-Type': 'application/vnd.apple.mpegurl',
                'Access-Control-Allow-Origin': '*',
                'Cache-Control': 'no-cache',
              });
              res.end(rewritten);
            });
          }).on('error', () => {
            res.writeHead(502);
            res.end('upstream error');
          });
        });
      },
    },
  ],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
    proxy: {
      '/api': {
        target: 'http://localhost:3001',
        changeOrigin: true,
      },
    },
  },
}));
