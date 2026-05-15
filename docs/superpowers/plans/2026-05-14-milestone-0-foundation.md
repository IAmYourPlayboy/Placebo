# Milestone 0: Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Подготовить чистую базу для дальнейшей работы: удалить мёртвый код, настроить генерацию TypeScript-типов из Rust, инфраструктуру i18n, систему тем через CSS-переменные, реорганизовать папки под новый shell.

**Architecture:** Никаких продуктовых фич. Только подготовка инструментов и фундамента. В конце milestone приложение **всё ещё запускается и работает как раньше**, но внутри есть готовые каналы для всех последующих milestone'ов.

**Tech Stack:** ts-rs (Rust → TS codegen), react-i18next, CSS variables, Tauri 2.

**Spec reference:** `docs/superpowers/specs/2026-05-14-alpha-design.md`, разделы 3.5 (темы), 3.6 (i18n), 4.2 (типогенерация), 7.5 (IPC cleanup).

**Общее правило:** после каждой задачи приложение должно запускаться. Если `npm run dev` или `cargo check` падает – не двигаемся дальше.

---

## File Map

### Создаются

- `src/i18n/index.ts` – инициализация react-i18next.
- `src/i18n/locales/ru.json` – русские переводы (первичный файл).
- `src/theme/ThemeProvider.tsx` – React context для темы.
- `src/theme/useTheme.ts` – хук для доступа к теме.
- `src/theme/variables.css` – CSS-переменные для обеих тем.
- `src/types/api/.gitkeep` – папка для авто-генерируемых типов.
- `crates/placebo-shared/Cargo.toml` (обновить) – добавить ts-rs feature.
- `crates/placebo-shared/src/codegen_tests.rs` – тест-ворота экспорта типов.
- `scripts/gen-types.sh` + `scripts/gen-types.ps1` – обёртки для npm-скрипта.
- `.gitignore` – добавить исключения для авто-генерируемых файлов.

### Модифицируются

- `src-tauri/src/lib.rs` – удалить legacy-комментарии если есть, оставить чистую регистрацию IPC.
- `src-tauri/src/main.rs` – только entry point, без дубликатов.
- `src/App.tsx` – обернуть в `ThemeProvider` + `I18nextProvider`, убрать логику screen-switching (она временно остаётся, но Shell придёт в M1).
- `src/App.css` – перенести цвета в `src/theme/variables.css`, здесь оставить только layout.
- `package.json` – добавить зависимости, скрипты `gen-types`, `predev`, `prebuild`.
- `crates/placebo-shared/src/lib.rs` – подключить `codegen_tests`.

### Удаляются

- `src/components/BottomNav.tsx` – скрыт CSS, не нужен в десктоп-альфе.
- Legacy-экраны НЕ трогаем в M0 (их переписывают в M2-M6), только `BottomNav` как очевидный мусор.

---

## Task 1: Git worktree и стартовая ветка

**Files:** `.git/` (ветка)

- [ ] **Step 1: Создать ветку для milestone 0**

```bash
git -C d:/Projects/Placebo checkout main
git -C d:/Projects/Placebo pull
git -C d:/Projects/Placebo checkout -b feat/m0-foundation
```

Esли на ветке `feat/auth-system` ещё не смердж-реквест был – сначала создать PR и дождаться мёрджа, но в альфа-режиме можно ветвиться от `feat/auth-system`:

```bash
git -C d:/Projects/Placebo checkout feat/auth-system
git -C d:/Projects/Placebo checkout -b feat/m0-foundation
```

Expected: `git branch --show-current` → `feat/m0-foundation`.

- [ ] **Step 2: Проверить, что `npm run dev` стартует**

```bash
cd d:/Projects/Placebo
npm run dev
```

Expected: Vite поднимается на `http://localhost:1420`, никаких ошибок в консоли. Остановить (`Ctrl+C`).

- [ ] **Step 3: Проверить, что `cargo check` проходит**

```bash
cd d:/Projects/Placebo
cargo check --workspace
```

Expected: `Finished dev [unoptimized + debuginfo] target(s)` без ошибок.

---

## Task 2: Удалить BottomNav (очевидный мусор)

**Files:**
- Delete: `src/components/BottomNav.tsx`
- Modify: `src/App.tsx` (убрать импорт и использование)

- [ ] **Step 1: Проверить, где используется BottomNav**

```bash
grep -rn "BottomNav" d:/Projects/Placebo/src/
```

Expected: только `App.tsx` импортирует и рендерит.

- [ ] **Step 2: Удалить файл**

```bash
rm d:/Projects/Placebo/src/components/BottomNav.tsx
```

- [ ] **Step 3: Убрать импорт и использование из `src/App.tsx`**

Заменить начало `src/App.tsx`, удалив строку `import BottomNav`:

```tsx
import { useState } from "react";
import HomeScreen from "./screens/HomeScreen";
import ExploreScreen from "./screens/ExploreScreen";
import CreateScreen from "./screens/CreateScreen";
import FriendsScreen from "./screens/FriendsScreen";
import ProfileScreen from "./screens/ProfileScreen";
import WatchRoomScreen from "./screens/WatchRoomScreen";
import World3DScreen from "./screens/World3DScreen";
```

