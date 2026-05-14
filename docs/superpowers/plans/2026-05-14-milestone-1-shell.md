# Milestone 1: Shell Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Построить полный shell приложения по Figma: сайдбар, верхняя панель, настоящая мульти-таб навигация с per-tab history-stack, breadcrumbs, skeleton-страницы для вкладок без содержимого.

**Architecture:** Центральный `TabManager` держит список табов. Каждый таб содержит свой `MemoryRouter` (react-router-dom v6). Неактивные табы не размонтируются (CSS `hidden` + React keep-alive). Сайдбар и topbar общие на всё приложение и никогда не перемонтируются. Breadcrumbs строятся из текущего route активного таба. 3D-сцены виртуализируются через `Scene3DRegistry` (подготовка; реальные сцены подключаются в M4).

**Tech Stack:** react-router-dom v6.26+ (memory router API), React context, CSS Grid для layout, clsx для условных классов.

**Spec reference:** `docs/superpowers/specs/2026-05-14-alpha-design.md`, разделы 3.1–3.6.

**Зависимость от M0:** `ThemeProvider`, `i18n`, `prefs_*` Tauri IPC должны работать.

**Правила для этого milestone:**
- Содержимое экранов пока заглушечное (реальный контент – M2-M6). Shell даёт "рамку" и правильную навигацию.
- Все UI-строки через `t('...')`.
- Все цвета/отступы через CSS-переменные из `src/theme/variables.css`. Никаких хардкод-цветов.
- Покрываем unit-тестами чистую логику (tab manager, breadcrumb generator), DOM-тесты – только критичные (клик по сайдбару открывает таб).

---

## File Map

### Создаются (shell core)

- `src/shell/ShellRoot.tsx` – корневой компонент shell, рендерит сайдбар + topbar + tab-bar + активный таб.
- `src/shell/ShellLayout.tsx` – CSS-grid layout без бизнес-логики.
- `src/shell/Sidebar.tsx` – левый сайдбар по Figma.
- `src/shell/SidebarItem.tsx` – одна строка сайдбара.
- `src/shell/TopBar.tsx` – верхняя панель (навигация + поиск-placeholder + переключатель темы).
- `src/shell/NavButtons.tsx` – кнопки `< > ↻`.
- `src/shell/ThemeToggle.tsx` – трёхпозиционный переключатель темы.
- `src/shell/TabBar.tsx` – ряд вкладок.
- `src/shell/Tab.tsx` – одна вкладка.
- `src/shell/Breadcrumbs.tsx` – хлебные крошки под табами.
- `src/shell/TabContent.tsx` – контейнер, который рендерит RouterProvider нужного таба.
- `src/shell/Logo.tsx` – логотип "Placebo" в левом верхнем углу.

### Создаются (tab manager)

- `src/shell/tabs/TabManager.tsx` – провайдер контекста табов.
- `src/shell/tabs/useTabs.ts` – хук доступа.
- `src/shell/tabs/types.ts` – типы `Tab`, `TabState`, `TabRoute`.
- `src/shell/tabs/tabTitles.ts` – маппинг path → заголовок таба (функция).
- `src/shell/tabs/TabManager.test.ts` – unit-тесты логики менеджера.

### Создаются (маршруты и скелеты)

- `src/shell/routes.tsx` – единый список маршрутов, которые может открывать любой таб.
- `src/screens/main/HomePlaceholder.tsx` – заглушка на M1 (настоящая Home – M4).
- `src/screens/skeletons/NotificationsScreen.tsx` – "Пока нет уведомлений".
- `src/screens/skeletons/HistoryScreen.tsx`
- `src/screens/skeletons/FavoritesScreen.tsx`
- `src/screens/skeletons/FoldersScreen.tsx`
- `src/screens/skeletons/PeopleScreen.tsx`
- `src/screens/settings/SettingsScreen.tsx` – минимальный (тема + язык + "выйти" disabled до M2).
- `src/screens/profile/ProfilePlaceholder.tsx` – M6 перепишет.

### Создаются (иконки и UI)

- `src/components/ui/Icon.tsx` – тонкая обёртка над SVG-иконками.
- `src/components/ui/Button.tsx` – базовая кнопка.
- `src/components/Icons.tsx` (модифицируем) – добавить недостающие иконки для сайдбара.

### Создаются (3D-виртуализация, подготовка)

- `src/shell/scene3d/Scene3DRegistry.tsx` – регистр активной сцены (контекст).
- `src/shell/scene3d/GlobalCanvas.tsx` – единственный `<Canvas>` в приложении.
- `src/shell/scene3d/types.ts` – интерфейс `Scene3DState`.

В M1 GlobalCanvas пустой. В M4 World3D-сцена подключается через `useScene3D`.

### Модифицируются

- `src/App.tsx` – становится тонкой обёрткой: `<ThemeProvider><I18nProvider><TabManager><ShellRoot /></TabManager></ThemeProvider>`.
- `src/App.css` – layout shell'а (grid), переменные не трогаем (они в theme/variables.css).
- `src/main.tsx` – без изменений (все провайдеры уже в App).
- `src/i18n/locales/ru.json` – добавить ключи для сайдбара, breadcrumbs, settings.
- `package.json` – добавить `react-router-dom`, `clsx`.

### Удаляются

В M1 **НИЧЕГО не удаляем из `src/screens/*Screen.tsx`**. Старые экраны продолжают работать как routes. Они переписываются в своих milestones (M2: auth; M4: home/categories/world; M5: room; M6: profile/create/friends).

---

## Task 1: Ветка и зависимости

- [ ] **Step 1: Новая ветка от main (или от merged M0)**

```bash
git -C d:/Projects/Placebo checkout main
git -C d:/Projects/Placebo pull
git -C d:/Projects/Placebo checkout -b feat/m1-shell
```

- [ ] **Step 2: Установить зависимости**

```bash
cd d:/Projects/Placebo
npm install react-router-dom clsx
```

Expected: `react-router-dom` версии 6.26+ (у нас v6, не v7 – они ломали API). `clsx` ^2.

- [ ] **Step 3: Проверить, что ничего не сломалось**

```bash
npm run dev
```

- [ ] **Step 4: Commit**

```bash
git add package.json package-lock.json
git commit -m "chore(deps): add react-router-dom and clsx for shell"
```

---

## Task 2: Типы таба и TabManager контекст

**Files:**
- Create: `src/shell/tabs/types.ts`
- Create: `src/shell/tabs/tabTitles.ts`
- Create: `src/shell/tabs/TabManager.tsx`
- Create: `src/shell/tabs/useTabs.ts`
- Create: `src/shell/tabs/TabManager.test.ts`

- [ ] **Step 1: Определить типы**

Создать `src/shell/tabs/types.ts`:

```ts
import type { Router } from "@remix-run/router";

export type TabId = string;

export type Tab = {
  id: TabId;
  title: string;
  /** Path при создании таба; текущий путь узнаётся через router.state.location. */
  initialPath: string;
  /** Per-tab memory router. Управляет per-tab историей. */
  router: Router;
  /** Момент создания (Date.now()) – используется для стабильной сортировки. */
  createdAt: number;
};

export type TabManagerApi = {
  tabs: Tab[];
  activeTabId: TabId;
  openTab(path: string, title?: string): TabId;
  closeTab(id: TabId): void;
  activateTab(id: TabId): void;
  renameTab(id: TabId, title: string): void;
  /** Навигация внутри активного таба по path. Создаёт новую запись в history. */
  navigateInActiveTab(path: string): void;
  /** Кнопки <, >, ↻ */
  goBack(): void;
  goForward(): void;
  reload(): void;
};
```

