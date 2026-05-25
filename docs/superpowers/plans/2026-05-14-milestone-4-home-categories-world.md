# Milestone 4: Home + Categories + World3D Integration Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Переписать Home и Categories по Figma, интегрировать существующий World3DScreen в shell, переключить 3D-мир на реальные камеры из API. В конце milestone пользователь проходит Main → Categories → Онлайн карта мира → клик маркер → видит живой HLS-стрим в 3D.

**Architecture:**
- `HomeScreen` (по Figma): аватар пользователя, "Открытые комнаты" (горизонтальные кружки с + и аватаром), "Популярное сейчас" (сетка карточек), боковой блок с быстрыми ссылками (Видеоигры, Фильмы, Стримы вне дома, Бравл старс – в альфе просто текст). Данные загружаются параллельно: комнаты через `GET /rooms?open=true` (эндпоинт подготовим-заглушку в M4, реализация роллирующих данных – M5), популярные комнаты – placeholder список, фактический сигнал "сколько людей смотрит" берётся из viewer-count эндпоинта в М5.
- `CategoriesScreen` (по Figma): тайлы с разными категориями. В альфе работает один – "Онлайн карта мира", остальные – disabled-tile с toast "Скоро".
- `World3DScreen` остаётся как сейчас – `<Canvas>` живёт **внутри** экрана. `GlobalCanvas` остаётся stub-ом (deferred decision: portal-based виртуализация откладывается до M7+, см. Task 2).
- **Камеры в 3D**: `useCamerasFromApi()` из M3 заменяет `useNearbyCameras` mock. Адаптер `cameraResponseToCamera3D()` маппит `CameraResponse` (плоские поля) в `Camera3D` (с `orientation`-объектом и `hlsUrl`), чтобы `WorldScene` остался без изменений сигнатуры.
- **HLS-плеер в 3D**: URL берётся из `CameraResponse.proxyManifestUrl` (`/api/v1/hls-proxy/:slug`), `CameraFrustum` его получает через `Camera3D.hlsUrl` как раньше.

**Note on existing types:** план опирается на реальный ts-rs тип `CameraResponse` (после M3) – `CameraSummary` упоминается в более ранних драфтах, но такого типа в `src/types/api/` нет.

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
- `src/screens/world/world.css`
- `src/components/ui/Toast.tsx` + `src/components/ui/toast.css`
- `src/api/camera3d.ts` – адаптер `cameraResponseToCamera3D(c: CameraResponse): Camera3D`.

### Modify

- `src/shell/routes.tsx` – `/home` → `HomeScreen`, `/categories` → `CategoriesScreen`, `/world` → новый `World3DScreen`, убрать старый импорт.
- `src/App.tsx` – добавить `ToastProvider` рядом с `ThemeProvider`.
- `src/i18n/locales/ru.json` – ключи home/categories/world.
- `docs/superpowers/specs/2026-05-14-alpha-design.md` – пункт в §10 про deferred GlobalCanvas virtualization.

### NOT modified (важно)

- `src/components/world3d/WorldScene.tsx` – остаётся 1:1 как сейчас (принимает `Camera3D`/`Camera3D[]`, своя `<Canvas>`). Адаптер на стороне screen-а конвертирует `CameraResponse` → `Camera3D`.
- `src/components/world3d/CameraFrustum.tsx` – тоже без изменений; HLS URL приходит в `Camera3D.hlsUrl`, который заполняется адаптером из `CameraResponse.proxyManifestUrl`.
- `src/shell/scene3d/GlobalCanvas.tsx` – остаётся пустым stub-ом (Task 2 фиксирует это решение).
- `src/hooks/useCamerasFromApi.ts` – уже создан в M3, в M4 только используется.

### Delete

- `src/screens/World3DScreen.tsx` (перенесли в `screens/world/`)
- `src/screens/HomeScreen.tsx` (старый прототип, заменяется на `screens/main/HomeScreen.tsx`)
- `src/screens/ExploreScreen.tsx` (заменяется на `screens/categories/CategoriesScreen.tsx`)
- `src/screens/main/HomePlaceholder.tsx`
- `src/hooks/useNearbyCameras.ts` (mock больше не нужен)
- `src/screens/WatchRoomScreen.tsx` старый – **не удаляем в M4**, это M5.