Убрать из JSX:

```tsx
// было:
//   <BottomNav active={screen} onChange={setScreen} />
// убрать целиком эту строку
```

В `src/App.tsx` временная заглушка навигации (альфа-shell придёт в M1). Пока оставляем экраны доступными через `useState`, но без BottomNav – значит, пока видно только `home`. Это нормально для M0, shell в M1 всё перепишет.

- [ ] **Step 4: Проверить сборку**

```bash
npm run dev
```

Expected: открывается Home, никаких ошибок в консоли.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "$(cat <<'EOF'
chore: remove unused BottomNav (hidden on desktop, replaced by sidebar in M1)

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: Добавить зависимости для i18n и темы

**Files:**
- Modify: `package.json`

- [ ] **Step 1: Установить npm-пакеты**

```bash
cd d:/Projects/Placebo
npm install i18next react-i18next i18next-browser-languagedetector
```

Expected: `package.json` содержит новые dependencies, никаких peer-warnings критичных уровней.

- [ ] **Step 2: Проверить версии**

Открыть `package.json` и убедиться, что версии: `i18next` ^23 или ^24, `react-i18next` ^14 или ^15, `i18next-browser-languagedetector` ^8.

- [ ] **Step 3: Commit**

```bash
git add package.json package-lock.json
git commit -m "$(cat <<'EOF'
chore(deps): add i18next + react-i18next + languagedetector

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: Настроить CSS-переменные для тем

**Files:**
- Create: `src/theme/variables.css`
- Modify: `src/App.css` (импорт variables.css в самом верху)

- [ ] **Step 1: Создать папку theme**

```bash
mkdir -p d:/Projects/Placebo/src/theme
```

- [ ] **Step 2: Создать `src/theme/variables.css`**

```css
/*
 * Placebo theme variables.
 * Two themes: light and dark. "auto" is runtime-selected via
 * prefers-color-scheme and resolves to one of the two below.
 */

:root[data-theme="light"] {
  /* Brand */
  --accent: #E8345A;
  --accent-hover: #D12850;

  /* Surfaces */
  --bg:    #FFFFFF;
  --bg-2:  #F5F5F7;
  --bg-3:  #EBEBEB;

  /* Text */
  --t1: #0F0F0F;
  --t2: #444444;
  --t3: #999999;

  /* Borders and dividers */
  --border: #E8E8EC;

  /* Shell metrics */
  --sidebar-w: 224px;
  --topbar-h: 56px;
  --tabbar-h: 36px;

  /* 3D scene */
  --scene-bg: #0a0a0f;
  --scene-fog: #0a0a1a;
  --wire-edge: #1e2332;
  --wire-fill: #0f121c;

  /* Category marker palette */
  --cat-city:    #E8345A;
  --cat-traffic: #FF9500;
  --cat-nature:  #34C759;
}

:root[data-theme="dark"] {
  --accent: #FF4A6B;
  --accent-hover: #E8345A;

  --bg:    #0F0F10;
  --bg-2:  #1A1A1C;
  --bg-3:  #262628;

  --t1: #F5F5F7;
  --t2: #BEBEC2;
  --t3: #78787C;

  --border: #2A2A2E;

  --sidebar-w: 224px;
  --topbar-h: 56px;
  --tabbar-h: 36px;

  --scene-bg: #000006;
  --scene-fog: #050511;
  --wire-edge: #3A4058;
  --wire-fill: #1A1D2B;

  --cat-city:    #FF4A6B;
  --cat-traffic: #FFAA33;
  --cat-nature:  #4ED87A;
}
```

- [ ] **Step 3: Импортировать variables.css в App.css**

В самом верху `src/App.css` добавить первой строкой:

```css
@import "./theme/variables.css";
```

- [ ] **Step 4: Переместить существующие CSS-переменные в variables.css**

Открыть `src/App.css` и найти блок `:root { --accent: ...; }`. Если переменные уже перечислены в variables.css – удалить их из `App.css`, чтобы не дублировались. Любые цвета/метрики, которых нет в variables.css, – перенести туда в нужный theme-блок (по умолчанию в `[data-theme="light"]`).

- [ ] **Step 5: Установить тему по умолчанию в `index.html`**

В `index.html` добавить в `<html>` атрибут:

```html
<html lang="ru" data-theme="light">
```

- [ ] **Step 6: Проверить, что приложение запускается и визуально не сломано**

```bash
npm run dev
```

Expected: ничего не изменилось визуально.

- [ ] **Step 7: Commit**

```bash
git add src/theme/variables.css src/App.css index.html
git commit -m "$(cat <<'EOF'
feat(theme): extract CSS variables into theme/variables.css

Adds light and dark theme tokens under [data-theme] attributes.
The HTML root has data-theme="light" by default; ThemeProvider
(next task) wires this up to user preference and system setting.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: ThemeProvider и useTheme хук

