# Placebo — Context for Claude Code

> Единый контекст проекта. Склейка PLACEBO_CLAUDE_CODE_CONTEXT.md + RUNBOOK.md + README.md.
> Детали — в исходных файлах (ссылки по тексту). Здесь — то, что нужно держать в голове.

---

## 0. Как со мной работать

- Два разработчика, один старший. Русский язык (кроме кода/команд).
- НЕ использовать длинное тире `—`, заменять на `–` или убирать.
- Коротко и по делу. Развёрнуто только там, где нужна архитектура/бизнес-логика.
- Умеренные эмодзи (✅ 🔲 ⚠️), не для красоты.
- Инициатива приветствуется. Несогласие – тоже. Без капитанства.
- Если можно в одном предложении – одно предложение.
- Никаких заглушек/placeholder – только реальный рабочий код.
- Перед решением: "что изменится для конечного пользователя?"
- Тесты к коду. Не уверен – говори прямо.

**RUNBOOK.md запрещено редактировать без явной просьбы пользователя.**

---

## 1. Что такое Placebo

Десктопное приложение для **совместного просмотра** видео и живых камер мира.
Комбинация живых камер + социальный слой (чат, голосовой созвон, совместный просмотр).

- **Платформа**: Windows (primary), macOS (secondary). Мобиль – не в планах.
- **Рынки**: СНГ + международный (фокус: Индия, Финляндия, Россия).
- **Репо**: https://github.com/IAmYourPlayboy/Placebo.git, ветка `main`.
- **Локально**: `d:\Projects\Placebo`.

---

## 2. Стек

### Клиент
- Tauri 2.0, React 18.3, TypeScript 5.5, Vite 5.4, Rust (2021).

### 3D-мир
- `three` + `@react-three/fiber` + `@react-three/drei` + `@react-three/postprocessing`
- `3d-tiles-renderer` (NASA AMMOS) – пока не установлен
- `hls.js` – HLS-потоки как видеотекстуры

### Rust (src-tauri)
- `tauri 2.0`, `serde`, `sqlx` (sqlite), `tokio`, `thiserror`, `anyhow`, `uuid`, `chrono`.

### Backend (placebo-api, axum)
- `axum`, `PostgreSQL 17 + PostGIS`, `Redis`, `Cloudflare R2`, `Better Auth` (планируется), `FFmpeg` (RTSP→HLS).

---

## 3. Структура репо (основное)

```
d:\Projects\Placebo\
├── CLAUDE.md                 ← этот файл
├── IDEAS.md                  ← идеи пользователя (append-only)
├── RUNBOOK.md                ← ЗАПРЕЩЕНО редактировать без просьбы
├── PLACEBO_CLAUDE_CODE_CONTEXT.md  ← полный исходный контекст
├── README.md
├── Cargo.toml / Cargo.lock   ← workspace для crates/
├── crates/                   ← backend crates (placebo-api и т.д.)
├── data/cameras-seed.json    ← 50 seed-камер
├── docs/
│   ├── ARCHITECTURE.md
│   └── ARCHITECTURE_3D.md
├── pipeline/                 ← OSM → 3D Tiles (Docker + bash)
│   ├── docker-compose.yml
│   ├── nginx.conf
│   ├── scripts/generate-tiles.sh
│   ├── scripts/youtube-stream.sh
│   ├── sql/
│   └── styles/default.style
├── src/                      ← React frontend
│   ├── App.tsx / App.css / main.tsx
│   ├── components/
│   │   ├── Sidebar.tsx / BottomNav.tsx / Icons.tsx
│   │   └── world3d/         ← WorldScene, BuildingsLayer, CameraMarker3D,
│   │                          CameraFrustum, Environment, NavigationControls
│   ├── hooks/                ← useNearbyCameras, useCityTiles
│   ├── screens/              ← Home, Explore, Cameras, Profile, WatchRoom,
│   │                          Friends, Create, World3DScreen
│   ├── services/cameras.ts
│   └── types/                ← camera.ts, world3d.ts
├── src-tauri/
│   ├── tauri.conf.json       ← 1280×800, min 900×600
│   ├── Cargo.toml / build.rs
│   ├── migrations/           ← 001_cameras.sql, 002_full_schema.sql
│   └── src/                  ← main.rs, lib.rs, db/, commands/
├── vite.config.ts            ← содержит HLS-proxy middleware (yt-dlp + CORS)
└── package.json
```

