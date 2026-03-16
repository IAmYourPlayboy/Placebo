import { useMemo, useState, useEffect, useRef } from 'react';
import * as THREE from 'three';
import { useFrame } from '@react-three/fiber';
import type { Camera3D } from '../../types/world3d';

interface CameraFrustumProps {
  /** Камера для которой строим конус видимости */
  camera: Camera3D;
  /** Показывать ли видеоплоскость внутри конуса */
  showVideo?: boolean;
  /** Глубина конуса (расстояние от камеры до дальнего плана) */
  frustumDepth?: number;
}

/**
 * CameraFrustum — конус видимости камеры.
 *
 * Визуально: полупрозрачная пирамида от точки камеры до "экрана".
 * Внутри пирамиды — плоскость с видеотекстурой (VideoPlane).
 *
 * Конус строится из fov_horizontal, fov_vertical, azimuth, elevation.
 * Ориентируется в направлении куда смотрит камера.
 */
export function CameraFrustum({
  camera,
  showVideo = true,
  frustumDepth = 80,
}: CameraFrustumProps) {
  const { orientation } = camera;

  // Размер дальней плоскости конуса (куда проецируется видео)
  const farWidth = useMemo(() => {
    const fovRad = (orientation.fovHorizontal * Math.PI) / 180;
    return 2 * frustumDepth * Math.tan(fovRad / 2);
  }, [orientation.fovHorizontal, frustumDepth]);

  const farHeight = useMemo(() => {
    const fovRad = (orientation.fovVertical * Math.PI) / 180;
    return 2 * frustumDepth * Math.tan(fovRad / 2);
  }, [orientation.fovVertical, frustumDepth]);

  // Геометрия конуса видимости (4 линии от камеры к углам)
  const frustumLines = useMemo(() => {
    const hw = farWidth / 2;
    const hh = farHeight / 2;
    const d = frustumDepth;

    // 4 угла дальней плоскости (в локальных координатах камеры)
    const corners = [
      new THREE.Vector3(-hw, hh, -d),   // top-left
      new THREE.Vector3(hw, hh, -d),    // top-right
      new THREE.Vector3(hw, -hh, -d),   // bottom-right
      new THREE.Vector3(-hw, -hh, -d),  // bottom-left
    ];

    const origin = new THREE.Vector3(0, 0, 0);
    const points: number[] = [];

    // Линии от камеры к каждому углу
    for (const corner of corners) {
      points.push(origin.x, origin.y, origin.z);
      points.push(corner.x, corner.y, corner.z);
    }

    // Рамка дальней плоскости
    for (let i = 0; i < 4; i++) {
      const a = corners[i];
      const b = corners[(i + 1) % 4];
      points.push(a.x, a.y, a.z);
      points.push(b.x, b.y, b.z);
    }

    return new Float32Array(points);
  }, [farWidth, farHeight, frustumDepth]);

  // Поворот конуса по azimuth и elevation
  const rotation: [number, number, number] = useMemo(() => {
    const azRad = ((orientation.azimuth - 90) * Math.PI) / 180;  // -90 т.к. Three.js Z-forward
    const elRad = (orientation.elevation * Math.PI) / 180;
    return [elRad, azRad, 0];
  }, [orientation.azimuth, orientation.elevation]);

  return (
    <group rotation={rotation}>
      {/* Линии конуса видимости */}
      <lineSegments>
        <bufferGeometry>
          <bufferAttribute
            attach="attributes-position"
            array={frustumLines}
            count={frustumLines.length / 3}
            itemSize={3}
          />
        </bufferGeometry>
        <lineBasicMaterial
          color="#E8345A"
          opacity={0.25}
          transparent
          linewidth={1}
        />
      </lineSegments>

      {/* Полупрозрачная заливка конуса */}
      <mesh position={[0, 0, -frustumDepth / 2]}>
        <coneGeometry args={[
          Math.max(farWidth, farHeight) / 2,  // radiusBottom
          frustumDepth,                         // height
          4,                                    // radialSegments (пирамида)
        ]} />
        <meshBasicMaterial
          color="#E8345A"
          opacity={0.03}
          transparent
          side={THREE.DoubleSide}
          depthWrite={false}
        />
      </mesh>

      {/* Видео-плоскость на дальнем конце конуса */}
      {showVideo && (
        <VideoPlane
          width={farWidth}
          height={farHeight}
          depth={frustumDepth}
          hlsUrl={camera.hlsUrl}
        />
      )}
    </group>
  );
}

