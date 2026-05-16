import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useTabs } from "./tabs/useTabs";

function labelFor(seg: string, t: (k: string) => string): string {
  switch (seg) {
    case "home": return t("shell.tab.home");
    case "categories": return t("shell.tab.categories");
    case "world": return t("shell.tab.world");
    case "create": return t("shell.tab.create");
    case "people": return t("shell.tab.people");
    case "notifications": return t("shell.tab.notifications");
    case "history": return t("shell.tab.history");
    case "favorites": return t("shell.tab.favorites");
    case "folders": return t("shell.tab.folders");
    case "settings": return t("shell.tab.settings");
    case "profile": return t("shell.tab.profile");
    case "room": return t("shell.tab.room");
    default: return seg;
  }
}

export default function Breadcrumbs() {
  const { tabs, activeTabId, navigateInActiveTab } = useTabs();
  const { t } = useTranslation();
  const active = tabs.find((x) => x.id === activeTabId);
  const [path, setPath] = useState<string>(active?.router.state.location.pathname ?? "/home");

  useEffect(() => {
    if (!active) return;
    setPath(active.router.state.location.pathname);
    const unsub = active.router.subscribe((state) => {
      setPath(state.location.pathname);
    });
    return () => unsub();
  }, [active]);

  const segments = path.replace(/^\/+/, "").split("/").filter(Boolean);
  if (segments.length === 0) return null;

  const crumbs = segments.map((seg, i) => {
    const target = "/" + segments.slice(0, i + 1).join("/");
    return { label: labelFor(seg, t), target, isLast: i === segments.length - 1 };
  });

  return (
    <nav className="shell-breadcrumbs" aria-label="breadcrumb">
      {crumbs.map((c, i) => (
        <span key={i} className="shell-breadcrumbs__crumb">
          {c.isLast ? (
            <span className="shell-breadcrumbs__leaf">{c.label}</span>
          ) : (
            <button onClick={() => navigateInActiveTab(c.target)}>{c.label}</button>
          )}
          {!c.isLast && <span className="shell-breadcrumbs__sep">/</span>}
        </span>
      ))}
    </nav>
  );
}
