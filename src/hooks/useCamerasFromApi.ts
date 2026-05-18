import { useEffect, useState } from "react";

import { listCameras } from "../api/cameras";
import type { CameraResponse } from "../types/api/CameraResponse";

interface State {
  data: CameraResponse[] | null;
  error: Error | null;
  loading: boolean;
}

/**
 * Fetch the camera roster from the placebo-api on mount.
 *
 * Lives next to the M3-era mock-driven `useNearbyCameras` for now;
 * M4 rewires World3D to use this hook (see plan
 * `docs/superpowers/plans/2026-05-14-milestone-4-home-categories-world.md`).
 */
export function useCamerasFromApi(perPage = 50): State {
  const [state, setState] = useState<State>({
    data: null,
    error: null,
    loading: true,
  });

  useEffect(() => {
    let cancelled = false;
    listCameras({ perPage })
      .then((response) => {
        if (cancelled) return;
        setState({ data: response.data, error: null, loading: false });
      })
      .catch((error: unknown) => {
        if (cancelled) return;
        const e = error instanceof Error ? error : new Error(String(error));
        setState({ data: null, error: e, loading: false });
      });
    return () => {
      cancelled = true;
    };
  }, [perPage]);

  return state;
}
