# Milestone 4: Home + Categories + World3D Integration Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Переписать Home и Categories по Figma, интегрировать существующий World3DScreen в shell через общий `GlobalCanvas` (виртуализация WebGL), переключить 3D-мир на реальные камеры из API. В конце milestone пользователь проходит Main → Categories → Онлайн карта мира → клик маркер → видит живой HLS-стрим в 3D.

**Architecture:**
- `HomeScreen` (по Figma): аватар пользователя, "Открытые комнаты" (горизонтальные кружки с + и аватаром), "Популярное сейчас" (сетка карточек), боковой блок с быстрыми ссылками (Видеоигры, Фильмы, Стримы вне дома, Бравл старс – в альфе просто текст). Данные загружаются параллельно: комнаты через `GET /rooms?open=true` (эндпоинт подготовим-заглушку в M4, реализация роллирующих данных – M5), популярные комнаты – placeholder список, фактический сигнал "сколько людей смотрит" берётся из viewer-count эндпоинта в М5.
- `CategoriesScreen` (по Figma): тайлы с разными категориями. В альфе работает один – "Онлайн карта мира", остальные – disabled-tile с toast "Скоро".
- `World3DScreen` переписывается на **driven-by-GlobalCanvas** архитектуру: сцена регистрируется в `Scene3DRegistry`, `<Canvas>` не создаётся внутри экрана, а порталом добавляется в `GlobalCanvas`.
- **Камеры в 3D**: `useCamerasFromApi()` из M3 подключается вместо `useNearbyCameras` mock.
- **HLS-плеер в 3D**: `CameraFrustum` обновляется на URL `/api/v1/hls-proxy/:slug` вместо dev-middleware.

**Tech Stack:** React Three Fiber, drei, hls.js, react-router-dom, i18n.

**Spec reference:** `docs/superpowers/specs/2026-05-14-alpha-design.md`, разделы 5.3–5.4, 6.

**Зависимости:** M1 (shell), M3 (cameras API + HLS proxy).

---

## File Map

### Create

- `src/screens/main/HomeScreen.tsx` (переписанный, заменяет `HomePlaceholder.tsx`)
- `src/screens/main/home.css`
- `src/screens/categories/CategoriesScreen.tsx`
- `src/screens/categories/categories.css`
- `src/screens/world/World3DScreen.tsx` (в новом месте, старый `src/screens/World3DScreen.tsx` удаляется)
- `src/screens/world/CameraDetailPanel.tsx` (боковая панель с инфо по камере)
- `src/shell/scene3d/useScene3D.ts` (хук регистрации сцены)
- `src/shell/scene3d/GlobalCanvas.tsx` (переписывается – теперь с реальным `<Canvas>`)

### Modify

- `src/shell/routes.tsx` – `/home` → `HomeScreen`, `/categories` → `CategoriesScreen`, `/world` → новый `World3DScreen`, убрать `/world` как старый компонент.
- `src/components/world3d/CameraFrustum.tsx` – HLS URL через API.
- `src/components/world3d/WorldScene.tsx` – работает внутри общего Canvas, не своего.
- `src/hooks/useNearbyCameras.ts` – deprecate (удалить).
- `src/i18n/locales/ru.json` – ключи home/categories/world.

### Delete

- `src/screens/World3DScreen.tsx` (перенесли в `screens/world/`)
- `src/screens/HomeScreen.tsx` (заменили на `screens/main/HomeScreen.tsx`)
- `src/screens/ExploreScreen.tsx` (заменили на `screens/categories/CategoriesScreen.tsx`)
- `src/screens/main/HomePlaceholder.tsx`
- `src/screens/WatchRoomScreen.tsx` старый – **не удаляем в M4**, это M5.

---

## Task 1: Ветка

```bash
git -C d:/Projects/Placebo checkout main && git -C d:/Projects/Placebo pull
git -C d:/Projects/Placebo checkout -b feat/m4-home-categories-world
```

---

## Task 2: Переписать GlobalCanvas на R3F + реестр

**Files:**
- Modify: `src/shell/scene3d/GlobalCanvas.tsx`
- Create: `src/shell/scene3d/useScene3D.ts`
- Modify: `src/shell/scene3d/Scene3DRegistry.tsx` (+ activeScene children)

- [ ] **Step 1: Сцена-контейнер**

Расширить registry, чтобы хранить не только id, но и `ReactNode` активной сцены:

