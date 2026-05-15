import { useScene3D } from "./Scene3DRegistry";

/**
 * Single shared GL context for the whole app. Currently a stub that
 * renders nothing when no scene is active; M4 mounts @react-three/fiber
 * Canvas here and hosts the active scene via portal.
 */
export default function GlobalCanvas() {
  const { activeSceneId } = useScene3D();
  if (!activeSceneId) return null;
  return <div className="global-canvas" data-scene-id={activeSceneId} />;
}