**Files:**
- Create: `src/theme/ThemeProvider.tsx`
- Create: `src/theme/useTheme.ts`
- Create: `src/theme/index.ts`
- Modify: `src/App.tsx` (обернуть в ThemeProvider)

- [ ] **Step 1: Написать тип темы и константы**

Создать `src/theme/index.ts`:

```ts
export type ThemeChoice = "light" | "dark" | "auto";
export type ResolvedTheme = "light" | "dark";

export const THEME_STORAGE_KEY = "placebo.theme";
export const DEFAULT_THEME: ThemeChoice = "auto";

export { ThemeProvider } from "./ThemeProvider";
export { useTheme } from "./useTheme";
```

- [ ] **Step 2: Написать ThemeProvider**

Создать `src/theme/ThemeProvider.tsx`:

```tsx
import { createContext, useEffect, useMemo, useState, ReactNode } from "react";
import type { ThemeChoice, ResolvedTheme } from "./index";
import { DEFAULT_THEME, THEME_STORAGE_KEY } from "./index";

type ThemeContextValue = {
  choice: ThemeChoice;
  resolved: ResolvedTheme;
  setChoice: (c: ThemeChoice) => void;
};

export const ThemeContext = createContext<ThemeContextValue | null>(null);

function resolveChoice(choice: ThemeChoice): ResolvedTheme {
  if (choice === "auto") {
    const prefersDark = typeof window !== "undefined"
      && window.matchMedia("(prefers-color-scheme: dark)").matches;
    return prefersDark ? "dark" : "light";
  }
  return choice;
}

function loadChoice(): ThemeChoice {
  if (typeof window === "undefined") return DEFAULT_THEME;
  const stored = window.localStorage.getItem(THEME_STORAGE_KEY);
  if (stored === "light" || stored === "dark" || stored === "auto") return stored;
  return DEFAULT_THEME;
}

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [choice, setChoiceState] = useState<ThemeChoice>(() => loadChoice());
  const [resolved, setResolved] = useState<ResolvedTheme>(() => resolveChoice(choice));

  useEffect(() => {
    setResolved(resolveChoice(choice));
  }, [choice]);

  // Respond to system theme changes while in "auto"
  useEffect(() => {
    if (choice !== "auto") return;
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const onChange = () => setResolved(mq.matches ? "dark" : "light");
    mq.addEventListener("change", onChange);
    return () => mq.removeEventListener("change", onChange);
  }, [choice]);

  // Sync to <html data-theme=...>
  useEffect(() => {
    document.documentElement.setAttribute("data-theme", resolved);
  }, [resolved]);

  const setChoice = (c: ThemeChoice) => {
    setChoiceState(c);
    window.localStorage.setItem(THEME_STORAGE_KEY, c);
  };

  const value = useMemo(() => ({ choice, resolved, setChoice }), [choice, resolved]);
  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
}
```

- [ ] **Step 3: Написать useTheme**

Создать `src/theme/useTheme.ts`:

```ts
import { useContext } from "react";
import { ThemeContext } from "./ThemeProvider";

export function useTheme() {
  const ctx = useContext(ThemeContext);
  if (!ctx) throw new Error("useTheme must be used within <ThemeProvider>");
  return ctx;
}
```

- [ ] **Step 4: Обернуть приложение в ThemeProvider**

В `src/App.tsx` добавить импорт и обёртку в самом верху компонента:

```tsx
import { ThemeProvider } from "./theme";

export default function App() {
  return (
    <ThemeProvider>
      <AppInner />
    </ThemeProvider>
  );
}

function AppInner() {
  // ... existing body of old App component
}
```

Текущий body App становится `AppInner`, чтобы хуки темы были доступны для детей.

- [ ] **Step 5: Дымовой тест**

```bash
npm run dev
```

Открыть DevTools → Console, запустить:

```js
document.documentElement.setAttribute("data-theme", "dark");
```

Expected: при наличии дизайна с `var(--bg)` что-то визуально меняется. Если пока вся вёрстка на хардкод-цветах – это нормально, будем переводить на variables в M1.

Вернуть:

```js
document.documentElement.setAttribute("data-theme", "light");
```

- [ ] **Step 6: Commit**

```bash
git add src/theme/ src/App.tsx
git commit -m "$(cat <<'EOF'
feat(theme): add ThemeProvider with light/dark/auto support

- React context wraps the whole app, persists choice in localStorage.
- "auto" follows prefers-color-scheme and reacts to system changes.
- Resolved theme is synced to <html data-theme=...> so variables.css
  picks the right token set.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 6: Инфраструктура i18n

**Files:**
- Create: `src/i18n/index.ts`
- Create: `src/i18n/locales/ru.json`
- Create: `src/i18n/locales/en.json` (пустая, заготовка)
- Modify: `src/main.tsx` (импорт `./i18n`)

- [ ] **Step 1: Создать стартовый словарь**

Создать `src/i18n/locales/ru.json`:

```json
{
  "app.loading": "Загрузка...",
  "app.error.generic": "Что-то пошло не так"
}
```

Создать `src/i18n/locales/en.json`:

```json
{
  "app.loading": "Loading...",
  "app.error.generic": "Something went wrong"
}
```

- [ ] **Step 2: Настроить i18next**

Создать `src/i18n/index.ts`:

```ts
import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import LanguageDetector from "i18next-browser-languagedetector";
import ru from "./locales/ru.json";
import en from "./locales/en.json";

