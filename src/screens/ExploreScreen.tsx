import { IcSearch, IcClock, IcBookmark } from "../components/Icons";

interface Category {
  id: number;
  label: string;
  emoji: string;
  bg: string;
  wide?: boolean;
}

const WORLD_CATEGORIES: Category[] = [
  { id: 1, label: "Онлайн карта мира",              emoji: "🌍", bg: "linear-gradient(140deg,#1a6b3a,#2d9e5f)", wide: true },
  { id: 2, label: "Видео-камеры всех уголков мира: В реальном времени", emoji: "📡", bg: "linear-gradient(140deg,#1a3a8f,#2d5fd4)" },
  { id: 3, label: "Посмотреть фильмы с людьми",     emoji: "🎬", bg: "linear-gradient(140deg,#1a6b6b,#2d9e9e)" },
  { id: 4, label: "Стримы в реальной жизни",        emoji: "🎥", bg: "linear-gradient(140deg,#1a4a8f,#2d6fd4)" },
  { id: 5, label: "Послушать радио из разных стран", emoji: "📻", bg: "linear-gradient(140deg,#5a1a8f,#8f2dd4)" },
  { id: 6, label: "Видеоигры",                      emoji: "🎮", bg: "linear-gradient(140deg,#2a0d3d,#5a1a8f)", wide: true },
];

const RELAX_CATEGORIES: Category[] = [
  { id: 7, label: "Телепередачи Сеул/Бостона",  emoji: "📺", bg: "linear-gradient(140deg,#1a6b6b,#2d9e9e)" },
  { id: 8, label: "Подписки на каналы",          emoji: "⭐", bg: "linear-gradient(140deg,#5a1a8f,#8f2dd4)" },
  { id: 9, label: "Клипы",                       emoji: "🎵", bg: "linear-gradient(140deg,#8f1a3a,#d42d5f)" },
];

function CategoryCard({ cat, idx }: { cat: Category; idx: number }) {
  return (
    <button
      className={`cat-card cr-pop${cat.wide ? " cat-card--wide" : ""}`}
      style={{
        background: cat.bg,
        animationDelay: `${idx * 0.06}s`,
      }}
      aria-label={cat.label}
    >
      <span className="cat-card__emoji">{cat.emoji}</span>
      <span className="cat-card__label">{cat.label}</span>
    </button>
  );
}

export default function ExploreScreen() {
  return (
    <div className="screen">
      <header className="topbar">
        <h1 className="topbar__title">Категории</h1>
        <div className="topbar__actions">
          <button className="icon-btn" aria-label="История"><IcClock /></button>
          <button className="icon-btn" aria-label="Закладки"><IcBookmark /></button>
          <button className="icon-btn" aria-label="Поиск"><IcSearch /></button>
        </div>
      </header>

      <div className="screen-body">
        <div className="section">
          <div className="section__header">
            <span className="section__title">Что сейчас происходит в мире:</span>
          </div>
          <div className="cat-grid">
            {WORLD_CATEGORIES.map((c, i) => <CategoryCard key={c.id} cat={c} idx={i} />)}
          </div>
        </div>

        <div className="section">
          <div className="section__header">
            <span className="section__title">Расслабиться и забыться:</span>
          </div>
          <div className="cat-grid">
            {RELAX_CATEGORIES.map((c, i) => <CategoryCard key={c.id} cat={c} idx={i} />)}
          </div>
        </div>

        <div className="spacer" />
      </div>
    </div>
  );
}
