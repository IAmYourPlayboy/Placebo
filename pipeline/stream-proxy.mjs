#!/usr/bin/env node
/**
 * Placebo Stream Proxy
 *
 * Проксирует YouTube Live HLS стримы с CORS headers для фронтенда.
 * yt-dlp извлекает прямой HLS URL, прокси кэширует его на 30 минут.
 *
 * Endpoints:
 *   GET /api/stream.m3u8?src=shibuya-crossing  → proxied YouTube HLS playlist
 *   GET /api/streams                            → list of configured streams
 *
 * Использование:
 *   node stream-proxy.mjs
 *   # или с custom port:
 *   PORT=1984 node stream-proxy.mjs
 */

import http from 'node:http';
import https from 'node:https';
import { execSync } from 'node:child_process';
import { URL } from 'node:url';

const PORT = parseInt(process.env.PORT || '1984', 10);

// ─── Stream config ───────────────────────────────────────────
const STREAMS = {
  'shibuya-crossing': 'https://www.youtube.com/watch?v=dfVK7ld38Ys',
  'shibuya-station':  'https://www.youtube.com/watch?v=8H3nRCFVR6Y',
  'hachiko-square':   'https://www.youtube.com/watch?v=ehJbjfH1dIo',
  'center-gai':       'https://www.youtube.com/watch?v=6dp-bvQ7RWo',
};

// ─── URL cache (YouTube HLS URLs expire after ~6 hours) ──────
const urlCache = new Map();   // slug → { url, expires }
const CACHE_TTL_MS = 30 * 60 * 1000; // 30 minutes

function getHlsUrl(slug) {
  const cached = urlCache.get(slug);
  if (cached && cached.expires > Date.now()) {
    return cached.url;
  }

  const youtubeUrl = STREAMS[slug];
  if (!youtubeUrl) return null;

  try {
    console.log(`[yt-dlp] extracting HLS URL for ${slug}...`);
    const url = execSync(
      `yt-dlp -f 93 --no-warnings -g "${youtubeUrl}"`,
      { timeout: 30000, encoding: 'utf-8' }
    ).trim();
    console.log(`[yt-dlp] ${slug} → ${url.substring(0, 80)}...`);
    urlCache.set(slug, { url, expires: Date.now() + CACHE_TTL_MS });
    return url;
  } catch (err) {
    console.error(`[yt-dlp] failed for ${slug}:`, err.message);
    try {
      const url = execSync(
        `yt-dlp -f best --no-warnings -g "${youtubeUrl}"`,
        { timeout: 30000, encoding: 'utf-8' }
      ).trim();
      urlCache.set(slug, { url, expires: Date.now() + CACHE_TTL_MS });
      return url;
    } catch {
      return null;
    }
  }
}

// ─── Keep-alive agents ───────────────────────────────────────
const httpsAgent = new https.Agent({ keepAlive: true, maxSockets: 10 });

// ─── Playlist cache (short TTL to reduce YouTube CDN hits) ───
const playlistCache = new Map(); // key → { body, headers, expires }
const PLAYLIST_CACHE_MS = 2000;  // 2 seconds

// ─── Segment cache (segments are immutable for their duration)
const segmentCache = new Map();  // url → { body, contentType, expires }
const SEGMENT_CACHE_MS = 10000;  // 10 seconds
const MAX_SEGMENT_CACHE = 100;   // limit memory usage

// ─── Buffered proxy fetch ────────────────────────────────────
function proxyFetch(targetUrl) {
  return new Promise((resolve, reject) => {
    const parsedUrl = new URL(targetUrl);
    const isHttps = parsedUrl.protocol === 'https:';
    const client = isHttps ? https : http;

    const req = client.get(targetUrl, {
      agent: isHttps ? httpsAgent : undefined,
      headers: {
        'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)',
        'Referer': 'https://www.youtube.com/',
        'Origin': 'https://www.youtube.com',
      },
    }, (res) => {
      const chunks = [];
      res.on('data', (chunk) => chunks.push(chunk));
      res.on('end', () => {
        resolve({
          status: res.statusCode,
          headers: res.headers,
          body: Buffer.concat(chunks),
        });
      });
    });
    req.on('error', reject);
    req.setTimeout(30000, () => { req.destroy(); reject(new Error('timeout')); });
  });
}

// ─── Streaming proxy (pipe YouTube → client directly) ────────
function proxyStream(targetUrl, clientRes) {
  return new Promise((resolve, reject) => {
    const parsedUrl = new URL(targetUrl);
    const isHttps = parsedUrl.protocol === 'https:';
    const client = isHttps ? https : http;

    const req = client.get(targetUrl, {
      agent: isHttps ? httpsAgent : undefined,
      headers: {
        'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)',
        'Referer': 'https://www.youtube.com/',
        'Origin': 'https://www.youtube.com',
      },
    }, (upstream) => {
      const contentType = upstream.headers['content-type'] || 'video/mp2t';
      clientRes.writeHead(upstream.statusCode, {
        'Content-Type': contentType,
        'Cache-Control': 'no-cache',
      });
      upstream.pipe(clientRes);
      upstream.on('end', resolve);
      upstream.on('error', reject);
    });
    req.on('error', reject);
    req.setTimeout(30000, () => { req.destroy(); reject(new Error('timeout')); });
  });
}

// ─── Rewrite m3u8 segment URLs to go through our proxy ───────
function rewriteM3u8(body, baseUrl) {
  const lines = body.toString('utf-8').split('\n');
  return lines.map((line) => {
    const trimmed = line.trim();
    if (trimmed.startsWith('#') || trimmed === '') return line;
    if (trimmed.startsWith('http')) {
      return `/api/proxy?url=${encodeURIComponent(trimmed)}`;
    }
    if (baseUrl) {
      const absUrl = new URL(trimmed, baseUrl).href;
      return `/api/proxy?url=${encodeURIComponent(absUrl)}`;
    }
    return line;
  }).join('\n');
}

