# Milestone 7: Polish + Acceptance + Distribution Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development. Steps use `- [ ]`.

**Goal:** Довести альфу до состояния, в котором друзья-тестеры (10-50 человек) могут скачать `.msi`, установить на чистой Windows 10/11 и пройти весь главный сценарий без блокеров. Это milestone про полировку, интеграционное тестирование по всем 18 acceptance criteria из спеки, безопасность путешествия приглашения, сборку инсталлятора, туннель и документацию.

**Architecture:**
- **Deep-link**: `placebo://r/<code>` зарегистрирован в Tauri как custom URL scheme. Когда пользователь кликает ссылку в браузере/мессенджере – открывается установленное приложение и навигирует на `/r/:code`.
- **Single-instance**: `tauri-plugin-single-instance` – вторая попытка запуска фокусирует существующее окно и передаёт deep-link туда (иначе на Windows пользователь каждый раз открывал бы новое окно).
- **Обновления**: Tauri updater – **не делаем в альфе**. Раздаём вручную.
- **Туннель**: `cloudflared tunnel --url http://localhost:3001`. URL-туннеля вшивается в сборку через `.env.production` и `VITE_API_BASE_URL`. Так как друзья получают свежую сборку после каждой смены туннеля – это acceptable.
- **Диагностика**: global ErrorBoundary на React, структурные логи на axum (tracing-subscriber уже есть), простая "About" страница с версией и ссылкой на фидбек.
- **Производительность**: измеряем FPS в World3D, память, CPU – на типичной тестерской машине. Если что-то явно тормозит – точечно чиним.

**Spec reference:** §8.6 (распространение), §12 (18 acceptance criteria), §13 (риски).

**Зависимости:** M0-M6 готовы.

---

## File Map

### Tauri

- Modify: `src-tauri/Cargo.toml` – `tauri-plugin-single-instance`, `tauri-plugin-deep-link`.
- Modify: `src-tauri/src/lib.rs` – регистрация плагинов.
- Modify: `src-tauri/tauri.conf.json` – `bundle`, `identifier`, `windows`, `updater` off, deep-link scheme.
- Create: `src-tauri/icons/` – набор иконок (если ещё не сгенерирован).

### Frontend

- Create: `src/components/ErrorBoundary.tsx`.
- Create: `src/screens/about/AboutScreen.tsx`.
- Create: `src/api/version.ts` (для /health + версия).
- Modify: `src/App.tsx` – ErrorBoundary + deep-link listener.
- Modify: `src/shell/routes.tsx` – `/about`.
- Modify: `src/shell/Sidebar.tsx` / settings – ссылка на About.
- Modify: `src/i18n/locales/ru.json` – ключи.

### Docs / ops

- Create: `docs/ALPHA_TESTERS.md` – инструкция для тестеров (скачать, установить, что делать, куда писать фидбек).
- Create: `docs/DISTRIBUTION.md` – для тебя: как собрать `.msi`, как поднять туннель, как раздать.
- Modify: `RUNBOOK.md` – **только если пользователь явно разрешит** (спеку помнить: RUNBOOK.md трогать только по явной просьбе).
- Create: `scripts/release.ps1` – полуавтоматический build + версия.

---

## Task 1: Ветка

```bash
git -C d:/Projects/Placebo checkout main && git pull
git -C d:/Projects/Placebo checkout -b feat/m7-polish
```

---

## Task 2: ErrorBoundary

**Files:** `src/components/ErrorBoundary.tsx`

- [ ] **Step 1:**

```tsx
import { Component, ReactNode } from "react";

type Props = { children: ReactNode };
type State = { error: Error | null };

export class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidCatch(error: Error, info: unknown) {
    // Surface to console; future: send to analytics/Sentry.
    console.error("[Placebo ErrorBoundary]", error, info);
  }

  render() {
    if (this.state.error) {
      return (
        <div style={{ padding: 32, maxWidth: 720 }}>
          <h2 style={{ color: "#D12850" }}>Что-то пошло не так</h2>
          <p>Текст ошибки: {this.state.error.message}</p>
          <p>Перезагрузите приложение. Если повторится — скиньте нам скриншот.</p>
          <pre style={{ whiteSpace: "pre-wrap", fontSize: 12, color: "#888" }}>
            {this.state.error.stack ?? ""}
          </pre>
          <button onClick={() => location.reload()}>Перезагрузить</button>
        </div>
      );
    }
    return this.props.children;
  }
}
```