- [ ] **Step 2: Маппинг path → заголовок таба**

Создать `src/shell/tabs/tabTitles.ts`:

```ts
/**
 * Given a path, return a human-readable tab title.
 * Uses the first segment for section names; deeper paths
 * can be made more specific later (e.g. /room/:id -> room name).
 */
export function titleForPath(path: string, tFn: (key: string) => string): string {
  const clean = path.split("?")[0].split("#")[0].replace(/^\/+/, "");
  const [head, ...rest] = clean.split("/");
  switch (head) {
    case "":
    case "home":
      return tFn("shell.tab.home");
    case "categories":
      return tFn("shell.tab.categories");
    case "world":
      return tFn("shell.tab.world");
    case "create":
      return tFn("shell.tab.create");
    case "people":
      return tFn("shell.tab.people");
    case "notifications":
      return tFn("shell.tab.notifications");
    case "history":
      return tFn("shell.tab.history");
    case "favorites":
      return tFn("shell.tab.favorites");
    case "folders":
      return tFn("shell.tab.folders");
    case "settings":
      return tFn("shell.tab.settings");
    case "profile":
      // path: /profile/:username -> show username if present
      return rest[0] ? `@${rest[0]}` : tFn("shell.tab.profile");
    case "room":
      return rest[0] ? tFn("shell.tab.room") + " " + rest[0].slice(0, 6) : tFn("shell.tab.room");
    case "welcome":
    case "login":
    case "register":
      return tFn("shell.tab.auth");
    default:
      return clean || tFn("shell.tab.home");
  }
}
```

- [ ] **Step 3: TabManager контекст-провайдер**

Создать `src/shell/tabs/TabManager.tsx`:

```tsx
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
  return crypto.randomUUID();
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
```

- [ ] **Step 4: Хук useTabs**

Создать `src/shell/tabs/useTabs.ts`:

```ts
import { useContext } from "react";
import { TabContext } from "./TabManager";

export function useTabs() {
  const ctx = useContext(TabContext);
  if (!ctx) throw new Error("useTabs must be used within <TabManager>");
  return ctx;
}
```

- [ ] **Step 5: Юнит-тесты для чистой логики**

Мы тестируем `titleForPath` как чистую функцию (не требует React):

Создать `src/shell/tabs/TabManager.test.ts`:

```ts
import { describe, it, expect } from "vitest";
import { titleForPath } from "./tabTitles";

const t = (k: string) => k; // identity translator for tests

describe("titleForPath", () => {
  it("returns home title for /home", () => {
    expect(titleForPath("/home", t)).toBe("shell.tab.home");
  });

  it("returns home title for empty/root", () => {
    expect(titleForPath("/", t)).toBe("shell.tab.home");
    expect(titleForPath("", t)).toBe("shell.tab.home");
  });

  it("returns profile title with @username when username present", () => {
    expect(titleForPath("/profile/zara", t)).toBe("@zara");
  });

  it("returns generic profile title when no username", () => {
    expect(titleForPath("/profile", t)).toBe("shell.tab.profile");
  });

  it("strips query and hash", () => {
    expect(titleForPath("/categories?foo=bar#x", t)).toBe("shell.tab.categories");
  });

  it("room title shows short id", () => {
    const out = titleForPath("/room/abc12345-xxxx", t);
    expect(out).toContain("abc123");
  });
});
```

- [ ] **Step 6: Добавить vitest в dev-deps и конфиг**

```bash
cd d:/Projects/Placebo
npm install -D vitest @testing-library/react @testing-library/dom @testing-library/user-event jsdom
```

Создать `vitest.config.ts`:

```ts
import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["./src/test-setup.ts"],
  },
});
```

Создать `src/test-setup.ts`:

```ts
import "@testing-library/jest-dom/vitest";
```

Добавить в `package.json` скрипт:

```json
"test": "vitest run",
"test:watch": "vitest"
```

- [ ] **Step 7: Запустить тесты**

```bash
npm test -- src/shell/tabs/TabManager.test.ts
```

Expected: 6 тестов проходят.

- [ ] **Step 8: Commit**

```bash
git add src/shell/tabs/ vitest.config.ts src/test-setup.ts package.json package-lock.json
git commit -m "$(cat <<'EOF'
feat(shell): tab manager with per-tab memory router

- Each tab holds its own createMemoryRouter instance for isolated
  history stacks (back/forward work per-tab, not globally).
- Tab titles derive from the current path via titleForPath().
- Pure functions covered by vitest; DOM tests come once we have
  real screens wired up.
- Closing the last tab replaces it with a fresh /home tab instead
  of leaving the shell empty.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: Маршруты (routes.tsx) и skeleton-экраны

**Files:**
- Create: `src/shell/routes.tsx`
- Create: `src/screens/main/HomePlaceholder.tsx`
- Create: `src/screens/skeletons/NotificationsScreen.tsx`
- Create: `src/screens/skeletons/HistoryScreen.tsx`
- Create: `src/screens/skeletons/FavoritesScreen.tsx`
- Create: `src/screens/skeletons/FoldersScreen.tsx`
- Create: `src/screens/skeletons/PeopleScreen.tsx`
- Create: `src/screens/settings/SettingsScreen.tsx`
- Create: `src/screens/profile/ProfilePlaceholder.tsx`

- [ ] **Step 1: Простой переиспользуемый skeleton-компонент**

Создать `src/screens/skeletons/EmptySection.tsx`:

```tsx
import { ReactNode } from "react";

type Props = {
  title: string;
  hint?: ReactNode;
};

export default function EmptySection({ title, hint }: Props) {
  return (
    <div className="empty-section">
      <div className="empty-section__title">{title}</div>
      {hint && <div className="empty-section__hint">{hint}</div>}
    </div>
  );
}
```

Добавить в `src/App.css`:

```css
.empty-section {
  padding: 48px 32px;
  color: var(--t2);
  text-align: center;
}
.empty-section__title { font-size: 18px; font-weight: 600; color: var(--t1); margin-bottom: 8px; }
.empty-section__hint { font-size: 14px; color: var(--t3); }
```

- [ ] **Step 2: Skeleton-экраны**

Создать `src/screens/skeletons/NotificationsScreen.tsx`:

```tsx
import { useTranslation } from "react-i18next";
import EmptySection from "./EmptySection";

export default function NotificationsScreen() {
  const { t } = useTranslation();
  return <EmptySection title={t("notifications.empty.title")} hint={t("notifications.empty.hint")} />;
}
```

Аналогично создать:

- `src/screens/skeletons/HistoryScreen.tsx` (ключи `history.empty.*`)
- `src/screens/skeletons/FavoritesScreen.tsx` (ключи `favorites.empty.*`)
- `src/screens/skeletons/FoldersScreen.tsx` (ключи `folders.empty.*`)
- `src/screens/skeletons/PeopleScreen.tsx` (ключи `people.empty.*`)

Каждый файл – 6 строк, без лишнего.

- [ ] **Step 3: Заглушка Home и Profile**

`src/screens/main/HomePlaceholder.tsx`:

```tsx
import { useTranslation } from "react-i18next";

