import {
  createContext, useCallback, useEffect, useMemo, useRef, useState, ReactNode,
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
  // useMemo с пустыми deps – tab/router создаются ровно один раз на инстанс TabManager.
  // eslint-disable-next-line react-hooks/exhaustive-deps
  const initial = useMemo(() => buildTab(initialPath, titleForPath(initialPath, t)), []);

  const [tabs, setTabs] = useState<Tab[]>(() => [initial]);
  const [activeTabId, setActiveTabId] = useState<TabId>(initial.id);

  // Refs для стабильных колбэков (api не пересобирается при каждом изменении tabs/activeTabId).
  const tabsRef = useRef<Tab[]>(tabs);
  useEffect(() => { tabsRef.current = tabs; }, [tabs]);

  const activeTabIdRef = useRef<TabId>(activeTabId);
  useEffect(() => { activeTabIdRef.current = activeTabId; }, [activeTabId]);

  // Освободить все роутеры при размонтировании TabManager (teardown приложения).
  useEffect(() => {
    return () => {
      tabsRef.current.forEach((tab) => tab.router.dispose?.());
    };
  }, []);

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
      const idx = prev.findIndex((x) => x.id === id);
      if (idx === -1) return prev;
      const closed = prev[idx];
      closed.router.dispose?.();

      if (prev.length <= 1) {
        // Последний таб – вместо закрытия сбросить на /home.
        const fresh = buildTab("/home", titleForPath("/home", t));
        // setActiveTabId вызывается ВНЕ updater'а setTabs (через microtask),
        // чтобы избежать побочных эффектов в reducer и сюрпризов в strict mode
        // (двойной вызов updater без двойного setActiveTabId).
        queueMicrotask(() => setActiveTabId(fresh.id));
        return [fresh];
      }

      const next = prev.filter((x) => x.id !== id);
      queueMicrotask(() => {
        setActiveTabId((curActive) => {
          if (curActive !== id) return curActive;
          const neighbor = next[Math.min(idx, next.length - 1)];
          return neighbor.id;
        });
      });
      return next;
    });
  }, [t]);

  const activateTab = useCallback<TabManagerApi["activateTab"]>((id) => {
    setActiveTabId(id);
  }, []);

  const renameTab = useCallback<TabManagerApi["renameTab"]>((id, title) => {
    setTabs((prev) => prev.map((x) => (x.id === id ? { ...x, title } : x)));
  }, []);

  const navigateInActiveTab = useCallback<TabManagerApi["navigateInActiveTab"]>((path) => {
    const curActive = activeTabIdRef.current;
    const tab = tabsRef.current.find((x) => x.id === curActive);
    if (!tab) return;
    tab.router.navigate(path);
    const newTitle = titleForPath(path, t);
    setTabs((prev) => prev.map((x) => (x.id === curActive ? { ...x, title: newTitle } : x)));
  }, [t]);

  const goBack = useCallback<TabManagerApi["goBack"]>(() => {
    const tab = tabsRef.current.find((x) => x.id === activeTabIdRef.current);
    tab?.router.navigate(-1);
  }, []);

  const goForward = useCallback<TabManagerApi["goForward"]>(() => {
    const tab = tabsRef.current.find((x) => x.id === activeTabIdRef.current);
    tab?.router.navigate(1);
  }, []);

  const reload = useCallback<TabManagerApi["reload"]>(() => {
    const tab = tabsRef.current.find((x) => x.id === activeTabIdRef.current);
    tab?.router.revalidate();
  }, []);

  const api = useMemo<TabManagerApi>(() => ({
    tabs, activeTabId,
    openTab, closeTab, activateTab, renameTab,
    navigateInActiveTab, goBack, goForward, reload,
  }), [tabs, activeTabId, openTab, closeTab, activateTab, renameTab, navigateInActiveTab, goBack, goForward, reload]);

  return <TabContext.Provider value={api}>{children}</TabContext.Provider>;
}
