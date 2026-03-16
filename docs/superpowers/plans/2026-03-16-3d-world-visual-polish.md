# 3D World Visual Polish Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Polish the 3D Tokyo scene into a clean minimalist "Clean Dark" visual style with three quality presets (low/medium/high), real OSM roads, improved sky/lighting, and trackpad-friendly navigation.

**Architecture:** Layered system — each visual domain (sky, ground, lighting, post) is an independent module reading from a shared QualityContext. Components compose in WorldScene via orchestrators. Time-of-day is a shared hook consumed by sky and lighting.

**Tech Stack:** React Three Fiber 8, three.js 0.183, @react-three/drei 9, @react-three/postprocessing (new), hls.js, TypeScript 5.5

**Spec:** `docs/superpowers/specs/2026-03-16-3d-world-visual-polish-design.md`

---

## Out of Scope (separate plan)

The following spec items require backend/pipeline work and will be implemented in a separate plan:
- `pipeline/sql/03_roads_view.sql` – SQL view for road data extraction
- `pipeline/scripts/generate-tiles.sh` – add roads GeoJSON export step
- `GET /api/world/roads` axum endpoint – serve static GeoJSON files
- Intersection glow rendering (depends on road data being available)

The `useRoadNetwork` hook is stubbed to return `[]` until the API endpoint is ready.

---

## Chunk 1: Foundation — Types, Quality Context, Time Hook

### Task 1: Install new dependencies

**Files:**
- Modify: `package.json`

- [ ] **Step 1: Install postprocessing packages**

```bash
npm install @react-three/postprocessing postprocessing
```

- [ ] **Step 2: Verify install**

```bash
node -e "require('@react-three/postprocessing'); console.log('ok')"
```

Expected: `ok`

- [ ] **Step 3: Commit**

```bash
git add package.json package-lock.json
git commit -m "deps: add @react-three/postprocessing for medium/high presets"
```

---

### Task 2: Replace QualitySettings with QualityConfig

**Files:**
- Modify: `src/types/world3d.ts` (lines 67–87: replace QualitySettings + DEFAULT_QUALITY)

- [ ] **Step 1: Replace QualitySettings type and defaults**

In `src/types/world3d.ts`, replace the `QualitySettings` interface (lines 67–78) and `DEFAULT_QUALITY` (lines 81–87) with:

```typescript
// ─── Quality Presets ────────────────────────────────────────
export type QualityPreset = 'low' | 'medium' | 'high';
export type SkyMode = 'gradient' | 'gradient-stars-sun' | 'atmospheric';
export type GroundDetail = 'roads-basic' | 'roads-detailed' | 'roads-ssr';
export type LightingMode = 'basic' | 'ssao-bloom' | 'full';
export type PostMode = 'none' | 'tonemap-vignette' | 'full';

export interface QualityConfig {
  preset: QualityPreset;
  sky: { mode: SkyMode };
  ground: { detail: GroundDetail; gridEnabled: boolean };
  lighting: { mode: LightingMode; shadowMapSize: number };
  post: { mode: PostMode };
  maxVideoTextures: number;
  fog: { near: number; far: number; color: string };
}

export const QUALITY_PRESETS: Record<QualityPreset, QualityConfig> = {
  low: {
    preset: 'low',
    sky: { mode: 'gradient' },
    ground: { detail: 'roads-basic', gridEnabled: false },
    lighting: { mode: 'basic', shadowMapSize: 1024 },
    post: { mode: 'none' },
    maxVideoTextures: 1,
    fog: { near: 300, far: 1500, color: '#0a0a1a' },
  },
  medium: {
    preset: 'medium',
    sky: { mode: 'gradient-stars-sun' },
    ground: { detail: 'roads-detailed', gridEnabled: true },
    lighting: { mode: 'ssao-bloom', shadowMapSize: 2048 },
    post: { mode: 'tonemap-vignette' },
    maxVideoTextures: 2,
    fog: { near: 500, far: 2000, color: '#0a0a1a' },
  },
  high: {
    preset: 'high',
    sky: { mode: 'atmospheric' },
    ground: { detail: 'roads-ssr', gridEnabled: true },
    lighting: { mode: 'full', shadowMapSize: 4096 },
    post: { mode: 'full' },
    maxVideoTextures: 4,
    fog: { near: 800, far: 3000, color: '#0a0a1a' },
  },
};

export const DEFAULT_QUALITY = QUALITY_PRESETS.medium;
```

- [ ] **Step 2: Fix type references in WorldScene.tsx**

In `src/components/world3d/WorldScene.tsx`, update the import and props:
- Change `QualitySettings` → `QualityConfig` in the import (line 4)
- Change `quality?: QualitySettings` → `quality?: QualityConfig` in WorldSceneProps (line ~22)
- The `DEFAULT_QUALITY` import stays the same

- [ ] **Step 3: Fix type references in World3DScreen.tsx**

No changes needed — World3DScreen doesn't reference QualitySettings directly, it passes `quality` prop to WorldScene which has a default.

- [ ] **Step 4: Verify build**

```bash
npx tsc --noEmit
```

Expected: No errors

- [ ] **Step 5: Commit**

```bash
git add src/types/world3d.ts src/components/world3d/WorldScene.tsx
git commit -m "refactor: replace QualitySettings with QualityConfig and three presets"
```

---

### Task 3: Create QualityContext + useQuality hook

**Files:**
- Create: `src/hooks/useQuality.ts`

- [ ] **Step 1: Create the quality context and hook**

