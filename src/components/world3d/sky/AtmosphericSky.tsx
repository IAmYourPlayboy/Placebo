import { Sky } from '@react-three/drei';
import { useTimeOfDay } from '../../../hooks/useTimeOfDay';

interface AtmosphericSkyProps {
  timezone?: string;
}

export function AtmosphericSky({ timezone = 'UTC' }: AtmosphericSkyProps) {
  const { sunPosition } = useTimeOfDay(timezone);

  return (
    <Sky
      distance={450000}
      sunPosition={sunPosition.toArray() as [number, number, number]}
      turbidity={8}
      rayleigh={0.5}
      mieCoefficient={0.005}
      mieDirectionalG={0.8}
    />
  );
}
