import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { useAuth } from "../../auth/useAuth";
import "./home.css";

const MOCK_POPULAR_KEYS = [
  { key: "concert", viewers: 65 },
  { key: "korea_news", viewers: 389 },
  { key: "harry_potter", viewers: 20 },
  { key: "pubg", viewers: 1014 },
  { key: "times_square", viewers: 93 },
  { key: "horror", viewers: 666 },
  { key: "bratishkin", viewers: 14888 },
  { key: "tv_2x2", viewers: 401 },
] as const;

const SIDE_LINKS = [
  "home.side.games",
  "home.side.films",
  "home.side.irl",
  "home.side.brawl",
] as const;

export default function HomeScreen() {
  const { t } = useTranslation();
  const { user } = useAuth();
  const nav = useNavigate();

  const initial = (user?.displayName?.[0] ?? user?.username?.[0] ?? "•").toUpperCase();
  const handle = user?.username ? `@${user.username}` : "";

  return (
    <div className="home">
      <header className="home__head">
        <h2 className="home__title">{t("home.open_rooms")}</h2>
        <button className="home__about" type="button" disabled>
          {t("home.about")}
        </button>
      </header>

      <div className="home__rooms-row" role="list">
        <button
          className="home__create"
          type="button"
          onClick={() => nav("/create")}
          role="listitem"
        >
          <span className="home__create-plus" aria-hidden>
            +
          </span>
          <span>{t("home.create_room")}</span>
        </button>

        <div className="home__avatar" role="listitem">
          <div className="home__avatar-circle">{initial}</div>
          {handle && <span className="home__avatar-handle">{handle}</span>}
        </div>

        {Array.from({ length: 12 }).map((_, i) => (
          <div key={i} className="home__empty-slot" role="listitem" aria-hidden>
            +
          </div>
        ))}
      </div>

      <div className="home__popular-head">
        <h2 className="home__title">{t("home.popular")}</h2>
        <button className="home__dropdown" type="button" disabled>
          {t("home.recommendations")} ▾
        </button>
      </div>

      <div className="home__layout">
        <div className="home__grid">
          {MOCK_POPULAR_KEYS.map(({ key, viewers }) => (
            <button
              key={key}
              className="home__card"
              type="button"
              onClick={() => nav(`/room/${key}`)}
            >
              <div className="home__thumb" />
              <div className="home__card-meta">
                <span className="home__viewers">👥 {viewers.toLocaleString()}</span>
              </div>
              <div className="home__card-title">{t(`home.mock.${key}`)}</div>
            </button>
          ))}
        </div>

        <aside className="home__side">
          {SIDE_LINKS.map((k) => (
            <button key={k} className="home__side-item" type="button" disabled>
              <span>{t(k)}</span>
              <span aria-hidden>▾</span>
            </button>
          ))}
        </aside>
      </div>
    </div>
  );
}