```typescript
import { createContext, useContext } from 'react';
import { QualityConfig, DEFAULT_QUALITY } from '../types/world3d';

export const QualityContext = createContext<QualityConfig>(DEFAULT_QUALITY);

export function useQuality(): QualityConfig {
  return useContext(QualityContext);
}
```

- [ ] **Step 2: Verify build**

```bash
npx tsc --noEmit
```

- [ ] **Step 3: Commit**

```bash
git add src/hooks/useQuality.ts
git commit -m "feat: add QualityContext and useQuality hook"
```

---

### Task 4: Create useTimeOfDay hook

**Files:**
- Create: `src/hooks/useTimeOfDay.ts`

- [ ] **Step 1: Create the time-of-day hook**

Extract and improve the time logic from `Environment.tsx` (lines 125–138):

```typescript
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
  // night again 21–24 handled by wrapping
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

  // Update every 60 seconds so the sky/lighting evolves in real time
  useEffect(() => {
    setMinute(Math.floor(Date.now() / 60000));
    const id = setInterval(() => setMinute(Math.floor(Date.now() / 60000)), 60000);
    return () => clearInterval(id);
  }, []);

  return useMemo(() => {
    const hour = getLocalHour(timezone);

    // Sun arc: east (+X) at 6:00 → overhead (+Y) at 12:00 → west (-X) at 18:00
    const sunAngle = ((hour - 6) / 12) * Math.PI;
    const sunPosition = new THREE.Vector3(
      Math.cos(sunAngle) * SUN_RADIUS,
      Math.sin(sunAngle) * SUN_RADIUS,
      0
    );

    // Moon: opposite arc
    const moonAngle = ((hour - 18) / 12) * Math.PI;
    const moonPosition = new THREE.Vector3(
      Math.cos(moonAngle) * SUN_RADIUS * 0.8,
      Math.sin(moonAngle) * SUN_RADIUS * 0.8,
      SUN_RADIUS * 0.3
    );

    // Phase detection
    let phase: TimePhase = 'night';
    let t = 0;

    for (const p of PHASES) {
      if (hour >= p.start && hour < p.end) {
        phase = p.name;
        t = (hour - p.start) / (p.end - p.start);
        break;
      }
    }
    // 21–24 is also night
    if (hour >= 21) {
      phase = 'night';
      t = (hour - 21) / (24 - 21 + 5); // progress through night 21→5
    }

    return { hour, sunPosition, moonPosition, phase, t };
  }, [timezone, minute]);
}
```

- [ ] **Step 2: Verify build**

```bash
npx tsc --noEmit
```

- [ ] **Step 3: Commit**

```bash
git add src/hooks/useTimeOfDay.ts
git commit -m "feat: add useTimeOfDay hook with sun/moon position and phase detection"
```

---

## Chunk 2: Ground System

### Task 5: Create GroundPlane component

**Files:**
- Create: `src/components/world3d/ground/GroundPlane.tsx`

- [ ] **Step 1: Create GroundPlane**

Move and improve from `Environment.tsx` (lines 4–25):

```typescript
import * as THREE from 'three';

interface GroundPlaneProps {
  radius?: number;
}

export function GroundPlane({ radius = 2000 }: GroundPlaneProps) {
  return (
    <mesh rotation={[-Math.PI / 2, 0, 0]} position={[0, -0.1, 0]} receiveShadow>
      <circleGeometry args={[radius, 64]} />
      <meshStandardMaterial
        color="#0a0e14"
        roughness={0.95}
        metalness={0}
      />
    </mesh>
  );
}
```

- [ ] **Step 2: Commit**

```bash
mkdir -p src/components/world3d/ground
git add src/components/world3d/ground/GroundPlane.tsx
git commit -m "feat: create ground/GroundPlane with improved color"
```

---

### Task 6: Create GroundGrid component

**Files:**
- Create: `src/components/world3d/ground/GroundGrid.tsx`

- [ ] **Step 1: Create GroundGrid**

```typescript
import { useMemo } from 'react';
import * as THREE from 'three';

interface GroundGridProps {
  size?: number;      // total grid size in meters
  cellSize?: number;  // cell size in meters
}

/**
 * Thin grid overlay on the ground plane (medium/high presets only).
 * Fades toward edges for a clean look.
 */
export function GroundGrid({ size = 2000, cellSize = 50 }: GroundGridProps) {
  const gridLines = useMemo(() => {
    const halfSize = size / 2;
    const count = Math.floor(size / cellSize);
    const points: number[] = [];

    for (let i = 0; i <= count; i++) {
      const pos = -halfSize + i * cellSize;
      // Horizontal line
      points.push(-halfSize, 0, pos, halfSize, 0, pos);
      // Vertical line
      points.push(pos, 0, -halfSize, pos, 0, halfSize);
    }

    return new Float32Array(points);
  }, [size, cellSize]);

  return (
    <lineSegments position={[0, 0.01, 0]}>
      <bufferGeometry>
        <bufferAttribute
          attach="attributes-position"
          array={gridLines}
          count={gridLines.length / 3}
          itemSize={3}
        />
      </bufferGeometry>
      <lineBasicMaterial
        color="#ffffff"
        opacity={0.02}
        transparent
        depthWrite={false}
      />
    </lineSegments>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add src/components/world3d/ground/GroundGrid.tsx
git commit -m "feat: create ground/GroundGrid for medium/high presets"
```

---

### Task 7: Create useRoadNetwork hook

**Files:**
- Create: `src/hooks/useRoadNetwork.ts`

- [ ] **Step 1: Create the road data hook**