i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources: {
      ru: { translation: ru },
      en: { translation: en },
    },
    fallbackLng: "ru",
    supportedLngs: ["ru", "en"],
    interpolation: { escapeValue: false },
    detection: {
      order: ["localStorage", "navigator"],
      lookupLocalStorage: "placebo.lang",
      caches: ["localStorage"],
    },
  });

export default i18n;
```

- [ ] **Step 3: Подключить в main**

В `src/main.tsx` добавить импорт до `ReactDOM.render`:

```ts
import "./i18n";
```

- [ ] **Step 4: Смоук-тест**

В любом экране (например, `HomeScreen.tsx`) временно добавить:

```tsx
import { useTranslation } from "react-i18next";

export default function HomeScreen(props: any) {
  const { t } = useTranslation();
  return (
    <div>
      <p>{t("app.loading")}</p>
      {/* ... existing content ... */}
    </div>
  );
}
```

```bash
npm run dev
```

Expected: видно текст "Загрузка..." из ru.json. Потом **откатить этот смоук-тест**:

```bash
git checkout src/screens/HomeScreen.tsx
```

- [ ] **Step 5: Commit**

```bash
git add src/i18n/ src/main.tsx
git commit -m "$(cat <<'EOF'
feat(i18n): set up react-i18next with ru as primary language

- Flat key structure (no nesting) for easier search&replace.
- Language detector: localStorage ("placebo.lang") > navigator.
- Starter dictionary limited to app-level keys; screens will add keys
  as they are rewritten in later milestones.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 7: Настроить ts-rs в placebo-shared

**Files:**
- Modify: `crates/placebo-shared/Cargo.toml`
- Modify: `crates/placebo-shared/src/lib.rs`
- Create: `crates/placebo-shared/src/codegen.rs`
- Create: `.gitignore` запись
- Modify: существующие структуры в `auth.rs`, `camera.rs`, `user.rs` и т.д. получат derive(`TS`) в M2-M5 когда до них дойдёт очередь. В M0 только настройка машины.

- [ ] **Step 1: Добавить ts-rs в зависимости**

Открыть `crates/placebo-shared/Cargo.toml` и в секцию `[dependencies]` добавить:

```toml
ts-rs = { version = "10", features = ["serde-compat", "chrono-impl", "uuid-impl"] }
```

Добавить секцию features (если её нет):

```toml
[features]
export-types = []
```

- [ ] **Step 2: Написать test-ворота экспорта**

Создать `crates/placebo-shared/src/codegen.rs`:

```rust
//! Codegen helper for ts-rs. Runs only under the `export-types` feature.
//!
//! All shared DTOs that need TypeScript bindings must derive `TS` and
//! invoke the `#[ts(export, export_to = "../../bindings/")]` attribute.
//! This module contains a single test that writes all registered types
//! to disk. A post-build step copies `bindings/` into `src/types/api/`.

#![cfg(feature = "export-types")]

