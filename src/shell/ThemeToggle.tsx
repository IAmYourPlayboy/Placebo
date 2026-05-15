import clsx from "clsx";
import { useTheme } from "../theme/useTheme";
import type { ThemeChoice } from "../theme";
import { Icon, type IconName } from "../components/ui/Icon";

const MAP: Array<{ value: ThemeChoice; icon: IconName }> = [
  { value: "dark", icon: "MoonIcon" },
  { value: "auto", icon: "ToggleIcon" },
  { value: "light", icon: "SunIcon" },
];

export default function ThemeToggle() {
  const { choice, setChoice } = useTheme();
  return (
    <div className="theme-toggle" role="radiogroup" aria-label="theme">
      {MAP.map((item) => {
        const active = choice === item.value;
        return (
          <button
            key={item.value}
            className={clsx("theme-toggle__btn", active && "theme-toggle__btn--active")}
            onClick={() => setChoice(item.value)}
            aria-pressed={active}
            aria-label={item.value}
          >
            <Icon name={item.icon} size={18} />
          </button>
        );
      })}
    </div>
  );
}