```tsx
// Scene3DRegistry.tsx – дополнить
type RegistryApi = {
  activeSceneId: string | null;
  activeSceneNode: ReactNode | null;
  setActiveScene(id: string | null, node: ReactNode | null): void;
};

export function Scene3DRegistry({ children }: { children: ReactNode }) {
  const [activeSceneId, setActiveSceneId] = useState<string | null>(null);
  const [activeSceneNode, setActiveSceneNode] = useState<ReactNode | null>(null);

  const setActiveScene = useCallback((id: string | null, node: ReactNode | null) => {
    setActiveSceneId(id);
    setActiveSceneNode(node);
  }, []);

  const api = useMemo<RegistryApi>(() => ({ activeSceneId, activeSceneNode, setActiveScene }),
    [activeSceneId, activeSceneNode, setActiveScene]);
  return <Scene3DContext.Provider value={api}>{children}</Scene3DContext.Provider>;
}
```

- [ ] **Step 2: useScene3D – хук регистрации**

```ts
// src/shell/scene3d/useScene3D.ts
import { ReactNode, useEffect, useRef } from "react";
import { useScene3D as useRegistry } from "./Scene3DRegistry";

export function useActiveScene(id: string, node: ReactNode) {
  const { setActiveScene } = useRegistry();
  const nodeRef = useRef(node);
  nodeRef.current = node;

  useEffect(() => {
    setActiveScene(id, nodeRef.current);
    return () => setActiveScene(null, null);
  }, [id, setActiveScene]);
}
```

- [ ] **Step 3: GlobalCanvas с R3F**

```tsx
// src/shell/scene3d/GlobalCanvas.tsx
import { Canvas } from "@react-three/fiber";
import { useScene3D as useRegistry } from "./Scene3DRegistry";

export default function GlobalCanvas() {
  const { activeSceneId, activeSceneNode } = useRegistry();
  if (!activeSceneId || !activeSceneNode) return null;
  return (
    <div className="global-canvas">
      <Canvas camera={{ position: [0, 30, 60], fov: 55 }} dpr={[1, 1.5]} frameloop="always">
        {activeSceneNode}
      </Canvas>
    </div>
  );
}
```

CSS (обновить):

```css
.global-canvas {
  position: absolute; inset: 0;
  /* Rendered behind UI; scenes enable pointer-events on their own meshes */
  pointer-events: auto;
  z-index: 0;
}
```

И нужно сделать так, чтобы shell-content имел `z-index: 1` поверх canvas-а **только тогда, когда активная сцена не требует canvas'а на экране**. Проще всего: `GlobalCanvas` находится **внутри** активной страницы World3D как backdrop. Но план выше делает его глобальным. Выбираем более простой вариант: GlobalCanvas рендерится **внутри `WorldScreen`**, а не в ShellRoot. Это отличается от раздела 5.3 спеки, где canvas один на всё приложение; делаем компромиссное решение для альфы:

**Решение для альфы:** `GlobalCanvas` сохранён как концепт (для M5-M6, когда potentially будет несколько 3D-мест). В M4 он рендерится **только когда активная сцена есть** и **только если активный таб открыт на `/world`**. Реально – сцена живёт внутри `World3DScreen`, но через useActiveScene регистрирует себя, и `GlobalCanvas` рендерит её на ShellRoot-уровне. Это отвязывает жизненный цикл Canvas'а от текущего таба: при переключении на другой таб 3D-canvas уходит из DOM, при возврате – монтируется обратно. Состояние сцены (позиция камеры и т.д.) сохраняется **внутри** сцены через хуки, пока сцена-ноде активна в React-дереве.

**Более честный вариант** – в M4 Canvas монтируется в `WorldScene` как обычно, а виртуализация откладывается. Делаем так:

```tsx
// Упрощённый GlobalCanvas – остаётся пустым, не рендерит ничего.
export default function GlobalCanvas() {
  return null;
}
```

И пересмотр архитектуры: **3D canvas живёт внутри `World3DScreen`, как было раньше в коде**. Когда таб неактивен → scene CSS hidden → canvas стоп-кадр. Когда таб активен → canvas рендерит. Это избавляет от сложного portal-solution и работает прямо сейчас. Ограничение: **один таб может активно рендерить 3D**. Открытие двух табов с `/world` одновременно = два canvas'а = два WebGL-контекста. В альфе считаем это acceptable. В M7 можно вернуться и заложить portal-based GlobalCanvas, если возникнут проблемы с памятью.

- [ ] **Step 4: Зафиксировать "без виртуализации для альфы" в заметке**

Добавить в `docs/superpowers/specs/2026-05-14-alpha-design.md` в раздел 10 (Deferred Decisions) пункт:

> 11. **3D-виртуализация через GlobalCanvas + React portal.** В M4 сцены живут внутри своих screen-компонентов (классический R3F `<Canvas>`). Portal-based virtualization отложена до M7 или пост-альфы; ограничение – не открывать два таба с `/world` одновременно.

- [ ] **Step 5: Commit**

