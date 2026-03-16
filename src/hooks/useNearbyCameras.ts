import { useState, useEffect } from 'react';
import type { Camera3D, CameraOrientation } from '../types/world3d';
import { DEFAULT_ORIENTATION, geoDistance } from '../types/world3d';

// ─── Stream URL helper ────────────────────────────────────────
// Если запущен go2rtc (VITE_GO2RTC_URL), используем HLS endpoint.
// Иначе — fallback на локальные test MP4.
const GO2RTC_URL = import.meta.env.VITE_GO2RTC_URL as string | undefined;
console.log('[Placebo] GO2RTC_URL =', GO2RTC_URL);

function streamUrl(slug: string): string {
  const url = GO2RTC_URL
    ? `${GO2RTC_URL}/api/stream.m3u8?src=${slug}`
    : `/test-streams/${slug}.mp4`;
  console.log('[Placebo] streamUrl', slug, '→', url);
  return url;
}

/**
 * Загружает камеры в радиусе от указанной точки.
 *
 * Сейчас: берёт данные из seed (mock) + test video URLs.
 * Потом: будет вызывать API GET /cameras/nearby.
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
 */
async function loadMockCameras(
  centerLat: number,
  centerLng: number,
  radiusMeters: number
): Promise<Camera3D[]> {
  const MOCK_CAMERAS: Camera3D[] = [
    // 4 камеры с видеопотоками (разные azimuth чтобы конусы не перекрывались)
    makeMock('shibuya-crossing',   'Shibuya Crossing',       35.6595, 139.7005, 'city',    8,
      { azimuth: 180, elevation: -20, fovHorizontal: 80, fovVertical: 50 },
      streamUrl('shibuya-crossing')),
    makeMock('shibuya-station',    'Shibuya Station East',   35.6580, 139.7020, 'traffic', 12,
      { azimuth: 270, elevation: -10, fovHorizontal: 75, fovVertical: 45 },
      streamUrl('shibuya-station')),
    makeMock('hachiko-square',     'Hachiko Square',         35.6590, 139.7003, 'city',    5,
      { azimuth: 90, elevation: -25, fovHorizontal: 85, fovVertical: 52 },
      streamUrl('hachiko-square')),
    makeMock('center-gai',        'Center Gai Street',      35.6603, 139.6998, 'city',    4,
      { azimuth: 0, elevation: -15, fovHorizontal: 70, fovVertical: 42 },
      streamUrl('center-gai')),

    // Камеры без видеопотока (только маркеры)
    makeMock('shibuya-109',        'Shibuya 109 Building',   35.6601, 139.6985, 'city',    35,
      { azimuth: 135, elevation: -30 }),
    makeMock('meiji-dori',         'Meiji Dori Avenue',      35.6610, 139.6995, 'traffic', 6,
      { azimuth: 225, elevation: -12 }),
    makeMock('dogenzaka',          'Dogenzaka Hill',         35.6570, 139.6970, 'city',    7,
      { azimuth: 45, elevation: -18 }),
    makeMock('miyashita-park',     'Miyashita Park Cam',     35.6623, 139.7008, 'nature',  15,
      { azimuth: 315, elevation: -8 }),

    // Далёкие камеры (не попадут в радиус 1000м)
    makeMock('tokyo-tower',        'Tokyo Tower',            35.6586, 139.7454, 'city',    250),
    makeMock('akihabara',          'Akihabara Station',      35.6984, 139.7731, 'city',    10),
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
