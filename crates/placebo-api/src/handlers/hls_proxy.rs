//! Public HLS proxy.
//!
//! `GET /api/v1/hls-proxy/:slug` – return a sanitized m3u8 manifest where
//! every segment URI points back at us. For loop_mp4 cameras the response
//! is a 307 redirect to a static asset served by the `/static` ServeDir.
//!
//! `GET /api/v1/hls-proxy/:slug/seg?u=<base64url>` – stream a single
//! segment from the upstream URL encoded into `u`. We never expose the
//! raw upstream URL to the client.

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Router,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use reqwest::Url;
use serde::Deserialize;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::services::hls_source::{self, ResolvedSource};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/hls-proxy/:slug", get(manifest))
        .route("/hls-proxy/:slug/seg", get(segment))
}

#[derive(Deserialize)]
struct SegQuery {
    u: String,
}

async fn manifest(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Response, AppError> {
    let resolved = hls_source::resolve(&state.db, &state.redis, &slug)
        .await
        .map_err(AppError::Internal)?;

    match resolved {
        ResolvedSource::NotFound => Err(AppError::NotFound(format!("camera '{slug}'"))),
        ResolvedSource::Unsupported => Err(AppError::NotFound(format!(
            "stream type for '{slug}' is not proxied in the alpha"
        ))),
        ResolvedSource::StaticLoop(rel_path) => Ok(Redirect::temporary(&rel_path).into_response()),
        ResolvedSource::Hls(upstream_url) => fetch_and_rewrite(&upstream_url, &slug).await,
    }
}

async fn fetch_and_rewrite(upstream_url: &str, slug: &str) -> Result<Response, AppError> {
    let body = reqwest::Client::new()
        .get(upstream_url)
        .send()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("upstream fetch: {e}")))?
        .text()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("upstream body: {e}")))?;

    let base = Url::parse(upstream_url)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("bad upstream url: {e}")))?;
    let rewritten = rewrite_m3u8(&body, &base, slug);

    let mut headers = HeaderMap::new();
    headers.insert(
        "content-type",
        HeaderValue::from_static("application/vnd.apple.mpegurl"),
    );
    headers.insert("cache-control", HeaderValue::from_static("no-cache"));
    headers.insert("access-control-allow-origin", HeaderValue::from_static("*"));
    Ok((StatusCode::OK, headers, rewritten).into_response())
}

/// Resolve every URI line (relative or absolute) to an absolute URL,
/// then rewrite the line to point at our own `/seg` endpoint, encoding
/// the absolute URL in base64url so it survives query-string transit.
///
/// `URI="..."` attributes inside `#EXT-X-KEY` / `#EXT-X-MAP` tags are
/// rewritten in-place; everything else (comments, blank lines, tag
/// metadata) is passed through unchanged.
fn rewrite_m3u8(body: &str, base: &Url, slug: &str) -> String {
    let mut out = String::with_capacity(body.len() + 256);
    for line in body.lines() {
        if line.is_empty() {
            out.push('\n');
            continue;
        }
        if line.starts_with('#') {
            // Tag line – rewrite any URI="..." attribute it carries.
            out.push_str(&rewrite_tag_uris(line, base, slug));
            out.push('\n');
            continue;
        }
        // Plain segment / sub-playlist URI.
        let absolute = absolutize(line, base);
        out.push_str(&proxy_url(slug, &absolute));
        out.push('\n');
    }
    out
}

fn absolutize(href: &str, base: &Url) -> String {
    if href.starts_with("http://") || href.starts_with("https://") {
        href.to_string()
    } else {
        base.join(href)
            .map(|u| u.to_string())
            .unwrap_or_else(|_| href.to_string())
    }
}

fn proxy_url(slug: &str, absolute_url: &str) -> String {
    let encoded = URL_SAFE_NO_PAD.encode(absolute_url.as_bytes());
    format!("/api/v1/hls-proxy/{slug}/seg?u={encoded}")
}

/// Find every `URI="<value>"` attribute on an HLS tag line and replace
/// each `<value>` with a proxied URL. HLS tags that may carry a URI:
/// `#EXT-X-KEY`, `#EXT-X-MAP`, `#EXT-X-MEDIA`, `#EXT-X-I-FRAME-STREAM-INF`,
/// `#EXT-X-SESSION-DATA`, `#EXT-X-PART`, `#EXT-X-PRELOAD-HINT`,
/// `#EXT-X-RENDITION-REPORT`. Rewriting URI on all of them is the safe
/// default: no other tag uses a `URI=` attribute, so non-matches simply
/// produce a passthrough copy.
fn rewrite_tag_uris(line: &str, base: &Url, slug: &str) -> String {
    let needle = "URI=\"";
    let mut result = String::with_capacity(line.len());
    let mut cursor = 0usize;
    while let Some(rel_start) = line[cursor..].find(needle) {
        let attr_start = cursor + rel_start;
        let value_start = attr_start + needle.len();
        let Some(rel_end) = line[value_start..].find('"') else {
            // Unterminated quote – give up and emit the rest verbatim.
            result.push_str(&line[cursor..]);
            cursor = line.len();
            break;
        };
        let value_end = value_start + rel_end;
        let original = &line[value_start..value_end];
        let absolute = absolutize(original, base);
        let rewritten = proxy_url(slug, &absolute);

        result.push_str(&line[cursor..value_start]);
        result.push_str(&rewritten);
        result.push('"');
        cursor = value_end + 1;
    }
    result.push_str(&line[cursor..]);
    result
}