---

## Task 1: Ветка

Ветка `feat/m4-home-categories-world` уже создана от свежего main и содержит черри-пик `7c04a09` (docs про gh-PAT + YouTube anti-bot, остался от M3-PR). Этот коммит – первый в M4-ветке, дополнительных шагов не нужно.

- [ ] **Step 1: Sanity check**

```bash
git -C d:/Projects/Placebo branch --show-current
# expected: feat/m4-home-categories-world
git -C d:/Projects/Placebo log --oneline origin/main..HEAD
# expected: 70277ad docs: note gh-PAT scope + YouTube anti-bot...
```

---

## Task 2: GlobalCanvas остаётся stub-ом + spec note

**Решение:** в альфе НЕ виртуализируем 3D-canvas через portal. Canvas живёт внутри `World3DScreen` (как сейчас). Когда таб неактивен – таб скрыт через CSS, R3F автоматически останавливает render-loop. Открытие двух табов с `/world` одновременно даст два WebGL-контекста; считаем это acceptable для альфы (в M7 пересмотрим).

**Files:**
- Modify: `src/shell/scene3d/GlobalCanvas.tsx` (если ещё не stub – привести к stub-у)
- Modify: `docs/superpowers/specs/2026-05-14-alpha-design.md` (Deferred decisions §10)

- [ ] **Step 1: Убедиться что `GlobalCanvas` – stub**

Читаем текущий файл. Если он уже `export default function GlobalCanvas() { return null; }` – ничего не трогаем. Если там осталась попытка реальной реализации, сводим к:

```tsx
// src/shell/scene3d/GlobalCanvas.tsx
/**
 * Stub. Portal-based 3D canvas virtualization is a deferred decision
 * (alpha-design.md §10). For the alpha, each World3DScreen owns its
 * own R3F <Canvas> directly; this component intentionally renders
 * nothing.
 */
export default function GlobalCanvas() {
  return null;
}
```

- [ ] **Step 2: Добавить пункт в Deferred Decisions спеки**

В `docs/superpowers/specs/2026-05-14-alpha-design.md` в раздел 10 (Deferred Decisions) добавить пункт:

> **3D-виртуализация через GlobalCanvas + React portal.** В альфе сцены живут внутри своих screen-компонентов (классический R3F `<Canvas>`). Portal-based virtualization отложена до M7 или пост-альфы; ограничение – не открывать два таба с `/world` одновременно (получим два WebGL-контекста).

- [ ] **Step 3: Commit**

```bash
git add src/shell/scene3d/GlobalCanvas.tsx docs/superpowers/specs/2026-05-14-alpha-design.md
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
git add src/screens/main/HomeScreen.tsx src/screens/main/home.css \
        src/shell/routes.tsx \
        src/i18n/locales/ru.json
git add -u src/screens/main/HomePlaceholder.tsx
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
          <button
            key={tile.key}
            className={`cats__tile cats__tile--${tile.variant}${tile.enabled ? "" : " cats__tile--disabled"}`}
            onClick={() => click(tile)}
            aria-disabled={!tile.enabled}
          >
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

/* Alpha layout: 7 tiles in a responsive grid without named areas.
   Hero tiles span 2 columns / 2 rows, small ones – 1 cell. The Figma
   mock uses irregular sizes, but a uniform grid is good enough for the
   alpha and saves us the named-areas hack. */
.cats__world {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
}
.cats__tile--hero-sky,
.cats__tile--hero-blue,
.cats__tile--hero-orange { grid-column: span 2; grid-row: span 2; }
@media (max-width: 1100px) {
  .cats__world { grid-template-columns: repeat(2, 1fr); }
  .cats__tile--hero-sky,
  .cats__tile--hero-blue,
  .cats__tile--hero-orange { grid-column: span 2; grid-row: auto; }
}

.cats__chill { display: grid; grid-template-columns: repeat(6, 1fr); gap: 16px; }
@media (max-width: 1100px) { .cats__chill { grid-template-columns: repeat(3, 1fr); } }

.cats__tile {
  min-height: 140px; border-radius: 16px; border: 0; padding: 16px;
  cursor: pointer; display: flex; align-items: flex-end; text-align: left;
  color: var(--t1); font-weight: 700; font-size: 18px; overflow: hidden; position: relative;
  transition: transform 120ms ease;
}
.cats__tile:hover:not(.cats__tile--disabled) { transform: translateY(-2px); }
.cats__tile--disabled { cursor: pointer; opacity: 0.85; }
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

Удаляем старый `ExploreScreen` и стейджим только то, что относится к этому таску – никакого `git add -A`.

```bash
rm d:/Projects/Placebo/src/screens/ExploreScreen.tsx
git add src/screens/categories/ \
        src/components/ui/Toast.tsx src/components/ui/toast.css \
        src/App.tsx src/App.css \
        src/shell/routes.tsx \
        src/i18n/locales/ru.json
