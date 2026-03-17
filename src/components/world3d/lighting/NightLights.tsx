import { useMemo } from 'react';
import { useTimeOfDay } from '../../../hooks/useTimeOfDay';
import { useQuality } from '../../../hooks/useQuality';
import type { RoadSegment } from '../../../types/world3d';

interface NightLightsProps {
  timezone?: string;
  roads: RoadSegment[];
}

function sampleRoadPositions(roads: RoadSegment[], spacing: number, maxLights: number): [number, number, number][] {
  const positions: [number, number, number][] = [];

  for (const road of roads) {
    if (!['primary', 'secondary', 'tertiary', 'trunk', 'motorway'].includes(road.highway)) continue;

    for (let i = 0; i < road.points.length - 1 && positions.length < maxLights; i++) {
      const a = road.points[i];
      const b = road.points[i + 1];
      const dx = b.x - a.x;
      const dz = b.z - a.z;
      const segLen = Math.sqrt(dx * dx + dz * dz);
      const steps = Math.floor(segLen / spacing);

      for (let s = 0; s < steps && positions.length < maxLights; s++) {
        const t = s / steps;
        positions.push([a.x + dx * t, 5, a.z + dz * t]);
      }
    }
  }

  return positions;
}

const FALLBACK_POSITIONS: [number, number, number][] = [
  [30, 5, 0],
  [-30, 5, 0],
  [0, 5, 30],
  [0, 5, -30],
];

export function NightLights({ timezone = 'UTC', roads }: NightLightsProps) {
  const { hour } = useTimeOfDay(timezone);
  const { preset } = useQuality();

  const isNight = hour >= 18 || hour < 6;
  const spacing = preset === 'low' ? 200 : 100;
  const maxLights = preset === 'low' ? 6 : preset === 'medium' ? 15 : 30;

  const positions = useMemo(() => {
    if (roads.length === 0) return FALLBACK_POSITIONS;
    return sampleRoadPositions(roads, spacing, maxLights);
  }, [roads, spacing, maxLights]);

  // Early return AFTER all hooks (Rules of Hooks)
  if (!isNight) return null;

  return (
    <group>
      {positions.map((pos, i) => (
        <pointLight
          key={i}
          position={pos}
          color="#ffaa44"
          intensity={0.3}
          distance={30}
          decay={2}
        />
      ))}
    </group>
  );
}