- [ ] **Step 2:** Wrap в App.tsx

```tsx
<ErrorBoundary>
  <ThemeProvider>...</ThemeProvider>
</ErrorBoundary>
```

- [ ] **Step 3:** Smoke-тест

Временно в HomeScreen вставить `throw new Error("test")` → убедиться что вместо белого экрана показывается ErrorBoundary. Откатить.

- [ ] **Step 4:** Commit

```bash
git add src/components/ErrorBoundary.tsx src/App.tsx
git commit -m "feat(diag): ErrorBoundary at app root"
```

---

## Task 3: About screen

**Files:** `src/screens/about/AboutScreen.tsx`

- [ ] **Step 1:**

```tsx
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

const VERSION = __APP_VERSION__ ?? "dev";

export default function AboutScreen() {
  const { t } = useTranslation();
  const [serverVersion, setServerVersion] = useState<string | null>(null);

  useEffect(() => {
    fetch((import.meta.env.VITE_API_BASE_URL ?? "http://localhost:3001/api/v1").replace(/\/api\/v1$/, "") + "/health")
      .then((r) => r.json())
      .then((j) => setServerVersion(j.version ?? "unknown"))
      .catch(() => setServerVersion("offline"));
  }, []);

  return (
    <div style={{ padding: 32, maxWidth: 640 }}>
      <h1>Placebo</h1>
      <p>{t("about.tagline")}</p>
      <p>Version: <code>{VERSION}</code></p>
      <p>Server: <code>{serverVersion ?? "..."}</code></p>
      <p>{t("about.feedback")}: <a href="mailto:alpha@placebo.local">alpha@placebo.local</a></p>
    </div>
  );
}
```

- [ ] **Step 2:** vite-define

`vite.config.ts`:

```ts
import pkg from "./package.json" assert { type: "json" };
// ...
define: {
  __APP_VERSION__: JSON.stringify(pkg.version),
},
```

- [ ] **Step 3:** i18n

```json
{
  "about.tagline": "Смотреть мир вместе.",
  "about.feedback": "Фидбек",
  "about.link": "О приложении"
}
```

- [ ] **Step 4:** Route + ссылка в Home

В `routes.tsx`:

```tsx
{ path: "/about", element: guarded(<AboutScreen />) },
```

В `HomeScreen.tsx` ссылка "О приложении Placebo" теперь навигирует:

```tsx
<button className="home__about" onClick={() => openTab("/about", t("about.link"))}>
  {t("home.about")}
</button>
```

- [ ] **Step 5:** Commit

```bash
git add -A
git commit -m "feat(about): About screen with client + server version"
```

---

## Task 4: /health endpoint с версией

**Files:** `crates/placebo-api/src/handlers/health.rs` (уже есть – расширить)

- [ ] **Step 1:**

```rust
use serde::Serialize;

#[derive(Serialize)]
pub struct Health {
    pub status: &'static str,
    pub version: &'static str,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/health", axum::routing::get(|| async {
        axum::Json(Health { status: "ok", version: env!("CARGO_PKG_VERSION") })
    }))
}
```

(или расширить существующий handler – ориентируемся на реальное состояние файла.)

- [ ] **Step 2:** Commit

```bash
git add crates/placebo-api/src/handlers/health.rs
git commit -m "feat(health): /health returns server version"
```

---

## Task 5: Deep-link и single-instance

**Files:** `src-tauri/Cargo.toml`, `src-tauri/src/lib.rs`, `src-tauri/tauri.conf.json`

- [ ] **Step 1:** Зависимости

```toml
tauri-plugin-single-instance = "2"
tauri-plugin-deep-link = "2"
```

- [ ] **Step 2:** Регистрация плагинов

