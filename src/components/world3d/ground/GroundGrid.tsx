import { useMemo } from 'react';

interface GroundGridProps {
  size?: number;
  cellSize?: number;
}

export function GroundGrid({ size = 2000, cellSize = 50 }: GroundGridProps) {
  const gridLines = useMemo(() => {
    const halfSize = size / 2;
    const count = Math.floor(size / cellSize);
    const points: number[] = [];

    for (let i = 0; i <= count; i++) {
      const pos = -halfSize + i * cellSize;
      points.push(-halfSize, 0, pos, halfSize, 0, pos);
      points.push(pos, 0, -halfSize, pos, 0, halfSize);
    }

    return new Float32Array(points);
  }, [size, cellSize]);

  return (
    <lineSegments position={[0, 0.01, 0]}>
      <bufferGeometry>
        <bufferAttribute
          attach="attributes-position"
          array={gridLines}
          count={gridLines.length / 3}
          itemSize={3}
        />
      </bufferGeometry>
      <lineBasicMaterial
        color="#ffffff"
        opacity={0.02}
        transparent
        depthWrite={false}
      />
    </lineSegments>
  );
}
