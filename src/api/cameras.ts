/**
 * Typed wrappers around the placebo-api `/cameras` endpoints.
 *
 * `CameraResponse` is generated from the Rust DTO by ts-rs (see
 * `npm run gen-types`) and lives under `src/types/api/`. Don't redefine it.
 *
 * The HLS playback URL for any returned camera is `proxyManifestUrl`:
 * `null` for `streamSourceType === "rtsp"` (not proxied in alpha) and a
 * relative path like `/api/v1/hls-proxy/<slug>` for everything else.
 * That path is relative to the API base; the helper `manifestUrlFor()`
 * absolutizes it for hls.js / <video src>.
 */

import type { CameraResponse } from "../types/api/CameraResponse";

import { apiRequest } from "./client";

export interface ListCamerasParams {
  page?: number;
  perPage?: number;
  category?: string;
  type?: string;
}

export interface PaginationMeta {
  page: number;
  perPage: number;
  total: number;
  totalPages: number;
}

export interface CameraListResponse {
  data: CameraResponse[];
  meta: PaginationMeta;
}

function buildQuery(params: Record<string, unknown>): string {
  const search = new URLSearchParams();
  for (const [key, value] of Object.entries(params)) {
    if (value === undefined || value === null) continue;
    search.set(key, String(value));
  }
  const qs = search.toString();
  return qs ? `?${qs}` : "";
}

export function listCameras(
  params: ListCamerasParams = {},
): Promise<CameraListResponse> {
  const query = buildQuery({
    page: params.page,
    per_page: params.perPage,
    category: params.category,
    type: params.type,
  });
  return apiRequest<CameraListResponse>(`/cameras${query}`, { auth: false });
}

export function getCamera(id: string): Promise<{ data: CameraResponse }> {
  return apiRequest<{ data: CameraResponse }>(`/cameras/${id}`, { auth: false });
}

export function getCamerasNearby(
  lat: number,
  lng: number,
  radiusM: number,
  limit = 50,
): Promise<{ data: CameraResponse[] }> {
  const query = buildQuery({ lat, lng, radius_m: radiusM, limit });
  return apiRequest<{ data: CameraResponse[] }>(`/cameras/nearby${query}`, {
    auth: false,
  });
}

export function searchCameras(
  q: string,
  limit = 50,
): Promise<{ data: CameraResponse[] }> {
  const query = buildQuery({ q, limit });
  return apiRequest<{ data: CameraResponse[] }>(`/cameras/search${query}`, {
    auth: false,
  });
}

/**
 * Build an absolute URL for `<video>` / hls.js from a camera's
 * `proxyManifestUrl`. Returns `null` if the camera has no proxied stream
 * (e.g. RTSP cameras in alpha).
 *
 * The DTO carries the path as `/api/v1/hls-proxy/<slug>`. We strip the
 * `/api/v1` prefix and prepend `VITE_API_BASE_URL` (which already ends
 * in `/api/v1`) so the result works in both vite-dev (where the proxy
 * does the forwarding) and Tauri builds (where it doesn't).
 */
export function manifestUrlFor(camera: Pick<CameraResponse, "proxyManifestUrl">): string | null {
  const path = camera.proxyManifestUrl;
  if (!path) return null;
  const base = (import.meta.env.VITE_API_BASE_URL as string | undefined) ??
    "http://localhost:3001/api/v1";
  // path is `/api/v1/<rest>` – strip the duplicate prefix before joining.
  const trimmed = path.startsWith("/api/v1") ? path.slice("/api/v1".length) : path;
  return `${base}${trimmed}`;
}
