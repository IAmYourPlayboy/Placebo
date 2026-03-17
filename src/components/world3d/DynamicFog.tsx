import { useRef, useEffect } from 'react';
import { useThree, useFrame } from '@react-three/fiber';
import * as THREE from 'three';
import { useTimeOfDay, type TimePhase } from '../../hooks/useTimeOfDay';

const FOG_COLORS: Record<TimePhase, string> = {
  night: '#050510',
  dawn: '#1a1020',
  morning: '#2a4060',
  day: '#4a6a8a',
  dusk: '#3a2020',
  twilight: '#0a0a1a',
};

interface DynamicFogProps {
  timezone: string;
  near: number;
  far: number;
}

export function DynamicFog({ timezone, near, far }: DynamicFogProps) {
  const { scene } = useThree();
  const timeOfDay = useTimeOfDay(timezone);
  const targetColor = useRef(new THREE.Color(FOG_COLORS.day));

  useEffect(() => {
    const colorHex = FOG_COLORS[timeOfDay.phase] || FOG_COLORS.day;
    targetColor.current.set(colorHex);
  }, [timeOfDay.phase]);

  useEffect(() => {
    scene.fog = new THREE.Fog(FOG_COLORS.day, near, far);
    return () => { scene.fog = null; };
  }, [scene, near, far]);

  useFrame(() => {
    if (scene.fog instanceof THREE.Fog) {
      scene.fog.color.lerp(targetColor.current, 0.02);
    }
  });

  return null;
}