```typescript
import { useState, useEffect } from 'react';
import { geoToLocal } from '../types/world3d';

export interface RoadSegment {
  /** Local XZ coordinates (meters from center) */
  points: { x: number; z: number }[];
  highway: string;
  name: string | null;
  /** Width in meters (from OSM or default by type) */
  width: number;
}

export interface RoadData {
  roads: RoadSegment[];
  loading: boolean;
}

const DEFAULT_WIDTHS: Record<string, number> = {
  motorway: 15,
  trunk: 14,
  primary: 12,
  secondary: 9,
  tertiary: 7,
  residential: 6,
  unclassified: 5,
  footway: 2.5,
  path: 2,
  cycleway: 2.5,
  service: 4,
  pedestrian: 4,
};

/**
 * Fetches road network data from our API.
 * Returns roads as local XZ coordinates relative to center lat/lng.
 *
 * NOTE: API endpoint not yet implemented. Returns empty array for now.
 * When API is ready, uncomment the fetch logic.
 */
export function useRoadNetwork(
  centerLat: number,
  centerLng: number,
  radiusMeters: number = 1000
): RoadData {
  const [roads, setRoads] = useState<RoadSegment[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    // TODO: Uncomment when /api/world/roads endpoint is implemented
    // For now, returns empty – the world renders without roads gracefully.
    //
    // setLoading(true);
    // fetch(`/api/world/roads?lat=${centerLat}&lng=${centerLng}&radius=${radiusMeters}`)
    //   .then(r => r.json())
    //   .then((geojson) => {
    //     const segments = geojson.features.map((f: any) => {
    //       const highway = f.properties.highway || 'residential';
    //       const width = f.properties.width || DEFAULT_WIDTHS[highway] || 6;
    //       const points = f.geometry.coordinates.map(([lng, lat]: [number, number]) => {
    //         const { x, z } = geoToLocal(lat, lng, centerLat, centerLng);
    //         return { x, z };
    //       });
    //       return { points, highway, name: f.properties.name, width };
    //     });
    //     setRoads(segments);
    //   })
    //   .catch(console.error)
    //   .finally(() => setLoading(false));

    setRoads([]);
    setLoading(false);
  }, [centerLat, centerLng, radiusMeters]);

  return { roads, loading };
}
```

- [ ] **Step 2: Commit**

```bash
git add src/hooks/useRoadNetwork.ts
git commit -m "feat: add useRoadNetwork hook (stub, ready for API)"
```

---

### Task 8: Create RoadNetwork renderer

**Files:**
- Create: `src/components/world3d/ground/RoadNetwork.tsx`

- [ ] **Step 1: Create RoadNetwork component**

```typescript
import { useMemo } from 'react';
import * as THREE from 'three';
import type { RoadSegment } from '../../../hooks/useRoadNetwork';

interface RoadNetworkProps {
  roads: RoadSegment[];
}

/** Road color by highway classification (Clean Dark palette) */
const ROAD_OPACITY: Record<string, number> = {
  motorway: 0.07,
  trunk: 0.06,
  primary: 0.06,
  secondary: 0.045,
  tertiary: 0.035,
  residential: 0.03,
  footway: 0.02,
  path: 0.02,
  cycleway: 0.02,
  service: 0.025,
  pedestrian: 0.025,
};

/**
 * Tessellates road LineStrings into flat ribbon meshes on the XZ plane.
 * Each road segment becomes a strip of quads with miter joins at corners.
 */
function tessellateRoad(road: RoadSegment): Float32Array | null {
  const pts = road.points;
  if (pts.length < 2) return null;

  const halfW = road.width / 2;
  const vertices: number[] = [];

  for (let i = 0; i < pts.length - 1; i++) {
    const curr = pts[i];
    const next = pts[i + 1];

    // Direction along road
    const dx = next.x - curr.x;
    const dz = next.z - curr.z;
    const len = Math.sqrt(dx * dx + dz * dz);
    if (len < 0.01) continue;

    // Perpendicular (on XZ plane)
    const px = -dz / len;
    const pz = dx / len;

    // Quad: two triangles
    const x0 = curr.x + px * halfW;
    const z0 = curr.z + pz * halfW;
    const x1 = curr.x - px * halfW;
    const z1 = curr.z - pz * halfW;
    const x2 = next.x + px * halfW;
    const z2 = next.z + pz * halfW;
    const x3 = next.x - px * halfW;
    const z3 = next.z - pz * halfW;

    // Triangle 1
    vertices.push(x0, 0.02, z0);
    vertices.push(x1, 0.02, z1);
    vertices.push(x2, 0.02, z2);
    // Triangle 2
    vertices.push(x1, 0.02, z1);
    vertices.push(x3, 0.02, z3);
    vertices.push(x2, 0.02, z2);
  }

  return vertices.length > 0 ? new Float32Array(vertices) : null;
}

export function RoadNetwork({ roads }: RoadNetworkProps) {
  const meshes = useMemo(() => {
    // Group roads by opacity level to batch draw calls
    const groups = new Map<number, Float32Array[]>();

    for (const road of roads) {
      const opacity = ROAD_OPACITY[road.highway] ?? 0.03;
      const geom = tessellateRoad(road);
      if (!geom) continue;

      if (!groups.has(opacity)) groups.set(opacity, []);
      groups.get(opacity)!.push(geom);
    }

    // Merge each group into a single BufferGeometry
    return Array.from(groups.entries()).map(([opacity, arrays]) => {
      const totalLen = arrays.reduce((sum, a) => sum + a.length, 0);
      const merged = new Float32Array(totalLen);
      let offset = 0;
      for (const arr of arrays) {
        merged.set(arr, offset);
        offset += arr.length;
      }
      return { opacity, vertices: merged };
    });
  }, [roads]);

  if (meshes.length === 0) return null;

  return (
    <group>
      {meshes.map(({ opacity, vertices }, i) => (
        <mesh key={i}>
          <bufferGeometry>
            <bufferAttribute
              attach="attributes-position"
              array={vertices}
              count={vertices.length / 3}
              itemSize={3}
            />
          </bufferGeometry>
          <meshBasicMaterial
            color="#ffffff"
            opacity={opacity}
            transparent
            depthWrite={false}
            side={THREE.DoubleSide}
          />
        </mesh>
      ))}
    </group>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add src/components/world3d/ground/RoadNetwork.tsx
git commit -m "feat: create ground/RoadNetwork renderer with ribbon tessellation"
```