git add -u src/screens/ExploreScreen.tsx
git commit -m "feat(categories): Categories screen per Figma, toast for disabled tiles"
```

---

## Task 5: World3DScreen в shell

**Files:**
- Create: `src/api/camera3d.ts` (адаптер `CameraResponse → Camera3D`).
- Create: `src/screens/world/World3DScreen.tsx` + `world.css`.
- Create: `src/screens/world/CameraDetailPanel.tsx`.
- Modify: `src/shell/routes.tsx` (новый импорт, убрать `onBack`-проп).
- Modify: `src/i18n/locales/ru.json`.
- Delete: `src/screens/World3DScreen.tsx`, `src/hooks/useNearbyCameras.ts`.

**НЕ модифицируем:** `src/components/world3d/WorldScene.tsx`, `src/components/world3d/CameraFrustum.tsx` – остаются 1:1 как сейчас.

- [ ] **Step 1: Адаптер `cameraResponseToCamera3D`**

`WorldScene` принимает локальный тип `Camera3D` (с группой `orientation`, скаляром `heightAboveGround`, полем `hlsUrl`). API даёт плоский `CameraResponse`. Не меняем сигнатуру `WorldScene` – пишем тонкий адаптер.

`src/api/camera3d.ts`:

```ts
import type { CameraResponse } from "../types/api/CameraResponse";
import type { Camera3D } from "../types/world3d";
import { DEFAULT_ORIENTATION } from "../types/world3d";

/**
 * Map the API camera DTO to the local Camera3D type used by WorldScene.
 * Fallbacks for orientation / height come from DEFAULT_ORIENTATION (5m / 0° / -15° / 90° / 58°),
 * matching the legacy mock so visuals do not regress when seed data omits these fields.
 */
export function cameraResponseToCamera3D(c: CameraResponse): Camera3D {
  // Build the absolute HLS URL. proxyManifestUrl is server-relative ("/api/v1/hls-proxy/<slug>"),
  // and in dev the Vite proxy forwards /api → http://localhost:3001 transparently.
  // In Tauri prod we will resolve via VITE_API_BASE_URL.
  const base = (import.meta.env.VITE_API_BASE_URL as string | undefined) ?? "";
  const hlsUrl = c.proxyManifestUrl
    ? base
      ? `${base.replace(/\/$/, "")}${c.proxyManifestUrl.replace(/^\/api\/v1/, "")}`
      : c.proxyManifestUrl
    : null;

  return {
    id: c.id,
    name: c.name,
    slug: c.slug,
    lat: c.lat,
    lng: c.lng,
    category: c.category,
    heightAboveGround: c.heightAboveGround ?? 5,
    orientation: {
      azimuth: c.cameraAzimuth ?? DEFAULT_ORIENTATION.azimuth,
      elevation: c.cameraElevation ?? DEFAULT_ORIENTATION.elevation,
      fovHorizontal: c.fovHorizontal ?? DEFAULT_ORIENTATION.fovHorizontal,
      fovVertical: c.fovVertical ?? DEFAULT_ORIENTATION.fovVertical,
    },
    hlsUrl,
    thumbnailUrl: c.thumbnailUrl ?? null,
    isOnline: true,            // M5: real signal from /cameras/:id/health.
    viewersNow: 0,             // M5: real viewer count via Redis hook.
  };
}
```

**Важно:** `VITE_API_BASE_URL` в dev, скорее всего, не задан, и Vite-прокси ловит `/api` → `http://localhost:3001`. Тогда `proxyManifestUrl` (вида `/api/v1/hls-proxy/yt-shibuya-crossing`) уйдёт прямо в `<video>.src` – это работает. Если `VITE_API_BASE_URL=http://localhost:3001` задан, мы убираем `/api/v1` и подменяем им хост, чтобы избежать двойного `/api/v1`. CameraFrustum и его манипуляции `hls.js` менять не надо.

