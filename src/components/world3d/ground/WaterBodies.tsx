import { useMemo } from 'react';
import * as THREE from 'three';
import type { WaterFeature } from '../../../types/world3d';

const WATER_COLOR = '#0a1a3a';
const WATER_OPACITY = 0.4;
const RIVER_WIDTH = 8; // meters

interface WaterBodiesProps {
  water: WaterFeature[];
}

function tessellateRibbon(points: { x: number; z: number }[], halfWidth: number): Float32Array {
  const verts: number[] = [];
  for (let i = 0; i < points.length - 1; i++) {
    const p0 = points[i];
    const p1 = points[i + 1];
    const dx = p1.x - p0.x;
    const dz = p1.z - p0.z;
    const len = Math.sqrt(dx * dx + dz * dz);
    if (len < 0.001) continue;
    const nx = (-dz / len) * halfWidth;
    const nz = (dx / len) * halfWidth;
    const y = 0.02;
    verts.push(p0.x - nx, y, p0.z - nz);
    verts.push(p0.x + nx, y, p0.z + nz);
    verts.push(p1.x - nx, y, p1.z - nz);
    verts.push(p1.x - nx, y, p1.z - nz);
    verts.push(p0.x + nx, y, p0.z + nz);
    verts.push(p1.x + nx, y, p1.z + nz);
  }
  return new Float32Array(verts);
}

export function WaterBodies({ water }: WaterBodiesProps) {
  const { polygonGeo, lineGeo } = useMemo(() => {
    // Polygon water (lakes, ponds)
    const shapes: THREE.Shape[] = [];
    for (const w of water) {
      if (w.geomType !== 'polygon' || w.points.length < 3) continue;
      const shape = new THREE.Shape();
      shape.moveTo(w.points[0].x, w.points[0].z);
      for (let i = 1; i < w.points.length; i++) {
        shape.lineTo(w.points[i].x, w.points[i].z);
      }
      shapes.push(shape);
    }
    const polygonGeo = shapes.length > 0
      ? new THREE.ShapeGeometry(shapes)
      : null;

    // Line water (rivers, streams) – ribbon mesh
    const allVerts: number[] = [];
    for (const w of water) {
      if (w.geomType !== 'line' || w.points.length < 2) continue;
      const ribbon = tessellateRibbon(w.points, RIVER_WIDTH / 2);
      for (let i = 0; i < ribbon.length; i++) allVerts.push(ribbon[i]);
    }
    const lineGeo = allVerts.length > 0
      ? new THREE.BufferGeometry().setAttribute(
          'position',
          new THREE.Float32BufferAttribute(allVerts, 3)
        )
      : null;

    return { polygonGeo, lineGeo };
  }, [water]);

  return (
    <group>
      {polygonGeo && (
        <mesh geometry={polygonGeo} rotation={[-Math.PI / 2, 0, 0]} position={[0, 0.02, 0]}>
          <meshBasicMaterial
            color={WATER_COLOR}
            opacity={WATER_OPACITY}
            transparent
            depthWrite={false}
            side={THREE.DoubleSide}
          />
        </mesh>
      )}
      {lineGeo && (
        <mesh geometry={lineGeo}>
          <meshBasicMaterial
            color={WATER_COLOR}
            opacity={WATER_OPACITY}
            transparent
            depthWrite={false}
            side={THREE.DoubleSide}
          />
        </mesh>
      )}
    </group>
  );
}