Голосую за чистку: `main.rs` имеет дубликат `fn main` (legacy + новый), legacy-команды (`greet`, `get_public_rooms`, `create_room`) в `lib.rs` – удалить когда будет API. `BottomNav` скрыт через CSS, можно удалить.

---

## 4. Текущая стадия

### Готово ✅
- Tauri 2 + React 18 скелет, 16 desktop-экранов.
- SQLite + 50 seed-камер (JSON + JSON Schema).
- 8 Tauri IPC команд (Rust), 25 Rust-тестов (camera + recording).
- CamerasScreen со SVG-картой.
- 3D-мир: архитектура + 7 компонентов + хук + типы (написаны).
- Pipeline OSM → 3D Tiles: Docker-compose + скрипты.
- Auth-бэкенд (ветка `feat/auth-system`): register, login, logout, refresh, password-reset.
- User session tracking в Redis + delete-all для logout-everywhere.
- HLS-прокси через Vite middleware (yt-dlp). Видео работает в 3D.

### НЕ готово 🔲
- Установка `3d-tiles-renderer` + раскомментирование кода в `BuildingsLayer.tsx`.
- Генерация реальных 3D Tiles (Tokyo и др.) – нужен Docker.
- Остальные axum-эндпоинты (рум/камеры/клипы).
- Интеграция Better Auth фронт.
- FFmpeg медиасервер (RTSP → HLS).
- FriendsScreen, ElevenLabs перевод речи.

### Milestones (alpha roadmap)

- [x] **M0 Foundation** (2026-05-15): ts-rs pipeline, react-i18next, ThemeProvider, user_preferences IPC, dead-code cleanup.
- [x] **M1 Shell** (2026-05-16): sidebar, topbar, per-tab memory router, breadcrumbs, theme toggle, skeleton screens, tab persistence.
- [x] **M2 Auth** (2026-05-17): welcome / register / login, AuthProvider + AuthGuard, OS-keychain token storage, /me endpoint, username + DOB on users.
- [ ] M3 Cameras seed + HLS proxy.
- [ ] M4 Home + Categories + World3D in shell.
- [ ] M5 Rooms + WebSocket + chat.
- [ ] M6 Profile + Friends + Settings + Create hub.
- [ ] M7 Polish + acceptance + distribution.

---

## 5. Дизайн-система

```css
--accent:    #E8345A   /* розово-красный, основной акцент */
--bg:        #FFFFFF
--bg-2:      #F5F5F7
--bg-3:      #EBEBEB
--t1:        #0F0F0F
--t2:        #444444
--t3:        #999999
--border:    #E8E8EC
--sidebar-w: 224px
```

3D: фон `#0a0a0f`, туман `#0a0a1a` 500–2000м. Wireframe рёбра `#1e2332`, заливка `#0f121c`. Маркеры: city `#E8345A`, traffic `#FF9500`, nature `#34C759`.

Шрифт: Nunito (400/600/700/800/900). Emoji: Twemoji через CDN – в коде пишем Unicode.

Clash Royale spring для карточек: `scale(0.55)→(1.06)→(0.97)→(1)`, stagger 0.05s. 3D-маркеры пульсируют sin 2Hz.

---

## 6. 3D-мир (ключевая концепция)

**3D-мир – это дефолтная среда просмотра камеры, не отдельный режим.**

Поток: открыл камеру → видео на весь экран → повернул ПКМ → увидел что стоишь в 3D-городе. Видео натянуто на плоскость перед "глазами", здания вне FOV – wireframe.

### Источник 3D-зданий
- **Свой хостинг**, НЕ Cesium ion.
- `Geofabrik (OSM PBF) → osm2pgsql → PostGIS → pg2b3dm → 3D Tiles → Cloudflare R2`.
- 8 городов: Tokyo, Moscow, NYC, Mumbai, Helsinki, London, Paris, Dubai. ~2.1GB, R2 ≈$0.03/мес.

### Управление
| Действие | Клавиша |
|---|---|
| Вращение | ПКМ + drag |
| Перемещение | WASD / стрелки |
| Вверх/вниз | E / Q |
| Zoom | колёсико |
| Сброс | Пробел |
| Перелёт к камере | клик на маркер |

### Координаты
- (0,0,0) = позиция активной камеры. X=восток, Y=высота, Z=север (метры).
- `dx = (lng2-lng1) * cos(lat) * 111320`; `dz = (lat2-lat1) * 111320`.