```rust
// src-tauri/src/lib.rs
tauri::Builder::default()
    .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
        // Focus main window; forward deep-link from argv[1] if present.
        if let Some(main) = app.webview_windows().values().next() {
            let _ = main.set_focus();
            if let Some(url) = argv.get(1) {
                let _ = main.eval(&format!("window.dispatchEvent(new CustomEvent('placebo:deep-link', {{ detail: {:?} }}))", url));
            }
        }
    }))
    .plugin(tauri_plugin_deep_link::init())
    .setup(|app| {
        // Register scheme on setup (no-op on desktop if already registered)
        #[cfg(desktop)]
        {
            use tauri_plugin_deep_link::DeepLinkExt;
            app.deep_link().register("placebo")?;
        }
        // existing setup ...
        Ok(())
    })
    // existing .plugin(tauri_plugin_opener::init()) etc.
```

- [ ] **Step 3:** tauri.conf.json

```json
{
  "identifier": "com.placebo.app",
  "plugins": {
    "deep-link": {
      "desktop": {
        "schemes": ["placebo"]
      }
    }
  },
  "bundle": {
    "active": true,
    "targets": ["msi", "nsis"],
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.ico",
      "icons/icon.icns"
    ]
  }
}
```

- [ ] **Step 4:** Frontend listener

`src/App.tsx`:

```tsx
import { useEffect } from "react";
import { useNavigate } from "react-router-dom";

function DeepLinkListener() {
  const nav = useNavigate();
  useEffect(() => {
    const handler = (e: any) => {
      const url: string = e.detail ?? "";
      // placebo://r/<code>
      const m = /^placebo:\/\/r\/([A-Za-z0-9]+)/.exec(url);
      if (m) nav(`/r/${m[1]}`);
    };
    window.addEventListener("placebo:deep-link", handler as any);
    return () => window.removeEventListener("placebo:deep-link", handler as any);
  }, [nav]);
  return null;
}
```

Проблема: `useNavigate` требует роутер. Пересмотр: слушатель внутри любого компонента, который живёт под `RouterProvider`. Альтернатива – внутри `ShellRoot` читать событие и вызывать `tabManager.openTab` с путём `/r/:code`. Более удобно это:

```tsx
// src/shell/DeepLinkBridge.tsx
import { useEffect } from "react";
import { useTabs } from "./tabs/useTabs";

export default function DeepLinkBridge() {
  const { openTab } = useTabs();
  useEffect(() => {
    const handler = (e: Event) => {
      const url = (e as CustomEvent<string>).detail ?? "";
      const m = /^placebo:\/\/r\/([A-Za-z0-9]+)/.exec(url);
      if (m) openTab(`/r/${m[1]}`);
    };
    window.addEventListener("placebo:deep-link", handler);
    return () => window.removeEventListener("placebo:deep-link", handler);
  }, [openTab]);
  return null;
}
```

Добавить `<DeepLinkBridge />` в `ShellRoot`.

- [ ] **Step 5:** Commit

```bash
git add -A
git commit -m "feat(tauri): placebo:// deep-link + single-instance plugin"
```

---

## Task 6: Performance pass

- [ ] **Step 1: Профиль в World3D**

Включить Performance monitor через drei:

```tsx
import { Stats } from "@react-three/drei";
// В Canvas:
{ import.meta.env.DEV && <Stats /> }
```

Открыть World3D, посмотреть FPS на типичной машине:
- Если < 30 – включить adaptive quality (снизить LOD, урезать количество маркеров).
- Если FPS > 45 стабильно – ок, ничего не трогаем.

- [ ] **Step 2: Мем-профиль**

DevTools → Memory → Heap snapshot в двух состояниях (idle, после 5-минут с активным стримом). Если heap растёт linearly – утечка в hls.js или в WS-слушателях. Смотрим, фиксим, если найдётся.

- [ ] **Step 3: WS reconnect-стресс**

Остановить API на 10 секунд, смотреть что фронт переподключится (без полного reload). Если клиент перестаёт работать – поправить backoff.

- [ ] **Step 4:** Фиксы коммитим по ходу.

---

## Task 7: Security hardening на уровне альфы

- [ ] **Step 1:** Rate-limit на auth

