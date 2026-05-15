import {
  createContext, useCallback, useContext, useMemo, useRef, useState,
  type ReactNode,
} from "react";

type RegistryApi = {
  activeSceneId: string | null;
  setActiveScene(id: string | null): void;
  registerSceneState<T>(id: string, state: T): void;
  getSceneState<T>(id: string): T | undefined;
};

const Scene3DContext = createContext<RegistryApi | null>(null);

export function Scene3DRegistry({ children }: { children: ReactNode }) {
  const [activeSceneId, setActiveSceneId] = useState<string | null>(null);
  const statesRef = useRef(new Map<string, unknown>());

  const registerSceneState = useCallback(<T,>(id: string, state: T) => {
    statesRef.current.set(id, state);
  }, []);

  const getSceneState = useCallback(<T,>(id: string): T | undefined => {
    return statesRef.current.get(id) as T | undefined;
  }, []);

  const api = useMemo<RegistryApi>(() => ({
    activeSceneId,
    setActiveScene: setActiveSceneId,
    registerSceneState,
    getSceneState,
  }), [activeSceneId, registerSceneState, getSceneState]);

  return <Scene3DContext.Provider value={api}>{children}</Scene3DContext.Provider>;
}

export function useScene3D() {
  const ctx = useContext(Scene3DContext);
  if (!ctx) throw new Error("useScene3D must be used within <Scene3DRegistry>");
  return ctx;
}