- [ ] **Step 2: Новый World3DScreen**

```tsx
// src/screens/world/World3DScreen.tsx
import { useEffect, useMemo, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { WorldScene } from "../../components/world3d/WorldScene";
import CameraDetailPanel from "./CameraDetailPanel";
import { useCamerasFromApi } from "../../hooks/useCamerasFromApi";
import { cameraResponseToCamera3D } from "../../api/camera3d";
import "./world.css";

export default function World3DScreen() {
  const { data, loading, error } = useCamerasFromApi();
  const { id } = useParams();
  const nav = useNavigate();
  const { t } = useTranslation();
  const [activeId, setActiveId] = useState<string | null>(null);

  const cameras = useMemo(() => (data ?? []).map(cameraResponseToCamera3D), [data]);

  // Sync the URL param (slug or id) with active state on mount/data change.
  useEffect(() => {
    if (!cameras.length) return;
    if (id) {
      const found = cameras.find((c) => c.slug === id || c.id === id);
      if (found) {
        setActiveId(found.id);
        return;
      }
    }
    setActiveId((prev) => prev ?? cameras[0].id);
  }, [cameras, id]);

  if (loading) return <div className="world-screen world-screen--loading">{t("world.loading")}</div>;
  if (error) return <div className="world-screen world-screen--error">{t("world.error", { msg: error.message })}</div>;
  if (!cameras.length) return <div className="world-screen world-screen--loading">{t("world.empty")}</div>;

  const active = cameras.find((c) => c.id === activeId) ?? cameras[0];

  return (
    <div className="world-screen">
      <WorldScene
        activeCamera={active}
        nearbyCameras={cameras}
        onCameraSelect={(cam) => {
          setActiveId(cam.id);
          nav(`/world/${cam.slug}`, { replace: true });
        }}
        timezone={"UTC"}
      />
      <CameraDetailPanel
        camera={active}
        onClose={() => nav("/world", { replace: true })}
      />
    </div>
  );
}
```

`world.css`:

```css
.world-screen { position: relative; width: 100%; height: 100%; background: #0a0a0f; overflow: hidden; }
.world-screen--loading,
.world-screen--error { display: grid; place-items: center; color: var(--t2); padding: 32px; }
.world-screen--error { color: #D12850; }
```

**Important:** `WorldScene` мы НЕ меняем. Он уже принимает `activeCamera`, `nearbyCameras`, `onCameraSelect` именно с такими именами и сам обрабатывает `cam.id !== activeCamera.id` фильтрацию. Просмотрев исходник до начала имплементации, имена пропсов сверить ещё раз.

- [ ] **Step 3: CameraDetailPanel**