---

### Task 9: Create GroundSystem orchestrator

**Files:**
- Create: `src/components/world3d/ground/GroundSystem.tsx`
- Create: `src/components/world3d/ground/index.ts`

- [ ] **Step 1: Create GroundSystem**

```typescript
import { useQuality } from '../../../hooks/useQuality';
import { GroundPlane } from './GroundPlane';
import { GroundGrid } from './GroundGrid';
import { RoadNetwork } from './RoadNetwork';
import type { RoadSegment } from '../../../hooks/useRoadNetwork';

interface GroundSystemProps {
  roads: RoadSegment[];
}

/**
 * Receives roads from parent (WorldScene) to share data with LightingSystem.
 */
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
```

- [ ] **Step 2: Create barrel export**

```typescript
// src/components/world3d/ground/index.ts
export { GroundSystem } from './GroundSystem';
export { GroundPlane } from './GroundPlane';
export { GroundGrid } from './GroundGrid';
export { RoadNetwork } from './RoadNetwork';
```

- [ ] **Step 3: Verify build**

```bash
npx tsc --noEmit
```

- [ ] **Step 4: Commit**

```bash
git add src/components/world3d/ground/
git commit -m "feat: create GroundSystem orchestrator"
```

---

## Chunk 3: Sky System

### Task 10: Create SkyGradient component

**Files:**
- Create: `src/components/world3d/sky/SkyGradient.tsx`

- [ ] **Step 1: Create SkyGradient**

Improved version of `SkyDome` from `Environment.tsx` with smooth gradient:

```typescript
import { useMemo, useEffect } from 'react';
import * as THREE from 'three';
import { useTimeOfDay, type TimePhase } from '../../../hooks/useTimeOfDay';

const SKY_COLORS: Record<TimePhase, { zenith: string; horizon: string }> = {
  night:    { zenith: '#020408', horizon: '#0a0e18' },
  dawn:     { zenith: '#0a0520', horizon: '#2a1535' },
  morning:  { zenith: '#1a3560', horizon: '#4a6590' },
  day:      { zenith: '#3a6a9f', horizon: '#7aa0c5' },
  dusk:     { zenith: '#15102a', horizon: '#6a3520' },
  twilight: { zenith: '#060812', horizon: '#12162a' },
};

interface SkyGradientProps {
  timezone?: string;
}

export function SkyGradient({ timezone = 'UTC' }: SkyGradientProps) {
  const { phase } = useTimeOfDay(timezone);
  const colors = SKY_COLORS[phase];

  const geometry = useMemo(() => {
    const geo = new THREE.SphereGeometry(3000, 32, 16, 0, Math.PI * 2, 0, Math.PI / 2);
    // Apply vertical color gradient via vertex colors
    const positions = geo.attributes.position;
    const colorsArr = new Float32Array(positions.count * 3);
    const zenith = new THREE.Color(colors.zenith);
    const horizon = new THREE.Color(colors.horizon);
    const tmp = new THREE.Color();

    for (let i = 0; i < positions.count; i++) {
      const y = positions.getY(i);
      const t = Math.max(0, y / 3000); // 0 at horizon, 1 at zenith
      tmp.copy(horizon).lerp(zenith, t);
      colorsArr[i * 3] = tmp.r;
      colorsArr[i * 3 + 1] = tmp.g;
      colorsArr[i * 3 + 2] = tmp.b;
    }

    geo.setAttribute('color', new THREE.BufferAttribute(colorsArr, 3));
    return geo;
  }, [colors.zenith, colors.horizon]);

  // Dispose previous geometry on change to prevent GPU memory leak
  useEffect(() => {
    return () => geometry.dispose();
  }, [geometry]);

  return (
    <mesh geometry={geometry} rotation={[0, 0, 0]}>
      <meshBasicMaterial
        vertexColors
        side={THREE.BackSide}
        depthWrite={false}
      />
    </mesh>
  );
}
```

- [ ] **Step 2: Commit**

```bash
mkdir -p src/components/world3d/sky
git add src/components/world3d/sky/SkyGradient.tsx
git commit -m "feat: create sky/SkyGradient with vertex color gradient"
```

---

### Task 11: Create StarField component

**Files:**
- Create: `src/components/world3d/sky/StarField.tsx`

- [ ] **Step 1: Create StarField**