// ─── CORS headers ────────────────────────────────────────────
function setCors(res) {
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Methods', 'GET, OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', '*');
}

// ─── Deduplicate in-flight fetches ───────────────────────────
const inFlight = new Map(); // url → Promise

function deduplicatedFetch(url) {
  if (inFlight.has(url)) {
    return inFlight.get(url);
  }
  const promise = proxyFetch(url).finally(() => inFlight.delete(url));
  inFlight.set(url, promise);
  return promise;
}

// ─── HTTP Server ─────────────────────────────────────────────
const server = http.createServer(async (req, res) => {
  setCors(res);

  if (req.method === 'OPTIONS') {
    res.writeHead(204);
    res.end();
    return;
  }

  const url = new URL(req.url, `http://localhost:${PORT}`);

  // GET /api/streams
  if (url.pathname === '/api/streams') {
    res.writeHead(200, { 'Content-Type': 'application/json' });
    const result = {};
    for (const [slug, ytUrl] of Object.entries(STREAMS)) {
      result[slug] = {
        producers: [{ url: ytUrl }],
        consumers: null,
        cached: urlCache.has(slug),
      };
    }
    res.end(JSON.stringify(result));
    return;
  }

  // GET /api/stream.m3u8?src=slug
  if (url.pathname === '/api/stream.m3u8') {
    const slug = url.searchParams.get('src');
    if (!slug || !STREAMS[slug]) {
      res.writeHead(404, { 'Content-Type': 'text/plain' });
      res.end(`Unknown stream: ${slug}`);
      return;
    }

    // Check playlist cache first
    const cacheKey = `playlist:${slug}`;
    const cached = playlistCache.get(cacheKey);
    if (cached && cached.expires > Date.now()) {
      res.writeHead(200, { 'Content-Type': 'application/vnd.apple.mpegurl' });
      res.end(cached.body);
      return;
    }

    const hlsUrl = getHlsUrl(slug);
    if (!hlsUrl) {
      res.writeHead(502, { 'Content-Type': 'text/plain' });
      res.end('Failed to extract YouTube HLS URL');
      return;
    }

    try {
      const upstream = await deduplicatedFetch(hlsUrl);
      const rewritten = rewriteM3u8(upstream.body, hlsUrl);
      // Cache the rewritten playlist
      playlistCache.set(cacheKey, {
        body: rewritten,
        expires: Date.now() + PLAYLIST_CACHE_MS,
      });
      res.writeHead(200, { 'Content-Type': 'application/vnd.apple.mpegurl' });
      res.end(rewritten);
    } catch (err) {
      console.error(`[proxy] m3u8 fetch failed for ${slug}:`, err.message);
      urlCache.delete(slug);
      res.writeHead(502, { 'Content-Type': 'text/plain' });
      res.end('Failed to fetch HLS playlist');
    }
    return;
  }

  // GET /api/proxy?url=... → proxy TS segments and sub-playlists
  if (url.pathname === '/api/proxy') {
    const targetUrl = url.searchParams.get('url');
    if (!targetUrl) {
      res.writeHead(400, { 'Content-Type': 'text/plain' });
      res.end('Missing url parameter');
      return;
    }

    // Check segment cache
    const segCached = segmentCache.get(targetUrl);
    if (segCached && segCached.expires > Date.now()) {
      res.writeHead(200, {
        'Content-Type': segCached.contentType,
        'Content-Length': segCached.body.length,
        'Cache-Control': 'no-cache',
      });
      res.end(segCached.body);
      return;
    }

    try {
      const upstream = await deduplicatedFetch(targetUrl);
      const contentType = upstream.headers['content-type'] || '';

      // m3u8 sub-playlist – rewrite URLs
      if (contentType.includes('mpegurl') || contentType.includes('m3u8')) {
        const rewritten = rewriteM3u8(upstream.body, targetUrl);
        res.writeHead(upstream.status, {
          'Content-Type': 'application/vnd.apple.mpegurl',
        });
        res.end(rewritten);
      } else {
        // TS segment – cache and serve
        if (segmentCache.size >= MAX_SEGMENT_CACHE) {
          // Evict oldest entries
          const keys = [...segmentCache.keys()];
          for (let i = 0; i < 20; i++) segmentCache.delete(keys[i]);
        }
        segmentCache.set(targetUrl, {
          body: upstream.body,
          contentType: contentType || 'video/mp2t',
          expires: Date.now() + SEGMENT_CACHE_MS,
        });

        res.writeHead(upstream.status, {
          'Content-Type': contentType || 'video/mp2t',
          'Content-Length': upstream.body.length,
          'Cache-Control': 'no-cache',
        });
        res.end(upstream.body);
      }
    } catch (err) {
      console.error(`[proxy] fetch failed:`, err.message);
      res.writeHead(502, { 'Content-Type': 'text/plain' });
      res.end('Proxy fetch failed');
    }
    return;
  }

  // 404
  res.writeHead(404, { 'Content-Type': 'text/plain' });
  res.end('Not found');
});

server.listen(PORT, () => {
  console.log(`[Placebo Stream Proxy] listening on http://localhost:${PORT}`);
  console.log(`[Placebo Stream Proxy] streams: ${Object.keys(STREAMS).join(', ')}`);

  // Pre-warm all streams on startup
  console.log('[Placebo Stream Proxy] pre-warming yt-dlp cache...');
  for (const slug of Object.keys(STREAMS)) {
    getHlsUrl(slug);
  }
  console.log('[Placebo Stream Proxy] all streams ready');
});
