import { useRef, useMemo } from 'react';
import * as THREE from 'three';
import { useFrame } from '@react-three/fiber';

interface CloudLayerProps {
  count?: number;
}

export function CloudLayer({ count = 6 }: CloudLayerProps) {
  const groupRef = useRef<THREE.Group>(null);

  useFrame((_, delta) => {
    if (groupRef.current) {
      groupRef.current.position.x += delta * 0.5;
      if (groupRef.current.position.x > 750) {
        groupRef.current.position.x = -750;
      }
    }
  });

  const clouds = useMemo(() => {
    return Array.from({ length: count }, () => ({
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
