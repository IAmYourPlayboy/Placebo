import { useState, useEffect, useMemo } from 'react';
import * as THREE from 'three';

export type TimePhase = 'night' | 'dawn' | 'morning' | 'day' | 'dusk' | 'twilight';

export interface TimeOfDay {
  hour: number;
  sunPosition: THREE.Vector3;
  moonPosition: THREE.Vector3;
  phase: TimePhase;
  /** 0–1 progress within current phase, for smooth lerp transitions */
  t: number;
}

const PHASES: { name: TimePhase; start: number; end: number }[] = [
  { name: 'night',    start: 0,  end: 5 },
  { name: 'dawn',     start: 5,  end: 7 },
  { name: 'morning',  start: 7,  end: 10 },
  { name: 'day',      start: 10, end: 16 },
  { name: 'dusk',     start: 16, end: 19 },
  { name: 'twilight', start: 19, end: 21 },
];

const SUN_RADIUS = 1000;

function getLocalHour(timezone: string): number {
  try {
    const fmt = new Intl.DateTimeFormat('en-US', {
      timeZone: timezone,
      hour: 'numeric',
      minute: 'numeric',
      hour12: false,
    });
    const parts = fmt.formatToParts(new Date());
    const h = parseInt(parts.find((p) => p.type === 'hour')?.value ?? '12', 10);
    const m = parseInt(parts.find((p) => p.type === 'minute')?.value ?? '0', 10);
    return h + m / 60;
  } catch {
    return 12;
  }
}

export function useTimeOfDay(timezone: string = 'UTC'): TimeOfDay {
  const [minute, setMinute] = useState(0);

  useEffect(() => {
    setMinute(Math.floor(Date.now() / 60000));
    const id = setInterval(() => setMinute(Math.floor(Date.now() / 60000)), 60000);
    return () => clearInterval(id);
  }, []);

  return useMemo(() => {
    const hour = getLocalHour(timezone);

    const sunAngle = ((hour - 6) / 12) * Math.PI;
    const sunPosition = new THREE.Vector3(
      Math.cos(sunAngle) * SUN_RADIUS,
      Math.sin(sunAngle) * SUN_RADIUS,
      0
    );

    const moonAngle = ((hour - 18) / 12) * Math.PI;
    const moonPosition = new THREE.Vector3(
      Math.cos(moonAngle) * SUN_RADIUS * 0.8,
      Math.sin(moonAngle) * SUN_RADIUS * 0.8,
      SUN_RADIUS * 0.3
    );

    let phase: TimePhase = 'night';
    let t = 0;

    for (const p of PHASES) {
      if (hour >= p.start && hour < p.end) {
        phase = p.name;
        t = (hour - p.start) / (p.end - p.start);
        break;
      }
    }
    if (hour >= 21) {
      phase = 'night';
      t = (hour - 21) / (24 - 21 + 5);
    }

    return { hour, sunPosition, moonPosition, phase, t };
  }, [timezone, minute]);
}