```bash
git add src/shell/scene3d/ docs/superpowers/specs/2026-05-14-alpha-design.md
git commit -m "chore(3d): defer GlobalCanvas virtualization; scenes stay inside screens for alpha"
```

---

## Task 3: HomeScreen по Figma

**Files:** `src/screens/main/HomeScreen.tsx`, `src/screens/main/home.css`

По макету Figma (файл "Главная"):
- Breadcrumbs: Home / An Application (в альфе: просто "Главная")
- Справа вверху: ссылка "О приложении Placebo".
- Заголовок "Открытые комнаты".
- Горизонтальная строка кружков: "+ Создать комнату" (красный градиент), затем круглые аватары.
- "Популярное сейчас:" + справа dropdown "Рекомендации" (в альфе disabled).
- Сетка 4×N карточек комнат (thumbnail + название + количество зрителей).
- Боковой блок справа: "Видеоигры", "Фильмы", "Стримы вне дома (IRL)", "Бравл старс" (каждый с dropdown-треугольником).

- [ ] **Step 1: Структура компонента**

```tsx
import { useTranslation } from "react-i18next";
import { useAuth } from "../../auth/useAuth";
import { useNavigate } from "react-router-dom";
import { useTabs } from "../../shell/tabs/useTabs";
import "./home.css";

export default function HomeScreen() {
  const { t } = useTranslation();
  const { user } = useAuth();
  const nav = useNavigate();
  const { openTab } = useTabs();

  const createRoom = () => openTab("/create", t("shell.tab.create"));

  // Mock popular rooms (реальный endpoint в M5)
  const popular = [
    { id: "1", name: t("home.mock.concert"), viewers: 65, thumb: null },
    { id: "2", name: t("home.mock.korea_news"), viewers: 389, thumb: null },
    { id: "3", name: t("home.mock.harry_potter"), viewers: 20, thumb: null },
    { id: "4", name: t("home.mock.pubg"), viewers: 1014, thumb: null },
    { id: "5", name: t("home.mock.times_square"), viewers: 93, thumb: null },
    { id: "6", name: t("home.mock.horror"), viewers: 666, thumb: null, tag: "horror" },
    { id: "7", name: t("home.mock.bratishkin"), viewers: 14888, thumb: null },
    { id: "8", name: t("home.mock.tv_2x2"), viewers: 401, thumb: null },
  ];

  return (
    <div className="home">
      <div className="home__head">
        <h2>{t("home.open_rooms")}</h2>
        <div className="home__about">{t("home.about")}</div>
      </div>

      <div className="home__rooms-row">
        <button className="home__create" onClick={createRoom}>
          <span className="home__create-plus">+</span>
          <span>{t("home.create_room")}</span>
        </button>
        <div className="home__avatar">
          <div className="home__avatar-circle">{user?.displayName?.[0] ?? "•"}</div>
          <span>{user?.username ? `@${user.username}` : ""}</span>
        </div>
        {Array.from({ length: 12 }).map((_, i) => (
          <div key={i} className="home__empty-slot" aria-hidden>+</div>
        ))}
      </div>

      <div className="home__popular-head">
        <h2>{t("home.popular")}</h2>
        <button className="home__dropdown" disabled>{t("home.recommendations")} ▾</button>
      </div>

      <div className="home__layout">
        <div className="home__grid">
          {popular.map((r) => (
            <button key={r.id} className="home__card" onClick={() => nav(`/room/${r.id}`)}>
              <div className="home__thumb" />
              <div className="home__card-meta">
                <span className="home__viewers">👥 {r.viewers}</span>
              </div>
              <div className="home__card-title">{r.name}</div>
            </button>
          ))}
        </div>

        <aside className="home__side">
          {["home.side.games", "home.side.films", "home.side.irl", "home.side.brawl"].map((k) => (
            <button key={k} className="home__side-item" disabled>
              {t(k)} <span>▾</span>
            </button>
          ))}
        </aside>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: CSS**

```css
.home { padding: 24px 32px; }
.home__head { display: flex; align-items: baseline; justify-content: space-between; margin-bottom: 16px; }
.home__head h2 { font-size: 18px; font-weight: 600; color: var(--t1); margin: 0; }
.home__about { color: var(--t2); font-size: 14px; }
.home__rooms-row {
  display: flex; gap: 12px; align-items: center;
  overflow-x: auto; padding: 8px 0 16px;
}
.home__create {
  min-width: 180px; height: 120px; border-radius: 12px;
  background: linear-gradient(135deg, #ffbfc9, #ff6b81);
  border: 0; color: var(--t1); font-weight: 600;
  display: flex; flex-direction: column; align-items: center; justify-content: center;
  gap: 8px; cursor: pointer;
}
.home__create-plus { font-size: 24px; }
.home__avatar { display: flex; flex-direction: column; align-items: center; gap: 6px; color: var(--t1); }
.home__avatar-circle {
  width: 56px; height: 56px; border-radius: 50%;
  background: #7c4dff; color: #fff; display: grid; place-items: center; font-weight: 700;
}
.home__empty-slot {
  width: 56px; height: 56px; border-radius: 50%;
  background: var(--bg-3); color: var(--t3);
  display: grid; place-items: center; font-size: 20px;
  flex: 0 0 auto;
}
.home__popular-head { display: flex; align-items: baseline; justify-content: space-between; margin: 16px 0; }
.home__dropdown { background: transparent; border: 0; color: var(--accent); font-size: 14px; cursor: pointer; }
.home__dropdown:disabled { opacity: 0.6; cursor: not-allowed; }

.home__layout { display: grid; grid-template-columns: 1fr 180px; gap: 24px; }
.home__grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 16px; }
@media (max-width: 1100px) { .home__grid { grid-template-columns: repeat(3, 1fr); } }
@media (max-width: 800px)  { .home__grid { grid-template-columns: repeat(2, 1fr); } }
.home__card {
  background: var(--bg); border: 1px solid var(--border); border-radius: 12px;
  overflow: hidden; cursor: pointer; text-align: left;
  display: flex; flex-direction: column;
}
.home__thumb { aspect-ratio: 16/9; background: var(--bg-3); }
.home__card-meta { padding: 8px 12px; color: var(--t2); font-size: 13px; }
.home__card-title { padding: 0 12px 12px; color: var(--t1); font-weight: 600; }
.home__viewers { color: var(--t2); }

