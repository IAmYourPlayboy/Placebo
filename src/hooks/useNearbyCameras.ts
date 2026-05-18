import { useState, useEffect } from 'react';
import type { Camera3D, CameraOrientation } from '../types/world3d';
import { DEFAULT_ORIENTATION, geoDistance } from '../types/world3d';

// ─── Stream URL helper ────────────────────────────────────────
// Since M3 the HLS proxy lives on the axum backend at
// /api/v1/hls-proxy/:slug. The slugs below MUST match real entries
// in the camera seed (migration 010); slugs that don't exist there
// will 404 from the proxy. yt-* slugs need yt-dlp to resolve
// upstream; demo-* slugs hit a static-file ServeDir.

function streamUrl(slug: string): string {
  const base = (import.meta.env.VITE_API_BASE_URL as string | undefined) ??
    'http://localhost:3001/api/v1';
  return `${base}/hls-proxy/${slug}`;
}

/**
 * Загружает камеры в радиусе от указанной точки.
 *
 * Сейчас: моковые маркеры, привязанные к реальным slug'ам из seed –
 * stream работает через axum-прокси без обращения к /cameras API.
 * Потом (M4): использовать useCamerasFromApi + GET /cameras/nearby.
 */
export function useNearbyCameras(
  centerLat: number,
  centerLng: number,
  radiusMeters: number = 1000
) {
  const [cameras, setCameras] = useState<Camera3D[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadMockCameras(centerLat, centerLng, radiusMeters).then((cams) => {
      setCameras(cams);
      setLoading(false);
    });
  }, [centerLat, centerLng, radiusMeters]);

  return { cameras, loading };
}

/**
 * Mock-камеры с уникальными направлениями и stream URL.
 * Slug'и подобраны так, чтобы соответствовать реальным записям из
 * migration 010 – тогда axum-прокси отдаёт настоящий manifest.
 */
async function loadMockCameras(
  centerLat: number,
  centerLng: number,
  radiusMeters: number
): Promise<Camera3D[]> {
  const MOCK_CAMERAS: Camera3D[] = [
    // 4 камеры с видеопотоками (разные azimuth, чтобы конусы не перекрывались).
    // Все 4 указывают на slug'и из seed: 2 youtube_live, 2 loop_mp4 demo.
    makeMock('yt-shibuya-crossing', 'Shibuya Crossing',     35.6595, 139.7005, 'city',    8,
      { azimuth: 180, elevation: -20, fovHorizontal: 80, fovVertical: 50 },
      streamUrl('yt-shibuya-crossing')),
    makeMock('yt-shibuya-station',  'Shibuya Station East', 35.6580, 139.7020, 'traffic', 12,
      { azimuth: 270, elevation: -10, fovHorizontal: 75, fovVertical: 45 },
      // No `yt-shibuya-station` in seed – fall back to the same proxy stream.
      streamUrl('yt-shibuya-crossing')),
    makeMock('demo-tokyo-alley',    'Hachiko Square',       35.6590, 139.7003, 'city',    5,
      { azimuth: 90, elevation: -25, fovHorizontal: 85, fovVertical: 52 },
      streamUrl('demo-tokyo-alley')),
    makeMock('demo-cafe-street',    'Center Gai Street',    35.6603, 139.6998, 'city',    4,
      { azimuth: 0, elevation: -15, fovHorizontal: 70, fovVertical: 42 },
      streamUrl('demo-cafe-street')),

    // Камеры без видеопотока (только маркеры).
    makeMock('shibuya-109',         'Shibuya 109 Building', 35.6601, 139.6985, 'city',    35,
      { azimuth: 135, elevation: -30 }),
    makeMock('meiji-dori',          'Meiji Dori Avenue',    35.6610, 139.6995, 'traffic', 6,
      { azimuth: 225, elevation: -12 }),
    makeMock('dogenzaka',           'Dogenzaka Hill',       35.6570, 139.6970, 'city',    7,
      { azimuth: 45, elevation: -18 }),
    makeMock('miyashita-park',      'Miyashita Park Cam',   35.6623, 139.7008, 'nature',  15,
      { azimuth: 315, elevation: -8 }),

    // Далёкие камеры (не попадут в радиус 1000м).
    makeMock('tokyo-tower',         'Tokyo Tower',          35.6586, 139.7454, 'city',    250),
    makeMock('akihabara',           'Akihabara Station',    35.6984, 139.7731, 'city',    10),
  ];

  return MOCK_CAMERAS.filter((cam) => {
    const dist = geoDistance(centerLat, centerLng, cam.lat, cam.lng);
    return dist <= radiusMeters;
  });
}

function makeMock(
  slug: string,
  name: string,
  lat: number,
  lng: number,
  category: string,
  height: number,
  orientation?: Partial<CameraOrientation>,
  hlsUrl?: string,
): Camera3D {
  return {
    id: `cam_${slug}`,
    name,
    slug,
    lat,
    lng,
    category,
    heightAboveGround: height,
    orientation: { ...DEFAULT_ORIENTATION, ...orientation },
    hlsUrl: hlsUrl ?? null,
    thumbnailUrl: null,
    isOnline: true,
    viewersNow: Math.floor(Math.random() * 50),
  };
}