export default function HomePlaceholder() {
  const { t } = useTranslation();
  return (
    <div style={{ padding: 32 }}>
      <h1>{t("shell.tab.home")}</h1>
      <p>{t("home.placeholder.hint")}</p>
    </div>
  );
}
```

`src/screens/profile/ProfilePlaceholder.tsx`:

```tsx
import { useTranslation } from "react-i18next";
import { useParams } from "react-router-dom";

export default function ProfilePlaceholder() {
  const { t } = useTranslation();
  const { username } = useParams();
  return (
    <div style={{ padding: 32 }}>
      <h1>{username ? `@${username}` : t("shell.tab.profile")}</h1>
      <p>{t("profile.placeholder.hint")}</p>
    </div>
  );
}
```

- [ ] **Step 4: Settings (минимальный)**

`src/screens/settings/SettingsScreen.tsx`:

```tsx
import { useTranslation } from "react-i18next";
import { useTheme } from "../../theme/useTheme";
import type { ThemeChoice } from "../../theme";

const CHOICES: ThemeChoice[] = ["light", "auto", "dark"];

export default function SettingsScreen() {
  const { t, i18n } = useTranslation();
  const { choice, setChoice } = useTheme();

  return (
    <div className="settings">
      <h1>{t("settings.title")}</h1>

      <section className="settings__group">
        <h2>{t("settings.theme.title")}</h2>
        <div className="settings__row">
          {CHOICES.map((c) => (
            <button
              key={c}
              className={"settings__chip" + (choice === c ? " settings__chip--active" : "")}
              onClick={() => setChoice(c)}
            >
              {t(`settings.theme.${c}`)}
            </button>
          ))}
        </div>
      </section>

      <section className="settings__group">
        <h2>{t("settings.language.title")}</h2>
        <div className="settings__row">
          {["ru", "en"].map((lng) => (
            <button
              key={lng}
              className={"settings__chip" + (i18n.resolvedLanguage === lng ? " settings__chip--active" : "")}
              onClick={() => i18n.changeLanguage(lng)}
            >
              {t(`settings.language.${lng}`)}
            </button>
          ))}
        </div>
      </section>

      <section className="settings__group">
        <h2>{t("settings.account.title")}</h2>
        <button className="settings__danger" disabled>
          {t("settings.account.logout")}
        </button>
        <p className="settings__hint">{t("settings.account.logout.hint")}</p>
      </section>
    </div>
  );
}
```

Стили добавить в `src/App.css`:

```css
.settings { padding: 24px 32px; max-width: 720px; }
.settings h1 { font-size: 24px; margin-bottom: 24px; }
.settings__group { margin-bottom: 32px; }
.settings__group h2 { font-size: 14px; color: var(--t3); text-transform: uppercase; letter-spacing: 0.04em; margin-bottom: 12px; }
.settings__row { display: flex; gap: 8px; flex-wrap: wrap; }
.settings__chip {
  padding: 8px 16px; border-radius: 999px;
  border: 1px solid var(--border); background: var(--bg);
  color: var(--t1); cursor: pointer; font-size: 14px;
}
.settings__chip--active { background: var(--t1); color: var(--bg); border-color: var(--t1); }
.settings__danger {
  padding: 10px 20px; border-radius: 10px;
  border: 1px solid var(--border); background: var(--bg);
  color: var(--t3); cursor: not-allowed; font-size: 14px;
}
.settings__hint { font-size: 13px; color: var(--t3); margin-top: 8px; }
```

- [ ] **Step 5: Собрать routes.tsx**

Создать `src/shell/routes.tsx`:

```tsx
import { RouteObject, Navigate } from "react-router-dom";
import HomePlaceholder from "../screens/main/HomePlaceholder";
import NotificationsScreen from "../screens/skeletons/NotificationsScreen";
import HistoryScreen from "../screens/skeletons/HistoryScreen";
import FavoritesScreen from "../screens/skeletons/FavoritesScreen";
import FoldersScreen from "../screens/skeletons/FoldersScreen";
import PeopleScreen from "../screens/skeletons/PeopleScreen";
import SettingsScreen from "../screens/settings/SettingsScreen";
import ProfilePlaceholder from "../screens/profile/ProfilePlaceholder";

// Старые экраны-прототипы: продолжают работать до момента своего переписывания.
import ExploreScreen from "../screens/ExploreScreen";
import CreateScreen from "../screens/CreateScreen";
import WatchRoomScreen from "../screens/WatchRoomScreen";
import World3DScreen from "../screens/World3DScreen";

export const routes: RouteObject[] = [
  { path: "/", element: <Navigate to="/home" replace /> },
  { path: "/home", element: <HomePlaceholder /> },
  { path: "/categories", element: <ExploreScreen /> },
  { path: "/create", element: <CreateScreen /> },
  { path: "/people", element: <PeopleScreen /> },
  { path: "/notifications", element: <NotificationsScreen /> },
  { path: "/history", element: <HistoryScreen /> },
  { path: "/favorites", element: <FavoritesScreen /> },
  { path: "/folders", element: <FoldersScreen /> },
  { path: "/settings", element: <SettingsScreen /> },
  { path: "/profile", element: <ProfilePlaceholder /> },
  { path: "/profile/:username", element: <ProfilePlaceholder /> },
  { path: "/room/:id", element: <WatchRoomScreen onBack={() => window.history.back()} /> },
  { path: "/world", element: <World3DScreen onBack={() => window.history.back()} /> },
  { path: "*", element: <Navigate to="/home" replace /> },
];
```

**Примечание про `onBack`:** старые компоненты `WatchRoomScreen`, `World3DScreen` ожидают `onBack` пропс. Временный костыль `window.history.back()` работает, но при переписывании этих экранов (M4, M5) мы уберём проп и перейдём на `useNavigate(-1)`.

- [ ] **Step 6: Пополнить ru.json**

Добавить в `src/i18n/locales/ru.json`:

```json
{
  "app.loading": "Загрузка...",
  "app.error.generic": "Что-то пошло не так",

  "shell.tab.home": "Главная",
  "shell.tab.categories": "Категории",
  "shell.tab.world": "Онлайн карта мира",
  "shell.tab.create": "Создать",
  "shell.tab.people": "Люди",
  "shell.tab.notifications": "Уведомления",
  "shell.tab.history": "История",
  "shell.tab.favorites": "Избранное",
  "shell.tab.folders": "Папки",
  "shell.tab.settings": "Настройки",
  "shell.tab.profile": "Профиль",
  "shell.tab.room": "Комната",
  "shell.tab.auth": "Вход",

  "sidebar.notifications": "Уведомления",
  "sidebar.profile": "Профиль",
  "sidebar.home": "Главная",
  "sidebar.create": "Создать",
  "sidebar.categories": "Категории",
  "sidebar.people": "Люди",
  "sidebar.history": "История",
  "sidebar.favorites": "Избранное",
  "sidebar.folders": "Папки",
  "sidebar.settings": "Настройки",

  "topbar.search.placeholder": "Поиск",
  "topbar.back": "Назад",
  "topbar.forward": "Вперёд",
  "topbar.reload": "Обновить",

  "tabbar.new": "Новая вкладка",
  "tabbar.close": "Закрыть",

  "notifications.empty.title": "Пока нет уведомлений",
  "notifications.empty.hint": "Здесь появятся события от друзей и камер.",
  "history.empty.title": "История пуста",
  "history.empty.hint": "Скоро появится: что ты смотрел и когда.",
  "favorites.empty.title": "Избранное пусто",
  "favorites.empty.hint": "Добавляй камеры и комнаты в избранное, чтобы быстро возвращаться.",
  "folders.empty.title": "Папок пока нет",
  "folders.empty.hint": "Группируй камеры в папки для удобства.",
  "people.empty.title": "Список пуст",
  "people.empty.hint": "Добавляй друзей по @юзернейму.",

  "home.placeholder.hint": "Настоящая главная появится в следующем milestone.",
  "profile.placeholder.hint": "Настоящий профиль появится в следующем milestone.",

  "settings.title": "Настройки",
  "settings.theme.title": "Тема",
  "settings.theme.light": "Светлая",
  "settings.theme.dark": "Тёмная",
  "settings.theme.auto": "Авто",
  "settings.language.title": "Язык",
  "settings.language.ru": "Русский",
  "settings.language.en": "English",
  "settings.account.title": "Аккаунт",
  "settings.account.logout": "Выйти",
  "settings.account.logout.hint": "Появится после входа в аккаунт."
}
```

- [ ] **Step 7: Сборка**

```bash
npm run dev
```

Expected: в консоли нет ошибок. Пока страница пустая (shell не смонтирован), это ок.

- [ ] **Step 8: Commit**

```bash
git add src/screens/ src/shell/routes.tsx src/i18n/locales/ru.json src/App.css
git commit -m "$(cat <<'EOF'
feat(shell): skeleton screens, settings screen, route table

