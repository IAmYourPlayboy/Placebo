export type Scene3DState = {
  /** Stable id for a 3D scene; usually tabId + ":world" */
  id: string;
  /** Arbitrary per-scene state the scene component manages itself. */
  state: Record<string, unknown>;
};