```typescript
import { useMemo } from 'react';
import * as THREE from 'three';
import { useTimeOfDay } from '../../../hooks/useTimeOfDay';

interface StarFieldProps {
  count?: number;
  timezone?: string;
}

export function StarField({ count = 500, timezone = 'UTC' }: StarFieldProps) {
  const { phase } = useTimeOfDay(timezone);

  // Stars visible during night, dawn, twilight
  const opacity = phase === 'night' ? 0.8
    : phase === 'twilight' ? 0.5
    : phase === 'dawn' ? 0.3
    : 0;

  const positions = useMemo(() => {
    const pts = new Float32Array(count * 3);
    const radius = 2800;

    for (let i = 0; i < count; i++) {
      // Random point on upper hemisphere
      const theta = Math.random() * Math.PI * 2;
      const phi = Math.random() * Math.PI * 0.45; // 0 to ~81° from zenith
      const r = radius + (Math.random() - 0.5) * 100;

      pts[i * 3] = r * Math.sin(phi) * Math.cos(theta);
      pts[i * 3 + 1] = r * Math.cos(phi); // y = up
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
```

- [ ] **Step 2: Commit**

```bash
git add src/components/world3d/sky/StarField.tsx
git commit -m "feat: create sky/StarField with time-based visibility"
```

---

### Task 12: Create CelestialBodies component

**Files:**
- Create: `src/components/world3d/sky/CelestialBodies.tsx`

- [ ] **Step 1: Create CelestialBodies**

```typescript
import * as THREE from 'three';
import { useTimeOfDay } from '../../../hooks/useTimeOfDay';

interface CelestialBodiesProps {
  timezone?: string;
}

export function CelestialBodies({ timezone = 'UTC' }: CelestialBodiesProps) {
  const { sunPosition, moonPosition, hour } = useTimeOfDay(timezone);

  const sunVisible = sunPosition.y > -100; // slightly below horizon still shows glow
  const moonVisible = hour >= 19 || hour < 6;

  return (
    <group>
      {/* Sun */}
      {sunVisible && (
        <group position={sunPosition.toArray()}>
          {/* Core disc */}
          <sprite>
            <spriteMaterial
              color="#fff8e0"
              opacity={Math.min(1, sunPosition.y / 200)}
              transparent
              depthWrite={false}
            />
          </sprite>
          {/* Glow */}
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

      {/* Moon */}
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
```

- [ ] **Step 2: Commit**

```bash
git add src/components/world3d/sky/CelestialBodies.tsx
git commit -m "feat: create sky/CelestialBodies with sun glow and moon"
```

---

### Task 13: Create CloudLayer and AtmosphericSky (high preset)

**Files:**
- Create: `src/components/world3d/sky/CloudLayer.tsx`
- Create: `src/components/world3d/sky/AtmosphericSky.tsx`

- [ ] **Step 1: Create CloudLayer**

```typescript
import { useRef, useMemo } from 'react';
import * as THREE from 'three';
import { useFrame } from '@react-three/fiber';

interface CloudLayerProps {
  count?: number;
}

export function CloudLayer({ count = 6 }: CloudLayerProps) {
  const groupRef = useRef<THREE.Group>(null);

  // Slow horizontal drift
  useFrame((_, delta) => {
    if (groupRef.current) {
      groupRef.current.position.x += delta * 0.5;
      // Wrap around
      if (groupRef.current.position.x > 500) {
        groupRef.current.position.x = -500;
      }
    }
  });

  const clouds = useMemo(() => {
    return Array.from({ length: count }, (_, i) => ({
      position: [
        (Math.random() - 0.5) * 1500,
        600 + Math.random() * 300,
        (Math.random() - 0.5) * 1500,
      ] as [number, number, number],
      scale: [
        80 + Math.random() * 120,
        1,
        40 + Math.random() * 60,
      ] as [number, number, number],
      rotation: Math.random() * Math.PI,
      opacity: 0.03 + Math.random() * 0.04,
    }));
  }, [count]);

  return (
    <group ref={groupRef}>
      {clouds.map((cloud, i) => (
        <mesh
          key={i}
          position={cloud.position}
          rotation={[-Math.PI / 2, 0, cloud.rotation]}
          scale={cloud.scale}
        >
          <planeGeometry args={[1, 1]} />
          <meshBasicMaterial
            color="#8899aa"
            opacity={cloud.opacity}
            transparent
            depthWrite={false}
            side={THREE.DoubleSide}
          />
        </mesh>
      ))}
    </group>
  );
}
```

- [ ] **Step 2: Create AtmosphericSky**

```typescript
import { Sky } from '@react-three/drei';
import { useTimeOfDay } from '../../../hooks/useTimeOfDay';

interface AtmosphericSkyProps {
  timezone?: string;
}

/**
 * Physically-based sky using drei <Sky> (Preetham model).
 * Muted parameters for Clean Dark aesthetic.
 * High preset only.
 */
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
```

- [ ] **Step 3: Commit**

```bash
git add src/components/world3d/sky/CloudLayer.tsx src/components/world3d/sky/AtmosphericSky.tsx
git commit -m "feat: create CloudLayer and AtmosphericSky for high preset"
```

---

### Task 14: Create SkySystem orchestrator

**Files:**
- Create: `src/components/world3d/sky/SkySystem.tsx`
- Create: `src/components/world3d/sky/index.ts`

- [ ] **Step 1: Create SkySystem**

```typescript
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

  // High preset: use atmospheric sky instead of gradient
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
```

- [ ] **Step 2: Create barrel export**

```typescript
// src/components/world3d/sky/index.ts
export { SkySystem } from './SkySystem';
export { SkyGradient } from './SkyGradient';
export { StarField } from './StarField';
export { CelestialBodies } from './CelestialBodies';
export { CloudLayer } from './CloudLayer';
export { AtmosphericSky } from './AtmosphericSky';
```

- [ ] **Step 3: Verify build**

```bash
npx tsc --noEmit
```

- [ ] **Step 4: Commit**

