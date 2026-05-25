import { useEffect, useMemo, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { WorldScene } from "../../components/world3d";
import { useCamerasFromApi } from "../../hooks/useCamerasFromApi";
import { cameraResponseToCamera3D } from "../../api/camera3d";
import CameraDetailPanel from "./CameraDetailPanel";
import "./world.css";

export default function World3DScreen() {
  const { t } = useTranslation();
  const { data, loading, error } = useCamerasFromApi();
  const { id } = useParams();
  const nav = useNavigate();
  const [activeId, setActiveId] = useState<string | null>(null);

  const cameras = useMemo(
    () => (data ?? []).map(cameraResponseToCamera3D),
    [data],
  );

  // Sync URL param (slug or id) with active state. If no param, default to the
  // first camera; preserve any prior selection so simply landing back on /world
  // doesn't reset the user's view.
  useEffect(() => {
    if (!cameras.length) return;
    if (id) {
      const found = cameras.find((c) => c.slug === id || c.id === id);
      if (found) {
        setActiveId(found.id);
        return;
      }
    }
    setActiveId((prev) => prev ?? cameras[0].id);
  }, [cameras, id]);

  if (loading) {
    return <div className="world-screen world-screen--message">{t("world.loading")}</div>;
  }
  if (error) {
    return (
      <div className="world-screen world-screen--message world-screen--error">
        {t("world.error", { msg: error.message })}
      </div>
    );
  }
  if (!cameras.length) {
    return <div className="world-screen world-screen--message">{t("world.empty")}</div>;
  }

  const active = cameras.find((c) => c.id === activeId) ?? cameras[0];

  return (
    <div className="world-screen">
      <WorldScene
        activeCamera={active}
        nearbyCameras={cameras}
        onCameraSelect={(cam) => {
          setActiveId(cam.id);
          nav(`/world/${cam.slug}`, { replace: true });
        }}
        timezone={"UTC"}
      />

      <div className="world-screen__hint">
        <span>{t("world.hint.rotate")}</span>
        <span>{t("world.hint.move")}</span>
        <span>{t("world.hint.zoom")}</span>
        <span>{t("world.hint.reset")}</span>
      </div>

      <CameraDetailPanel
        camera={active}
        nearbyCount={cameras.length}
        onClose={() => nav("/world", { replace: true })}
      />
    </div>
  );
}