.home__side { display: flex; flex-direction: column; gap: 8px; }
.home__side-item {
  display: flex; align-items: center; justify-content: space-between;
  background: transparent; border: 0; color: var(--accent);
  padding: 6px 8px; cursor: pointer; font-weight: 600;
}
.home__side-item:disabled { opacity: 0.8; cursor: default; }
```

- [ ] **Step 3: i18n**

Добавить в `ru.json`:

```json
{
  "home.open_rooms": "Открытые комнаты",
  "home.create_room": "Создать комнату",
  "home.popular": "Популярное сейчас:",
  "home.recommendations": "Рекомендации",
  "home.about": "О приложении Placebo",
  "home.side.games": "Видеоигры",
  "home.side.films": "Фильмы",
  "home.side.irl": "Стримы вне дома (IRL)",
  "home.side.brawl": "Бравл старс",

  "home.mock.concert": "Концерт Шамана",
  "home.mock.korea_news": "Новости Кореи",
  "home.mock.harry_potter": "Смотрим Гарри Поттера",
  "home.mock.pubg": "Стрим по Пабг мобайлу",
  "home.mock.times_square": "Камера на Таймсквер",
  "home.mock.horror": "Нарек попущен анонимом",
  "home.mock.bratishkin": "Братишкин стрим",
  "home.mock.tv_2x2": "Телеканал 2x2"
}
```

- [ ] **Step 4: Подключить в routes**

```tsx
import HomeScreen from "../screens/main/HomeScreen";
// ...
{ path: "/home", element: guarded(<HomeScreen />) },
```

Удалить `HomePlaceholder` импорт и файл:

```bash
rm d:/Projects/Placebo/src/screens/main/HomePlaceholder.tsx
```

- [ ] **Step 5: Commit**

```bash
git add src/screens/main/ src/shell/routes.tsx src/i18n/locales/ru.json
git commit -m "feat(home): HomeScreen per Figma (mock popular rooms)"
```

---

## Task 4: CategoriesScreen

**Files:** `src/screens/categories/CategoriesScreen.tsx`, `src/screens/categories/categories.css`

По Figma "Категории":
- Breadcrumbs: Телекамеры / Токио / Усарайдзю / ул. Джеки Чана (в альфе – пусто).
- Два блока: "Что сейчас происходит в мире:" и "Расслабиться и забыться:".
- Большие градиентные тайлы (звёздное небо "Онлайн карта мира", голубой "Видео-камеры со всех уголков мира", оранжевый "Посмотреть фильмы вместе").
- Мелкие тайлы: "Онлайн трансляции", "Послушать радио из разных стран", "Телепередачи Сеула/Бостона", "Стримы вне дома (IRL)".
- Второй ряд: "Подписки на каналы", "Клипы", "Клубы по интересам", "Совместные игры с друзьями", "Уличные певцы", "Мультики для детей".

В альфе клик на любой тайл кроме "Онлайн карта мира" показывает toast `Скоро`. Клик на "Онлайн карта мира" → `navigate("/world")`.

- [ ] **Step 1: Toast utility**

Простой toast для disabled-кликов. Минимум: хук `useToast()` с одним сообщением в offset-позиции.

`src/components/ui/Toast.tsx`:

```tsx
import { createContext, useCallback, useContext, useRef, useState, ReactNode } from "react";