В `auth_service` убедиться что:
- `login` – не более 10 попыток за 15 минут с одного IP (Redis counter).
- `register` – не более 5 регистраций за час с IP.

Если отсутствует – добавить middleware `rate_limit_ip(...)`.

- [ ] **Step 2:** CORS на axum

После добавления туннеля фронт может оказаться на другом origin. `tower-http::cors::CorsLayer` с `Any` – в альфе приемлемо; в прод – список origins из config.

```rust
use tower_http::cors::{CorsLayer, Any};
// ...
.layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
```

- [ ] **Step 3:** Удалить dev-auto-verify после релиза

В `auth_service::register` не использовать `auto_verify = Environment::Dev` для production-туннеля. Для альфы – оставляем `auto_verify = true` безусловно, потому что в альфе у нас вообще нет email-сервера. Пометить TODO: подключить email-verification в бете.

- [ ] **Step 4:** Commit

```bash
git add crates/placebo-api/src/
git commit -m "chore(security): CORS + rate-limit on auth for alpha"
```

---

## Task 8: Acceptance criteria прогон

Целевой milestone spec §12. Прогоняем все 18 пунктов. Создаём чек-лист в `docs/ALPHA_ACCEPTANCE.md`.

- [ ] **Step 1: Подготовка**

```bash
cd crates/placebo-api && cargo run &
cd d:/Projects/Placebo && npm run tauri dev
```

- [ ] **Step 2: Прогон**

Для каждого пункта из §12 спеки: тикнуть ✅ или описать баг.

Файл:

```markdown
# Alpha Acceptance Run — <дата>

Tester: <ник>
Machine: <OS/CPU/GPU>

1. [ ] Два разных пользователя регистрируются на двух разных машинах.
2. [ ] Оба видят главный экран по Figma.
...
18. [ ] Нет unhandled промисов, белых экранов, или потерянных сессий.
```

- [ ] **Step 3: Фиксы багов**

Каждый фикс – отдельный commit с ссылкой на пункт acceptance.

- [ ] **Step 4: Commit acceptance-лога**

```bash
git add docs/ALPHA_ACCEPTANCE.md
git commit -m "docs: alpha acceptance run log (clean)"
```

---

## Task 9: Сборка .msi / .exe инсталлятора

- [ ] **Step 1: Генерация иконок**

Если `src-tauri/icons/` пуст или содержит tauri-default:

```bash
cd d:/Projects/Placebo
npm install -D @tauri-apps/cli
npx tauri icon path/to/logo.png  # берём реальный логотип
```

- [ ] **Step 2: Выставить версию**

```bash
# package.json
"version": "0.1.0-alpha.1"

# src-tauri/Cargo.toml
[package]
version = "0.1.0-alpha.1"
```

- [ ] **Step 3: .env.production**

```
VITE_API_BASE_URL=https://<cloudflared-tunnel-id>.trycloudflare.com/api/v1
VITE_WS_BASE_URL=wss://<cloudflared-tunnel-id>.trycloudflare.com/ws
```

(`.env.production` — коммитим без чувствительных секретов, URL туннеля у нас не секрет, но меняется. Обновляем перед каждой сборкой.)

- [ ] **Step 4: Build**

```bash
cd d:/Projects/Placebo
npm run tauri build
```

Expected: артефакты в `src-tauri/target/release/bundle/msi/Placebo_0.1.0-alpha.1_x64_en-US.msi` и `bundle/nsis/*.exe`.

- [ ] **Step 5: Проверить установку**

На чистой виртуалке Windows 10 (Hyper-V / VirtualBox) – установить `.msi`. Запустить → открывается Welcome → регистрация работает, подключается к туннелю.

- [ ] **Step 6: Commit assets**

Инсталлятор НЕ коммитим в git (большой). Публикуем в Google Drive / Dropbox / GitHub Releases (если есть репозиторий publicless).

---

## Task 10: Cloudflared tunnel

- [ ] **Step 1:**

```bash
cloudflared tunnel --url http://localhost:3001
```

Expected: выдаёт URL `https://<random>.trycloudflare.com`. Этот URL кладём в `.env.production` (Step 3 задачи 9).

- [ ] **Step 2:** Добавить скрипт запуска

