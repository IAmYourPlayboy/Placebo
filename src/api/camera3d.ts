import type { CameraResponse } from "../types/api/CameraResponse";
import type { Camera3D } from "../types/world3d";
import { DEFAULT_ORIENTATION } from "../types/world3d";

/**
 * Map the API camera DTO to the local Camera3D type used by WorldScene.
 *
 * Fallbacks (height = 5m, orientation 0° / -15° / 90° / 58°) come from
 * DEFAULT_ORIENTATION so visuals do not regress when seed rows omit
 * those fields.
 *
 * The HLS URL passes through to <video>.src as-is. proxyManifestUrl from
 * the API is server-relative ("/api/v1/hls-proxy/<slug>"); in dev the Vite
 * proxy forwards /api → http://localhost:3001 transparently. If
 * VITE_API_BASE_URL is set (Tauri prod or remote dev), we replace the
 * leading /api/v1 with the configured base so we don't end up with double
 * /api/v1 segments.
 */
export function cameraResponseToCamera3D(c: CameraResponse): Camera3D {
  return {
    id: c.id,
    name: c.name,
    slug: c.slug,
    lat: c.lat,
    lng: c.lng,
    category: c.category,
    heightAboveGround: c.heightAboveGround ?? 5,
    orientation: {
      azimuth: c.cameraAzimuth ?? DEFAULT_ORIENTATION.azimuth,
      elevation: c.cameraElevation ?? DEFAULT_ORIENTATION.elevation,
      fovHorizontal: c.fovHorizontal ?? DEFAULT_ORIENTATION.fovHorizontal,
      fovVertical: c.fovVertical ?? DEFAULT_ORIENTATION.fovVertical,
    },
    hlsUrl: resolveHlsUrl(c.proxyManifestUrl),
    thumbnailUrl: c.thumbnailUrl,
    isOnline: true,        // M5: real signal from /cameras/:id/health.
    viewersNow: 0,         // M5: real viewer count via Redis hook.
  };
}

function resolveHlsUrl(proxyManifestUrl: string | null): string | null {
  if (!proxyManifestUrl) return null;
  const raw = (import.meta.env.VITE_API_BASE_URL as string | undefined)?.replace(/\/$/, "");
  if (!raw) return proxyManifestUrl;
  // VITE_API_BASE_URL may be either the host ("http://localhost:3001") or
  // include the API prefix ("http://localhost:3001/api/v1") - .env.example
  // in M3 ships the latter. Strip a trailing /api/v1 so we don't end up
  // duplicating it onto the already-prefixed proxyManifestUrl.
  const host = raw.replace(/\/api\/v1$/, "");
  return `${host}${proxyManifestUrl}`;
}
