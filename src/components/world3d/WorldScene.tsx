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
import { DynamicFog } from './DynamicFog';
import { QualityContext } from '../../hooks/useQuality';
import { useCityTiles } from '../../hooks/useCityTiles';
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
  /** Показывать ли FPS-счётчик (dev mode) */
  showStats?: boolean;
}

/**
 * WorldScene — корневой компонент 3D-мира Placebo.
 *
 * Рендерит реальные OSM данные (дороги, вода, парки, здания),
 * видеопотоки камер, маркеры и окружение (sky, lighting, fog).
 */
export function WorldScene({
  activeCamera,
  nearbyCameras,
  onCameraSelect,
  timezone = 'UTC',
  quality = DEFAULT_QUALITY,
  showStats = false,
}: WorldSceneProps) {
  const [isExploring, setIsExploring] = useState(false);
  const canvasRef = useRef<HTMLCanvasElement>(null);

  const handleExplorationStart = useCallback(() => {
    setIsExploring(true);
  }, []);

  const { roads, water, parks, buildings } = useCityTiles(
    activeCamera.lat, activeCamera.lng, 16
  );

  return (
    <Canvas
      ref={canvasRef}
      camera={{
        fov: 75,
        near: 0.1,
        far: 5000,
        position: [0, activeCamera.heightAboveGround, 0],
      }}
      gl={{
        antialias: true,
        alpha: false,
        powerPreference: 'high-performance',
        stencil: quality.post.mode !== 'none',
      }}
      dpr={[1, 2]}
      style={{
        position: 'absolute',
        top: 0,
        left: 0,
        width: '100%',
        height: '100%',
        background: '#0a0a0f',
      }}
    >
      <QualityContext.Provider value={quality}>
        {showStats && <Stats />}

        <SkySystem timezone={timezone} />
        <LightingSystem timezone={timezone} roads={roads} />
        <DynamicFog timezone={timezone} near={quality.fog.near} far={quality.fog.far} />

        <Suspense fallback={null}>
          <GroundSystem roads={roads} water={water} parks={parks} />

          <BuildingsLayer buildings={buildings} />

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
