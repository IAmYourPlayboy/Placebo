import { useState, useEffect } from 'react';

export interface RoadSegment {
  points: { x: number; z: number }[];
  highway: string;
  name: string | null;
  width: number;
}

export interface RoadData {
  roads: RoadSegment[];
  loading: boolean;
}

export const DEFAULT_WIDTHS: Record<string, number> = {
  motorway: 15,
  trunk: 14,
  primary: 12,
  secondary: 9,
  tertiary: 7,
  residential: 6,
  unclassified: 5,
  footway: 2.5,
  path: 2,
  cycleway: 2.5,
  service: 4,
  pedestrian: 4,
};

/**
 * Fetches road network data from our API.
 * NOTE: API endpoint not yet implemented. Returns empty array for now.
 */
export function useRoadNetwork(
  centerLat: number,
  centerLng: number,
  radiusMeters: number = 1000
): RoadData {
  const [roads, setRoads] = useState<RoadSegment[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    // TODO: Uncomment when /api/world/roads endpoint is implemented
    setRoads([]);
    setLoading(false);
  }, [centerLat, centerLng, radiusMeters]);

  return { roads, loading };
}