async fn segment(
    Path(_slug): Path<String>,
    Query(q): Query<SegQuery>,
) -> Result<Response, AppError> {
    let decoded = URL_SAFE_NO_PAD
        .decode(q.u.as_bytes())
        .map_err(|_| AppError::Validation("invalid base64url in u".into()))?;
    let url = String::from_utf8(decoded)
        .map_err(|_| AppError::Validation("u is not valid utf-8".into()))?;

    let upstream = reqwest::Client::new()
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("upstream segment: {e}")))?;

    let status = upstream.status();
    let mut headers = HeaderMap::new();
    if let Some(ct) = upstream.headers().get("content-type") {
        headers.insert("content-type", ct.clone());
    }
    headers.insert("access-control-allow-origin", HeaderValue::from_static("*"));
    headers.insert("cache-control", HeaderValue::from_static("no-cache"));

    let body = Body::from_stream(upstream.bytes_stream());
    Ok((status, headers, body).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decode_proxy_url(line: &str) -> String {
        let encoded = line.split("u=").nth(1).unwrap();
        let bytes = URL_SAFE_NO_PAD.decode(encoded.as_bytes()).unwrap();
        String::from_utf8(bytes).unwrap()
    }

    #[test]
    fn rewrite_keeps_comments_and_blanks() {
        let base = Url::parse("https://cdn.example.com/path/index.m3u8").unwrap();
        let body = "#EXTM3U\n\
#EXT-X-VERSION:3\n\
\n\
#EXTINF:4.000,\n\
seg-001.ts\n\
#EXTINF:4.000,\n\
https://other.example.com/seg-002.ts\n";
        let out = rewrite_m3u8(body, &base, "yt-test");
        // Tag lines without URI attrs pass through.
        assert!(out.contains("#EXTM3U"));
        assert!(out.contains("#EXT-X-VERSION:3"));
        assert!(out.contains("#EXTINF:4.000,"));
        // Both URI lines became proxy paths.
        let proxy_lines: Vec<&str> =
            out.lines().filter(|l| l.starts_with("/api/v1/hls-proxy/")).collect();
        assert_eq!(proxy_lines.len(), 2);
        // No raw upstream segment URLs leak.
        assert!(!out.contains("seg-001.ts\n"));
        assert!(!out.contains("https://other.example.com/seg-002.ts"));
    }

    #[test]
    fn rewrite_resolves_relative_against_base() {
        let base = Url::parse("https://cdn.example.com/playlists/master.m3u8").unwrap();
        let body = "#EXTM3U\n#EXTINF:4.000,\nseg-1.ts\n";
        let out = rewrite_m3u8(body, &base, "slug");
        let line = out.lines().find(|l| l.starts_with("/api/v1/hls-proxy/")).unwrap();
        assert_eq!(
            decode_proxy_url(line),
            "https://cdn.example.com/playlists/seg-1.ts"
        );
    }

    #[test]
    fn rewrite_preserves_absolute_segment_url() {
        let base = Url::parse("https://cdn.example.com/p/index.m3u8").unwrap();
        let body = "#EXTM3U\n#EXTINF:4.000,\nhttps://other.example.com/seg-1.ts\n";
        let out = rewrite_m3u8(body, &base, "slug");
        let line = out.lines().find(|l| l.starts_with("/api/v1/hls-proxy/")).unwrap();
        assert_eq!(
            decode_proxy_url(line),
            "https://other.example.com/seg-1.ts"
        );
    }

    #[test]
    fn rewrite_uri_attribute_in_ext_x_key() {
        let base = Url::parse("https://cdn.example.com/p/index.m3u8").unwrap();
        let body = "#EXTM3U\n\
#EXT-X-KEY:METHOD=AES-128,URI=\"key.bin\",IV=0x0\n\
#EXTINF:4.000,\n\
seg-1.ts\n";
        let out = rewrite_m3u8(body, &base, "slug");
        let key_line = out
            .lines()
            .find(|l| l.starts_with("#EXT-X-KEY"))
            .expect("key line preserved");
        // Plain raw key URL is gone, replaced with a proxy path.
        assert!(!key_line.contains("key.bin\""));
        assert!(key_line.contains("URI=\"/api/v1/hls-proxy/slug/seg?u="));
        // Tag metadata around URI must survive intact.
        assert!(key_line.contains("METHOD=AES-128"));
        assert!(key_line.contains("IV=0x0"));
    }

    #[test]
    fn rewrite_uri_attribute_in_ext_x_map() {
        let base = Url::parse("https://cdn.example.com/p/index.m3u8").unwrap();
        let body = "#EXTM3U\n#EXT-X-MAP:URI=\"init.mp4\"\n";
        let out = rewrite_m3u8(body, &base, "slug");
        let map_line = out
            .lines()
            .find(|l| l.starts_with("#EXT-X-MAP"))
            .expect("map line preserved");
        assert!(map_line.contains("URI=\"/api/v1/hls-proxy/slug/seg?u="));
    }
}