### Производительность
- 60 FPS на GTX 1060 / RX 580. Макс 500K треугольников.
- LOD: 0–200 полный, 200–500 средний, 500–1000 низкий, >1км не грузить.
- Макс 1 видеотекстура (активная камера), до 10 thumbnail.
- Adaptive quality: FPS<30 → снижаем LOD multiplier.

### Поля камеры для 3D (миграция позже)
`height_above_ground`, `camera_azimuth`, `camera_elevation`, `fov_horizontal`, `fov_vertical`. Дефолты: 5м / 0° / -15° / 90° / 58°.

Подробнее: [docs/ARCHITECTURE_3D.md](docs/ARCHITECTURE_3D.md).

---

## 7. Типы камер

| Тип | Доступ | Хранение | Шифрование |
|---|---|---|---|
| **public** | все | по тирам | нет |
| **enterprise** | B2B, закрытая сеть | 6 мес (закон) | AES-256 E2E |
| **yourself** (Premium) | владелец + кого пригласил | 45 дней | AES-256 E2E, адрес скрыт |

---

## 8. Записи: тиры и бусты

| Тир | Просмотры/мес | Retention | С бустами |
|---|---|---|---|
| tier1 | 1000+ | 14 дней | до 365 |
| tier2 | 500–999 | 7 дней | до 365 |
| tier3 | 100–499 | 5 дней | до 365 |
| tier4 | 20–99 | 2 дня | до 365 |
| tier5 | 0–19 | 0 (live only) | – |

**Бусты**: Premium = 4 токена/мес (сгорают). 1 токен = +3 дня любой камере. Потолок 365 (продуктовый). Каждый буст – отдельная запись с `expires_at`. Доп: +4 за 59₽, +10 за 99₽. Показ: "Поддержано N" + аватары. Стоимость ≈10% от Premium revenue.

**Хранение**: HOT 0–24ч (H.264 HLS .ts) → WARM 1–7д (H.265 MP4) → COLD 8–30д (H.265 CRF28) → ARCHIVE 30+ (AV1 ночным батчем).

---

## 9. Поля камеры (главное)

**Никогда не в API**: `stream_url`, `backup_url`, `external_id`, `frame_rate` (последнее – только для FFmpeg).

Идентификация: `id` (UUID v4), `name` (оригинальный язык), `slug`, `camera_type` (public|enterprise|yourself).
Локация: `country`, `country_code`, `region`, `city`, `district`, `address` (только public), `custom_label` (yourself/enterprise), `lat`, `lng`, `timezone` (IANA).
Стрим: `stream_type` (rtsp|hls|youtube|dash|webrtc|mjpeg), `stream_protocol`, `available_qualities` (JSON), `bitrate_kbps`, `codec`, `resolution_w/h`, `latency_ms`.
Возможности: `has_audio`, `has_night_vision`, `is_underwater`.
Мета: `category` (default `city`), `subcategory`, `tags` (JSON), `description_en` (pivot), `thumbnail_url`, `attribution`.
Запись: `recording_enabled`, `retention_tier`, `recording_retention_days`, `recording_codec`.
Runtime (Redis): `viewers_now`, `total_views`, `health_score`, `uptime_pct_30d`, `is_online`.

Гео-URL (Google Maps и пр.) – **не хранить**, генерировать из lat/lng.

---

## 10. Языки

- 28 языков планируется.
- Никнеймы хранятся в оригинале, не переводятся.
- `name` камеры – оригинальный язык страны.
- `description_en` – pivot. Перевод `description_en → target` через DeepL/LibreTranslate, кеш в `camera_translations`.
- Формат дат/времени – locale системы.

---

## 11. Монетизация

- **Free**: public-камеры, комнаты до 4, клипы только скачать, без yourself.
- **Premium 199₽/мес или 1490₽/год**: комнаты до 20, 4 буст-токена, yourself-камеры, облако 7GB, watch_history sync, анимированные обои, голосовой созвон, видеорегистратор.
- **Enterprise**: закрытая сеть, 6 мес, AES-256 E2E, SLA.

---

## 12. Инфраструктура

```
Cloudflare (WAF + Anycast DDoS)
   ↓
Load Balancer (Hetzner)
   ↓
2-3 axum API
   ↓
PostgreSQL+PostGIS | Redis | Cloudflare R2

Медиа: Cameras (RTSP) → FFmpeg workers → HLS → SSD (hot 24h) → R2 (warm/cold/archive)
```

Путь масштабирования: Docker Compose → Swarm → K8s (только с DevOps-командой).

---

## 13. Архитектурные решения (зафиксированы)

