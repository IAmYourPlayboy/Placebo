import { useState } from "react";
import { IcBell, IcClock, IcBookmark, IcSearch, IcHeart } from "../components/Icons";

interface HomeProps { onEnterRoom?: () => void; onEnter3D?: () => void; }

const ROOMS = [
  { id: 1, title: "Концерт Шамана",          loc: "г. Тольятти",     viewers: 52,  hot: true,  live: true,  bg: "linear-gradient(140deg,#1a0e2e,#2d1845)" },
  { id: 2, title: "Stand-up Ивана Абрамова",  loc: "Москва",          viewers: 134, hot: true,  live: true,  bg: "linear-gradient(140deg,#0d2035,#1a3a50)" },
  { id: 3, title: "Аниме-марафон",            loc: "Онлайн",          viewers: 38,  hot: false, live: false, bg: "linear-gradient(140deg,#1a0a22,#2d0a3a)" },
  { id: 4, title: "Dune: Part Two",           loc: "Онлайн",          viewers: 217, hot: true,  live: false, bg: "linear-gradient(140deg,#1f1000,#3a2000)" },
  { id: 5, title: "Ночной джаз-сет",          loc: "Санкт-Петербург", viewers: 29,  hot: false, live: true,  bg: "linear-gradient(140deg,#001515,#002a2a)" },
  { id: 6, title: "React Summit 2025",        loc: "Амстердам",       viewers: 408, hot: true,  live: true,  bg: "linear-gradient(140deg,#00153d,#002a6b)" },
];

// World map SVG simplified continent silhouettes
function WorldMapCard({ onClick }: { onClick?: () => void }) {
  return (
    <button className="world-map-card cr-pop" onClick={onClick} aria-label="Онлайн карта мира">
      {/* Ocean background */}
      <div className="world-map-card__ocean" />
      {/* CSS grid of "countries" — stylized dots/blocks */}
      <div className="world-map-card__dots">
        {Array.from({ length: 48 }).map((_, i) => (
          <span key={i} className="wm-dot" style={{ animationDelay: `${(i * 0.12) % 3}s` }} />
        ))}
      </div>
      {/* Pulsing live markers */}
      <div className="wm-marker wm-marker--1" />
      <div className="wm-marker wm-marker--2" />
      <div className="wm-marker wm-marker--3" />
      <div className="wm-marker wm-marker--4" />
      {/* Info overlay */}
      <div className="world-map-card__info">
        <div className="wm-badge">
          <span className="wm-badge__dot" />
          LIVE
        </div>
        <p className="world-map-card__title">🌍 Онлайн карта мира</p>
        <p className="world-map-card__sub">Камеры в реальном времени · 12 430 онлайн</p>
      </div>
    </button>
  );
}

function RoomCard({ room, onClick }: { room: typeof ROOMS[number]; onClick?: () => void }) {
  const [liked, setLiked] = useState(false);
  return (
    <div className="room-card cr-pop" onClick={onClick} style={{ cursor: onClick ? "pointer" : "default" }}>
      <div className="room-card__thumb" style={{ background: room.bg }}>
        <div className="room-card__play">▶</div>
        <div className="room-card__viewers">
          <svg width="11" height="11" viewBox="0 0 24 24" fill="none">
            <path d="M17 21v-2a4 4 0 00-4-4H5a4 4 0 00-4 4v2" stroke="rgba(255,255,255,.8)" strokeWidth="2" />
            <circle cx="9" cy="7" r="4" stroke="rgba(255,255,255,.8)" strokeWidth="2" />
          </svg>
          {room.viewers}
          {room.hot && <span>🔥</span>}
          {room.live && <span className="live-badge">LIVE</span>}
        </div>
        <button
          className="room-card__like"
          onClick={(e) => { e.stopPropagation(); setLiked((l) => !l); }}
        >
          <IcHeart size={14} filled={liked} />
        </button>
      </div>
      <div className="room-card__info">
        <p className="room-card__title">{room.title}</p>
        <p className="room-card__loc"><span className="loc-dot" />{room.loc}</p>
      </div>
    </div>
  );
}

export default function HomeScreen({ onEnterRoom, onEnter3D }: HomeProps) {
  return (
    <div className="screen">
      <header className="topbar">
        <h1 className="topbar__title">Главная</h1>
        <div className="topbar__actions">
          <button className="icon-btn" aria-label="Уведомления">
            <IcBell /><span className="icon-btn__dot" />
          </button>
          <button className="icon-btn" aria-label="История"><IcClock /></button>
          <button className="icon-btn" aria-label="Закладки"><IcBookmark /></button>
          <button className="icon-btn" aria-label="Поиск"><IcSearch /></button>
        </div>
      </header>

      <div className="screen-body">
        {/* Онлайн карта мира */}
        <div className="section">
          <div className="section__header">
            <span className="section__title">Прямо сейчас в мире:</span>
            <button className="section__link">Все →</button>
          </div>
          <WorldMapCard onClick={onEnter3D} />
        </div>

        {/* Популярное */}
        <div className="section">
          <div className="section__header">
            <span className="section__title">Популярное сейчас:</span>
            <button className="section__link">Все →</button>
          </div>
          <div className="cards-grid">
            {ROOMS.map((r) => <RoomCard key={r.id} room={r} onClick={onEnterRoom} />)}
          </div>
        </div>

        <div className="spacer" />
      </div>
    </div>
  );
}
