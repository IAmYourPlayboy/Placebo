import { useTranslation } from "react-i18next";
import SidebarItem from "./SidebarItem";
import Logo from "./Logo";

export default function Sidebar() {
  const { t } = useTranslation();
  return (
    <aside className="shell-sidebar">
      <Logo />

      <nav className="shell-sidebar__top">
        <SidebarItem icon="BellIcon" label={t("sidebar.notifications")} path="/notifications" />
        <SidebarItem icon="UserIcon" label={t("sidebar.profile")} path="/profile" />
      </nav>

      <nav className="shell-sidebar__main">
        <SidebarItem size="lg" icon="HomeIcon" label={t("sidebar.home")} path="/home" />
        <SidebarItem size="lg" icon="PlusIcon" label={t("sidebar.create")} path="/create" />
        <SidebarItem size="lg" icon="GridIcon" label={t("sidebar.categories")} path="/categories" />
        <SidebarItem size="lg" icon="UsersIcon" label={t("sidebar.people")} path="/people" />
      </nav>

      <nav className="shell-sidebar__bottom">
        <SidebarItem icon="ClockIcon" label={t("sidebar.history")} path="/history" />
        <SidebarItem icon="StarIcon" label={t("sidebar.favorites")} path="/favorites" />
        <SidebarItem icon="FolderIcon" label={t("sidebar.folders")} path="/folders" />
      </nav>

      <nav className="shell-sidebar__footer">
        <SidebarItem icon="GearIcon" label={t("sidebar.settings")} path="/settings" />
      </nav>
    </aside>
  );
}