// ─── useManualVideoTexture ───────────────────────────────────────
// Ручной хук: создаёт <video> → THREE.VideoTexture без suspend-react.
// Надёжнее чем drei useVideoTexture в R3F Suspense контексте.

function useManualVideoTexture(src: string | null): THREE.VideoTexture | null {
  const [texture, setTexture] = useState<THREE.VideoTexture | null>(null);
  const videoRef = useRef<HTMLVideoElement | null>(null);
  const hlsRef = useRef<any>(null);
  const abortedRef = useRef(false);

  useEffect(() => {
    abortedRef.current = false;

    if (!src) {
      setTexture(null);
      return;
    }

    const video = document.createElement('video');
    video.crossOrigin = 'anonymous';
    video.muted = true;
    video.loop = true;
    video.playsInline = true;
    video.autoplay = true;
    videoRef.current = video;

    const onReady = () => {
      if (abortedRef.current) return;
      const tex = new THREE.VideoTexture(video);
      tex.minFilter = THREE.LinearFilter;
      tex.magFilter = THREE.LinearFilter;
      tex.colorSpace = THREE.SRGBColorSpace;
      setTexture(tex);
      video.play().catch(() => {});
    };

    // HLS streams (.m3u8) → use hls.js
    const isHls = src.includes('.m3u8');
    if (isHls) {
      import('hls.js').then(({ default: Hls }) => {
        // Guard: if cleanup ran before async import resolved, bail out
        if (abortedRef.current) return;
        if (!Hls.isSupported()) return;

        const hls = new Hls({
          enableWorker: true,
          lowLatencyMode: false,
          liveSyncDurationCount: 3,
          liveMaxLatencyDurationCount: 10,
          fragLoadingMaxRetry: 3,
          fragLoadingRetryDelay: 2000,
          levelLoadingMaxRetry: 3,
          levelLoadingRetryDelay: 2000,
          manifestLoadingMaxRetry: 3,
          maxBufferLength: 10,
          maxMaxBufferLength: 30,
        });
        hlsRef.current = hls;
        hls.loadSource(src);
        hls.attachMedia(video);
        hls.on(Hls.Events.MANIFEST_PARSED, () => {
          if (!abortedRef.current) video.play().catch(() => {});
        });
        hls.on(Hls.Events.ERROR, (_: any, data: any) => {
          if (data.fatal) {
            console.warn('[HLS] fatal:', data.details);
            if (data.type === 'networkError') hls.startLoad();
            else if (data.type === 'mediaError') hls.recoverMediaError();
          }
        });
        video.addEventListener('loadedmetadata', onReady);
      });
    } else {
      video.src = src;
      video.addEventListener('loadedmetadata', onReady);
      video.load();
    }

    return () => {
      abortedRef.current = true;
      if (hlsRef.current) {
        hlsRef.current.destroy();
        hlsRef.current = null;
      }
      video.pause();
      video.removeAttribute('src');
      video.load();
      videoRef.current = null;
      setTexture((prev) => {
        if (prev) prev.dispose();
        return null;
      });
    };
  }, [src]);

  // Обновляем текстуру каждый кадр
  useFrame(() => {
    if (texture && videoRef.current && !videoRef.current.paused) {
      texture.needsUpdate = true;
    }
  });

  return texture;
}

// ─── VideoPlane ─────────────────────────────────────────────────

interface VideoPlaneProps {
  width: number;
  height: number;
  depth: number;
  hlsUrl: string | null;
}

/**
 * VideoPlane — плоскость с видео или серый fallback.
 * Без Suspense – видео загружается асинхронно, показывает fallback пока грузится.
 */
function VideoPlane({ width, height, depth, hlsUrl }: VideoPlaneProps) {
  const texture = useManualVideoTexture(hlsUrl);

  return (
    <mesh position={[0, 0, -depth]}>
      <planeGeometry args={[width, height]} />
      {texture ? (
        <meshBasicMaterial map={texture} toneMapped={false} side={THREE.DoubleSide} />
      ) : (
        <meshBasicMaterial
          color="#1a1a2e"
          opacity={0.9}
          transparent
          side={THREE.DoubleSide}
        />
      )}
    </mesh>
  );
}