- EmptySection component for reusable empty states.
- Skeleton screens: Notifications, History, Favorites, Folders, People.
- Settings screen with theme and language pickers (logout disabled
  until M2 wires auth).
- HomePlaceholder and ProfilePlaceholder — real versions land in M4/M6.
- routes.tsx wires all paths to components; legacy screens keep their
  current behaviour until their own milestone rewrites them.
- ru.json enriched with sidebar/topbar/tabbar/settings keys.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: Иконки сайдбара

**Files:**
- Modify: `src/components/Icons.tsx`
- Create: `src/components/ui/Icon.tsx` (тонкая обёртка для использования по имени)

- [ ] **Step 1: Проверить существующие иконки**

```bash
grep -n "export" d:/Projects/Placebo/src/components/Icons.tsx | head -40
```

Expected: увидим какие уже есть. Нужны: `BellIcon` (Уведомления), `UserIcon` (Профиль), `HomeIcon`, `PlusIcon` (Создать), `GridIcon` (Категории), `UsersIcon` (Люди), `ClockIcon` (История), `StarIcon` (Избранное), `FolderIcon` (Папки), `GearIcon` (Настройки), `ArrowLeftIcon`, `ArrowRightIcon`, `RefreshIcon`, `SearchIcon`, `MoonIcon`, `SunIcon`, `ToggleIcon` (средняя позиция "auto"), `CloseIcon` (крестик на табе), `PlusSmallIcon` (+ на таб-баре).

- [ ] **Step 2: Добавить недостающие в Icons.tsx**

Дописать в `src/components/Icons.tsx` те SVG-компоненты, которых нет. Формат:

```tsx
export function HomeIcon(props: React.SVGProps<SVGSVGElement>) {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" {...props}>
      <path d="M3 10l9-7 9 7v11a1 1 0 0 1-1 1h-5v-7h-6v7H4a1 1 0 0 1-1-1V10z"/>
    </svg>
  );
}
```

Весь список иконок – в одном подкоммите. Стиль SF-style stroke (1.5), `currentColor`, без заливки.

- [ ] **Step 3: Обёртка по имени**

Создать `src/components/ui/Icon.tsx`:

```tsx
import * as Icons from "../Icons";
import { SVGProps } from "react";

type IconName = keyof typeof Icons;

type Props = SVGProps<SVGSVGElement> & {
  name: IconName;
  size?: number;
};

export function Icon({ name, size = 20, style, ...rest }: Props) {
  const Component = Icons[name] as (p: SVGProps<SVGSVGElement>) => JSX.Element;
  return <Component width={size} height={size} style={style} {...rest} />;
}
```

- [ ] **Step 4: Проверить, что все имена существуют**

```bash
npm run dev
```

В DevTools Console:

```js
// временно в каком-нибудь экране:
// <Icon name="HomeIcon" />
```

Expected: иконка отображается без TS-ошибок.

- [ ] **Step 5: Commit**

```bash
git add src/components/
git commit -m "feat(icons): add sidebar/topbar/tabbar icons + Icon name wrapper"
```

---

## Task 5: Sidebar и SidebarItem

**Files:**
- Create: `src/shell/Sidebar.tsx`
- Create: `src/shell/SidebarItem.tsx`
- Create: `src/shell/Logo.tsx`

- [ ] **Step 1: Logo**

Создать `src/shell/Logo.tsx`:

```tsx
import { useTabs } from "./tabs/useTabs";

export default function Logo() {
  const { openTab, tabs, activateTab } = useTabs();

  const go = () => {
    const home = tabs.find((t) => t.initialPath === "/home");
    if (home) activateTab(home.id);
    else openTab("/home");
  };

  return (
    <button className="shell-logo" onClick={go}>
      <span className="shell-logo__text">Placebo</span>
    </button>
  );
}
```

CSS:

```css
.shell-logo {
  background: transparent; border: 0; cursor: pointer;
  padding: 16px 24px; text-align: left;
}
.shell-logo__text {
  font-family: "Nunito", sans-serif; font-weight: 800;
  font-size: 28px; color: var(--t1);
}
```

- [ ] **Step 2: SidebarItem**

Создать `src/shell/SidebarItem.tsx`:

```tsx
import { useTabs } from "./tabs/useTabs";
import { Icon } from "../components/ui/Icon";
import clsx from "clsx";
import * as Icons from "../components/Icons";

type IconName = keyof typeof Icons;

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
    >
      <Icon name={icon} size={size === "lg" ? 24 : 20} className="sidebar-item__icon" />
      <span className="sidebar-item__label">{label}</span>
    </button>
  );
}
```

CSS (добавить в App.css):

```css
.sidebar-item {
  display: flex; align-items: center; gap: 12px;
  width: 100%; padding: 10px 20px;
  background: transparent; border: 0; border-radius: 10px;
  color: var(--t1); cursor: pointer; text-align: left;
  font-size: 16px; line-height: 1.2;
}
.sidebar-item:hover { background: var(--bg-2); }
.sidebar-item--active {
  background: var(--bg-2);
  /* Figma-style "bubble" outline for active item */
  box-shadow: inset 0 0 0 1px var(--t1);
}
.sidebar-item--lg .sidebar-item__label { font-weight: 700; font-size: 18px; }
.sidebar-item__icon { flex: 0 0 auto; color: var(--t1); }
.sidebar-item__label { flex: 1 1 auto; }
```

