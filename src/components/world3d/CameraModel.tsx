import { useRef } from 'react';
import * as THREE from 'three';

interface CameraModelProps {
  /** Цвет подсветки (по категории камеры) */
  color: string;
  /** Онлайн ли камера */
  isOnline: boolean;
}

/**
 * CameraModel — процедурная 3D-модель security camera.
 *
 * Построена из примитивов (box, cylinder, torus, sphere).
 * Никаких внешних GLTF — zero dependencies.
 */
export function CameraModel({ color, isOnline }: CameraModelProps) {
  const groupRef = useRef<THREE.Group>(null);

  return (
    <group ref={groupRef}>
      {/* Корпус камеры */}
      <mesh castShadow>
        <boxGeometry args={[1.8, 1.2, 2.4]} />
        <meshStandardMaterial
          color="#2a2a3a"
          roughness={0.7}
          metalness={0.3}
        />
      </mesh>

      {/* Объектив (цилиндр, торчит вперёд по -Z) */}
      <mesh
        position={[0, 0, -1.7]}
        rotation={[Math.PI / 2, 0, 0]}
        castShadow
      >
        <cylinderGeometry args={[0.35, 0.45, 1.0, 16]} />
        <meshStandardMaterial
          color="#1a1a2a"
          roughness={0.4}
          metalness={0.5}
        />
      </mesh>

      {/* Кольцо вокруг объектива */}
      <mesh
        position={[0, 0, -2.2]}
        rotation={[Math.PI / 2, 0, 0]}
      >
        <torusGeometry args={[0.45, 0.06, 8, 24]} />
        <meshStandardMaterial
          color={color}
          emissive={color}
          emissiveIntensity={0.6}
          roughness={0.3}
          metalness={0.6}
        />
      </mesh>

      {/* Стекло объектива (тёмный диск) */}
      <mesh position={[0, 0, -2.25]}>
        <circleGeometry args={[0.35, 16]} />
        <meshStandardMaterial
          color="#050510"
          roughness={0.1}
          metalness={0.9}
        />
      </mesh>

      {/* Крепёжный кронштейн (сверху) */}
      <mesh position={[0, 1.0, 0]}>
        <boxGeometry args={[0.3, 0.8, 0.3]} />
        <meshStandardMaterial
          color="#222233"
          roughness={0.8}
          metalness={0.2}
        />
      </mesh>

      {/* Горизонтальная часть кронштейна */}
      <mesh position={[0, 1.4, 0.4]}>
        <boxGeometry args={[0.3, 0.15, 1.0]} />
        <meshStandardMaterial
          color="#222233"
          roughness={0.8}
          metalness={0.2}
        />
      </mesh>

      {/* LED индикатор */}
      <mesh position={[0.7, 0.5, -1.0]}>
        <sphereGeometry args={[0.08, 8, 8]} />
        <meshStandardMaterial
          color={isOnline ? '#00ff44' : '#ff0033'}
          emissive={isOnline ? '#00ff44' : '#ff0033'}
          emissiveIntensity={isOnline ? 1.5 : 0.8}
          toneMapped={false}
        />
      </mesh>

      {/* Полоска акцентного цвета по корпусу */}
      <mesh position={[0, -0.55, 0]}>
        <boxGeometry args={[1.85, 0.05, 2.45]} />
        <meshStandardMaterial
          color={color}
          emissive={color}
          emissiveIntensity={0.4}
          toneMapped={false}
        />
      </mesh>
    </group>
  );
}