// Currently there are no shared DTOs marked for export. When the first
// type with #[derive(TS)] is added (M2: auth), cargo test --features
// export-types will write it to crates/placebo-shared/bindings/.
```

- [ ] **Step 3: Зарегистрировать модуль**

В `crates/placebo-shared/src/lib.rs` добавить в самом верху:

```rust
#[cfg(feature = "export-types")]
pub mod codegen;
```

- [ ] **Step 4: Добавить bindings/ в .gitignore**

В корневом `.gitignore` добавить:

```
# ts-rs auto-generated bindings
crates/placebo-shared/bindings/
src/types/api/
```

**Важно:** `src/types/api/` также в gitignore, потому что скрипт gen-types копирует туда. Исходник правды – Rust-структуры, TypeScript – производный артефакт.

- [ ] **Step 5: Создать пустой скелет для src/types/api/**

```bash
mkdir -p d:/Projects/Placebo/src/types/api
echo "// Auto-generated from Rust structs via ts-rs. Do not edit." > d:/Projects/Placebo/src/types/api/README.md
```

`README.md` – не в gitignore (не подходит под паттерн `.ts`), чтобы папка существовала в git.

Перефиксировать .gitignore так, чтобы README оставался:

```
crates/placebo-shared/bindings/
src/types/api/*
!src/types/api/README.md
```

- [ ] **Step 6: Проверить, что cargo собирается**

```bash
cd d:/Projects/Placebo
cargo check --workspace
cargo check --workspace --features placebo-shared/export-types
```

Expected: обе команды проходят. Feature-флаг требует от caller'а явно указывать `-p placebo-shared`, проверим в следующем шаге.

- [ ] **Step 7: Проверить, что фича включается на уровне crate'а**

```bash
cargo check -p placebo-shared --features export-types
```

Expected: проходит без ошибок.

- [ ] **Step 8: Commit**

```bash
git add crates/placebo-shared/ .gitignore src/types/api/README.md
git commit -m "$(cat <<'EOF'
feat(codegen): wire up ts-rs for Rust -> TypeScript type generation

- Added ts-rs dependency with serde-compat, chrono-impl, uuid-impl.
- Introduced "export-types" cargo feature that activates the codegen
  module. No shared DTOs are exported yet; they are marked with
  #[derive(TS)] + #[ts(export)] as they are introduced in M2-M5.
- bindings/ and src/types/api/* are gitignored; the README anchors the
  destination directory in git.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 8: npm-скрипт gen-types и pre-hooks

**Files:**
- Create: `scripts/gen-types.sh`
- Create: `scripts/gen-types.ps1`
- Modify: `package.json`

- [ ] **Step 1: Написать bash-скрипт**

Создать `scripts/gen-types.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "[gen-types] running cargo test export_bindings ..."
cargo test -p placebo-shared --features export-types export_bindings -- --nocapture || true

SRC="$ROOT/crates/placebo-shared/bindings"
DST="$ROOT/src/types/api"

if [ -d "$SRC" ]; then
  echo "[gen-types] copying bindings -> $DST"
  mkdir -p "$DST"
  # Remove stale files first (but keep README.md)
  find "$DST" -type f -name "*.ts" -delete
  cp -R "$SRC"/. "$DST"/
else
  echo "[gen-types] no bindings/ yet, nothing to copy (OK on first run)."
fi

echo "[gen-types] done"
```

Сделать исполняемым:

```bash
chmod +x d:/Projects/Placebo/scripts/gen-types.sh
```

- [ ] **Step 2: Написать PowerShell-вариант**

Создать `scripts/gen-types.ps1`:

```powershell
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

Write-Host "[gen-types] running cargo test export_bindings ..."
cargo test -p placebo-shared --features export-types export_bindings -- --nocapture
if ($LASTEXITCODE -ne 0) { Write-Host "[gen-types] cargo test returned non-zero; continuing" }

$src = Join-Path $root "crates/placebo-shared/bindings"
$dst = Join-Path $root "src/types/api"

if (Test-Path $src) {
  Write-Host "[gen-types] copying bindings -> $dst"
  New-Item -ItemType Directory -Force -Path $dst | Out-Null
  Get-ChildItem $dst -Filter *.ts | Remove-Item -Force
  Copy-Item -Recurse -Force "$src\*" $dst
} else {
  Write-Host "[gen-types] no bindings/ yet, nothing to copy (OK on first run)."
}

Write-Host "[gen-types] done"
```

- [ ] **Step 3: Добавить npm-скрипты**

В `package.json` секцию `scripts` добавить:

```json
{
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "tauri": "tauri",
    "gen-types": "bash scripts/gen-types.sh",
    "gen-types:win": "powershell -ExecutionPolicy Bypass -File scripts/gen-types.ps1",
    "predev": "bash scripts/gen-types.sh",
    "prebuild": "bash scripts/gen-types.sh"
  }
}
```

**Примечание:** мы используем `bash` (в Windows доступен через Git Bash; у пользователя в окружении shell = bash по инструкции CLAUDE.md). Для чистого PowerShell-окружения есть `gen-types:win`.

- [ ] **Step 4: Запустить gen-types первый раз**

```bash
npm run gen-types
```

Expected: скрипт запускает `cargo test`, пишет "no bindings yet" (так как мы ещё не пометили ни одной структуры `#[derive(TS)]`), завершается кодом 0.

- [ ] **Step 5: Commit**

```bash
git add scripts/ package.json package-lock.json
git commit -m "$(cat <<'EOF'
build: add gen-types npm scripts (bash + powershell)

- scripts/gen-types.sh invokes cargo test --features export-types and
  copies crates/placebo-shared/bindings/ -> src/types/api/.
- predev and prebuild hooks run gen-types automatically so frontend
  always has fresh TS bindings.
- gen-types:win is a pure PowerShell fallback.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 9: Локальное хранилище настроек пользователя (Tauri SQLite)

**Files:**
- Create: `src-tauri/migrations/local/001_user_preferences.sql`
- Modify: `src-tauri/src/db/mod.rs`
- Create: `src-tauri/src/db/preferences.rs`
- Create: `src-tauri/src/commands/preferences.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Create: `src/services/preferences.ts`

Мы храним тему, язык, состояние табов локально. Shell'у в M1 это уже понадобится.

- [ ] **Step 1: Посмотреть существующую схему миграций в Tauri**

```bash
ls d:/Projects/Placebo/src-tauri/migrations/
```

Expected: видны `001_cameras.sql` и `002_full_schema.sql`. Поскольку мы уже имеем номера 001-002, **удобнее** перевести локальные миграции в отдельную директорию `migrations/local/`. Плюс этого: разделение "каталожной" и "персональной" структуры.

- [ ] **Step 2: Добавить таблицу user_preferences**

Создать `src-tauri/migrations/003_user_preferences.sql` (номер следующий за существующими):

```sql
CREATE TABLE IF NOT EXISTS user_preferences (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Seed defaults; INSERT OR IGNORE to avoid overwriting user choices.
INSERT OR IGNORE INTO user_preferences (key, value) VALUES
    ('theme', 'auto'),
    ('lang',  'ru');
```

- [ ] **Step 3: Модель и запросы**

Создать `src-tauri/src/db/preferences.rs`:

```rust
use sqlx::SqlitePool;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preference {
    pub key: String,
    pub value: String,
}

pub async fn get(pool: &SqlitePool, key: &str) -> Result<Option<String>, sqlx::Error> {
    let row = sqlx::query_scalar::<_, String>("SELECT value FROM user_preferences WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

pub async fn set(pool: &SqlitePool, key: &str, value: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO user_preferences (key, value, updated_at) \
         VALUES (?, ?, datetime('now')) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn all(pool: &SqlitePool) -> Result<Vec<Preference>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (String, String)>("SELECT key, value FROM user_preferences")
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().map(|(k, v)| Preference { key: k, value: v }).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn test_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::query("CREATE TABLE user_preferences (key TEXT PRIMARY KEY, value TEXT NOT NULL, updated_at TEXT)")
            .execute(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn set_then_get_returns_value() {
        let pool = test_pool().await;
        set(&pool, "theme", "dark").await.unwrap();
        assert_eq!(get(&pool, "theme").await.unwrap(), Some("dark".to_string()));
    }

    #[tokio::test]
    async fn set_overwrites_existing_key() {
        let pool = test_pool().await;
        set(&pool, "theme", "light").await.unwrap();
        set(&pool, "theme", "dark").await.unwrap();
        assert_eq!(get(&pool, "theme").await.unwrap(), Some("dark".to_string()));
    }

    #[tokio::test]
    async fn get_unknown_key_returns_none() {
        let pool = test_pool().await;
        assert_eq!(get(&pool, "nope").await.unwrap(), None);
    }
}
```

- [ ] **Step 4: Подключить модуль**

В `src-tauri/src/db/mod.rs` добавить:

```rust
pub mod preferences;
```

- [ ] **Step 5: Tauri-команды**

Создать `src-tauri/src/commands/preferences.rs`:

```rust
use crate::{db, AppState};
use tauri::State;

#[tauri::command]
pub async fn prefs_get(
    state: State<'_, AppState>,
    key: String,
) -> Result<Option<String>, String> {
    db::preferences::get(&state.db, &key)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn prefs_set(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> Result<(), String> {
    db::preferences::set(&state.db, &key, &value)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn prefs_all(
    state: State<'_, AppState>,
) -> Result<Vec<db::preferences::Preference>, String> {
    db::preferences::all(&state.db)
        .await
        .map_err(|e| e.to_string())
}
```

- [ ] **Step 6: Зарегистрировать модуль**

В `src-tauri/src/commands/mod.rs` добавить:

```rust
pub mod preferences;
```

В `src-tauri/src/lib.rs` в `invoke_handler![...]` добавить:

```rust
commands::preferences::prefs_get,
commands::preferences::prefs_set,
commands::preferences::prefs_all,
```

- [ ] **Step 7: Проверить, что миграции прикладываются**

```bash
cd d:/Projects/Placebo
cargo check --workspace
cd src-tauri && cargo test --lib db::preferences
```

Expected: 3 unit-теста (`set_then_get_returns_value`, `set_overwrites_existing_key`, `get_unknown_key_returns_none`) проходят.

- [ ] **Step 8: Клиентский сервис**

Создать `src/services/preferences.ts`:

```ts
import { invoke } from "@tauri-apps/api/core";

export async function prefsGet(key: string): Promise<string | null> {
  return await invoke<string | null>("prefs_get", { key });
}

export async function prefsSet(key: string, value: string): Promise<void> {
  await invoke<void>("prefs_set", { key, value });
}

export async function prefsAll(): Promise<Array<{ key: string; value: string }>> {
  return await invoke<Array<{ key: string; value: string }>>("prefs_all");
}
```

- [ ] **Step 9: Интеграция с ThemeProvider (сохранение в Tauri SQLite)**

В `src/theme/ThemeProvider.tsx` заменить `loadChoice` / `setChoice` так, чтобы они сначала **пытались** читать/писать через Tauri, а fallback оставался `localStorage` (на случай запуска через `npm run dev` в браузере без Tauri-бэкенда).

```tsx
import { prefsGet, prefsSet } from "../services/preferences";

async function loadChoice(): Promise<ThemeChoice> {
  try {
    const v = await prefsGet("theme");
    if (v === "light" || v === "dark" || v === "auto") return v;
  } catch {
    /* not running in Tauri; fall back to localStorage below */
  }
  if (typeof window !== "undefined") {
    const stored = window.localStorage.getItem(THEME_STORAGE_KEY);
    if (stored === "light" || stored === "dark" || stored === "auto") return stored;
  }
  return DEFAULT_THEME;
}
```

А инициализация `useState<ThemeChoice>(() => loadChoice())` превращается в `useState<ThemeChoice>(DEFAULT_THEME)` + `useEffect` при монтаже:

```tsx
const [choice, setChoiceState] = useState<ThemeChoice>(DEFAULT_THEME);