- [ ] **Step 3: Sidebar**

Создать `src/shell/Sidebar.tsx`:

```tsx
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
```

CSS:

```css
.shell-sidebar {
  grid-area: sidebar;
  background: var(--bg); border-right: 1px solid var(--border);
  display: flex; flex-direction: column;
  padding: 8px 12px 16px; gap: 4px;
  height: 100vh; overflow-y: auto;
}
.shell-sidebar__top,
.shell-sidebar__main,
.shell-sidebar__bottom,
.shell-sidebar__footer { display: flex; flex-direction: column; gap: 2px; }
.shell-sidebar__main { margin-top: 24px; }
.shell-sidebar__bottom { margin-top: 24px; padding-top: 16px; border-top: 1px solid var(--border); }
.shell-sidebar__footer { margin-top: auto; padding-top: 16px; border-top: 1px solid var(--border); }
```

- [ ] **Step 4: Smoke**

Сайдбар пока не смонтирован (нет ShellRoot). Проверим только, что файлы компилируются:

```bash
npm run build
```

Expected: build проходит (или падает на других вещах, но не из-за Sidebar).

- [ ] **Step 5: Commit**

```bash
git add src/shell/Sidebar.tsx src/shell/SidebarItem.tsx src/shell/Logo.tsx src/App.css
git commit -m "feat(shell): Sidebar, SidebarItem, Logo per Figma layout"
```

---

## Task 6: TopBar, NavButtons, ThemeToggle

**Files:**
- Create: `src/shell/TopBar.tsx`
- Create: `src/shell/NavButtons.tsx`
- Create: `src/shell/ThemeToggle.tsx`

- [ ] **Step 1: NavButtons (кнопки <, >, ↻)**

Создать `src/shell/NavButtons.tsx`:

```tsx
import { useTabs } from "./tabs/useTabs";
import { Icon } from "../components/ui/Icon";
import { useTranslation } from "react-i18next";

export default function NavButtons() {
  const { goBack, goForward, reload } = useTabs();
  const { t } = useTranslation();
  return (
    <div className="nav-buttons">
      <button aria-label={t("topbar.back")} onClick={goBack}><Icon name="ArrowLeftIcon" size={18} /></button>
      <button aria-label={t("topbar.forward")} onClick={goForward}><Icon name="ArrowRightIcon" size={18} /></button>
      <button aria-label={t("topbar.reload")} onClick={reload}><Icon name="RefreshIcon" size={18} /></button>
    </div>
  );
}
```

CSS:

```css
.nav-buttons { display: flex; gap: 4px; }
.nav-buttons button {
  width: 32px; height: 32px; border-radius: 8px;
  background: transparent; border: 0; color: var(--t2);
  cursor: pointer; display: grid; place-items: center;
}
.nav-buttons button:hover { background: var(--bg-2); color: var(--t1); }
```

- [ ] **Step 2: ThemeToggle (трёхпозиционный)**

`src/shell/ThemeToggle.tsx`:

```tsx
import { useTheme } from "../theme/useTheme";
import { Icon } from "../components/ui/Icon";
import clsx from "clsx";
import type { ThemeChoice } from "../theme";

const MAP: Array<{ value: ThemeChoice; icon: "MoonIcon" | "ToggleIcon" | "SunIcon" }> = [
  { value: "dark", icon: "MoonIcon" },
  { value: "auto", icon: "ToggleIcon" },
  { value: "light", icon: "SunIcon" },
];

export default function ThemeToggle() {
  const { choice, setChoice } = useTheme();
  return (
    <div className="theme-toggle">
      {MAP.map((item) => (
        <button
          key={item.value}
          className={clsx("theme-toggle__btn", choice === item.value && "theme-toggle__btn--active")}
          onClick={() => setChoice(item.value)}
          aria-pressed={choice === item.value}
        >
          <Icon name={item.icon} size={18} />
        </button>
      ))}
    </div>
  );
}
```

CSS:

```css
.theme-toggle {
  display: flex; gap: 2px;
  background: var(--bg-2); padding: 2px; border-radius: 999px;
}
.theme-toggle__btn {
  width: 32px; height: 28px; border-radius: 999px;
  background: transparent; border: 0; color: var(--t2);
  cursor: pointer; display: grid; place-items: center;
}
.theme-toggle__btn--active { background: var(--bg); color: var(--t1); box-shadow: 0 1px 3px rgba(0,0,0,0.08); }
```

- [ ] **Step 3: TopBar**

`src/shell/TopBar.tsx`:

```tsx
import { useTranslation } from "react-i18next";
import NavButtons from "./NavButtons";
import ThemeToggle from "./ThemeToggle";
import { Icon } from "../components/ui/Icon";

export default function TopBar() {
  const { t } = useTranslation();
  return (
    <div className="shell-topbar">
      <NavButtons />
      <label className="shell-topbar__search">
        <Icon name="SearchIcon" size={16} />
        <input
          type="text"
          placeholder={t("topbar.search.placeholder")}
          disabled
          aria-disabled="true"
        />
      </label>
      <ThemeToggle />
    </div>
  );
}
```

CSS:

```css
.shell-topbar {
  grid-area: topbar;
  height: var(--topbar-h);
  display: flex; align-items: center; gap: 16px;
  padding: 0 16px;
  border-bottom: 1px solid var(--border);
  background: var(--bg);
}
.shell-topbar__search {
  flex: 1 1 auto; display: flex; align-items: center; gap: 8px;
  background: var(--bg-2); border: 1px solid var(--border);
  border-radius: 10px; padding: 0 12px; height: 36px;
  color: var(--t3);
}
.shell-topbar__search input {
  flex: 1; background: transparent; border: 0; outline: none;
  color: var(--t1); font-size: 14px;
}
```

- [ ] **Step 4: Commit**

```bash
git add src/shell/TopBar.tsx src/shell/NavButtons.tsx src/shell/ThemeToggle.tsx src/App.css
git commit -m "feat(shell): TopBar with nav buttons, search (disabled), theme toggle"
```

---

## Task 7: TabBar и Tab

**Files:**
- Create: `src/shell/TabBar.tsx`
- Create: `src/shell/Tab.tsx`

- [ ] **Step 1: Tab**

```tsx
import { Icon } from "../components/ui/Icon";
import clsx from "clsx";
import { useTabs } from "./tabs/useTabs";
import type { Tab as TabData } from "./tabs/types";

export default function Tab({ tab }: { tab: TabData }) {
  const { activeTabId, activateTab, closeTab } = useTabs();
  const isActive = tab.id === activeTabId;

  const onMiddleClick = (e: React.MouseEvent) => {
    if (e.button === 1) {
      e.preventDefault();
      closeTab(tab.id);
    }
  };

  return (
    <div
      className={clsx("shell-tab", isActive && "shell-tab--active")}
      onClick={() => activateTab(tab.id)}
      onMouseDown={onMiddleClick}
      role="tab"
      aria-selected={isActive}
    >
      <span className="shell-tab__title" title={tab.title}>{tab.title}</span>
      <button
        className="shell-tab__close"
        onClick={(e) => { e.stopPropagation(); closeTab(tab.id); }}
        aria-label="close tab"
      >
        <Icon name="CloseIcon" size={14} />
      </button>
    </div>
  );
}
```

