import { useQuality } from '../../../hooks/useQuality';
import { GroundPlane } from './GroundPlane';
import { GroundGrid } from './GroundGrid';
import { RoadNetwork } from './RoadNetwork';
import { WaterBodies } from './WaterBodies';
import { Parks } from './Parks';
import type { RoadSegment, WaterFeature, ParkFeature } from '../../../types/world3d';

interface GroundSystemProps {
  roads: RoadSegment[];
  water: WaterFeature[];
  parks: ParkFeature[];
}

export function GroundSystem({ roads, water, parks }: GroundSystemProps) {
  const quality = useQuality();

  return (
    <group>
      <GroundPlane />
      {quality.ground.gridEnabled && <GroundGrid />}
      <RoadNetwork roads={roads} />
      <WaterBodies water={water} />
      <Parks parks={parks} />
    </group>
  );
}