useEffect(() => {
  loadChoice().then(setChoiceState);
}, []);

const setChoice = (c: ThemeChoice) => {
  setChoiceState(c);
  prefsSet("theme", c).catch(() => {
    window.localStorage.setItem(THEME_STORAGE_KEY, c);
  });
};
```

- [ ] **Step 10: Запустить и проверить**

```bash
npm run tauri dev
```

Expected: приложение открывается, в DevTools Console:

```js
await __TAURI_INTERNALS__.invoke("prefs_set", { key: "theme", value: "dark" });
await __TAURI_INTERNALS__.invoke("prefs_get", { key: "theme" });
```

Второй вызов возвращает `"dark"`. Перезапустить приложение – первый `prefs_get` после старта возвращает `"dark"`.

- [ ] **Step 11: Commit**

```bash
git add src-tauri/migrations/003_user_preferences.sql src-tauri/src/db/ src-tauri/src/commands/ src-tauri/src/lib.rs src/services/preferences.ts src/theme/ThemeProvider.tsx
git commit -m "$(cat <<'EOF'
feat(prefs): add user_preferences table + Tauri IPC + React service

- SQLite migration 003 adds key/value/updated_at storage with seed
  defaults (theme=auto, lang=ru).
- Rust db::preferences has get/set/all + unit tests.
- Tauri commands: prefs_get, prefs_set, prefs_all.
- src/services/preferences.ts wraps invoke calls.
- ThemeProvider now persists via Tauri, falls back to localStorage
  when running outside the Tauri shell (pure vite dev).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 10: Вычистить legacy Tauri-команды