CSS:

```css
.shell-tab {
  display: flex; align-items: center; gap: 8px;
  max-width: 220px; padding: 0 10px; height: calc(var(--tabbar-h) - 4px);
  border: 1px solid transparent; border-radius: 8px 8px 0 0;
  cursor: pointer; color: var(--t2);
}
.shell-tab__title {
  flex: 1 1 auto; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  font-size: 13px;
}
.shell-tab__close {
  flex: 0 0 auto; width: 20px; height: 20px; border: 0; border-radius: 4px;
  background: transparent; color: var(--t3); cursor: pointer;
  display: grid; place-items: center; opacity: 0;
}
.shell-tab:hover { background: var(--bg-2); color: var(--t1); }
.shell-tab:hover .shell-tab__close { opacity: 1; }
.shell-tab--active {
  background: var(--bg); color: var(--t1);
  border-color: var(--border); border-bottom-color: var(--bg);
}
.shell-tab--active .shell-tab__close { opacity: 1; }
```

- [ ] **Step 2: TabBar**

```tsx
import { useTabs } from "./tabs/useTabs";
import Tab from "./Tab";
import { Icon } from "../components/ui/Icon";
import { useTranslation } from "react-i18next";

export default function TabBar() {
  const { tabs, openTab } = useTabs();
  const { t } = useTranslation();
  return (
    <div className="shell-tabbar" role="tablist">
      {tabs.map((t) => <Tab key={t.id} tab={t} />)}
      <button className="shell-tabbar__new" onClick={() => openTab("/home")} aria-label={t("tabbar.new")}>
        <Icon name="PlusSmallIcon" size={16} />
      </button>
    </div>
  );
}
```

CSS:

```css
.shell-tabbar {
  grid-area: tabbar;
  height: var(--tabbar-h);
  display: flex; align-items: flex-end; gap: 2px;
  padding: 4px 12px 0; background: var(--bg-2);
  border-bottom: 1px solid var(--border);
  overflow-x: auto; overflow-y: hidden;
}
.shell-tabbar__new {
  width: 28px; height: 28px; border-radius: 6px;
  background: transparent; border: 0; color: var(--t2);
  cursor: pointer; display: grid; place-items: center;
  margin-bottom: 2px;
}
.shell-tabbar__new:hover { background: var(--bg-3); color: var(--t1); }
```

- [ ] **Step 3: Commit**

```bash
git add src/shell/TabBar.tsx src/shell/Tab.tsx src/App.css
git commit -m "feat(shell): TabBar and Tab with middle-click close"
```

---

## Task 8: Breadcrumbs

**Files:**
- Create: `src/shell/Breadcrumbs.tsx`

Breadcrumbs нужны **внутри** активного таба (они знают текущий path). Мы их строим из `router.state.location.pathname` активного таба.

- [ ] **Step 1: Компонент**

```tsx
import { useEffect, useState } from "react";
import { useTabs } from "./tabs/useTabs";
import { useTranslation } from "react-i18next";

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
  const [path, setPath] = useState(active?.router.state.location.pathname ?? "/home");

  useEffect(() => {
    if (!active) return;
    setPath(active.router.state.location.pathname);
    const unsub = active.router.subscribe((state) => {
      setPath(state.location.pathname);
    });
    return () => unsub();
  }, [active]);

  const segments = path.replace(/^\/+/, "").split("/").filter(Boolean);
  const crumbs = segments.map((seg, i) => {
    const target = "/" + segments.slice(0, i + 1).join("/");
    return { label: labelFor(seg, t), target, isLast: i === segments.length - 1 };
  });

  if (crumbs.length === 0) return null;

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
```

CSS:

```css
.shell-breadcrumbs {
  grid-area: breadcrumbs;
  padding: 8px 32px; color: var(--t3); font-size: 13px;
  display: flex; align-items: center; gap: 4px;
  background: var(--bg); border-bottom: 1px solid var(--border);
}
.shell-breadcrumbs__crumb { display: inline-flex; align-items: center; gap: 6px; }
.shell-breadcrumbs__crumb button {
  background: transparent; border: 0; color: var(--t3); cursor: pointer;
  padding: 0; font-size: 13px;
}
.shell-breadcrumbs__crumb button:hover { color: var(--t1); }
.shell-breadcrumbs__leaf { color: var(--t1); font-weight: 600; }
.shell-breadcrumbs__sep { color: var(--t3); }
```

- [ ] **Step 2: Commit**

```bash
git add src/shell/Breadcrumbs.tsx src/App.css
git commit -m "feat(shell): Breadcrumbs subscribing to active tab router"
```

---

## Task 9: 3D виртуализация — скелет (в M1 пока пустой)

**Files:**
- Create: `src/shell/scene3d/types.ts`
- Create: `src/shell/scene3d/Scene3DRegistry.tsx`
- Create: `src/shell/scene3d/GlobalCanvas.tsx`

Цель задачи: заложить архитектуру виртуализации сейчас, чтобы в M4, когда World3D подключается, всё уже работало. В M1 `<GlobalCanvas />` – пустой Three.js canvas, который не рендерится (null).

- [ ] **Step 1: Типы**

```ts
// src/shell/scene3d/types.ts
export type Scene3DState = {
  /** Stable id for a 3D scene; usually tabId + ":world" */
  id: string;
  /** Arbitrary per-scene state the scene component manages itself. */
  state: Record<string, unknown>;
};
```

- [ ] **Step 2: Registry**

```tsx
// src/shell/scene3d/Scene3DRegistry.tsx
import { createContext, useCallback, useContext, useMemo, useRef, useState, ReactNode } from "react";

type RegistryApi = {
  activeSceneId: string | null;
  setActiveScene(id: string | null): void;
  registerSceneState<T>(id: string, state: T): void;
  getSceneState<T>(id: string): T | undefined;
};

const Scene3DContext = createContext<RegistryApi | null>(null);

export function Scene3DRegistry({ children }: { children: ReactNode }) {
  const [activeSceneId, setActiveSceneId] = useState<string | null>(null);
  const statesRef = useRef(new Map<string, unknown>());

  const registerSceneState = useCallback(<T,>(id: string, state: T) => {
    statesRef.current.set(id, state);
  }, []);

  const getSceneState = useCallback(<T,>(id: string): T | undefined => {
    return statesRef.current.get(id) as T | undefined;
  }, []);

  const api = useMemo<RegistryApi>(() => ({
    activeSceneId,
    setActiveScene: setActiveSceneId,
    registerSceneState,
    getSceneState,
  }), [activeSceneId, registerSceneState, getSceneState]);

  return <Scene3DContext.Provider value={api}>{children}</Scene3DContext.Provider>;
}

export function useScene3D() {
  const ctx = useContext(Scene3DContext);
  if (!ctx) throw new Error("useScene3D must be used within <Scene3DRegistry>");
  return ctx;
}
```