`scripts/serve-alpha.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
(cd crates/placebo-api && cargo run --release) &
API_PID=$!
trap "kill $API_PID" EXIT
cloudflared tunnel --url http://localhost:3001
```

- [ ] **Step 3: Commit**

```bash
git add scripts/serve-alpha.sh
git commit -m "build: serve-alpha.sh launches API + cloudflared tunnel"
```

---

## Task 11: Документация для тестеров

**Files:** `docs/ALPHA_TESTERS.md`

- [ ] **Step 1:**

```markdown
# Placebo — Alpha Testing

Спасибо, что согласился потестить! Вот как быстро запуститься.

## 1. Установка

1. Скачай `Placebo_0.1.0-alpha.1_x64_en-US.msi` (ссылка прислана отдельно).
2. Запусти. Windows может показать предупреждение SmartScreen – жми "Подробнее" → "Запустить всё равно". Это ожидаемо: сборка не подписана.
3. После установки найди "Placebo" в меню Пуск.

## 2. Регистрация

1. Запусти приложение. Откроется экран Welcome.
2. Нажми "Зарегистрироваться".
3. Заполни минимум: имя (никнейм), email, пароль (8+ символов).
4. Опционально: юзернейм (латиница, @), дата рождения.

## 3. Что делаем

- Зайди в "Категории" → "Онлайн карта мира" → покликай камеры.
- Нажми "Смотреть вместе" – создаётся комната, ссылка копируется в буфер.
- Скинь ссылку вида `placebo://r/<code>` другому тестеру (или `https://<tunnel>/r/<code>` – работает так же).
- Пообщайтесь в чате.

## 4. Что ломается – пиши сюда

Телеграм: @<твой ник>
Email: alpha@placebo.local

Пожалуйста, прикладывай:
- Шаги воспроизведения.
- Скриншот (Win+PrtScn).
- Версию (видна в About: Главная → "О приложении Placebo").

## 5. Известные ограничения

- Соцсети-кнопки на входе не работают.
- Поиск сверху не работает.
- Голосового созвона нет.
- История, Избранное, Папки, Уведомления – пустые скелеты.
- Сервер туннелируется через cloudflared; если сломалось – жди минуту, админ перезапустит.
```

- [ ] **Step 2: Commit**

```bash
git add docs/ALPHA_TESTERS.md
git commit -m "docs: alpha testers getting-started"
```

---

## Task 12: DISTRIBUTION.md (для тебя)

**Files:** `docs/DISTRIBUTION.md`

- [ ] **Step 1:**

```markdown
# Placebo — Distribution runbook (internal)

## Сборка альфа-билда

1. Убедись что `main` зелёный (CI, тесты, `cargo test`, `npm test`).
2. Запусти туннель: `cloudflared tunnel --url http://localhost:3001`. Скопируй URL.
3. Обнови `.env.production`:
   ```
   VITE_API_BASE_URL=https://<tunnel-id>.trycloudflare.com/api/v1
   VITE_WS_BASE_URL=wss://<tunnel-id>.trycloudflare.com/ws
   ```
4. Обнови версию в `package.json` и `src-tauri/Cargo.toml` (семвер `0.1.0-alpha.N`).
5. `npm run tauri build`.
6. Найди артефакты: `src-tauri/target/release/bundle/msi/*.msi`.
7. Залей в приватный Google Drive / выдели ссылку для тестеров.

## Поднятие сервера для альфы

1. Убедись что PostgreSQL 17 + PostGIS запущен (`pg_isready`).
2. Убедись что Redis запущен (`redis-cli ping`).
3. Прогони миграции: `cd crates/placebo-api && sqlx migrate run`.
4. `scripts/serve-alpha.sh` – запускает API и cloudflared.
5. Скопируй URL туннеля тестерам (если URL изменился – пересобери клиент).

## Если туннель меняет URL

Вариант A: ручной пересбор.
Вариант B: купить постоянный домен + cloudflared named-tunnel (post-альфа).
```

- [ ] **Step 2: Commit**

```bash
git add docs/DISTRIBUTION.md
git commit -m "docs: distribution runbook for alpha releases"
```

---

## Task 13: release.ps1 (опционально)

```powershell
# scripts/release.ps1
$ErrorActionPreference = "Stop"
param([string]$Version = "0.1.0-alpha.1", [string]$TunnelUrl)