```tsx
// src/screens/world/CameraDetailPanel.tsx
import type { Camera3D } from "../../types/world3d";
import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";

type Props = { camera: Camera3D; onClose: () => void };

export default function CameraDetailPanel({ camera, onClose }: Props) {
  const { t } = useTranslation();
  const nav = useNavigate();

  const watchTogether = () => {
    // M5 will create a real room via API; for now route to the existing CreateScreen with camera id.
    nav(`/create?camera=${camera.id}`);
  };

  return (
    <div className="world-panel">
      <div className="world-panel__head">
        <div>
          <div className="world-panel__title">{camera.name}</div>
          <div className="world-panel__subtitle">{camera.category}</div>
        </div>
        <button className="world-panel__close" onClick={onClose} aria-label={t("world.close")}>✕</button>
      </div>
      <button className="world-panel__watch" onClick={watchTogether}>
        {t("world.watch_together")}
      </button>
    </div>
  );
}
```

Панель использует `Camera3D` (после адаптации). `city`/`country` тут нет – мы их потеряли в адаптере; если хочется показывать локацию, расширим `Camera3D` отдельным полем `location?: string` в M5. Для альфы достаточно `name` + `category`.

CSS дополнить в `world.css`:

```css
.world-panel {
  position: absolute; top: 24px; right: 24px;
  width: 320px; background: var(--bg); color: var(--t1);
  border-radius: 16px; border: 1px solid var(--border);
  padding: 16px; box-shadow: 0 10px 30px rgba(0,0,0,0.25);
  z-index: 10;
}
.world-panel__head { display: flex; justify-content: space-between; align-items: flex-start; gap: 12px; }
.world-panel__title { font-size: 18px; font-weight: 700; }
.world-panel__subtitle { color: var(--t2); font-size: 13px; text-transform: capitalize; }
.world-panel__close {
  background: transparent; border: 0; color: var(--t2);
  font-size: 18px; cursor: pointer; padding: 4px 8px;
}
.world-panel__watch {
  margin-top: 16px; width: 100%; padding: 12px;
  background: var(--accent); color: #fff; border: 0; border-radius: 10px;
  cursor: pointer; font-weight: 600;
}
```

- [ ] **Step 4: Route**

`src/shell/routes.tsx`:

```tsx
import World3DScreen from "../screens/world/World3DScreen";
// убрать старый импорт `import World3DScreen from "../screens/World3DScreen"`
// убрать `onBack`-проп
{ path: "/world", element: guarded(<World3DScreen />) },
{ path: "/world/:id", element: guarded(<World3DScreen />) },
```

- [ ] **Step 5: i18n**

`src/i18n/locales/ru.json`:

```json
"world.watch_together": "Смотреть вместе",
"world.close": "Закрыть",
"world.loading": "Загружаем камеры...",
"world.empty": "Нет камер для отображения",
"world.error": "Не удалось загрузить камеры: {{msg}}"
```

- [ ] **Step 6: Удалить мёртвые файлы**

```bash
rm d:/Projects/Placebo/src/screens/World3DScreen.tsx
rm d:/Projects/Placebo/src/hooks/useNearbyCameras.ts
# Проверить, что никто не ссылается:
grep -rn "useNearbyCameras\|screens/World3DScreen" d:/Projects/Placebo/src/
# expected: пусто
```

- [ ] **Step 7: Commit**

```bash
git add src/api/camera3d.ts \
        src/screens/world/ \
        src/shell/routes.tsx \
        src/i18n/locales/ru.json
git add -u src/screens/World3DScreen.tsx src/hooks/useNearbyCameras.ts
git commit -m "$(cat <<'EOF'
feat(world): World3DScreen driven by /cameras API + HLS proxy

- New cameraResponseToCamera3D adapter maps the API DTO to the local
  Camera3D type WorldScene expects, leaving WorldScene untouched.
- World3DScreen now lives under src/screens/world/, reads /cameras via
  useCamerasFromApi, and syncs the active camera with the /world/:slug URL.
- CameraDetailPanel shows name + category and routes "Watch Together"
  to /create?camera=<id> until M5 lands real room creation.
- HLS playback flows through the M3 axum proxy via CameraResponse.proxyManifestUrl
  → Camera3D.hlsUrl; CameraFrustum is unchanged.
- Legacy useNearbyCameras mock and the old src/screens/World3DScreen.tsx
  are removed.

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