```bash
git add src/components/world3d/sky/
git commit -m "feat: create SkySystem orchestrator with preset switching"
```

---

## Chunk 4: Lighting + Post-processing

### Task 15: Create BasicLights component

**Files:**
- Create: `src/components/world3d/lighting/BasicLights.tsx`

- [ ] **Step 1: Create BasicLights**

Improved version of `WorldLights` from `Environment.tsx`:

```typescript
import { useMemo } from 'react';
import * as THREE from 'three';
import { useTimeOfDay } from '../../../hooks/useTimeOfDay';
import { useQuality } from '../../../hooks/useQuality';

interface BasicLightsProps {
  timezone?: string;
}

export function BasicLights({ timezone = 'UTC' }: BasicLightsProps) {
  const { hour, sunPosition, phase, t } = useTimeOfDay(timezone);
  const { lighting } = useQuality();

  const { ambientColor, ambientIntensity, sunColor, sunIntensity } = useMemo(() => {
    // Continuous interpolation based on phase
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
          ambientIntensity: 0.08 + t * 0.12, // 0.08 → 0.20
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
```

- [ ] **Step 2: Commit**

```bash
mkdir -p src/components/world3d/lighting
git add src/components/world3d/lighting/BasicLights.tsx
git commit -m "feat: create lighting/BasicLights with continuous phase interpolation"
```

---

### Task 16: Create NightLights component

**Files:**
- Create: `src/components/world3d/lighting/NightLights.tsx`

- [ ] **Step 1: Create NightLights**

```typescript
import { useMemo } from 'react';
import { useTimeOfDay } from '../../../hooks/useTimeOfDay';
import { useQuality } from '../../../hooks/useQuality';
import type { RoadSegment } from '../../../hooks/useRoadNetwork';

interface NightLightsProps {
  timezone?: string;
  roads: RoadSegment[];
}

/** Sample points along roads to place street lights */
function sampleRoadPositions(roads: RoadSegment[], spacing: number, maxLights: number): [number, number, number][] {
  const positions: [number, number, number][] = [];

  for (const road of roads) {
    // Only light primary/secondary/tertiary roads
    if (!['primary', 'secondary', 'tertiary', 'trunk', 'motorway'].includes(road.highway)) continue;

    for (let i = 0; i < road.points.length - 1 && positions.length < maxLights; i++) {
      const a = road.points[i];
      const b = road.points[i + 1];
      const dx = b.x - a.x;
      const dz = b.z - a.z;
      const segLen = Math.sqrt(dx * dx + dz * dz);
      const steps = Math.floor(segLen / spacing);

      for (let s = 0; s < steps && positions.length < maxLights; s++) {
        const t = s / steps;
        positions.push([a.x + dx * t, 5, a.z + dz * t]);
      }
    }
  }

  return positions;
}

/** Fallback grid when no road data is available */
const FALLBACK_POSITIONS: [number, number, number][] = [
  [30, 5, 0],
  [-30, 5, 0],
  [0, 5, 30],
  [0, 5, -30],
];

export function NightLights({ timezone = 'UTC', roads }: NightLightsProps) {
  const { hour } = useTimeOfDay(timezone);
  const { preset } = useQuality();

  const isNight = hour >= 18 || hour < 6;
  const spacing = preset === 'low' ? 200 : 100;
  const maxLights = preset === 'low' ? 6 : preset === 'medium' ? 15 : 30;

  const positions = useMemo(() => {
    if (roads.length === 0) return FALLBACK_POSITIONS;
    return sampleRoadPositions(roads, spacing, maxLights);
  }, [roads, spacing, maxLights]);

  // Early return AFTER all hooks (Rules of Hooks)
  if (!isNight) return null;

  return (
    <group>
      {positions.map((pos, i) => (
        <pointLight
          key={i}
          position={pos}
          color="#ffaa44"
          intensity={0.3}
          distance={30}
          decay={2}
        />
      ))}
    </group>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add src/components/world3d/lighting/NightLights.tsx
git commit -m "feat: create lighting/NightLights with road-based placement"
```

---

### Task 17: Create LightingSystem orchestrator

**Files:**
- Create: `src/components/world3d/lighting/LightingSystem.tsx`
- Create: `src/components/world3d/lighting/index.ts`

- [ ] **Step 1: Create LightingSystem**

```typescript
import { BasicLights } from './BasicLights';
import { NightLights } from './NightLights';
import type { RoadSegment } from '../../../hooks/useRoadNetwork';

interface LightingSystemProps {
  timezone?: string;
  roads: RoadSegment[];
}

/**
 * Receives roads from parent (WorldScene) to avoid duplicate useRoadNetwork fetch.
 * GroundSystem and LightingSystem share the same road data.
 */
export function LightingSystem({ timezone = 'UTC', roads }: LightingSystemProps) {
  return (
    <group>
      <BasicLights timezone={timezone} />
      <NightLights timezone={timezone} roads={roads} />
    </group>
  );
}
```

- [ ] **Step 2: Create barrel export**

```typescript
// src/components/world3d/lighting/index.ts
export { LightingSystem } from './LightingSystem';
export { BasicLights } from './BasicLights';
export { NightLights } from './NightLights';
```

- [ ] **Step 3: Commit**

```bash
git add src/components/world3d/lighting/
git commit -m "feat: create LightingSystem orchestrator"
```

---

### Task 18: Create PostStack

**Files:**
- Create: `src/components/world3d/post/PostStack.tsx`
- Create: `src/components/world3d/post/index.ts`

- [ ] **Step 1: Create PostStack**

