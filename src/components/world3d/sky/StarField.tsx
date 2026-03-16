import { useMemo } from 'react';
import { useTimeOfDay } from '../../../hooks/useTimeOfDay';

interface StarFieldProps {
  count?: number;
  timezone?: string;
}

export function StarField({ count = 500, timezone = 'UTC' }: StarFieldProps) {
  const { phase } = useTimeOfDay(timezone);

  const opacity = phase === 'night' ? 0.8
    : phase === 'twilight' ? 0.5
    : phase === 'dawn' ? 0.3
    : 0;

  const positions = useMemo(() => {
    const pts = new Float32Array(count * 3);
    const radius = 2800;

    for (let i = 0; i < count; i++) {
      const theta = Math.random() * Math.PI * 2;
      const phi = Math.random() * Math.PI * 0.45;
      const r = radius + (Math.random() - 0.5) * 100;

      pts[i * 3] = r * Math.sin(phi) * Math.cos(theta);
      pts[i * 3 + 1] = r * Math.cos(phi);
      pts[i * 3 + 2] = r * Math.sin(phi) * Math.sin(theta);
    }
    return pts;
  }, [count]);

  if (opacity <= 0) return null;

  return (
    <points>
      <bufferGeometry>
        <bufferAttribute attach="attributes-position" array={positions} count={count} itemSize={3} />
      </bufferGeometry>
      <pointsMaterial
        color="#ffffff"
        size={2}
        sizeAttenuation={false}
        transparent
        opacity={opacity}
        depthWrite={false}
      />
    </points>
  );
}
