import { useState, useEffect, useRef } from 'react';
import type { RoadSegment, WaterFeature, ParkFeature, BuildingFootprint } from '../types/world3d';

// API base – empty string = same origin (Vite proxy forwards /api to axum)
const API_BASE = '';

interface CityTilesResult {
  roads: RoadSegment[];
  water: WaterFeature[];
  parks: ParkFeature[];
  buildings: BuildingFootprint[];
  loading: boolean;
  error: string | null;
}

interface TileCoord { z: number; x: number; y: number }

function latLngToTile(lat: number, lng: number, zoom: number): { x: number; y: number } {
  const n = 2 ** zoom;
  const x = Math.floor(((lng + 180) / 360) * n);
  const latRad = (lat * Math.PI) / 180;
  const y = Math.floor(
    ((1 - Math.log(Math.tan(latRad) + 1 / Math.cos(latRad)) / Math.PI) / 2) * n
  );
  return { x, y };
}

function getVisibleTiles(lat: number, lng: number, zoom: number): TileCoord[] {
  const center = latLngToTile(lat, lng, zoom);
  const tiles: TileCoord[] = [];
  for (let dx = -1; dx <= 1; dx++) {
    for (let dy = -1; dy <= 1; dy++) {
      tiles.push({ z: zoom, x: center.x + dx, y: center.y + dy });
    }
  }
  return tiles;
}

function tileCacheKey(tiles: TileCoord[]): string {
  return tiles.map(t => `${t.z}/${t.x}/${t.y}`).sort().join(',');
}

export function useCityTiles(
  centerLat: number,
  centerLng: number,
  zoom: number = 16,
): CityTilesResult {
  const [roads, setRoads] = useState<RoadSegment[]>([]);
  const [water, setWater] = useState<WaterFeature[]>([]);
  const [parks, setParks] = useState<ParkFeature[]>([]);
  const [buildings, setBuildings] = useState<BuildingFootprint[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const prevTileKey = useRef<string>('');

  useEffect(() => {
    const tiles = getVisibleTiles(centerLat, centerLng, zoom);
    const key = tileCacheKey(tiles);

    // Skip if same tiles
    if (key === prevTileKey.current) return;
    prevTileKey.current = key;

    const controller = new AbortController();
    setLoading(true);
    setError(null);

    const fetchTiles = async () => {
      try {
        const results = await Promise.allSettled(
          tiles.map(t =>
            fetch(
              `${API_BASE}/api/v1/world/tile?z=${t.z}&x=${t.x}&y=${t.y}&center_lat=${centerLat}&center_lng=${centerLng}`,
              { signal: controller.signal }
            ).then(res => {
              if (!res.ok) throw new Error(`Tile ${t.z}/${t.x}/${t.y}: ${res.status}`);
              return res.json();
            })
          )
        );

        const allRoads: RoadSegment[] = [];
        const allWater: WaterFeature[] = [];
        const allParks: ParkFeature[] = [];
        const allBuildings: BuildingFootprint[] = [];

        for (const result of results) {
          if (result.status === 'fulfilled') {
            const data = result.value;
            // Map API response → frontend types
            // API: { coords: [x,z][], highway, name, width_meters, ... }
            // Frontend: { points: {x,z}[], highway, name, width, ... }
            if (data.roads) {
              for (const r of data.roads) {
                allRoads.push({
                  points: (r.coords ?? []).map((c: number[]) => ({ x: c[0], z: c[1] })),
                  highway: r.highway,
                  name: r.name,
                  width: r.width_meters ?? 5,
                });
              }
            }
            if (data.water) {
              for (const w of data.water) {
                allWater.push({
                  points: (w.coords ?? []).map((c: number[]) => ({ x: c[0], z: c[1] })),
                  type: w.type,
                  geomType: w.geomType ?? w.geom_type ?? 'polygon',
                  name: w.name,
                });
              }
            }
            if (data.parks) {
              for (const p of data.parks) {
                allParks.push({
                  points: (p.coords ?? []).map((c: number[]) => ({ x: c[0], z: c[1] })),
                  type: p.park_type ?? p.type ?? 'park',
                  name: p.name,
                });
              }
            }
            if (data.buildings) {
              for (const b of data.buildings) {
                allBuildings.push({
                  outline: (b.coords ?? []).map((c: number[]) => ({ x: c[0], z: c[1] })),
                  height: b.height_meters ?? 10,
                });
              }
            }
          }
        }

        setRoads(allRoads);
        setWater(allWater);
        setParks(allParks);
        setBuildings(allBuildings);
      } catch (err: unknown) {
        if (err instanceof Error && err.name !== 'AbortError') {
          setError(err.message);
        }
      } finally {
        setLoading(false);
      }
    };

    fetchTiles();
    return () => {
      controller.abort();
      // Reset key so StrictMode re-run will re-fetch
      prevTileKey.current = '';
    };
  }, [centerLat, centerLng, zoom]);

  return { roads, water, parks, buildings, loading, error };
}