Мы сохраняем `commands::camera::*` (они пока работают с локальной SQLite-базой камер; в M3 мы их переделаем). Но `greet`, `get_public_rooms`, `create_room` – рудименты.

- [ ] **Step 1: Найти legacy-команды**

```bash
grep -rn "greet\|get_public_rooms\|create_room" d:/Projects/Placebo/src-tauri/src/ d:/Projects/Placebo/src/
```

Expected: всплывут строки в `lib.rs` или `commands/mod.rs`, либо их там нет (уже удалено). **Если их нет – пропустить этот Task целиком.**

- [ ] **Step 2: Удалить из `invoke_handler!`**

Если в `src-tauri/src/lib.rs` есть регистрация legacy-команд, удалить соответствующие строки.

- [ ] **Step 3: Удалить их определения (если есть в `commands/*.rs`)**

- [ ] **Step 4: Обновить frontend-референсы**

```bash
grep -rn '"greet"\|"get_public_rooms"\|"create_room"' d:/Projects/Placebo/src/
```

Если вызовы есть – удалить или заменить на заглушку.

- [ ] **Step 5: Проверить сборку**

```bash
cd d:/Projects/Placebo
cargo check --workspace
npm run dev
```

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "$(cat <<'EOF'
chore: remove legacy Tauri commands (greet/get_public_rooms/create_room)

These were prototype stubs; all production flows go through the axum
API. Tauri IPC is reserved for local-only concerns (preferences,
keychain, future: tabs state, notifications).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 11: Сверить и подчистить main.rs / lib.rs

CLAUDE.md упоминает, что `main.rs` может содержать дублированный код (два `fn main`). Проверить.

- [ ] **Step 1: Прочитать main.rs и lib.rs**

```bash
cat d:/Projects/Placebo/src-tauri/src/main.rs
cat d:/Projects/Placebo/src-tauri/src/lib.rs
```

Expected: `main.rs` – только `fn main() { placebo_lib::run(); }`. `lib.rs` содержит `run()` и регистрирует команды.

- [ ] **Step 2: Если есть дубликаты – удалить**

Если `main.rs` содержит второй `fn main` или какие-то `#[tauri::command]` напрямую – вычистить. Всё должно быть в `lib.rs`.

- [ ] **Step 3: Проверить, что `webrtc.rs` не содержит стартовой логики**

CLAUDE.md и ls показали, что в `src-tauri/src/` есть `webrtc.rs`. Проверить, что он либо (а) используется через `mod webrtc` в lib.rs, либо (б) удалён как мёртвый код.

```bash
grep -n "mod webrtc\|webrtc::" d:/Projects/Placebo/src-tauri/src/
```

Если не используется – удалить `webrtc.rs`. Если используется – оставить, в M0 не трогаем.

- [ ] **Step 4: Сборка**

```bash
cd d:/Projects/Placebo
cargo check --workspace
cargo test --workspace --lib
```

- [ ] **Step 5: Commit (только если были изменения)**

