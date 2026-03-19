import { useState, useCallback } from 'react';
import { WorldScene } from '../components/world3d';
import { useNearbyCameras } from '../hooks/useNearbyCameras';
import type { Camera3D } from '../types/world3d';
import { DEFAULT_ORIENTATION } from '../types/world3d';

interface World3DScreenProps {
  onBack: () => void;
}

// HLS proxy on Vite dev server (yt-dlp + CORS proxy)
function streamUrl(slug: string): string {
  return `/hls-proxy?src=${slug}`;
}

// Стартовая камера – Shibuya Crossing
const INITIAL_CAMERA: Camera3D = {
  id: 'cam_shibuya-crossing',
  name: 'Shibuya Crossing',
  slug: 'shibuya-crossing',
  lat: 35.6595,
  lng: 139.7005,
  category: 'city',
  heightAboveGround: 8,
  orientation: { ...DEFAULT_ORIENTATION, azimuth: 180, elevation: -20, fovHorizontal: 80, fovVertical: 50 },
  hlsUrl: streamUrl('shibuya-crossing'),
  thumbnailUrl: null,
  isOnline: true,
  viewersNow: 42,
};

export default function World3DScreen({ onBack }: World3DScreenProps) {
  const [activeCamera, setActiveCamera] = useState<Camera3D>(INITIAL_CAMERA);
  const { cameras, loading } = useNearbyCameras(activeCamera.lat, activeCamera.lng, 1000);

  const handleCameraSelect = useCallback((cam: Camera3D) => {
    setActiveCamera(cam);
  }, []);

  return (
    <div className="world3d-screen">
      {/* 3D Canvas – full screen */}
      <WorldScene
        activeCamera={activeCamera}
        nearbyCameras={cameras}
        onCameraSelect={handleCameraSelect}
        timezone="Asia/Tokyo"
        showStats={true}
      />

      {/* Overlay UI */}
      <div className="world3d-overlay">
        <div className="world3d-header">
          <button className="world3d-back-btn" onClick={onBack}>
            <svg width={20} height={20} viewBox="0 0 24 24" fill="none">
              <path d="M15 18l-6-6 6-6" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round"/>
            </svg>
            Назад
          </button>

          <div className="world3d-camera-info">
            <div className="world3d-camera-name">{activeCamera.name}</div>
            <div className="world3d-camera-meta">
              {activeCamera.isOnline && <span className="world3d-live">LIVE</span>}
              {cameras.length > 0 && <span>{cameras.length} камер рядом</span>}
            </div>
          </div>
        </div>
      </div>

      {/* Подсказка управления */}
      <div className="world3d-controls-hint">
        <span>ПКМ – вращать</span>
        <span>WASD – двигаться</span>
        <span>Колёсико – zoom</span>
        <span>Пробел – сброс</span>
      </div>

      {loading && (
        <div className="world3d-loading">
          Загрузка камер...
        </div>
      )}
    </div>
  );
}
