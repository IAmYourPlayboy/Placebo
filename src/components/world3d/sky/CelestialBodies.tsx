import * as THREE from 'three';
import { useTimeOfDay } from '../../../hooks/useTimeOfDay';

interface CelestialBodiesProps {
  timezone?: string;
}

export function CelestialBodies({ timezone = 'UTC' }: CelestialBodiesProps) {
  const { sunPosition, moonPosition, hour } = useTimeOfDay(timezone);

  const sunVisible = sunPosition.y > -100;
  const moonVisible = hour >= 19 || hour < 6;

  return (
    <group>
      {sunVisible && (
        <group position={sunPosition.toArray()}>
          <sprite>
            <spriteMaterial
              color="#fff8e0"
              opacity={Math.min(1, sunPosition.y / 200)}
              transparent
              depthWrite={false}
            />
          </sprite>
          <sprite scale={[80, 80, 1]}>
            <spriteMaterial
              color="#ffcc44"
              opacity={Math.min(0.3, sunPosition.y / 500)}
              transparent
              depthWrite={false}
              blending={THREE.AdditiveBlending}
            />
          </sprite>
        </group>
      )}

      {moonVisible && (
        <sprite position={moonPosition.toArray()} scale={[30, 30, 1]}>
          <spriteMaterial
            color="#c8d0e0"
            opacity={0.6}
            transparent
            depthWrite={false}
          />
        </sprite>
      )}
    </group>
  );
}
