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
 * Used by World3DScreen since M4 to drive the 3D map; the legacy
 * mock-driven `useNearbyCameras` was removed in the same milestone.
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
