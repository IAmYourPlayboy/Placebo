import { Suspense, useState, useCallback, useRef } from 'react';
import { Canvas } from '@react-three/fiber';
import { Stats } from '@react-three/drei';
import type { Camera3D, QualityConfig } from '../../types/world3d';
import { DEFAULT_QUALITY, geoToLocal } from '../../types/world3d';
import { BuildingsLayer } from './BuildingsLayer';
import { CameraMarker3D } from './CameraMarker3D';
import { CameraFrustum } from './CameraFrustum';
import { SkySystem } from './sky';
import { GroundSystem } from './ground';
import { LightingSystem } from './lighting';
import { PostStack } from './post';
import { QualityContext } from '../../hooks/useQuality';
import { useRoadNetwork } from '../../hooks/useRoadNetwork';
import { NavigationControls } from './NavigationControls';

interface WorldSceneProps {
  /** Активная камера (та что пользователь смотрит) */
  activeCamera: Camera3D;
  /** Список соседних камер (для маркеров) */
  nearbyCameras: Camera3D[];
  /** Колбэк при клике на маркер соседней камеры */
  onCameraSelect: (camera: Camera3D) => void;
  /** Часовой пояс для освещения (день/ночь) */
  timezone?: string;
  /** Настройки качества */
  quality?: QualityConfig;
  /** URL до 3D Tiles (tileset.json) */
  tilesUrl: string;
  /** Показывать ли FPS-счётчик (dev mode) */
  showStats?: boolean;
}

/**
 * WorldScene — корневой компонент 3D-мира Placebo.
 *
 * Это R3F Canvas который рендерит:
 * - 3D-здания из OSM (через 3D Tiles)
 * - Видеопоток активной камеры на плоскости
 * - Маркеры соседних камер
 * - Wireframe-эффект для зданий вне FOV
 * - Освещение (день/ночь по timezone)
 *
 * По умолчанию камера смотрит прямо на VideoPlane (как обычный плеер).
 * При вращении мышью — открывается 3D-мир вокруг.
 */
export function WorldScene({
  activeCamera,
  nearbyCameras,
  onCameraSelect,
  timezone = 'UTC',
  quality = DEFAULT_QUALITY,
  tilesUrl,
  showStats = false,
}: WorldSceneProps) {
  const [isExploring, setIsExploring] = useState(false);
  const canvasRef = useRef<HTMLCanvasElement>(null);

  // Когда пользователь начинает вращать камеру — переходим в exploration mode
  const handleExplorationStart = useCallback(() => {
    setIsExploring(true);
  }, []);

  const { roads } = useRoadNetwork(activeCamera.lat, activeCamera.lng);

  return (
    <Canvas
      ref={canvasRef}
      camera={{
        fov: 75,
        near: 0.1,
        far: 5000,   // 5км видимость
        position: [0, activeCamera.heightAboveGround, 0],
      }}
      gl={{
        antialias: true,
        alpha: false,
        powerPreference: 'high-performance',
        // Stencil buffer нужен для outline post-processing
        stencil: quality.post.mode !== 'none',
      }}
      dpr={[1, 2]}   // device pixel ratio: min 1, max 2
      style={{
        position: 'absolute',
        top: 0,
        left: 0,
        width: '100%',
        height: '100%',
        background: '#0a0a0f',  // тёмный фон для загрузки
      }}
    >
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
    </Canvas>
  );
}
