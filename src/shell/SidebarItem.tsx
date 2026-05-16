import clsx from "clsx";
import { useTabs } from "./tabs/useTabs";
import { Icon, type IconName } from "../components/ui/Icon";

type Props = {
  icon: IconName;
  label: string;
  path: string;
  size?: "sm" | "md" | "lg";
};

export default function SidebarItem({ icon, label, path, size = "md" }: Props) {
  const { tabs, activeTabId, openTab, activateTab } = useTabs();

  const existing = tabs.find((t) => t.initialPath === path);
  const isActive = !!existing && existing.id === activeTabId;

  const handleClick = () => {
    if (existing) activateTab(existing.id);
    else openTab(path);
  };

  return (
    <button
      className={clsx("sidebar-item", `sidebar-item--${size}`, isActive && "sidebar-item--active")}
      onClick={handleClick}
      aria-current={isActive ? "page" : undefined}
    >
      <Icon name={icon} size={size === "lg" ? 24 : 20} className="sidebar-item__icon" />
      <span className="sidebar-item__label">{label}</span>
    </button>
  );
}
