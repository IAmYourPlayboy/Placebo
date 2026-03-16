import { useMemo, useEffect } from 'react';
import * as THREE from 'three';
import { useTimeOfDay, type TimePhase } from '../../../hooks/useTimeOfDay';

const SKY_COLORS: Record<TimePhase, { zenith: string; horizon: string }> = {
  night:    { zenith: '#020408', horizon: '#0a0e18' },
  dawn:     { zenith: '#0a0520', horizon: '#2a1535' },
  morning:  { zenith: '#1a3560', horizon: '#4a6590' },
  day:      { zenith: '#3a6a9f', horizon: '#7aa0c5' },
  dusk:     { zenith: '#2a2050', horizon: '#c06030' },
  twilight: { zenith: '#101030', horizon: '#2a2045' },
};

interface SkyGradientProps {
  timezone?: string;
}

export function SkyGradient({ timezone = 'UTC' }: SkyGradientProps) {
  const { phase } = useTimeOfDay(timezone);
  const colors = SKY_COLORS[phase];

  const geometry = useMemo(() => {
    const geo = new THREE.SphereGeometry(3000, 32, 16, 0, Math.PI * 2, 0, Math.PI / 2);
    const positions = geo.attributes.position;
    const colorsArr = new Float32Array(positions.count * 3);
    const zenith = new THREE.Color(colors.zenith);
    const horizon = new THREE.Color(colors.horizon);
    const tmp = new THREE.Color();

    for (let i = 0; i < positions.count; i++) {
      const y = positions.getY(i);
      const t = Math.max(0, y / 3000);
      tmp.copy(horizon).lerp(zenith, t);
      colorsArr[i * 3] = tmp.r;
      colorsArr[i * 3 + 1] = tmp.g;
      colorsArr[i * 3 + 2] = tmp.b;
    }

    geo.setAttribute('color', new THREE.BufferAttribute(colorsArr, 3));
    return geo;
  }, [colors.zenith, colors.horizon]);

  // Dispose previous geometry on change to prevent GPU memory leak
  useEffect(() => {
    return () => geometry.dispose();
  }, [geometry]);

  return (
    <mesh geometry={geometry} rotation={[0, 0, 0]}>
      <meshBasicMaterial
        vertexColors
        side={THREE.BackSide}
        depthWrite={false}
      />
    </mesh>
  );
}