- [ ] **Step 3: GlobalCanvas-стаб**

```tsx
// src/shell/scene3d/GlobalCanvas.tsx
import { useScene3D } from "./Scene3DRegistry";

/**
 * Single shared GL context for the whole app. Currently a stub that
 * renders nothing when no scene is active; M4 will mount @react-three/fiber
 * Canvas here and host the active scene via portal.
 */
export default function GlobalCanvas() {
  const { activeSceneId } = useScene3D();
  if (!activeSceneId) return null;
  return (
    <div className="global-canvas" data-scene-id={activeSceneId}>
      {/* M4: <Canvas> and active scene content will live here */}
    </div>
  );
}
```

CSS:

```css
.global-canvas {
  position: absolute; inset: 0;
  pointer-events: none; /* The scene component enables events as needed */
}
```

- [ ] **Step 4: Commit**

```bash
git add src/shell/scene3d/ src/App.css
git commit -m "feat(shell): Scene3DRegistry scaffolding for future 3D virtualization"
```

---

## Task 10: TabContent — рендер активного таба (и hidden неактивных)

**Files:**
- Create: `src/shell/TabContent.tsx`

- [ ] **Step 1: Компонент**

RouterProvider (v6) ожидает, что его можно рендерить. Мы рендерим **все табы** одновременно, но неактивные – `hidden`. У каждого таба свой `RouterProvider` с его собственным `router`.

```tsx
import { RouterProvider } from "react-router-dom";
import { useTabs } from "./tabs/useTabs";

export default function TabContent() {
  const { tabs, activeTabId } = useTabs();
  return (
    <main className="shell-content">
      {tabs.map((tab) => (
        <div
          key={tab.id}
          className={"shell-content__tab" + (tab.id === activeTabId ? " shell-content__tab--active" : "")}
          hidden={tab.id !== activeTabId}
        >
          <RouterProvider router={tab.router} />
        </div>
      ))}
    </main>
  );
}
```

CSS:

```css
.shell-content {
  grid-area: content;
  position: relative;
  overflow: hidden;
  background: var(--bg);
}
.shell-content__tab {
  position: absolute; inset: 0;
  overflow: auto;
}
.shell-content__tab[hidden] { display: none !important; }
```

- [ ] **Step 2: Commit**

```bash
git add src/shell/TabContent.tsx src/App.css
git commit -m "feat(shell): TabContent keeps inactive tabs mounted but hidden"
```

---

## Task 11: ShellRoot и ShellLayout — собираем всё вместе

**Files:**
- Create: `src/shell/ShellLayout.tsx`
- Create: `src/shell/ShellRoot.tsx`
- Modify: `src/App.tsx` (подключаем)

- [ ] **Step 1: ShellLayout (чистый CSS-grid)**

```tsx
import { ReactNode } from "react";

type Props = {
  sidebar: ReactNode;
  topbar: ReactNode;
  tabbar: ReactNode;
  breadcrumbs: ReactNode;
  content: ReactNode;
};

export default function ShellLayout({ sidebar, topbar, tabbar, breadcrumbs, content }: Props) {
  return (
    <div className="shell-layout">
      {sidebar}
      {topbar}
      {tabbar}
      {breadcrumbs}
      {content}
    </div>
  );
}
```

CSS (добавить в App.css):

```css
.shell-layout {
  display: grid;
  grid-template-columns: var(--sidebar-w) 1fr;
  grid-template-rows: var(--topbar-h) var(--tabbar-h) auto 1fr;
  grid-template-areas:
    "sidebar topbar"
    "sidebar tabbar"
    "sidebar breadcrumbs"
    "sidebar content";
  height: 100vh; width: 100vw;
  background: var(--bg); color: var(--t1);
  font-family: "Nunito", system-ui, sans-serif;
}
```

- [ ] **Step 2: ShellRoot**

```tsx
import Sidebar from "./Sidebar";
import TopBar from "./TopBar";
import TabBar from "./TabBar";
import Breadcrumbs from "./Breadcrumbs";
import TabContent from "./TabContent";
import ShellLayout from "./ShellLayout";
import GlobalCanvas from "./scene3d/GlobalCanvas";

export default function ShellRoot() {
  return (
    <>
      <ShellLayout
        sidebar={<Sidebar />}
        topbar={<TopBar />}
        tabbar={<TabBar />}
        breadcrumbs={<Breadcrumbs />}
        content={<TabContent />}
      />
      <GlobalCanvas />
    </>
  );
}
```

- [ ] **Step 3: Переписать App.tsx**

```tsx
import { ThemeProvider } from "./theme";
import { TabManager } from "./shell/tabs/TabManager";
import { Scene3DRegistry } from "./shell/scene3d/Scene3DRegistry";
import ShellRoot from "./shell/ShellRoot";

export default function App() {
  return (
    <ThemeProvider>
      <Scene3DRegistry>
        <TabManager initialPath="/home">
          <ShellRoot />
        </TabManager>
      </Scene3DRegistry>
    </ThemeProvider>
  );
}
```

Старая логика `useState<Screen>("home")`, `inRoom`, `in3DWorld` – **полностью удалена**. Навигация теперь через TabManager + routes.

- [ ] **Step 4: Запустить и проверить**

```bash
npm run dev
```

Expected:
- Сайдбар слева со всеми пунктами.
- Верхняя панель с `< > ↻`, поиском (disabled) и переключателем темы.
- Один таб "Главная".
- Breadcrumbs: "Главная" (без кликабельной ссылки, так как он единственный сегмент).
- Клик на "Настройки" → открывается **новый таб** "Настройки" с экраном SettingsScreen.
- Клик на "Настройки" второй раз → таб **активируется**, не создаётся дубликат.
- Переключение темы в SettingsScreen → визуально меняется (если переменные используются – в M1 они используются в самом shell).
- Закрытие таба крестиком → активируется соседний.
- Закрытие последнего таба → появляется новый "Главная".
- Средний клик мышью по табу → закрывает.

- [ ] **Step 5: Тест на Tauri-runtime (не только Vite)**

```bash
npm run tauri dev
```

Expected: то же, но в Tauri-окне. Настройки сохраняются через `prefs_set` после перезапуска приложения (тема остаётся выбранной).

- [ ] **Step 6: Commit**

```bash
git add src/shell/ShellRoot.tsx src/shell/ShellLayout.tsx src/App.tsx src/App.css
git commit -m "$(cat <<'EOF'
feat(shell): wire up ShellRoot + ShellLayout + App entry

App.tsx is now a thin wrapper: ThemeProvider → Scene3DRegistry →
TabManager → ShellRoot. All previous ad-hoc screen state is removed;
navigation flows through per-tab memory routers and the sidebar.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 12: Доменные тесты на сайдбар → таб

**Files:**
- Create: `src/shell/Sidebar.test.tsx`

- [ ] **Step 1: Тест**

```tsx
import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import { ThemeProvider } from "../theme";
import { TabManager } from "./tabs/TabManager";
import { Scene3DRegistry } from "./scene3d/Scene3DRegistry";
import ShellRoot from "./ShellRoot";
import "../i18n";

