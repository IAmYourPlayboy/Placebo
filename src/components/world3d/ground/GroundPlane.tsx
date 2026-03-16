interface GroundPlaneProps {
  radius?: number;
}

export function GroundPlane({ radius = 2000 }: GroundPlaneProps) {
  return (
    <mesh rotation={[-Math.PI / 2, 0, 0]} position={[0, -0.1, 0]} receiveShadow>
      <circleGeometry args={[radius, 64]} />
      <meshStandardMaterial
        color="#141a22"
        roughness={0.95}
        metalness={0}
      />
    </mesh>
  );
}
