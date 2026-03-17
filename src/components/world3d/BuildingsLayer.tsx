import { useMemo } from 'react';
import * as THREE from 'three';
import { mergeGeometries } from 'three/examples/jsm/utils/BufferGeometryUtils.js';
import type { BuildingFootprint } from '../../types/world3d';

const FILL_COLOR = '#0a0f18';
const FILL_OPACITY = 0.06;
const EDGE_COLOR = '#1e2840';
const EDGE_OPACITY = 0.4;

interface BuildingsLayerProps {
  buildings: BuildingFootprint[];
}

export function BuildingsLayer({ buildings }: BuildingsLayerProps) {
  const { fillGeo, edgeGeo } = useMemo(() => {
    if (buildings.length === 0) return { fillGeo: null, edgeGeo: null };

    const fillGeometries: THREE.BufferGeometry[] = [];
    const edgeGeometries: THREE.BufferGeometry[] = [];

    for (const b of buildings) {
      if (b.outline.length < 3 || b.height <= 0) continue;

      const shape = new THREE.Shape();
      shape.moveTo(b.outline[0].x, b.outline[0].z);
      for (let i = 1; i < b.outline.length; i++) {
        shape.lineTo(b.outline[i].x, b.outline[i].z);
      }

      const extruded = new THREE.ExtrudeGeometry(shape, {
        depth: b.height,
        bevelEnabled: false,
      });

      // ExtrudeGeometry extrudes along local Z → rotate to Y-up
      extruded.rotateX(-Math.PI / 2);

      fillGeometries.push(extruded);
      edgeGeometries.push(new THREE.EdgesGeometry(extruded));
    }

    const fillGeo = fillGeometries.length > 0
      ? mergeGeometries(fillGeometries, false)
      : null;
    const edgeGeo = edgeGeometries.length > 0
      ? mergeGeometries(edgeGeometries, false)
      : null;

    // Dispose individual geometries
    for (const g of fillGeometries) g.dispose();
    for (const g of edgeGeometries) g.dispose();

    return { fillGeo: fillGeo ?? null, edgeGeo: edgeGeo ?? null };
  }, [buildings]);

  return (
    <group>
      {fillGeo && (
        <mesh geometry={fillGeo}>
          <meshBasicMaterial
            color={FILL_COLOR}
            opacity={FILL_OPACITY}
            transparent
            depthWrite={false}
            side={THREE.DoubleSide}
          />
        </mesh>
      )}
      {edgeGeo && (
        <lineSegments geometry={edgeGeo}>
          <lineBasicMaterial
            color={EDGE_COLOR}
            opacity={EDGE_OPACITY}
            transparent
            depthWrite={false}
          />
        </lineSegments>
      )}
    </group>
  );
}