function mount() {
  render(
    <ThemeProvider>
      <Scene3DRegistry>
        <TabManager initialPath="/home">
          <ShellRoot />
        </TabManager>
      </Scene3DRegistry>
    </ThemeProvider>,
  );
}

describe("Sidebar → tab opening", () => {
  it("opens Settings tab when clicking the sidebar item", () => {
    mount();
    // initial tab is Главная
    expect(screen.getByText("Главная")).toBeInTheDocument();

    fireEvent.click(screen.getByText("Настройки"));

    // tab title "Настройки" appears in tab bar (appears twice: in sidebar and in tab bar)
    const settingsNodes = screen.getAllByText("Настройки");
    expect(settingsNodes.length).toBeGreaterThanOrEqual(2);
  });

  it("activates existing tab instead of duplicating", () => {
    mount();
    fireEvent.click(screen.getByText("Настройки"));
    fireEvent.click(screen.getByText("Главная"));
    fireEvent.click(screen.getByText("Настройки"));

    // only one Settings tab (+1 in sidebar = 2 matches in DOM)
    const settingsNodes = screen.getAllByText("Настройки");
    expect(settingsNodes).toHaveLength(2);
  });
});
```

- [ ] **Step 2: Запустить**

```bash
npm test -- src/shell/Sidebar.test.tsx
```

Expected: 2 теста проходят.

- [ ] **Step 3: Commit**

```bash
git add src/shell/Sidebar.test.tsx
git commit -m "test(shell): sidebar opens/activates tabs correctly"
```

---

## Task 13: Восстановление табов после перезапуска

**Files:**
- Modify: `src-tauri/migrations/003_user_preferences.sql` уже содержит таблицу. Переиспользуем.
- Modify: `src/shell/tabs/TabManager.tsx` – сохранять snapshot при каждом изменении, читать при старте.
- Create: `src/shell/tabs/persistence.ts`

- [ ] **Step 1: Persistence utility**

`src/shell/tabs/persistence.ts`:

```ts
import { prefsGet, prefsSet } from "../../services/preferences";

const KEY = "tabs.snapshot.v1";

export type Snapshot = {
  tabs: Array<{ id: string; initialPath: string; title: string; currentPath: string }>;
  activeTabId: string;
};

export async function loadSnapshot(): Promise<Snapshot | null> {
  try {
    const raw = await prefsGet(KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as Snapshot;
    if (!parsed.tabs?.length) return null;
    return parsed;
  } catch {
    return null;
  }
}

export async function saveSnapshot(s: Snapshot): Promise<void> {
  try {
    await prefsSet(KEY, JSON.stringify(s));
  } catch {
    /* non-Tauri dev run: localStorage fallback */
    try {
      localStorage.setItem(KEY, JSON.stringify(s));
    } catch { /* ignore */ }
  }
}
```

- [ ] **Step 2: Интеграция в TabManager**

В `TabManager.tsx` добавить:

```tsx
import { useEffect } from "react";
import { loadSnapshot, saveSnapshot } from "./persistence";

// в TabManager после useState:
useEffect(() => {
  let cancelled = false;
  loadSnapshot().then((snap) => {
    if (cancelled || !snap) return;
    const restored = snap.tabs.map((s) => {
      const router = createMemoryRouter(routes, { initialEntries: [s.currentPath] });
      return {
        id: s.id,
        title: s.title,
        initialPath: s.initialPath,
        router,
        createdAt: Date.now(),
      } as Tab;
    });
    setTabs(restored);
    const active = restored.find((x) => x.id === snap.activeTabId) ?? restored[0];
    setActiveTabId(active.id);
  });
  return () => { cancelled = true; };
}, []);

// Автосохранение при изменении:
useEffect(() => {
  const snap = {
    tabs: tabs.map((t) => ({
      id: t.id,
      initialPath: t.initialPath,
      title: t.title,
      currentPath: t.router.state.location.pathname,
    })),
    activeTabId,
  };
  saveSnapshot(snap);
}, [tabs, activeTabId]);
```

- [ ] **Step 3: Smoke-тест восстановления**

```bash
npm run tauri dev
```

В приложении:
1. Открыть 3 таба: Главная, Настройки, Категории.
2. Активировать Настройки.
3. Закрыть приложение полностью.
4. Открыть снова.

Expected: три таба восстановились, активен Настройки.

- [ ] **Step 4: Commit**

```bash
git add src/shell/tabs/persistence.ts src/shell/tabs/TabManager.tsx
git commit -m "$(cat <<'EOF'
feat(shell): persist tab state across app restarts

Snapshot saves the list of tabs (id, title, initialPath, currentPath)
and activeTabId into user_preferences under tabs.snapshot.v1. On next
launch the tab list is restored; if there's nothing saved the default
single /home tab is used.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 14: README милестоуна и merge

- [ ] **Step 1: Обновить CLAUDE.md**

В разделе "Milestones" поставить галочку M1:

```markdown
- [x] **M0 Foundation** (2026-05-??)
- [x] **M1 Shell** (2026-05-??): sidebar, topbar, per-tab memory router, breadcrumbs, theme toggle, skeleton screens, persistence.
- [ ] M2 Auth.
...
```

- [ ] **Step 2: Финальный прогон**

```bash
cd d:/Projects/Placebo
cargo check --workspace
cd src-tauri && cargo test --lib
cd ..
npm test
npm run build
npm run tauri dev
```

Expected: всё зелёное.

- [ ] **Step 3: Commit и PR**

```bash
git add CLAUDE.md
git commit -m "docs: mark M1 Shell complete in CLAUDE.md"
git push -u origin feat/m1-shell
```

PR: `feat/m1-shell → main`.

---

## Acceptance Criteria для Milestone 1

1. ✅ Приложение запускается с shell по Figma (сайдбар, верхняя панель, таб-бар, breadcrumbs).
2. ✅ Клик на любой пункт сайдбара открывает новый таб (или активирует существующий).
3. ✅ В таб-баре можно открывать новые табы кнопкой "+", закрывать крестиком или средним кликом.
4. ✅ При закрытии последнего таба автоматически создаётся новый `/home`.
5. ✅ Кнопки `< > ↻` перемещают по истории **внутри активного таба**, не затрагивая другие.
6. ✅ Breadcrumbs отображают текущий path активного таба и обновляются при навигации.
7. ✅ Переключатель темы переключает между светлой/авто/тёмной, визуально меняется, сохраняется после перезапуска.
8. ✅ Все UI-строки проходят через i18n (нет ru-строк, захардкоженных в JSX).
9. ✅ Скелет-экраны (Уведомления, История, Избранное, Папки, Люди) отображают понятную заглушку.
10. ✅ `SettingsScreen` позволяет менять тему и язык (кнопка "Выйти" disabled).
11. ✅ Состояние табов восстанавливается после перезапуска приложения.
12. ✅ `npm test` проходит (включая тесты на `titleForPath` и Sidebar → tab).
13. ✅ `cargo test --workspace --lib` проходит.
14. ✅ `Scene3DRegistry` и `GlobalCanvas` существуют и компилируются (в M1 не отображают ничего, M4 их задействует).

---

## Что идёт дальше

После approval M1 – переход к `2026-05-14-milestone-2-auth.md`: Welcome screen, Register, Login, auth context, API-клиент с refresh-tokens.
