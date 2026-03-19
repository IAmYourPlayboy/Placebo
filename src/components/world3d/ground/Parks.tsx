import { useMemo } from 'react';
import * as THREE from 'three';
import type { ParkFeature } from '../../../types/world3d';

const PARK_COLOR = '#0a2a0a';
const PARK_OPACITY = 0.3;

interface ParksProps {
  parks: ParkFeature[];
}

export function Parks({ parks }: ParksProps) {
  const geometry = useMemo(() => {
    const shapes: THREE.Shape[] = [];
    for (const p of parks) {
      if (p.points.length < 3) continue;
      const shape = new THREE.Shape();
      shape.moveTo(p.points[0].x, p.points[0].z);
      for (let i = 1; i < p.points.length; i++) {
        shape.lineTo(p.points[i].x, p.points[i].z);
      }
      shapes.push(shape);
    }
    return shapes.length > 0 ? new THREE.ShapeGeometry(shapes) : null;
  }, [parks]);

  if (!geometry) return null;

  return (
    <mesh geometry={geometry} rotation={[-Math.PI / 2, 0, 0]} position={[0, 0.01, 0]}>
      <meshBasicMaterial
        color={PARK_COLOR}
        opacity={PARK_OPACITY}
        transparent
        depthWrite={false}
        side={THREE.DoubleSide}
      />
    </mesh>
  );
}
