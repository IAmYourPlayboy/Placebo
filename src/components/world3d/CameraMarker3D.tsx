import { useRef, useState, useMemo } from 'react';
import { useFrame } from '@react-three/fiber';
import { Html, Billboard } from '@react-three/drei';
import * as THREE from 'three';
import type { Camera3D } from '../../types/world3d';
import { geoToLocal } from '../../types/world3d';
import { CameraModel } from './CameraModel';

interface CameraMarker3DProps {
  /** Данные камеры для отображения */
  camera: Camera3D;
  /** Центральная камера (для вычисления локальных координат) */
  centerCamera: Camera3D;
  /** Колбэк при клике */
  onClick: () => void;
}

/** Цвета категорий камер */
const CATEGORY_COLORS: Record<string, string> = {
  city:     '#E8345A',  // accent Placebo
  traffic:  '#FF9500',
  nature:   '#34C759',
  beach:    '#00C7BE',
  airport:  '#5856D6',
  wildlife: '#AF52DE',
  space:    '#007AFF',
  event:    '#FF2D55',
};

/**
 * CameraMarker3D — маркер соседней камеры в 3D-мире.
 *
 * Визуально: светящаяся сфера на позиции камеры + HTML-лейбл с именем.
 * При наведении: увеличивается + показывается подробная информация.
 * При клике: инициирует перелёт к этой камере.
 */
export function CameraMarker3D({
  camera,
  centerCamera,
  onClick,
}: CameraMarker3DProps) {
  const groupRef = useRef<THREE.Group>(null);
  const [hovered, setHovered] = useState(false);

  // Позиция маркера в локальных координатах
  const { x, z } = geoToLocal(
    camera.lat,
    camera.lng,
    centerCamera.lat,
    centerCamera.lng
  );
  const y = camera.heightAboveGround;

  const color = CATEGORY_COLORS[camera.category] || '#E8345A';

  // Ротация модели камеры по azimuth (объектив смотрит в направлении камеры)
  const cameraRotationY = useMemo(() => {
    return ((camera.orientation.azimuth - 90) * Math.PI) / 180;
  }, [camera.orientation.azimuth]);

  // Пульсация (масштаб осциллирует)
  useFrame(({ clock }) => {
    if (!groupRef.current) return;
    const pulse = 1 + Math.sin(clock.elapsedTime * 2) * 0.1;
    const scale = hovered ? 1.5 : pulse;
    groupRef.current.scale.setScalar(scale);
  });

  return (
    <group position={[x, y, z]}>
      {/* 3D модель security camera */}
      <group
        ref={groupRef}
        rotation={[0, cameraRotationY, 0]}
        onClick={(e) => {
          e.stopPropagation();
          onClick();
        }}
        onPointerEnter={() => {
          setHovered(true);
          document.body.style.cursor = 'pointer';
        }}
        onPointerLeave={() => {
          setHovered(false);
          document.body.style.cursor = 'default';
        }}
      >
        <CameraModel color={color} isOnline={camera.isOnline} />
      </group>

      {/* Вертикальная линия от земли до камеры */}
      <line>
        <bufferGeometry>
          <bufferAttribute
            attach="attributes-position"
            array={new Float32Array([0, 0, 0, 0, -y, 0])}
            count={2}
            itemSize={3}
          />
        </bufferGeometry>
        <lineBasicMaterial color={color} opacity={0.3} transparent />
      </line>

      {/* HTML лейбл */}
      <Billboard follow lockX={false} lockY={false} lockZ={false}>
        <Html
          center
          distanceFactor={200}
          style={{
            pointerEvents: 'none',
            userSelect: 'none',
          }}
        >
          <div
            style={{
              background: hovered ? 'rgba(15,15,15,0.95)' : 'rgba(15,15,15,0.75)',
              color: '#fff',
              padding: hovered ? '8px 14px' : '4px 10px',
              borderRadius: '8px',
              border: `1px solid ${color}`,
              fontFamily: 'Nunito, sans-serif',
              fontSize: hovered ? '13px' : '11px',
              whiteSpace: 'nowrap',
              transition: 'all 0.2s ease',
              boxShadow: hovered ? `0 0 20px ${color}40` : 'none',
            }}
          >
            <div style={{ fontWeight: 700 }}>{camera.name}</div>
            {hovered && (
              <div style={{ fontSize: '10px', color: '#999', marginTop: '2px' }}>
                {camera.isOnline ? '🔴 LIVE' : '⚫ Offline'}
                {camera.viewersNow > 0 && ` · ${camera.viewersNow} зрителей`}
              </div>
            )}
          </div>
        </Html>
      </Billboard>
    </group>
  );
}
