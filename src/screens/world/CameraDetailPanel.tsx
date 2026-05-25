import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import type { Camera3D } from "../../types/world3d";

type Props = {
  camera: Camera3D;
  nearbyCount: number;
  onClose: () => void;
};

export default function CameraDetailPanel({ camera, nearbyCount, onClose }: Props) {
  const { t } = useTranslation();
  const nav = useNavigate();

  // M5 will mint a real room via API; until then route to the existing
  // CreateScreen with the camera id as a query param.
  const watchTogether = () => nav(`/create?camera=${camera.id}`);

  return (
    <aside className="world-panel">
      <header className="world-panel__head">
        <div>
          <div className="world-panel__title">{camera.name}</div>
          <div className="world-panel__subtitle">
            {camera.isOnline && <span className="world-panel__live">{t("world.live")}</span>}
            <span className="world-panel__category">{camera.category}</span>
            <span className="world-panel__nearby">{t("world.nearby", { count: nearbyCount })}</span>
          </div>
        </div>
        <button
          type="button"
          className="world-panel__close"
          onClick={onClose}
          aria-label={t("world.close")}
        >
          ✕
        </button>
      </header>
      <button type="button" className="world-panel__watch" onClick={watchTogether}>
        {t("world.watch_together")}
      </button>
    </aside>
  );
}
