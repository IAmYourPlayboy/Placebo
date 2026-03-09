import { useState } from "react";
import { IcMoreH, IcHeart, IcComment, IcShare, IcPin, IcVerified } from "../components/Icons";

const POSTS = [
  { id: 1, loc: "Испания",   time: "19 ч",  likes: 42,  comments: "Хорошо выглядишь", bg: "linear-gradient(160deg,#c2a86c,#8c6a30)" },
  { id: 2, loc: "Испания",   time: "2 дн",  likes: 43,  comments: "Nice",             bg: "linear-gradient(160deg,#7090c0,#3050a0)" },
  { id: 3, loc: "Марокко",   time: "5 дн",  likes: 88,  comments: "Обои 🔥",          bg: "linear-gradient(160deg,#d4826a,#a04030)" },
  { id: 4, loc: "Токио",     time: "1 нед", likes: 201, comments: "Завидую",          bg: "linear-gradient(160deg,#202040,#100830)" },
  { id: 5, loc: "Барселона", time: "2 нед", likes: 57,  comments: "💫",               bg: "linear-gradient(160deg,#f0a060,#c06020)" },
  { id: 6, loc: "Бали",      time: "3 нед", likes: 134, comments: "Мечта",            bg: "linear-gradient(160deg,#40b090,#206050)" },
];

const WALL_LEFT  = ["🌊", "❄️", "🌙"];
const WALL_RIGHT = ["🌸", "✨", "🦋"];

function PostCard({ post }: { post: typeof POSTS[number] }) {
  const [liked, setLiked] = useState(false);
  return (
    <div className="post-card cr-pop">
      <div className="post-thumb" style={{ background: post.bg }} />
      <div className="post-footer">
        <div className="post-loc">
          <IcPin size={9} />
          {post.loc}
          <span style={{ marginLeft: "auto", color: "var(--t3)", fontSize: 9 }}>{post.time}</span>
        </div>
        <div className="post-actions">
          <button className="post-act" onClick={() => setLiked((l) => !l)}>
            <IcHeart size={11} filled={liked} />
            <span>{post.likes + (liked ? 1 : 0)}</span>
          </button>
          <button className="post-act">
            <IcComment size={11} />
            <span style={{ maxWidth: 44, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
              {post.comments}
            </span>
          </button>
          <button className="post-act post-act--push">
            <IcShare size={11} />
          </button>
        </div>
      </div>
    </div>
  );
}

function WallpaperStrip({ decos, side }: { decos: string[]; side: "left" | "right" }) {
  const bg = side === "left"
    ? "linear-gradient(180deg, #dbeeff 0%, #a8d4f5 40%, #7ab8e8 80%, #5a9cd4 100%)"
    : "linear-gradient(180deg, #fce4ec 0%, #f8b4c8 40%, #f48fb1 80%, #e91e63 100%)";

  return (
    <div className="wallpaper-strip" style={{ "--ws-bg": bg } as React.CSSProperties}>
      <div className="wallpaper-strip__bg" />
      <div className="wallpaper-strip__art">
        {decos.map((d, i) => (
          <span key={i} className="wallpaper-strip__deco">{d}</span>
        ))}
        <span className="wallpaper-strip__label">Обои</span>
      </div>
      <span className="wallpaper-strip__buy">Купить</span>
    </div>
  );
}

export default function ProfileScreen() {
  const [followed, setFollowed] = useState(false);

  return (
    <div className="screen">
      {/* Topbar */}
      <header className="topbar" style={{ borderBottom: "none" }}>
        <div className="profile-topbar-left">
          {/* Bookmark icon from Figma */}
          <button className="icon-btn" aria-label="Закладки">
            <svg width={22} height={22} viewBox="0 0 24 24" fill="none">
              <path d="M6 3h12a1 1 0 011 1v17l-7-4-7 4V4a1 1 0 011-1z"
                stroke="var(--t1)" strokeWidth={1.75} strokeLinejoin="round"/>
            </svg>
          </button>
        </div>
        <div className="topbar__actions">
          <button className="icon-btn" aria-label="Ещё"><IcMoreH /></button>
        </div>
      </header>

      {/* Profile hero */}
      <div className="profile-hero">
        <div className="profile-avatar-wrap">
          <div className="profile-avatar">С</div>
        </div>
        <div className="profile-meta">
          <div className="profile-name-row">
            <span className="profile-name">Сабрина Крухова</span>
            <IcVerified size={16} />
          </div>
          <div className="profile-bio">Люблю пёсиков 🐾</div>
          <div className="profile-btns">
            <button
              className={`btn-follow${followed ? " btn-follow--active" : ""}`}
              onClick={() => setFollowed((f) => !f)}
            >
              {followed ? "Вы подписаны" : "Добавить в друзья"}
            </button>
            <button className="btn-msg">Написать сообщение</button>
          </div>
        </div>
      </div>

      {/* Links row */}
      <div className="profile-links-row">
        <button className="profile-link">
          <span className="link-star">★</span>
          Обзоры фильмов
          <span className="link-arrow">›</span>
        </button>
        <span className="profile-link-sep" />
        <button className="profile-link">
          Состоит в клубах:
          <span className="link-arrow">›</span>
        </button>
      </div>

      <div className="divider" />

      {/* 3-column layout: wallpaper | posts | wallpaper */}
      <div className="profile-layout">
        <WallpaperStrip decos={WALL_LEFT} side="left" />

        <div className="posts-col">
          <div className="posts-grid">
            {POSTS.map((p) => <PostCard key={p.id} post={p} />)}
          </div>
        </div>

        <WallpaperStrip decos={WALL_RIGHT} side="right" />
      </div>
    </div>
  );
}