if (-not $TunnelUrl) { Write-Error "Pass -TunnelUrl https://..." }

# Update version
(Get-Content package.json) -replace '"version": "[^"]+"', "`"version`": `"$Version`"" | Set-Content package.json
(Get-Content src-tauri/Cargo.toml) -replace 'version = "[^"]+"', "version = `"$Version`"" | Set-Content src-tauri/Cargo.toml

# Write .env.production
@"
VITE_API_BASE_URL=$TunnelUrl/api/v1
VITE_WS_BASE_URL=$($TunnelUrl -replace '^https://', 'wss://')/ws
"@ | Set-Content .env.production

npm run tauri build
```

```bash
git add scripts/release.ps1
git commit -m "build: release.ps1 helper"
```

---

## Task 14: Финальный push + CLAUDE.md

- [ ] **Step 1:** Обновить CLAUDE.md

```markdown
### Milestones
- [x] M0 Foundation
- [x] M1 Shell
- [x] M2 Auth
- [x] M3 Cameras + HLS proxy
- [x] M4 Home + Categories + World3D
- [x] M5 Rooms + WebSocket + Chat
- [x] M6 Profile + Friends + Create hub
- [x] M7 Polish + Distribution — Alpha 0.1.0-alpha.1
```

- [ ] **Step 2:**

```bash
git add CLAUDE.md
git commit -m "docs: mark M7 complete – alpha 0.1.0-alpha.1 ready"
git push -u origin feat/m7-polish
```

PR → мердж в main. Теги:

```bash
git tag -a v0.1.0-alpha.1 -m "Placebo Alpha 0.1.0-alpha.1"
git push origin v0.1.0-alpha.1
```

---

## Acceptance Criteria (M7-specific)

1. ✅ Global ErrorBoundary ловит любой render-crash и показывает понятный экран.
2. ✅ About screen показывает клиентскую и серверную версию.
3. ✅ `/health` возвращает `{ status: "ok", version: "0.1.0-alpha.1" }`.
4. ✅ Deep-link `placebo://r/<code>` открывает приложение и переходит в `/r/<code>`.
5. ✅ Single-instance: повторный запуск фокусирует существующее окно.
6. ✅ Все 18 пунктов §12 спеки отмечены ✅ в `docs/ALPHA_ACCEPTANCE.md`.
7. ✅ `npm run tauri build` производит `.msi`, который ставится на чистой Win10/11.
8. ✅ Приложение, установленное из `.msi`, на чистой машине проходит сценарий: регистрация → World3D → "Смотреть вместе" → чат между двумя машинами.
9. ✅ `scripts/serve-alpha.sh` поднимает API + cloudflared одной командой.
10. ✅ `docs/ALPHA_TESTERS.md` и `docs/DISTRIBUTION.md` закоммичены.
11. ✅ Тег `v0.1.0-alpha.1` запушен в remote.
12. ✅ `.msi` залит в раздачу, тестерам отправлены ссылки.

---

## Что дальше (post-alpha)

Список задач для Beta:

- OAuth (Google / Telegram).
- Голосовой созвон (WebRTC).
- Email verification.
- Tauri auto-updater.
- Partner-HLS-источники вместо yt-dlp.
- Собственный FFmpeg-ingest.
- Перенос на Hetzner VPS.
- Multi-region + anti-DDoS архитектура (см. IDEAS.md).
- Виртуализация 3D через portal-based GlobalCanvas (отложенное решение M4).
- Redis pub/sub вместо in-process Bus (отложенное решение M5).
- Записи камер + tier-based retention + boost система.
- Клипы.
- yourself / enterprise камеры.
- Карточка фильма и "совместный просмотр фильма" pipeline.
- Premium подписка.

---

## Финальная галочка

После всех M0–M7 – `docs/superpowers/specs/2026-05-14-alpha-design.md` можно помечать как "delivered". Следующая брейншторм-сессия начинается с бета-плана.
