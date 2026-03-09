import type { Screen } from "../App";
import { IcHome, IcGrid, IcPlus, IcUsers, IcUser } from "./Icons";

interface Props { active: Screen; onChange: (s: Screen) => void; }

export default function BottomNav({ active, onChange }: Props) {
  return (
    <nav className="bottom-nav">
      <button
        className={`nav-item ${active === "home" ? "nav-item--active" : ""}`}
        onClick={() => onChange("home")}
      >
        <IcHome active={active === "home"} />
        <span className="nav-label">Главная</span>
      </button>

      <button
        className={`nav-item ${active === "explore" ? "nav-item--active" : ""}`}
        onClick={() => onChange("explore")}
      >
        <IcGrid active={active === "explore"} />
        <span className="nav-label">Каталог</span>
      </button>

      <button className="nav-create" onClick={() => onChange("create")} aria-label="Создать">
        <IcPlus />
      </button>

      <button
        className={`nav-item ${active === "friends" ? "nav-item--active" : ""}`}
        onClick={() => onChange("friends")}
      >
        <IcUsers active={active === "friends"} />
        <span className="nav-label">Друзья</span>
      </button>

      <button
        className={`nav-item ${active === "profile" ? "nav-item--active" : ""}`}
        onClick={() => onChange("profile")}
      >
        <IcUser active={active === "profile"} />
        <span className="nav-label">Профиль</span>
      </button>
    </nav>
  );
}