```bash
git add -A
git commit -m "$(cat <<'EOF'
chore: clean up main.rs/lib.rs duplicates and dead modules

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 12: Папочная структура под Shell (подготовка для M1)

Создаём пустые папки, чтобы M1 сразу начинал с правильной иерархии.

**Files:**
- Create: `src/shell/` (пустая, placeholder README)
- Create: `src/screens/auth/`, `src/screens/main/`, `src/screens/categories/`, `src/screens/room/`, `src/screens/profile/`, `src/screens/settings/`, `src/screens/skeletons/`
- Create: `src/components/ui/` (переиспользуемые компоненты: Button, Input и т.д.; в M0 пусто)
- Create: `src/api/` (HTTP-клиент + WS-клиент; в M0 пусто)

- [ ] **Step 1: Создать каркас**

```bash
cd d:/Projects/Placebo/src
mkdir -p shell screens/auth screens/main screens/categories screens/room screens/profile screens/settings screens/skeletons components/ui api
for d in shell screens/auth screens/main screens/categories screens/room screens/profile screens/settings screens/skeletons components/ui api; do
  echo "// Milestone placeholder. Will be populated as screens are implemented." > "$d/.keep.ts"
done
```

**Примечание:** `.keep.ts` – пустой `.ts`, gitignore не затрагивает. Файл удаляется когда в папке появляется реальный код.

- [ ] **Step 2: Старые файлы – НЕ перемещать в M0**

Старые `HomeScreen.tsx`, `ExploreScreen.tsx` и т.д. **остаются в `src/screens/` в плоском виде до момента, когда мы их переписываем в соответствующих milestone'ах**. Перенос = переименование = риск поломки для M0. В M1-M6 каждый экран, когда его переписывают, сразу кладётся в правильную подпапку.

- [ ] **Step 3: Commit**

```bash
git add src/shell/ src/screens/auth/ src/screens/main/ src/screens/categories/ src/screens/room/ src/screens/profile/ src/screens/settings/ src/screens/skeletons/ src/components/ui/ src/api/
git commit -m "$(cat <<'EOF'
chore(structure): scaffold shell/, screens/*, components/ui/, api/ dirs

Directories are anchored by .keep.ts placeholders; real files replace
them as each milestone populates its area. Existing flat screen files
are migrated one-by-one as they are rewritten (M1-M6), not upfront.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 13: README и контрольная точка M0

- [ ] **Step 1: Добавить заметку в CLAUDE.md о завершении M0**

В `CLAUDE.md` в раздел "Текущая стадия" (или создать новый раздел "Milestones") добавить:

```markdown
### Milestones

- [x] **M0 Foundation** (2026-05-??): ts-rs, i18n, ThemeProvider, user_preferences IPC, cleanup.
- [ ] M1 Shell (sidebar + tabs + per-tab router + breadcrumbs).
- [ ] M2 Auth (welcome / register / login).
- [ ] M3 Cameras seed + HLS proxy.
- [ ] M4 Home + Categories + World3D in shell.
- [ ] M5 Rooms + WebSocket + chat.
- [ ] M6 Profile + Friends + Settings + Create hub.
- [ ] M7 Polish + acceptance + distribution.
```

Дату проставить при завершении.

- [ ] **Step 2: Финальный smoke-check**

```bash
cd d:/Projects/Placebo
cargo check --workspace
cd src-tauri && cargo test --lib
cd ..
npm run gen-types
npm run tauri dev
```

Expected:
- cargo check проходит.
- cargo test – все unit-тесты проходят (включая 3 новых в `db::preferences`).
- gen-types завершается OK.
- Приложение запускается, тема по умолчанию "auto" → "light" на светлой системе.

- [ ] **Step 3: Commit и PR**

```bash
git add CLAUDE.md
git commit -m "$(cat <<'EOF'
docs: mark M0 Foundation complete in CLAUDE.md

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"

git push -u origin feat/m0-foundation
```

Послать PR: `feat/m0-foundation → main`, название "M0: Foundation", в описании – ссылка на этот план.

---

## Acceptance Criteria для Milestone 0

1. ✅ `cargo check --workspace` проходит без warnings важного уровня.
2. ✅ `cargo test --workspace --lib` – все тесты проходят, включая 3 новых в `db::preferences`.
3. ✅ `npm run gen-types` запускается без ошибок и в первый раз пишет "no bindings yet".
4. ✅ `npm run dev` и `npm run tauri dev` поднимаются.
5. ✅ В DevTools Console работает `await __TAURI_INTERNALS__.invoke("prefs_set", { key: "theme", value: "dark" })` и после перезапуска `prefs_get` возвращает сохранённое.
6. ✅ Переключение `document.documentElement.setAttribute("data-theme", "dark")` меняет CSS-переменные (визуально – в M0 может быть незаметно, но `getComputedStyle(document.documentElement).getPropertyValue('--bg')` должно меняться).
7. ✅ `useTranslation` + `t('app.loading')` возвращает "Загрузка...".
8. ✅ В `src/` есть созданные папки `shell/`, `screens/{auth,main,categories,room,profile,settings,skeletons}/`, `components/ui/`, `api/`.
9. ✅ `BottomNav.tsx` удалён.
10. ✅ Legacy-команды (`greet`, `get_public_rooms`, `create_room`) удалены из Rust и frontend (если были).
11. ✅ Нет дубликатов в `main.rs`; `webrtc.rs` либо используется, либо удалён.

---

## Что идёт дальше

После approval M0 – переход к `2026-05-14-milestone-1-shell.md`.