1. SQLite локально – только личные данные пользователя.
2. PostgreSQL+PostGIS на сервере – всё общее.
3. Redis – только real-time (viewers, sessions, rate limits).
4. Cloudflare с первого дня (R2 + WAF + CDN).
5. axum (Rust) – API.
6. Seed: JSON + JSON Schema.
7. RTSP URL никогда не в API – только через медиасервер.
8. Docker Compose → Swarm → K8s.
9. Три типа камер: public / enterprise / yourself.
10. Пять тиров + бусты (потолок 365).
11. **Клипы** = нарезки с камер. **Моменты** = пользовательский контент.
12. **3D-мир = дефолт просмотра** (не отдельный режим).
13. **3D здания – свой хостинг** (не Cesium ion).
14. **React Three Fiber** (не CesiumJS, не MapLibre).
15. **При наведении только thumbnail**, live preview HLS – позже.

---

## 14. Dev — команды

```bash
# Frontend only (Vite + HLS-proxy middleware)
npm run dev               # http://localhost:1420

# Полный Tauri (когда актуально)
npm run tauri dev
npm run tauri build       # .exe инсталлятор

# Rust тесты
cd src-tauri && cargo test

# axum API
cd placebo-api && cargo run    # listening on 0.0.0.0:3001

# Миграции
cd placebo-api && sqlx migrate run
```

Windows: shell – bash (не PowerShell). Unix-синтаксис (`/dev/null`, forward slashes). PowerShell-тул доступен отдельно.

Pipeline bash-скрипты на Windows запускать через Git Bash или WSL.

---

## 15. Runtime-зависимости

Перед запуском (всё руками):

1. Docker Desktop – "Docker is running".
2. PostgreSQL 17 с PostGIS и OSM-данными региона (Канто для Tokyo-камер): `pg_isready`.
3. Redis: `redis-cli ping`.
4. `yt-dlp` в PATH (для HLS-proxy).
5. Порты 1420 (Vite) и 3001 (axum) свободны.

Подробнее см. [RUNBOOK.md](RUNBOOK.md).

---

## 16. Известные проблемы / заметки

- `3d-tiles-renderer` загрузчик закомментирован в `BuildingsLayer.tsx` до `npm install`.
- HLS-видеотекстура закомментирована в `CameraFrustum.tsx` – живая часть идёт через hls.js proxy.
- Белые плоскости вместо видео = запросы на go2rtc. Убрать `VITE_GO2RTC_URL` из `.env` и `launch.json`.
- Сегменты 403/410 = истёк YouTube HLS URL → перезапустить Vite (сбросит кеш yt-dlp).
- Тайлы 502 = не запущен axum API.
- `gh auth login` сам по себе бесполезен – дефолтный fine-grained PAT не
  имеет прав на этот репо. Нужен токен с явно выбранным `IAmYourPlayboy/Placebo`
  и правами `Contents` + `Pull requests` (read & write). Подробности в RUNBOOK §6.
- YouTube anti-bot блокирует анонимный `yt-dlp -g` (с 2026). С residential
  IP вернёт `Sign in to confirm you're not a bot`. axum-прокси отрабатывает
  ошибку как 500+JSON. Пока не решено – планируется EU-VPS для прода.

---

## 17. Скиллы (используй, когда подходит)

- `tauri`, `tauri-architecture` – Tauri.
- `rust-pro` – Rust.
- `react-guidelines`, `frontend-design` – React/UI.
- `systematic-debugging` – любой баг (не "просто дебажу").
- `better-auth-best-practices`, `create-auth-skill`, `email-and-password-best-practices`, `two-factor-authentication-best-practices`, `better-auth-security-best-practices` – авторизация.
- `typescript-advanced-types` – сложные TS.
- `superpowers:brainstorming` – перед любой creative-работой.
- `superpowers:test-driven-development` – при имплементации.
- `superpowers:verification-before-completion` – перед "готово".

---

## 18. Ссылки

- Полный исходный контекст: [PLACEBO_CLAUDE_CODE_CONTEXT.md](PLACEBO_CLAUDE_CODE_CONTEXT.md).
- Runbook (**не редактировать без просьбы**): [RUNBOOK.md](RUNBOOK.md).
- Архитектура: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md), [docs/ARCHITECTURE_3D.md](docs/ARCHITECTURE_3D.md).
- Идеи пользователя: [IDEAS.md](IDEAS.md).
- Репо: https://github.com/IAmYourPlayboy/Placebo.git.
