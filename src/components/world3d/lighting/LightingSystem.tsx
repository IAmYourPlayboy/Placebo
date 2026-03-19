import { BasicLights } from './BasicLights';
import { NightLights } from './NightLights';
import type { RoadSegment } from '../../../types/world3d';

interface LightingSystemProps {
  timezone?: string;
  roads: RoadSegment[];
}

export function LightingSystem({ timezone = 'UTC', roads }: LightingSystemProps) {
  return (
    <group>
      <BasicLights timezone={timezone} />
      <NightLights timezone={timezone} roads={roads} />
    </group>
  );
}