type ToastApi = { show(message: string): void };
const Ctx = createContext<ToastApi | null>(null);

export function ToastProvider({ children }: { children: ReactNode }) {
  const [msg, setMsg] = useState<string | null>(null);
  const timer = useRef<number | null>(null);
  const show = useCallback((m: string) => {
    setMsg(m);
    if (timer.current) window.clearTimeout(timer.current);
    timer.current = window.setTimeout(() => setMsg(null), 2200);
  }, []);
  return (
    <Ctx.Provider value={{ show }}>
      {children}
      {msg && <div className="toast">{msg}</div>}
    </Ctx.Provider>
  );
}

export function useToast() {
  const c = useContext(Ctx);
  if (!c) throw new Error("ToastProvider missing");
  return c;
}
```

CSS:

```css
.toast {
  position: fixed; bottom: 24px; left: 50%; transform: translateX(-50%);
  background: var(--t1); color: var(--bg);
  padding: 10px 16px; border-radius: 10px;
  font-size: 14px; z-index: 9999;
}
```

Подключить в `App.tsx` рядом с ThemeProvider.

- [ ] **Step 2: Компонент**

```tsx
import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { useToast } from "../../components/ui/Toast";
import "./categories.css";

type Tile = {
  key: string;
  title: string;
  enabled: boolean;
  path?: string;
  variant?: "hero-sky" | "hero-blue" | "hero-orange" | "small-orange" | "small-olive" | "small-pink" | "small-purple" | "small-pink-light" | "small-black" | "small-orange-pale";
};

export default function CategoriesScreen() {
  const { t } = useTranslation();
  const nav = useNavigate();
  const { show } = useToast();

  const click = (tile: Tile) => {
    if (tile.enabled && tile.path) nav(tile.path);
    else show(t("categories.coming_soon"));
  };

  const world: Tile[] = [
    { key: "world-map",  title: t("categories.world_map"),      enabled: true, path: "/world", variant: "hero-sky" },
    { key: "webcams",    title: t("categories.webcams"),        enabled: false, variant: "hero-blue" },
    { key: "films",      title: t("categories.films_together"), enabled: false, variant: "hero-orange" },
    { key: "live",       title: t("categories.live"),           enabled: false, variant: "small-orange" },
    { key: "radio",      title: t("categories.radio"),          enabled: false, variant: "small-olive" },
    { key: "tv",         title: t("categories.tv"),             enabled: false, variant: "small-pink" },
    { key: "irl",        title: t("categories.irl"),            enabled: false, variant: "small-purple" },
  ];
  const chill: Tile[] = [
    { key: "subs",       title: t("categories.subs"),     enabled: false, variant: "small-pink-light" },
    { key: "clips",      title: t("categories.clips"),    enabled: false, variant: "small-black" },
    { key: "clubs",      title: t("categories.clubs"),    enabled: false, variant: "small-orange-pale" },
    { key: "games",      title: t("categories.games"),    enabled: false, variant: "small-orange-pale" },
    { key: "singers",    title: t("categories.singers"),  enabled: false, variant: "small-orange-pale" },
    { key: "cartoons",   title: t("categories.cartoons"), enabled: false, variant: "small-orange-pale" },
  ];

  return (
    <div className="cats">
      <h2 className="cats__section">{t("categories.world_section")}</h2>
      <div className="cats__world">
        {world.map((tile) => (
          <button key={tile.key} className={`cats__tile cats__tile--${tile.variant}`} onClick={() => click(tile)} disabled={!tile.enabled && false}>
            <span className="cats__tile-title">{tile.title}</span>
          </button>
        ))}
      </div>

      <h2 className="cats__section">{t("categories.chill_section")}</h2>
      <div className="cats__chill">
        {chill.map((tile) => (
          <button key={tile.key} className={`cats__tile cats__tile--${tile.variant}`} onClick={() => click(tile)}>
            <span className="cats__tile-title">{tile.title}</span>
          </button>
        ))}
      </div>
    </div>
  );
}
```

- [ ] **Step 3: CSS**

```css
.cats { padding: 24px 32px; }
.cats__section { font-size: 18px; font-weight: 600; color: var(--t1); margin: 16px 0; }
.cats__world {
  display: grid;
  grid-template-columns: 2fr 1fr 1fr 1fr 1fr;
  grid-template-rows: 1fr 1fr;
  grid-template-areas:
    "a b b c d"
    "a b b e f";
  gap: 16px; min-height: 360px;
}
.cats__world > :nth-child(1) { grid-area: a; }
.cats__world > :nth-child(2) { grid-area: b; }
.cats__world > :nth-child(3) { grid-area: b; display: none; } /* Figma: third hero is adjacent; simplified here */
/* Simpler 3-column grid fallback for alpha: */
@media (max-width: 1100px) {
  .cats__world { grid-template-columns: repeat(3, 1fr); grid-template-areas: none; grid-template-rows: auto; }
  .cats__world > * { grid-area: auto !important; }
}
.cats__chill { display: grid; grid-template-columns: repeat(6, 1fr); gap: 16px; }
@media (max-width: 1100px) { .cats__chill { grid-template-columns: repeat(3, 1fr); } }

