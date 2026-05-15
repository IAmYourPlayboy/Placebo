import {
  createContext, useCallback, useMemo, useRef, useState, ReactNode,
} from "react";
import { createMemoryRouter } from "react-router-dom";
import { routes } from "../routes";
import type { Tab, TabId, TabManagerApi } from "./types";
import { titleForPath } from "./tabTitles";
import { useTranslation } from "react-i18next";

export const TabContext = createContext<TabManagerApi | null>(null);

function newId(): TabId {
  return crypto.randomUUID?.() ?? Math.random().toString(36).slice(2);
}

function buildTab(path: string, title: string): Tab {
  const router = createMemoryRouter(routes, { initialEntries: [path] });
  return {
    id: newId(),
    title,
    initialPath: path,
    router,
    createdAt: Date.now(),
  };
}

export function TabManager({ children, initialPath = "/home" }: { children: ReactNode; initialPath?: string }) {
  const { t } = useTranslation();
  const initialTab = useRef<Tab>(buildTab(initialPath, titleForPath(initialPath, t)));

  const [tabs, setTabs] = useState<Tab[]>([initialTab.current]);
  const [activeTabId, setActiveTabId] = useState<TabId>(initialTab.current.id);

  const openTab = useCallback<TabManagerApi["openTab"]>(
    (path, title) => {
      const tab = buildTab(path, title ?? titleForPath(path, t));
      setTabs((prev) => [...prev, tab]);
      setActiveTabId(tab.id);
      return tab.id;
    },
    [t],
  );

  const closeTab = useCallback<TabManagerApi["closeTab"]>((id) => {
    setTabs((prev) => {
      if (prev.length <= 1) {
        // Последний таб – вместо закрытия сбросить на /home.
        const fresh = buildTab("/home", titleForPath("/home", t));
        setActiveTabId(fresh.id);
        return [fresh];
      }
      const idx = prev.findIndex((x) => x.id === id);
      if (idx === -1) return prev;
      const next = prev.filter((x) => x.id !== id);
      if (activeTabId === id) {
        const neighbor = next[Math.min(idx, next.length - 1)];
        setActiveTabId(neighbor.id);
      }
      return next;
    });
  }, [activeTabId, t]);

  const activateTab = useCallback<TabManagerApi["activateTab"]>((id) => {
    setActiveTabId(id);
  }, []);

  const renameTab = useCallback<TabManagerApi["renameTab"]>((id, title) => {
    setTabs((prev) => prev.map((x) => (x.id === id ? { ...x, title } : x)));
  }, []);

  const navigateInActiveTab = useCallback<TabManagerApi["navigateInActiveTab"]>((path) => {
    const tab = tabs.find((x) => x.id === activeTabId);
    if (!tab) return;
    tab.router.navigate(path);
    const newTitle = titleForPath(path, t);
    setTabs((prev) => prev.map((x) => (x.id === activeTabId ? { ...x, title: newTitle } : x)));
  }, [tabs, activeTabId, t]);

  const goBack = useCallback<TabManagerApi["goBack"]>(() => {
    const tab = tabs.find((x) => x.id === activeTabId);
    tab?.router.navigate(-1);
  }, [tabs, activeTabId]);

  const goForward = useCallback<TabManagerApi["goForward"]>(() => {
    const tab = tabs.find((x) => x.id === activeTabId);
    tab?.router.navigate(1);
  }, [tabs, activeTabId]);

  const reload = useCallback<TabManagerApi["reload"]>(() => {
    const tab = tabs.find((x) => x.id === activeTabId);
    tab?.router.revalidate();
  }, [tabs, activeTabId]);

  const api = useMemo<TabManagerApi>(() => ({
    tabs, activeTabId,
    openTab, closeTab, activateTab, renameTab,
    navigateInActiveTab, goBack, goForward, reload,
  }), [tabs, activeTabId, openTab, closeTab, activateTab, renameTab, navigateInActiveTab, goBack, goForward, reload]);

  return <TabContext.Provider value={api}>{children}</TabContext.Provider>;
}
