import { useMemo } from 'react';
import { useTimeOfDay } from '../../../hooks/useTimeOfDay';
import { useQuality } from '../../../hooks/useQuality';

interface BasicLightsProps {
  timezone?: string;
}

export function BasicLights({ timezone = 'UTC' }: BasicLightsProps) {
  const { sunPosition, phase, t } = useTimeOfDay(timezone);
  const { lighting } = useQuality();

  const { ambientColor, ambientIntensity, sunColor, sunIntensity } = useMemo(() => {
    switch (phase) {
      case 'night':
        return {
          ambientColor: '#667799',
          ambientIntensity: 0.08,
          sunColor: '#2244aa',
          sunIntensity: 0.05,
        };
      case 'dawn':
        return {
          ambientColor: '#887766',
          ambientIntensity: 0.08 + t * 0.12,
          sunColor: '#ff6633',
          sunIntensity: 0.1 + t * 0.3,
        };
      case 'morning':
        return {
          ambientColor: '#99aabb',
          ambientIntensity: 0.2 + t * 0.1,
          sunColor: '#ffcc88',
          sunIntensity: 0.4 + t * 0.4,
        };
      case 'day':
        return {
          ambientColor: '#aabbcc',
          ambientIntensity: 0.3,
          sunColor: '#ffffff',
          sunIntensity: 0.8,
        };
      case 'dusk':
        return {
          ambientColor: '#886655',
          ambientIntensity: 0.3 - t * 0.15,
          sunColor: '#ff8844',
          sunIntensity: 0.8 - t * 0.5,
        };
      case 'twilight':
        return {
          ambientColor: '#556688',
          ambientIntensity: 0.15 - t * 0.07,
          sunColor: '#334488',
          sunIntensity: 0.3 - t * 0.25,
        };
      default:
        return {
          ambientColor: '#aabbcc',
          ambientIntensity: 0.3,
          sunColor: '#ffffff',
          sunIntensity: 0.8,
        };
    }
  }, [phase, t]);

  return (
    <group>
      <ambientLight color={ambientColor} intensity={ambientIntensity} />
      <directionalLight
        color={sunColor}
        intensity={sunIntensity}
        position={sunPosition.toArray()}
        castShadow
        shadow-mapSize-width={lighting.shadowMapSize}
        shadow-mapSize-height={lighting.shadowMapSize}
        shadow-camera-far={1000}
        shadow-camera-left={-200}
        shadow-camera-right={200}
        shadow-camera-top={200}
        shadow-camera-bottom={-200}
      />
    </group>
  );
}
