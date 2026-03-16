import { useQuality } from '../../../hooks/useQuality';
import { SkyGradient } from './SkyGradient';
import { StarField } from './StarField';
import { CelestialBodies } from './CelestialBodies';
import { CloudLayer } from './CloudLayer';
import { AtmosphericSky } from './AtmosphericSky';

interface SkySystemProps {
  timezone?: string;
}

export function SkySystem({ timezone = 'UTC' }: SkySystemProps) {
  const { sky } = useQuality();

  const useAtmospheric = sky.mode === 'atmospheric';
  const showStars = sky.mode !== 'gradient';
  const showCelestial = sky.mode !== 'gradient';
  const showClouds = sky.mode === 'atmospheric';

  return (
    <group>
      {useAtmospheric ? (
        <AtmosphericSky timezone={timezone} />
      ) : (
        <SkyGradient timezone={timezone} />
      )}
      {showStars && <StarField timezone={timezone} />}
      {showCelestial && <CelestialBodies timezone={timezone} />}
      {showClouds && <CloudLayer />}
    </group>
  );
}