.cats__tile {
  min-height: 140px; border-radius: 16px; border: 0; padding: 16px;
  cursor: pointer; display: flex; align-items: flex-end; text-align: left;
  color: var(--t1); font-weight: 700; font-size: 18px; overflow: hidden; position: relative;
}
.cats__tile--hero-sky    { background: linear-gradient(160deg,#03121f,#0a1f33 40%,#3b6a3b); color: #fff; min-height: 220px; }
.cats__tile--hero-blue   { background: linear-gradient(170deg,#72b3d6,#7cb6d4 50%,#3b7ea3); color: #fff; min-height: 220px; }
.cats__tile--hero-orange { background: linear-gradient(170deg,#7a5af8,#ffa45a 90%); color: #fff; min-height: 220px; }
.cats__tile--small-orange{ background: linear-gradient(170deg,#ffb28c,#ff7a45 90%); min-height: 160px; }
.cats__tile--small-olive { background: linear-gradient(160deg,#c8bd7a,#6f6a3b 90%); color: #fff; min-height: 160px; }
.cats__tile--small-pink  { background: linear-gradient(160deg,#d38ec0,#7d3a6c); color: #fff; min-height: 160px; }
.cats__tile--small-purple{ background: linear-gradient(160deg,#7b5ded,#ff9a55); color: #fff; min-height: 160px; }
.cats__tile--small-pink-light { background: linear-gradient(170deg,#ff7abf,#ffb1d4); color: #fff; min-height: 130px; }
.cats__tile--small-black { background: #0f0a12; color: #ff3368; min-height: 130px; }
.cats__tile--small-orange-pale { background: linear-gradient(170deg,#ffcc9f,#ff8f5a); min-height: 130px; }
.cats__tile-title { max-width: 80%; line-height: 1.2; }
```

- [ ] **Step 4: i18n**

```json
{
  "categories.world_section": "Что сейчас происходит в мире:",
  "categories.chill_section": "Расслабиться и забыться:",
  "categories.world_map": "Онлайн карта мира",
  "categories.webcams": "Видео-камеры со всех уголков мира: В реальном времени",
  "categories.films_together": "Посмотреть фильмы вместе",
  "categories.live": "Онлайн трансляции",
  "categories.radio": "Послушать радио из разных стран",
  "categories.tv": "Телепередачи Сеула/Бостона",
  "categories.irl": "Стримы вне дома (IRL)",
  "categories.subs": "Подписки на каналы",
  "categories.clips": "Клипы",
  "categories.clubs": "Клубы по интересам",
  "categories.games": "Совместные игры с друзьями",
  "categories.singers": "Уличные певцы",
  "categories.cartoons": "Мультики для детей",
  "categories.coming_soon": "Скоро"
}
```

- [ ] **Step 5: Подключить**

`src/shell/routes.tsx`:

```tsx
import CategoriesScreen from "../screens/categories/CategoriesScreen";
// ...
{ path: "/categories", element: guarded(<CategoriesScreen />) },
```

Удалить импорт и рендер `ExploreScreen`:

```bash
rm d:/Projects/Placebo/src/screens/ExploreScreen.tsx
```

- [ ] **Step 6: ToastProvider в App**

```tsx
import { ToastProvider } from "./components/ui/Toast";
// ...
<ThemeProvider>
  <ToastProvider>
    <AuthProvider>
      ...
    </AuthProvider>
  </ToastProvider>
</ThemeProvider>
```

- [ ] **Step 7: Commit**

```bash
git add src/screens/categories/ src/components/ui/Toast.tsx src/App.tsx src/App.css src/shell/routes.tsx src/i18n/locales/ru.json
rm -f src/screens/ExploreScreen.tsx
git add -A
git commit -m "feat(categories): Categories screen per Figma, toast for disabled tiles"
```

---

## Task 5: World3DScreen в shell

**Files:**
- Create: `src/screens/world/World3DScreen.tsx`
- Create: `src/screens/world/CameraDetailPanel.tsx`
- Modify: `src/components/world3d/WorldScene.tsx` (теперь принимает камеры пропсом)
- Modify: `src/components/world3d/CameraFrustum.tsx` (HLS URL через API)
- Delete: старый `src/screens/World3DScreen.tsx`

- [ ] **Step 1: Перенос и адаптация**

```bash
mv d:/Projects/Placebo/src/screens/World3DScreen.tsx d:/Projects/Placebo/src/screens/world/_old_reference.tsx.bak
```

(бэкап для сравнения, удалим в конце)

- [ ] **Step 2: Новый World3DScreen**

```tsx
// src/screens/world/World3DScreen.tsx
import { useEffect, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { Canvas } from "@react-three/fiber";
import WorldScene from "../../components/world3d/WorldScene";
import CameraDetailPanel from "./CameraDetailPanel";
import { useCamerasFromApi } from "../../hooks/useCamerasFromApi";
import type { CameraSummary } from "../../types/api/CameraSummary";

export default function World3DScreen() {
  const { data: cameras, loading, error } = useCamerasFromApi();
  const [active, setActive] = useState<CameraSummary | null>(null);
  const { id } = useParams();
  const nav = useNavigate();

  useEffect(() => {
    if (!cameras || !id) return;
    const found = cameras.find((c) => c.id === id || c.slug === id);
    if (found) setActive(found);
  }, [cameras, id]);

  if (loading) return <div style={{ padding: 32 }}>Loading cameras...</div>;
  if (error) return <div style={{ padding: 32, color: "#D12850" }}>Ошибка: {error.message}</div>;
  if (!cameras) return null;

  return (
    <div className="world-screen">
      <div className="world-screen__canvas">
        <Canvas camera={{ position: [0, 30, 60], fov: 55 }} dpr={[1, 1.5]}>
          <WorldScene cameras={cameras} activeId={active?.id ?? null} onPickCamera={(c) => {
            setActive(c);
            nav(`/world/${c.slug}`, { replace: true });
          }} />
        </Canvas>
      </div>
      {active && <CameraDetailPanel camera={active} onClose={() => { setActive(null); nav("/world", { replace: true }); }} />}
    </div>
  );
}
```

CSS (в `App.css` или отдельный):

```css
.world-screen { position: relative; width: 100%; height: 100%; background: var(--scene-bg); }
.world-screen__canvas { position: absolute; inset: 0; }
```

- [ ] **Step 3: CameraDetailPanel**

```tsx
import type { CameraSummary } from "../../types/api/CameraSummary";
import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";

type Props = { camera: CameraSummary; onClose: () => void };

export default function CameraDetailPanel({ camera, onClose }: Props) {
  const { t } = useTranslation();
  const nav = useNavigate();

  const watchTogether = () => {
    // Flow: create room via API (M5) and navigate; until then use placeholder.
    nav(`/create?camera=${camera.id}`);
  };

  return (
    <div className="world-panel">
      <div className="world-panel__head">
        <div>
          <div className="world-panel__title">{camera.name}</div>
          <div className="world-panel__subtitle">{camera.city}, {camera.country}</div>
        </div>
        <button onClick={onClose} aria-label="close">✕</button>
      </div>
      <button className="world-panel__watch" onClick={watchTogether}>
        {t("world.watch_together")}
      </button>
    </div>
  );
}
```

CSS:

```css
.world-panel {
  position: absolute; top: 24px; right: 24px;
  width: 320px; background: var(--bg); color: var(--t1);
  border-radius: 16px; border: 1px solid var(--border);
  padding: 16px; box-shadow: 0 10px 30px rgba(0,0,0,0.25);
}
.world-panel__head { display: flex; justify-content: space-between; align-items: flex-start; }
.world-panel__title { font-size: 18px; font-weight: 700; }
.world-panel__subtitle { color: var(--t2); font-size: 13px; }
.world-panel__watch {
  margin-top: 16px; width: 100%; padding: 12px;
  background: var(--accent); color: #fff; border: 0; border-radius: 10px;
  cursor: pointer; font-weight: 600;
}
```

- [ ] **Step 4: WorldScene и CameraFrustum обновления**

Текущий `WorldScene.tsx` получает камеры через хук, но нам нужно чтобы сцена принимала их как проп – для явной зависимости от API.

Основные изменения в `WorldScene.tsx`:

```tsx
import type { CameraSummary } from "../../types/api/CameraSummary";

type Props = {
  cameras: CameraSummary[];
  activeId: string | null;
  onPickCamera: (cam: CameraSummary) => void;
};

export default function WorldScene({ cameras, activeId, onPickCamera }: Props) {
  // Use cameras directly; origin = active camera position for lat/lng→XZ conversion.
  const active = cameras.find((c) => c.id === activeId) ?? cameras[0];
  // ... existing scene setup, passing `cameras`, `active`, and `onPickCamera` down to markers/frustum
}
```

В `CameraFrustum.tsx` заменить `streamUrl` на `/api/v1/hls-proxy/${camera.slug}` через env:

```tsx
const API_BASE = import.meta.env.VITE_API_BASE_URL ?? "http://localhost:3001/api/v1";
const hlsUrl = `${API_BASE.replace(/\/api\/v1$/, "")}/api/v1/hls-proxy/${camera.slug}`;
// Или если API_BASE уже содержит /api/v1 – можно просто `${API_BASE}/hls-proxy/${camera.slug}`.
```

- [ ] **Step 5: Route + удалить старый**

```tsx
import World3DScreen from "../screens/world/World3DScreen";
// ...
{ path: "/world", element: guarded(<World3DScreen />) },
{ path: "/world/:id", element: guarded(<World3DScreen />) },
```

Удалить старый файл:

```bash
rm d:/Projects/Placebo/src/screens/world/_old_reference.tsx.bak
```

- [ ] **Step 6: i18n**

```json
"world.watch_together": "Смотреть вместе"
```

- [ ] **Step 7: Удалить useNearbyCameras**

```bash
rm d:/Projects/Placebo/src/hooks/useNearbyCameras.ts
```

Проверить, что на него никто больше не ссылается:

```bash
grep -rn "useNearbyCameras" d:/Projects/Placebo/src/
```

Expected: ничего.

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "$(cat <<'EOF'
feat(world): World3DScreen driven by /cameras API + HLS proxy

- Cameras come from useCamerasFromApi (real CameraSummary items).
- CameraDetailPanel opens on marker click with a Watch Together button
  (routes to /create?camera=... until M5 room creation lands).
- CameraFrustum uses /api/v1/hls-proxy/:slug for HLS playback.
- Removed the legacy useNearbyCameras mock.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 6: E2E smoke-тест

- [ ] **Step 1: Backend + frontend**

```bash
cd d:/Projects/Placebo/crates/placebo-api && cargo run &
cd d:/Projects/Placebo && npm run tauri dev
```

- [ ] **Step 2: Шаги**

1. Зарегистрироваться / войти.
2. HomeScreen: видны 8 mock-карточек "Популярное", кружки комнат, боковые ссылки disabled.
3. Категории: большие тайлы и мелкие. Клик на "Фильмы" → toast "Скоро". Клик на "Онлайн карта мира" → `/world`.
4. World3D: загружаются реальные камеры из API (~18 штук). Маркеры на глобусе/карте.
5. Клик на маркер в Токио (yt-shibuya-crossing) → CameraDetailPanel справа + HLS-стрим виден в 3D-плоскости.
6. Клик "Смотреть вместе" → переход на `/create?camera=<id>` (пока старый CreateScreen, M5 переделает).

- [ ] **Step 3: Commit фиксов**

Любые баги – incremental commits.

- [ ] **Step 4: CLAUDE.md + push**

```bash
# обновить Milestones
git commit -am "docs: M4 done"
git push -u origin feat/m4-home-categories-world
```

---

## Acceptance Criteria

1. ✅ HomeScreen загружается после логина на `/home`, показывает 8 mock-карточек и 12 пустых слотов для комнат.
2. ✅ Клик "Создать комнату" открывает `/create` в новом табе.
3. ✅ CategoriesScreen – 7 тайлов в первом ряду и 6 во втором, активен только "Онлайн карта мира".
4. ✅ Toast показывается при клике на любой disabled-тайл и исчезает через 2.2 секунды.
5. ✅ Переход в 3D-мир работает, загружаются ≥ 13 камер-маркеров.
6. ✅ Клик на маркер → открывается CameraDetailPanel, в 3D-плоскости начинает играть HLS-поток (5-10 секунд до первого кадра).
7. ✅ Кнопка "Смотреть вместе" переводит на `/create?camera=<id>`.
8. ✅ Переключение между табами не роняет приложение; 3D-canvas в неактивном табе не рендерит (CSS hidden).
9. ✅ Старые файлы `src/screens/HomeScreen.tsx`, `src/screens/ExploreScreen.tsx`, `src/screens/World3DScreen.tsx`, `src/hooks/useNearbyCameras.ts` удалены.
10. ✅ `npm run build` зелёный, `cargo test --workspace --lib` зелёный.

---

## Дальше

М5: Rooms + WebSocket + chat. Создание реальной комнаты, синхронизация камеры, чат, счётчик viewers.
