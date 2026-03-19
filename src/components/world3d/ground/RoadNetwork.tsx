import { useMemo } from 'react';
import * as THREE from 'three';
import type { RoadSegment } from '../../../types/world3d';

interface RoadNetworkProps {
  roads: RoadSegment[];
}

const ROAD_OPACITY: Record<string, number> = {
  motorway: 0.07,
  trunk: 0.06,
  primary: 0.06,
  secondary: 0.045,
  tertiary: 0.035,
  residential: 0.03,
  footway: 0.02,
  path: 0.02,
  cycleway: 0.02,
  service: 0.025,
  pedestrian: 0.025,
};

function tessellateRoad(road: RoadSegment): Float32Array | null {
  const pts = road.points;
  if (pts.length < 2) return null;

  const halfW = road.width / 2;
  const vertices: number[] = [];

  for (let i = 0; i < pts.length - 1; i++) {
    const curr = pts[i];
    const next = pts[i + 1];

    const dx = next.x - curr.x;
    const dz = next.z - curr.z;
    const len = Math.sqrt(dx * dx + dz * dz);
    if (len < 0.01) continue;

    const px = -dz / len;
    const pz = dx / len;

    const x0 = curr.x + px * halfW;
    const z0 = curr.z + pz * halfW;
    const x1 = curr.x - px * halfW;
    const z1 = curr.z - pz * halfW;
    const x2 = next.x + px * halfW;
    const z2 = next.z + pz * halfW;
    const x3 = next.x - px * halfW;
    const z3 = next.z - pz * halfW;

    vertices.push(x0, 0.02, z0);
    vertices.push(x1, 0.02, z1);
    vertices.push(x2, 0.02, z2);
    vertices.push(x1, 0.02, z1);
    vertices.push(x3, 0.02, z3);
    vertices.push(x2, 0.02, z2);
  }

  return vertices.length > 0 ? new Float32Array(vertices) : null;
}

export function RoadNetwork({ roads }: RoadNetworkProps) {
  const meshes = useMemo(() => {
    const groups = new Map<number, Float32Array[]>();

    for (const road of roads) {
      const opacity = ROAD_OPACITY[road.highway] ?? 0.03;
      const geom = tessellateRoad(road);
      if (!geom) continue;

      if (!groups.has(opacity)) groups.set(opacity, []);
      groups.get(opacity)!.push(geom);
    }

    return Array.from(groups.entries()).map(([opacity, arrays]) => {
      const totalLen = arrays.reduce((sum, a) => sum + a.length, 0);
      const merged = new Float32Array(totalLen);
      let offset = 0;
      for (const arr of arrays) {
        merged.set(arr, offset);
        offset += arr.length;
      }
      return { opacity, vertices: merged };
    });
  }, [roads]);

  if (meshes.length === 0) return null;

  return (
    <group>
      {meshes.map(({ opacity, vertices }, i) => (
        <mesh key={i}>
          <bufferGeometry>
            <bufferAttribute
              attach="attributes-position"
              array={vertices}
              count={vertices.length / 3}
              itemSize={3}
            />
          </bufferGeometry>
          <meshBasicMaterial
            color="#ffffff"
            opacity={opacity}
            transparent
            depthWrite={false}
            side={THREE.DoubleSide}
          />
        </mesh>
      ))}
    </group>
  );
}