```typescript
import { lazy, Suspense } from 'react';
import { useQuality } from '../../../hooks/useQuality';

// Lazy load to avoid bundling for low preset
const PostEffects = lazy(() => import('./PostEffects'));

export function PostStack() {
  const { post } = useQuality();

  if (post.mode === 'none') return null;

  return (
    <Suspense fallback={null}>
      <PostEffects mode={post.mode} />
    </Suspense>
  );
}
```

- [ ] **Step 2: Create PostEffects (lazy-loaded)**

Create file `src/components/world3d/post/PostEffects.tsx`:

```typescript
import {
  EffectComposer,
  Bloom,
  ToneMapping,
  Vignette,
  SSAO,
} from '@react-three/postprocessing';
import { ToneMappingMode } from 'postprocessing';
import type { PostMode } from '../../../types/world3d';

interface PostEffectsProps {
  mode: PostMode;
}

export default function PostEffects({ mode }: PostEffectsProps) {
  const isFull = mode === 'full';

  return (
    <EffectComposer>
      <SSAO
        radius={0.1}
        intensity={1.5}
        luminanceInfluence={0.5}
      />
      <Bloom
        mipmapBlur
        luminanceThreshold={0.9}
        intensity={0.3}
      />
      <ToneMapping mode={ToneMappingMode.ACES_FILMIC} />
      {isFull && (
        <Vignette offset={0.3} darkness={0.6} />
      )}
    </EffectComposer>
  );
}
```

- [ ] **Step 3: Create barrel export**

```typescript
// src/components/world3d/post/index.ts
export { PostStack } from './PostStack';
```

- [ ] **Step 4: Verify build**

```bash
npx tsc --noEmit
```

- [ ] **Step 5: Commit**

```bash
mkdir -p src/components/world3d/post
git add src/components/world3d/post/
git commit -m "feat: create PostStack with lazy-loaded SSAO/Bloom/ToneMapping"
```

---

## Chunk 5: Integration — WorldScene, BuildingsLayer, Navigation, Cleanup

### Task 19: Refactor WorldScene to use new systems

**Files:**
- Modify: `src/components/world3d/WorldScene.tsx`

- [ ] **Step 1: Update imports**

Replace the Environment imports with new system imports:

```typescript
// Remove:
import { GroundPlane, SkyDome, WorldLights } from './Environment';

// Add:
import { SkySystem } from './sky';
import { GroundSystem } from './ground';
import { LightingSystem } from './lighting';
import { PostStack } from './post';
import { QualityContext } from '../../hooks/useQuality';
import { useRoadNetwork } from '../../hooks/useRoadNetwork';
import { QualityConfig, DEFAULT_QUALITY } from '../../types/world3d';
```

- [ ] **Step 2: Wrap Canvas children in QualityContext.Provider**

Update the component to wrap scene content with QualityContext:

Add `useRoadNetwork` call inside the component (shared between Ground and Lighting systems):

```tsx
const { roads } = useRoadNetwork(activeCamera.lat, activeCamera.lng);
```

Replace the scene content inside `<Canvas>` with:

```tsx
<QualityContext.Provider value={quality}>
  {showStats && <Stats />}

  <SkySystem timezone={timezone} />
  <LightingSystem timezone={timezone} roads={roads} />
  <fog attach="fog" args={[quality.fog.color, quality.fog.near, quality.fog.far]} />

  <Suspense fallback={null}>
    <GroundSystem roads={roads} />

    <BuildingsLayer
      tilesUrl={tilesUrl}
      centerLat={activeCamera.lat}
      centerLng={activeCamera.lng}
      activeCamera={activeCamera}
    />

    <CameraFrustum camera={activeCamera} showVideo={true} />

    {nearbyCameras
      .filter((cam) => cam.id !== activeCamera.id)
      .map((cam) => (
        <CameraMarker3D
          key={cam.id}
          camera={cam}
          centerCamera={activeCamera}
          onClick={() => onCameraSelect(cam)}
        />
      ))}

    {nearbyCameras
      .filter((cam) => cam.id !== activeCamera.id && cam.hlsUrl)
      .slice(0, quality.maxVideoTextures - 1)
      .map((cam) => {
        const { x, z } = geoToLocal(cam.lat, cam.lng, activeCamera.lat, activeCamera.lng);
        return (
          <group key={`frustum-${cam.id}`} position={[x, cam.heightAboveGround, z]}>
            <CameraFrustum camera={cam} showVideo={true} frustumDepth={60} />
          </group>
        );
      })}
  </Suspense>

  <PostStack />

  <NavigationControls
    onExplorationStart={handleExplorationStart}
    isExploring={isExploring}
  />
</QualityContext.Provider>
```

- [ ] **Step 3: Remove old quality prop drilling**

Remove `lodMultiplier` and `wireframeEnabled` from `BuildingsLayer` props — they'll read from context or be unused (no mocks).

- [ ] **Step 4: Verify build**

```bash
npx tsc --noEmit
```

- [ ] **Step 5: Commit**

```bash
git add src/components/world3d/WorldScene.tsx
git commit -m "refactor: WorldScene uses SkySystem/GroundSystem/LightingSystem/PostStack"
```

---

### Task 20: Clean up BuildingsLayer (remove mocks)

**Files:**
- Modify: `src/components/world3d/BuildingsLayer.tsx`

- [ ] **Step 1: Remove mock buildings**

Strip out:
- `seededRandom` function and mock building generation (lines ~120–186)
- `generateMockBuildings` function
- The JSX that renders mock buildings (the `.map()` over mockBuildings)
- Wireframe shader code (vertex/fragment shaders, lines ~190–226)
- `litMaterial` and `wireframeMaterial` useMemo blocks (lines ~47–77)

