import { useRef } from 'react';
import { useThree } from '@react-three/fiber';
import * as THREE from 'three';
import type { Camera3D } from '../../types/world3d';

// 3D Tiles Renderer (NASA AMMOS)
// import { TilesRenderer } from '3d-tiles-renderer';
// ^ Этот импорт раскомментировать после npm install 3d-tiles-renderer

interface BuildingsLayerProps {
  /** URL до tileset.json (наш сервер или localhost:8090) */
  tilesUrl: string;
  /** Центр мира (координаты активной камеры) */
  centerLat: number;
  centerLng: number;
  /** Активная камера */
  activeCamera: Camera3D;
}

/**
 * BuildingsLayer — загружает 3D Tiles зданий.
 *
 * Использует NASA 3DTilesRendererJS для стриминга тайлов:
 * - Автоматический LOD по расстоянию от камеры
 * - Frustum culling
 * - Lazy loading (грузит только видимые тайлы)
 */
export function BuildingsLayer({
  tilesUrl: _tilesUrl,
  centerLat: _centerLat,
  centerLng: _centerLng,
  activeCamera: _activeCamera,
}: BuildingsLayerProps) {
  const groupRef = useRef<THREE.Group>(null);
  const { camera: _camera, gl: _renderer } = useThree();

  // ─── 3D Tiles Loader ──────────────────────────────────────
  //
  // ВАЖНО: Этот код закомментирован до установки 3d-tiles-renderer.
  // Раскомментировать после: npm install 3d-tiles-renderer
  //
  // useEffect(() => {
  //   if (!groupRef.current) return;
  //
  //   const tilesRenderer = new TilesRenderer(tilesUrl);
  //   tilesRenderer.setCamera(camera);
  //   tilesRenderer.setResolutionFromRenderer(camera, renderer);
  //
  //   // LOD настройка
  //   tilesRenderer.errorTarget = 6;
  //   tilesRenderer.maxDepth = 15;
  //
  //   // Центрируем тайлсет на начале координат
  //   tilesRenderer.addEventListener('load-root-tileset', () => {
  //     const sphere = new THREE.Sphere();
  //     tilesRenderer.getBoundingSphere(sphere);
  //     tilesRenderer.group.position.copy(sphere.center).multiplyScalar(-1);
  //
  //     // Поворачиваем так чтобы Y=вверх (3D Tiles используют Z-up)
  //     tilesRenderer.group.rotation.x = -Math.PI / 2;
  //   });
  //
  //   groupRef.current.add(tilesRenderer.group);
  //
  //   return () => {
  //     tilesRenderer.dispose();
  //     groupRef.current?.remove(tilesRenderer.group);
  //   };
  // }, [tilesUrl, camera, renderer]);
  //
  // useFrame(() => {
  //   camera.updateMatrixWorld();
  //   tilesRenderer?.update();
  // });

  return <group ref={groupRef} />;
}
