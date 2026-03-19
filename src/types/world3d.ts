// ─── Placebo 3D World — TypeScript Types ───────────────────────
// Типы для 3D-среды просмотра камер.

/** Конфигурация 3D Tiles сервера */
export interface TilesConfig {
  /** URL до index.json с реестром городов */
  indexUrl: string;
  /** Базовый URL для тайлов (без trailing slash) */
  baseUrl: string;
}

/** Город в реестре 3D Tiles */
export interface CityTileset {
  slug: string;
  tileset: string;  // относительный путь к tileset.json
}

/** Позиция камеры в 3D-мире (локальные координаты) */
export interface WorldPosition {
  x: number;  // восток (метры от центра)
  y: number;  // высота над землёй (метры)
  z: number;  // север (метры от центра)
}

/** Ориентация камеры в 3D-мире */
export interface CameraOrientation {
  /** Направление взгляда: 0=север, 90=восток, 180=юг, 270=запад */
  azimuth: number;
  /** Вертикальный угол: 0=горизонт, -90=вниз, 90=вверх */
  elevation: number;
  /** Горизонтальный угол обзора (градусы) */
  fovHorizontal: number;
  /** Вертикальный угол обзора (градусы) */
  fovVertical: number;
}

/** Данные камеры для размещения в 3D-мире */
export interface Camera3D {
  id: string;
  name: string;
  slug: string;
  lat: number;
  lng: number;
  category: string;
  /** Высота камеры над землёй (метры). Дефолт: 5 */
  heightAboveGround: number;
  /** Ориентация камеры. Дефолт: azimuth=0, elevation=-15, fov=90 */
  orientation: CameraOrientation;
  /** URL HLS потока (null если нет медиасервера) */
  hlsUrl: string | null;
  /** URL превью-картинки */
  thumbnailUrl: string | null;
  /** Онлайн ли камера */
  isOnline: boolean;
  /** Кол-во зрителей */
  viewersNow: number;
}

/** Состояние 3D-мира */
export type WorldState =
  | { status: 'loading'; progress: number }
  | { status: 'ready' }
  | { status: 'transitioning'; from: Camera3D; to: Camera3D; progress: number }
  | { status: 'error'; message: string };

// ─── Quality Presets ────────────────────────────────────────
export type QualityPreset = 'low' | 'medium' | 'high';
export type SkyMode = 'gradient' | 'gradient-stars-sun' | 'atmospheric';
export type GroundDetail = 'roads-basic' | 'roads-detailed' | 'roads-ssr';
export type LightingMode = 'basic' | 'ssao-bloom' | 'full';
export type PostMode = 'none' | 'tonemap-vignette' | 'full';

export interface QualityConfig {
  preset: QualityPreset;
  sky: { mode: SkyMode };
  ground: { detail: GroundDetail; gridEnabled: boolean };
  lighting: { mode: LightingMode; shadowMapSize: number };
  post: { mode: PostMode };
  maxVideoTextures: number;
  fog: { near: number; far: number; color: string };
}

export const QUALITY_PRESETS: Record<QualityPreset, QualityConfig> = {
  low: {
    preset: 'low',
    sky: { mode: 'gradient' },
    ground: { detail: 'roads-basic', gridEnabled: false },
    lighting: { mode: 'basic', shadowMapSize: 1024 },
    post: { mode: 'none' },
    maxVideoTextures: 1,
    fog: { near: 300, far: 1500, color: '#0a0a1a' },
  },
  medium: {
    preset: 'medium',
    sky: { mode: 'gradient-stars-sun' },
    ground: { detail: 'roads-detailed', gridEnabled: true },
    lighting: { mode: 'ssao-bloom', shadowMapSize: 2048 },
    post: { mode: 'tonemap-vignette' },
    maxVideoTextures: 2,
    fog: { near: 500, far: 2000, color: '#0a0a1a' },
  },
  high: {
    preset: 'high',
    sky: { mode: 'atmospheric' },
    ground: { detail: 'roads-ssr', gridEnabled: true },
    lighting: { mode: 'full', shadowMapSize: 4096 },
    post: { mode: 'full' },
    maxVideoTextures: 4,
    fog: { near: 800, far: 3000, color: '#0a0a1a' },
  },
};

export const DEFAULT_QUALITY = QUALITY_PRESETS.medium;

/** Дефолтная ориентация камеры (если нет данных) */
export const DEFAULT_ORIENTATION: CameraOrientation = {
  azimuth: 0,
  elevation: -15,
  fovHorizontal: 90,
  fovVertical: 58,  // 90° / (16/9) ≈ 58°
};

// ─── Утилиты координат ────────────────────────────────────────

const METERS_PER_DEGREE_LAT = 111320;

/**
 * Конвертирует lat/lng в локальные координаты (метры)
 * относительно центра (centerLat, centerLng).
 *
 * X = восток, Z = север, Y = высота (передаётся отдельно).
 */
export function geoToLocal(
  lat: number,
  lng: number,
  centerLat: number,
  centerLng: number
): { x: number; z: number } {
  const cosLat = Math.cos((centerLat * Math.PI) / 180);
  return {
    x: (lng - centerLng) * cosLat * METERS_PER_DEGREE_LAT,
    z: (lat - centerLat) * METERS_PER_DEGREE_LAT,
  };
}

/**
 * Конвертирует локальные координаты обратно в lat/lng.
 */
export function localToGeo(
  x: number,
  z: number,
  centerLat: number,
  centerLng: number
): { lat: number; lng: number } {
  const cosLat = Math.cos((centerLat * Math.PI) / 180);
  return {
    lat: centerLat + z / METERS_PER_DEGREE_LAT,
    lng: centerLng + x / (cosLat * METERS_PER_DEGREE_LAT),
  };
}

/**
 * Расстояние между двумя точками в метрах (Haversine).
 */
export function geoDistance(
  lat1: number, lng1: number,
  lat2: number, lng2: number
): number {
  const R = 6371e3;
  const φ1 = (lat1 * Math.PI) / 180;
  const φ2 = (lat2 * Math.PI) / 180;
  const Δφ = ((lat2 - lat1) * Math.PI) / 180;
  const Δλ = ((lng2 - lng1) * Math.PI) / 180;

  const a =
    Math.sin(Δφ / 2) ** 2 +
    Math.cos(φ1) * Math.cos(φ2) * Math.sin(Δλ / 2) ** 2;
  return R * 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1 - a));
}

// ─── City Tile Types ────────────────────────────────────────

export interface RoadSegment {
  points: { x: number; z: number }[];
  highway: string;
  name: string | null;
  width: number;
}

export const DEFAULT_ROAD_WIDTHS: Record<string, number> = {
  motorway: 15, trunk: 14, primary: 12, secondary: 9,
  tertiary: 7, residential: 6, unclassified: 5, service: 4,
  footway: 2, cycleway: 2, pedestrian: 3, path: 1.5, steps: 1.5,
};

export interface WaterFeature {
  points: { x: number; z: number }[];
  type: string;
  geomType: 'polygon' | 'line';
  name: string | null;
}

export interface ParkFeature {
  points: { x: number; z: number }[];
  type: string;
  name: string | null;
}

export interface BuildingFootprint {
  outline: { x: number; z: number }[];
  height: number;
}