Keep:
- The component shell (exports, interface)
- The commented-out 3D Tiles Renderer code path
- Return `<group />` (empty) so the component renders nothing

- [ ] **Step 2: Simplify the interface**

Remove `lodMultiplier` and `wireframeEnabled` from `BuildingsLayerProps`. Keep `tilesUrl`, `centerLat`, `centerLng`, `activeCamera`.

- [ ] **Step 3: Verify build**

```bash
npx tsc --noEmit
```

- [ ] **Step 4: Commit**

```bash
git add src/components/world3d/BuildingsLayer.tsx
git commit -m "refactor: remove mock buildings from BuildingsLayer, keep 3D Tiles shell"
```

---

### Task 21: Improve NavigationControls for trackpad

**Files:**
- Modify: `src/components/world3d/NavigationControls.tsx`

- [ ] **Step 1: Add left-click drag for rotation**

In the `onPointerDown` handler (currently checks for `button === 2` or `button === 1`), also allow `button === 0` (left-click) for rotation. This makes trackpad two-finger click-drag work.

Change the condition from:
```typescript
if (e.button === 2 || e.button === 1)
```
to:
```typescript
// Left-click (0), middle-click (1), or right-click (2) all start rotation
if (e.button === 0 || e.button === 1 || e.button === 2)
```

- [ ] **Step 2: Improve wheel/pinch zoom**

The current wheel handler uses `deltaY * 0.1`. On trackpad, pinch gestures emit wheel events with `ctrlKey === true`. Normalize the zoom speed:

```typescript
const onWheel = (e: WheelEvent) => {
  e.preventDefault();
  const isPinch = e.ctrlKey; // trackpad pinch gesture
  const speed = isPinch ? 0.5 : 0.1;
  const delta = -e.deltaY * speed;

  const direction = new THREE.Vector3();
  camera.getWorldDirection(direction);
  camera.position.addScaledVector(direction, delta);

  if (!isExploring) onExplorationStart();
};
```

- [ ] **Step 3: Prevent camera marker click conflicts**

Left-click drag shouldn't trigger camera marker `onClick`. Add a threshold: only count as "click" if mouse moved less than 5px total. Add `dragDistanceRef`:

```typescript
const dragStartRef = useRef<{ x: number; y: number } | null>(null);

const onPointerDown = (e: PointerEvent) => {
  isDraggingRef.current = true;
  dragStartRef.current = { x: e.clientX, y: e.clientY };
  // ... existing logic
};

const onPointerUp = (e: PointerEvent) => {
  isDraggingRef.current = false;
  dragStartRef.current = null;
};
```

The `CameraMarker3D` onClick should check this via a shared ref or by R3F's built-in click vs drag detection (which already works — R3F's `onClick` only fires if pointer didn't move significantly).

- [ ] **Step 4: Verify build**

```bash
npx tsc --noEmit
```

- [ ] **Step 5: Commit**

```bash
git add src/components/world3d/NavigationControls.tsx
git commit -m "feat: improve NavigationControls for trackpad (left-click drag, pinch zoom)"
```

---

### Task 22: Delete Environment.tsx, update index.ts

**Files:**
- Delete: `src/components/world3d/Environment.tsx`
- Modify: `src/components/world3d/index.ts`

- [ ] **Step 1: Delete Environment.tsx**

```bash
rm src/components/world3d/Environment.tsx
```

- [ ] **Step 2: Update barrel exports**

Replace `src/components/world3d/index.ts` content:

```typescript
export { WorldScene } from './WorldScene';
export { BuildingsLayer } from './BuildingsLayer';
export { CameraMarker3D } from './CameraMarker3D';
export { CameraFrustum } from './CameraFrustum';
export { NavigationControls } from './NavigationControls';
export { CameraModel } from './CameraModel';

// Systems
export { SkySystem } from './sky';
export { GroundSystem } from './ground';
export { LightingSystem } from './lighting';
export { PostStack } from './post';
```

- [ ] **Step 3: Check no imports reference Environment.tsx**

```bash
grep -r "from.*Environment" src/ --include="*.ts" --include="*.tsx"
```

Expected: No results (WorldScene.tsx no longer imports from Environment)

- [ ] **Step 4: Verify build**

```bash
npx tsc --noEmit
```

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "cleanup: delete Environment.tsx, update barrel exports"
```

---

### Task 23: Visual verification

- [ ] **Step 1: Start dev server and verify 3D world renders**

Open the 3D world, check:
- Ground plane renders (dark blue `#0a0e14`)
- Sky gradient is visible (smooth zenith→horizon)
- Stars visible if local time is night (for Tokyo timezone)
- Sun/moon sprites visible at correct positions
- Camera frustum with live video still works
- No console errors
- FPS stays at 60

- [ ] **Step 2: Test quality presets**

Temporarily change `DEFAULT_QUALITY` to `QUALITY_PRESETS.low` and verify:
- No grid, no stars, no sun/moon, no post effects
- Basic lighting only

Change to `QUALITY_PRESETS.high` and verify:
- Grid visible, stars, sun/moon, clouds, atmospheric sky, SSAO+bloom

Revert to `medium`.

- [ ] **Step 3: Test trackpad navigation**

On MacBook:
- Left-click drag rotates camera
- Two-finger scroll zooms
- Pinch zooms
- WASD still works
- Space resets

- [ ] **Step 4: Final commit**

```bash
git add -A
git commit -m "verified: 3D world visual polish complete with quality presets"
```
