import { useQuality } from '../../../hooks/useQuality';
import { GroundPlane } from './GroundPlane';
import { GroundGrid } from './GroundGrid';
import { RoadNetwork } from './RoadNetwork';
import type { RoadSegment } from '../../../types/world3d';

interface GroundSystemProps {
  roads: RoadSegment[];
}

export function GroundSystem({ roads }: GroundSystemProps) {
  const quality = useQuality();

  return (
    <group>
      <GroundPlane />
      {quality.ground.gridEnabled && <GroundGrid />}
      <RoadNetwork roads={roads} />
    </group>
  );
}
