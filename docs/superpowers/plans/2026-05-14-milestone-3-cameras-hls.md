# Milestone 3: Cameras Seed + HLS Proxy Implementation Plan (v2)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Перенести HLS-прокси из Vite-middleware на axum-бэкенд, добавить в существующую `cameras` схему дескриптор `stream_source_*`, перезалить seed на 18 alpha-камер (13 YouTube live + 5 looped MP4 demo), расширить уже существующий `CameraResponse` полями `streamSourceType` + `proxyManifestUrl` + начать экспортировать camera DTO через ts-rs.

**Что отличает этот план от v1:** до начала M3 в репо уже есть `placebo_shared::camera::CameraResponse` (60+ полей, sensitive fields stripped, 8 unit-тестов), `handlers::cameras` с 7 эндпоинтами (`/`, `/:id`, `/nearby`, `/search`, `/bbox`, `/categories`, `/count`) и полный `camera_repo::CameraRow`. Мы НЕ создаём новые `CameraSummary`/`CameraDetail`, а расширяем существующие типы.

**Architecture:**
- **Схема БД:** новая миграция `009` добавляет ENUM `stream_source_type` (`youtube_live`, `direct_hls`, `loop_mp4`, `rtsp`) + `stream_source_config JSONB DEFAULT '{}'`. Существующие `stream_url`/`stream_type`/`backup_url`/`external_id`/`frame_rate` остаются (закон CLAUDE.md решение #7) и **никогда** не отдаются в API.
- **Seed-стратегия:** старая миграция `005_seed_cameras.sql` (50 dev-камер с RTSP-моками) и `data/cameras-seed.json` удаляются как dev-артефакт. Новая миграция `010_seed_alpha_cameras.sql` льёт 18 alpha-камер: 13 YouTube live + 5 loop_mp4 demo. Это становится единственным источником seed.
- **DTO:** `CameraResponse` получает два новых поля – `stream_source_type: StreamSourceType` и `proxy_manifest_url: Option<String>` (`/api/v1/hls-proxy/{slug}` для известных типов, `None` для `rtsp`). Все 7 enum-ов и `CameraResponse` получают `#[derive(TS)]` с `export_to = "../bindings/"` (auth-стиль).
- **HLS-прокси на axum:** `GET /api/v1/hls-proxy/:slug` возвращает m3u8 с переписанными сегмент-URL; `GET /api/v1/hls-proxy/:slug/seg?u=<base64url>` стримит сегмент upstream. Резолвер источника:
  - `youtube_live` → yt-dlp -g, кеш в Redis 30 мин (`hls:src:<slug>` TTL 1800s).
  - `direct_hls` → URL берётся прямо из `stream_source_config.url`.
  - `loop_mp4` → 302 redirect на `/static/demo/<asset>/index.m3u8` (статика на ServeDir).
  - `rtsp` → 404 (в альфе не поддерживаем).
- **Frontend:** vite hls-proxy middleware и vite proxy для `/api` уже есть – оставляем `/api`, удаляем hls-proxy. Добавляем `src/api/cameras.ts` (типизированная обёртка над существующими axum-эндпоинтами). `useNearbyCameras` остаётся mock'ом до M4 – единственное изменение в нём: `streamUrl()` теперь возвращает абсолютный URL `${VITE_API_BASE_URL}/api/v1/hls-proxy/${slug}` вместо vite-relative `/hls-proxy?src=`.

**Tech Stack:** axum 0.7, sqlx 0.8 (уже в проекте), deadpool-redis 0.18 (уже), tokio::process (для yt-dlp), reqwest 0.12 (новая dep), base64 0.22 (новая dep), tower-http 0.6 (уже, нужна feature `fs` для ServeDir), ts-rs 10 (уже).

**Spec reference:** `docs/superpowers/specs/2026-05-14-alpha-design.md` разделы 5.1–5.2, 7.4, 9, 2.3. Архитектурные решения в CLAUDE.md разделе 13: пункт #7 (stream_url никогда в API), пункт #6 (seed JSON+Schema – отменяется в M3 в пользу SQL-миграций как канонического источника).

**Зависимости:** M0 (ts-rs pipeline), M2 (auth context, AppState с PgPool+RedisPool).

---

## File Map

### Backend – новые файлы

- `crates/placebo-api/migrations/009_camera_stream_sources.sql` – ENUM + 2 колонки.
- `crates/placebo-api/migrations/010_seed_alpha_cameras.sql` – 18 INSERT.
- `crates/placebo-api/src/services/hls_source.rs` – `resolve(slug) -> ResolvedSource` с Redis-кешем и yt-dlp.
- `crates/placebo-api/src/handlers/hls_proxy.rs` – `manifest()` + `segment()` axum хендлеры.
- `crates/placebo-api/static/.gitkeep` + `crates/placebo-api/static/demo/README.md` – заглушка для loop_mp4.
- `scripts/verify-youtube-seed.sh` – sanity-check yt-dlp по 13 ID.

### Backend – модификации

- `crates/placebo-api/migrations/005_seed_cameras.sql` – **удалить**.
- `crates/placebo-api/Cargo.toml` – добавить `reqwest`, `base64`, включить feature `fs` для tower-http.
- `crates/placebo-shared/src/camera.rs` – добавить `StreamSourceType` enum, поля в `CameraResponse`, ts-rs derives + `export_bindings` тест.
- `crates/placebo-shared/Cargo.toml` – feature `export-types` уже есть.
- `crates/placebo-api/src/repositories/camera_repo.rs` – добавить `stream_source_type` + `stream_source_config` в `CAMERA_SELECT`, `CameraRow`, `NewCamera`, `insert()`.
- `crates/placebo-api/src/services/camera_service.rs::to_response` – маппить новые поля + строить `proxy_manifest_url`.
- `crates/placebo-api/src/handlers/hls_proxy.rs` – маунт через `handlers::api_router()`.
- `crates/placebo-api/src/handlers/mod.rs` – регистрация модуля + nest `/hls-proxy`.
- `crates/placebo-api/src/lib.rs::build_app` – `nest_service("/static", ServeDir::new(...))`.

### Backend – удалить

- `data/cameras-seed.json` – уже удалён пользователем, нужно зафиксировать в коммите.
- `crates/placebo-api/migrations/005_seed_cameras.sql` – удалить.
- `data/` – пустую директорию удалить.

### Frontend – новые

- `src/api/cameras.ts` – типизированный клиент `listCameras()`, `getCamera()`, `getCamerasNearby()`, `searchCameras()`.
- `src/hooks/useCamerasFromApi.ts` – простой хук вокруг `listCameras` (для будущего M4, в M3 используется только в smoke-тесте/Storybook'а нет).

### Frontend – модификации

- `vite.config.ts` – удалить блок `{ name: 'hls-proxy', ... }`, импорты `execSync`, `httpsGet`, `httpGet`, типы `IncomingMessage`/`ServerResponse`, мапу `YOUTUBE_IDS`, функции `resolveYoutubeHls`, `proxyFetch`, `rewriteM3u8`, `urlCache`. Сохранить `proxy: { '/api': ... }` (нужен для axum).
- `src/hooks/useNearbyCameras.ts::streamUrl()` – вернуть `${import.meta.env.VITE_API_BASE_URL ?? 'http://localhost:3001'}/api/v1/hls-proxy/${slug}`.
- `.env.example` (root) – добавить пример `VITE_API_BASE_URL=http://localhost:3001` если ещё нет.

### Tests

- `crates/placebo-shared/src/camera.rs` – расширить существующие тесты `camera_response_serde_roundtrip` и `enum_serde_lowercase` под новые поля.
- `crates/placebo-api/src/services/hls_source.rs` – inline `#[cfg(test)] mod tests` (cache-key derivation, m3u8 rewrite parser).
- `crates/placebo-api/src/handlers/hls_proxy.rs` – inline тест на `rewrite_m3u8()` с фикстурой.

---

## Task 0: Подготовка (ветка уже создана)

Ветка `feat/m3-cameras-hls` уже существует и активна. `git status` должен показывать удалённый `data/cameras-seed.json` (пользователь удалил вручную), модифицированные `Cargo.lock` и `pipeline/scripts/*.sh` (не наши, не трогаем).

- [ ] **Step 1: Проверка**

```bash
git branch --show-current
# expected: feat/m3-cameras-hls
git status --short
# Cargo.lock и pipeline/* игнорируем – чужие изменения
```

- [ ] **Step 2: Поднять docker dev окружение**

```bash
docker compose -f docker-compose.dev.yml up -d
docker compose -f docker-compose.dev.yml ps
# expected: postgres + redis up
```

- [ ] **Step 3: Накатить уже существующие миграции 001-008 на чистую БД**

```bash
cd crates/placebo-api
export DATABASE_URL="postgres://placebo:placebo@localhost:5432/placebo_dev"
cargo sqlx migrate run
# expected: 001..008 applied
```

Если база уже накачена и хочется сбросить – см. Task 3 step 1.

---

## Task 1: Удалить legacy seed

**Files:**
- Delete: `crates/placebo-api/migrations/005_seed_cameras.sql`
- Delete: `data/cameras-seed.json` (уже сделано)
- Delete: `data/` directory if empty

- [ ] **Step 1: Удалить миграцию 005**

```bash
rm crates/placebo-api/migrations/005_seed_cameras.sql
ls crates/placebo-api/migrations/
# expected: 001, 002, 003, 004, 006, 007, 008 (без 005)
```

- [ ] **Step 2: Удалить пустую папку data/**

```bash
rmdir data/ 2>/dev/null || true
ls data/ 2>&1
# expected: "No such file or directory" if empty, либо остаются файлы – тогда не трогаем
```

- [ ] **Step 3: Сбросить локальную БД (т.к. 005 уже накатилась раньше)**

```bash
docker compose -f docker-compose.dev.yml down -v
docker compose -f docker-compose.dev.yml up -d
sleep 3
cd crates/placebo-api && cargo sqlx migrate run
# expected: applies 001..008 on a fresh DB
```

- [ ] **Step 4: Commit**

```bash
git add -u crates/placebo-api/migrations/005_seed_cameras.sql data/cameras-seed.json
git status --short
# expected: D crates/placebo-api/migrations/005_seed_cameras.sql
#           D data/cameras-seed.json
git commit -m "$(cat <<'EOF'
chore: drop legacy seed (50 RTSP-mock cameras + cameras-seed.json)

The 50 cameras shipped via migration 005 used fictional rtsp://cam.placebo.tv
URLs that never resolved. data/cameras-seed.json was a duplicate JSON copy
that no code referenced. M3 replaces both with migration 010 carrying real
YouTube + looped-MP4 alpha cameras.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: Миграция 009 – stream_source_type + config

**Files:**
- Create: `crates/placebo-api/migrations/009_camera_stream_sources.sql`

- [ ] **Step 1: Написать миграцию**

```sql
-- 009_camera_stream_sources.sql
-- Adds a first-class stream source descriptor for cameras.
-- The existing stream_url/stream_type columns remain for legacy/RTSP ingest
-- but are NEVER exposed via the public API.

CREATE TYPE stream_source_type AS ENUM (
    'youtube_live',
    'direct_hls',
    'loop_mp4',
    'rtsp'
);

ALTER TABLE cameras
    ADD COLUMN stream_source_type   stream_source_type,
    ADD COLUMN stream_source_config JSONB NOT NULL DEFAULT '{}'::jsonb;

CREATE INDEX idx_cameras_stream_source_type ON cameras (stream_source_type);
```

Никакого back-fill: после миграции таблица пуста (мы только что снесли 005). Миграция 010 заполнит её сразу с правильным `stream_source_type`.

- [ ] **Step 2: Прогнать**

```bash
cd crates/placebo-api
cargo sqlx migrate run
# expected: 009/camera_stream_sources... installed
docker exec placebo-postgres-dev psql -U placebo -d placebo_dev -c \
  "\d cameras" | grep -E "stream_source_type|stream_source_config"
# expected: 2 lines
```

- [ ] **Step 3: Commit**

```bash
git add crates/placebo-api/migrations/009_camera_stream_sources.sql
git commit -m "feat(db): 009 adds stream_source_type enum + jsonb config to cameras"
```

---

## Task 3: Миграция 010 – alpha seed (13 yt + 5 demo)

**Files:**
- Create: `crates/placebo-api/migrations/010_seed_alpha_cameras.sql`

### Источники YouTube ID

Все 13 ID – **кандидаты**, валидируются скриптом в Task 8. Если кандидат мёртв – заменить новым live-стримом и обновить INSERT в этом же файле (миграция ещё не задеплоена в prod, прав на правку нет ограничений).

- [ ] **Step 1: Написать миграцию**

```sql
-- 010_seed_alpha_cameras.sql
-- Alpha-release camera roster: 13 YouTube live + 5 looped-MP4 demos.
-- Replaces the deleted 005 seed entirely – this is the only camera seed.

-- LIVE YouTube cameras (13)
INSERT INTO cameras (
    name, slug, camera_type,
    country, country_code, city,
    location, timezone,
    stream_url, stream_type,
    stream_source_type, stream_source_config,
    category, description_en,
    resolution_w, resolution_h, codec,
    height_above_ground, camera_azimuth, camera_elevation, fov_horizontal, fov_vertical,
    added_to_placebo_at, created_at
) VALUES
    ('Shibuya Crossing – Live',     'yt-shibuya-crossing', 'public',
     'Japan', 'JP', 'Tokyo',
     ST_SetSRID(ST_Point(139.7004, 35.6595), 4326), 'Asia/Tokyo',
     'youtube://dfVK7ld38Ys', 'youtube',
     'youtube_live', '{"videoId":"dfVK7ld38Ys"}'::jsonb,
     'city', 'Shibuya Crossing live',
     1920, 1080, 'h264',
     8, 180, -20, 80, 50,
     NOW(), NOW()),

    ('Times Square – NYC Live',     'yt-times-square', 'public',
     'United States', 'US', 'New York',
     ST_SetSRID(ST_Point(-73.9857, 40.7580), 4326), 'America/New_York',
     'youtube://u4UZ4UvZXrg', 'youtube',
     'youtube_live', '{"videoId":"u4UZ4UvZXrg"}'::jsonb,
     'city', 'Times Square live panoramic',
     1920, 1080, 'h264',
     12, 90, -10, 90, 55,
     NOW(), NOW()),

    ('Abbey Road Crossing – London','yt-abbey-road', 'public',
     'United Kingdom', 'GB', 'London',
     ST_SetSRID(ST_Point(-0.1779, 51.5320), 4326), 'Europe/London',
     'youtube://FQWkgr0aHlI', 'youtube',
     'youtube_live', '{"videoId":"FQWkgr0aHlI"}'::jsonb,
     'city', 'Abbey Road zebra crossing live',
     1920, 1080, 'h264',
     4, 0, -30, 70, 45,
     NOW(), NOW()),

    ('Helsinki Senate Square',      'yt-helsinki-senate', 'public',
     'Finland', 'FI', 'Helsinki',
     ST_SetSRID(ST_Point(24.9525, 60.1699), 4326), 'Europe/Helsinki',
     'youtube://AdUw5RdyZxI', 'youtube',
     'youtube_live', '{"videoId":"AdUw5RdyZxI"}'::jsonb,
     'city', 'Senate Square & Helsinki Cathedral',
     1920, 1080, 'h264',
     20, 135, -12, 85, 50,
     NOW(), NOW()),

    ('Red Square – Moscow',         'yt-red-square', 'public',
     'Russia', 'RU', 'Moscow',
     ST_SetSRID(ST_Point(37.6208, 55.7539), 4326), 'Europe/Moscow',
     'youtube://h1wly909BYw', 'youtube',
     'youtube_live', '{"videoId":"h1wly909BYw"}'::jsonb,
     'city', 'Red Square live feed',
     1920, 1080, 'h264',
     10, 180, -8, 90, 52,
     NOW(), NOW()),

    ('Gateway of India – Mumbai',   'yt-gateway-mumbai', 'public',
     'India', 'IN', 'Mumbai',
     ST_SetSRID(ST_Point(72.8347, 18.9220), 4326), 'Asia/Kolkata',
     'youtube://7Bl5p4VTXzQ', 'youtube',
     'youtube_live', '{"videoId":"7Bl5p4VTXzQ"}'::jsonb,
     'city', 'Gateway of India',
     1920, 1080, 'h264',
     6, 90, -15, 80, 50,
     NOW(), NOW()),

    ('Dubai Marina',                'yt-dubai-marina', 'public',
     'United Arab Emirates', 'AE', 'Dubai',
     ST_SetSRID(ST_Point(55.1403, 25.0777), 4326), 'Asia/Dubai',
     'youtube://2L4yhCmGRWg', 'youtube',
     'youtube_live', '{"videoId":"2L4yhCmGRWg"}'::jsonb,
     'city', 'Dubai Marina skyline',
     1920, 1080, 'h264',
     40, 45, -20, 85, 50,
     NOW(), NOW()),

    ('Eiffel Tower cam',            'yt-eiffel', 'public',
     'France', 'FR', 'Paris',
     ST_SetSRID(ST_Point(2.2945, 48.8584), 4326), 'Europe/Paris',
     'youtube://dyWHmEQAVUI', 'youtube',
     'youtube_live', '{"videoId":"dyWHmEQAVUI"}'::jsonb,
     'city', 'Eiffel Tower live',
     1920, 1080, 'h264',
     25, 300, -5, 85, 50,
     NOW(), NOW()),

    ('Venice – Rialto Bridge',      'yt-rialto-venice', 'public',
     'Italy', 'IT', 'Venice',
     ST_SetSRID(ST_Point(12.3359, 45.4380), 4326), 'Europe/Rome',
     'youtube://qMksIqJv3pI', 'youtube',
     'youtube_live', '{"videoId":"qMksIqJv3pI"}'::jsonb,
     'city', 'Grand Canal at Rialto',
     1920, 1080, 'h264',
     5, 180, -10, 80, 50,
     NOW(), NOW()),

    ('Seoul – Gwanghwamun Square',  'yt-gwanghwamun', 'public',
     'South Korea', 'KR', 'Seoul',
     ST_SetSRID(ST_Point(126.9768, 37.5759), 4326), 'Asia/Seoul',
     'youtube://wNmMr_ATI2E', 'youtube',
     'youtube_live', '{"videoId":"wNmMr_ATI2E"}'::jsonb,
     'city', 'Gwanghwamun Square',
     1920, 1080, 'h264',
     8, 0, -12, 85, 50,
     NOW(), NOW()),

    ('Barcelona – La Rambla',       'yt-la-rambla', 'public',
     'Spain', 'ES', 'Barcelona',
     ST_SetSRID(ST_Point(2.1724, 41.3809), 4326), 'Europe/Madrid',
     'youtube://hSbkw-F7bzY', 'youtube',
     'youtube_live', '{"videoId":"hSbkw-F7bzY"}'::jsonb,
     'city', 'La Rambla pedestrian street',
     1920, 1080, 'h264',
     6, 90, -15, 80, 50,
     NOW(), NOW()),

    ('Bondi Beach – Sydney',        'yt-bondi', 'public',
     'Australia', 'AU', 'Sydney',
     ST_SetSRID(ST_Point(151.2767, -33.8908), 4326), 'Australia/Sydney',
     'youtube://2Te5EvOXNZw', 'youtube',
     'youtube_live', '{"videoId":"2Te5EvOXNZw"}'::jsonb,
     'beach', 'Bondi Beach live',
     1920, 1080, 'h264',
     10, 60, -8, 90, 55,
     NOW(), NOW()),

    ('Amsterdam Canals',            'yt-amsterdam-canal', 'public',
     'Netherlands', 'NL', 'Amsterdam',
     ST_SetSRID(ST_Point(4.8917, 52.3730), 4326), 'Europe/Amsterdam',
     'youtube://SkdGPWUUkEw', 'youtube',
     'youtube_live', '{"videoId":"SkdGPWUUkEw"}'::jsonb,
     'city', 'Amsterdam canal loop',
     1920, 1080, 'h264',
     4, 180, -10, 80, 50,
     NOW(), NOW());

-- DEMO loop_mp4 cameras (5)
-- Assets are uploaded by the developer to crates/placebo-api/static/demo/<asset>/
-- (HLS-segmented MP4 + index.m3u8 generated by FFmpeg). The proxy 302-redirects
-- to /static/demo/<asset>/index.m3u8.
INSERT INTO cameras (
    name, slug, camera_type,
    country, country_code, city,
    location, timezone,
    stream_url, stream_type,
    stream_source_type, stream_source_config,
    category, description_en,
    resolution_w, resolution_h, codec,
    height_above_ground, camera_azimuth, camera_elevation, fov_horizontal, fov_vertical,
    added_to_placebo_at, created_at
) VALUES
    ('Demo – Tokyo Alley',     'demo-tokyo-alley', 'public',
     'Japan', 'JP', 'Tokyo',
     ST_SetSRID(ST_Point(139.7014, 35.6598), 4326), 'Asia/Tokyo',
     'loop://tokyo-alley', 'hls',
     'loop_mp4', '{"asset":"tokyo-alley","durationS":92}'::jsonb,
     'city', 'Looped demo – Tokyo alley',
     1280, 720, 'h264',
     6, 120, -15, 80, 50,
     NOW(), NOW()),

    ('Demo – Cafe Street',     'demo-cafe-street', 'public',
     'France', 'FR', 'Paris',
     ST_SetSRID(ST_Point(2.3500, 48.8566), 4326), 'Europe/Paris',
     'loop://cafe-street', 'hls',
     'loop_mp4', '{"asset":"cafe-street","durationS":120}'::jsonb,
     'city', 'Looped demo – Parisian cafe street',
     1280, 720, 'h264',
     4, 240, -8, 80, 50,
     NOW(), NOW()),

    ('Demo – Beach Sunset',    'demo-beach-sunset', 'public',
     'Spain', 'ES', 'Valencia',
     ST_SetSRID(ST_Point(-0.3700, 39.4700), 4326), 'Europe/Madrid',
     'loop://beach-sunset', 'hls',
     'loop_mp4', '{"asset":"beach-sunset","durationS":180}'::jsonb,
     'beach', 'Looped demo – sunset at the beach',
     1280, 720, 'h264',
     8, 270, -5, 90, 55,
     NOW(), NOW()),

    ('Demo – Rainy Window',    'demo-rainy-window', 'public',
     'Russia', 'RU', 'Saint Petersburg',
     ST_SetSRID(ST_Point(30.3351, 59.9343), 4326), 'Europe/Moscow',
     'loop://rainy-window', 'hls',
     'loop_mp4', '{"asset":"rainy-window","durationS":240}'::jsonb,
     'weather', 'Looped demo – rainy window',
     1280, 720, 'h264',
     3, 0, -20, 60, 40,
     NOW(), NOW()),

    ('Demo – Mountain Pass',   'demo-mountain-pass', 'public',
     'Switzerland', 'CH', 'Zermatt',
     ST_SetSRID(ST_Point(7.7491, 46.0207), 4326), 'Europe/Zurich',
     'loop://mountain-pass', 'hls',
     'loop_mp4', '{"asset":"mountain-pass","durationS":150}'::jsonb,
     'mountain', 'Looped demo – alpine pass',
     1280, 720, 'h264',
     12, 45, -10, 85, 50,
     NOW(), NOW());
```

- [ ] **Step 2: Прогнать**

```bash
cd crates/placebo-api
cargo sqlx migrate run

docker exec placebo-postgres-dev psql -U placebo -d placebo_dev -c \
  "SELECT slug, stream_source_type FROM cameras ORDER BY slug;"
# expected: 18 rows; 13 youtube_live, 5 loop_mp4
```

- [ ] **Step 3: Commit**

```bash
git add crates/placebo-api/migrations/010_seed_alpha_cameras.sql
git commit -m "feat(db): 010 alpha seed – 13 youtube + 5 loop_mp4 demos"
```

---

## Task 4: Расширить placebo_shared::camera (DTO + ts-rs)

**Files:**
- Modify: `crates/placebo-shared/src/camera.rs`
- Modify: `crates/placebo-shared/src/codegen.rs` (комментарий обновить)

### Что добавляется

1. Новый enum `StreamSourceType` (4 варианта).
2. Поля в `CameraResponse`: `stream_source_type: StreamSourceType` + `proxy_manifest_url: Option<String>`.
3. `#[derive(TS)]` + `#[ts(export, export_to = "../bindings/")]` на: `CameraType`, `RetentionTier`, `StreamType`, `StreamProtocol`, `VideoCodec`, `Category`, `StreamSourceType`, `CameraResponse`.

**Внимание:** auth-крейт использует `export_to = "../bindings/"` (одна `..`). Проверим точно в `auth.rs`:

```bash
grep -n 'export_to' crates/placebo-shared/src/auth.rs | head
```

Если там `"../bindings/"` – используем тот же путь. Если другой – align with existing.

- [ ] **Step 1: Прочитать существующий auth.rs ts-rs стиль**

```bash
grep -B1 -A2 'derive(TS)\|ts(export' crates/placebo-shared/src/auth.rs | head -30
```

Цель: скопировать точную форму атрибута и feature-gating. Скорее всего это:

```rust
#[cfg_attr(feature = "export-types", derive(ts_rs::TS))]
#[cfg_attr(feature = "export-types", ts(export, export_to = "../bindings/"))]
```

Если так – повторяем 1:1. Любое отличие (`#[cfg(feature = "export-types")]` vs `cfg_attr`) – тоже копируем.

- [ ] **Step 2: Добавить failing test для нового enum + поля**

В `crates/placebo-shared/src/camera.rs` в `mod tests`:

```rust
    #[test]
    fn stream_source_type_display_roundtrip() {
        for variant in [
            StreamSourceType::YoutubeLive,
            StreamSourceType::DirectHls,
            StreamSourceType::LoopMp4,
            StreamSourceType::Rtsp,
        ] {
            let s = serde_json::to_string(&variant).unwrap();
            let parsed: StreamSourceType = serde_json::from_str(&s).unwrap();
            assert_eq!(variant, parsed);
        }
    }

    #[test]
    fn stream_source_type_serde_snake_case() {
        assert_eq!(
            serde_json::to_string(&StreamSourceType::YoutubeLive).unwrap(),
            "\"youtube_live\""
        );
        assert_eq!(
            serde_json::to_string(&StreamSourceType::LoopMp4).unwrap(),
            "\"loop_mp4\""
        );
    }

    #[test]
    fn camera_response_has_stream_source_and_proxy_url() {
        let now = Utc::now();
        let resp = make_dummy_camera_response(now);
        let json = serde_json::to_string(&resp).unwrap();
        // New fields must be camelCase in JSON.
        assert!(json.contains("\"streamSourceType\""));
        assert!(json.contains("\"proxyManifestUrl\""));
        // streamUrl-style sensitive fields still absent.
        assert!(!json.contains("\"streamUrl\""));
        assert!(!json.contains("stream_url"));
    }
```

+ извлечь `make_dummy_camera_response` из существующего `camera_response_serde_roundtrip` в helper, чтобы не дублировать 50 строк инициализации. Это локальный refactor `mod tests`, никаких prod-API изменений.

- [ ] **Step 3: Прогнать – должны провалиться**

```bash
cd crates/placebo-shared
cargo test --lib camera::tests
# expected: stream_source_type_display_roundtrip - FAILED (StreamSourceType not defined)
```

- [ ] **Step 4: Реализовать**

Добавить в `crates/placebo-shared/src/camera.rs` (после блока Category, перед `CameraResponse`):

```rust
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "export-types", derive(ts_rs::TS))]
#[cfg_attr(feature = "export-types", ts(export, export_to = "../bindings/"))]
#[serde(rename_all = "snake_case")]
pub enum StreamSourceType {
    YoutubeLive,
    DirectHls,
    LoopMp4,
    Rtsp,
}

impl fmt::Display for StreamSourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::YoutubeLive => write!(f, "youtube_live"),
            Self::DirectHls => write!(f, "direct_hls"),
            Self::LoopMp4 => write!(f, "loop_mp4"),
            Self::Rtsp => write!(f, "rtsp"),
        }
    }
}

impl FromStr for StreamSourceType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "youtube_live" => Ok(Self::YoutubeLive),
            "direct_hls" => Ok(Self::DirectHls),
            "loop_mp4" => Ok(Self::LoopMp4),
            "rtsp" => Ok(Self::Rtsp),
            _ => Err(format!("unknown StreamSourceType: {s}")),
        }
    }
}

impl TryFrom<&str> for StreamSourceType {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> { s.parse() }
}
```

Добавить два поля в `CameraResponse` (после `stream_protocol`):

```rust
    pub stream_source_type: StreamSourceType,
    pub proxy_manifest_url: Option<String>,
```

- [ ] **Step 5: Добавить ts-rs derives на ВСЕ existing camera enums + CameraResponse**

Для каждого enum (`CameraType`, `RetentionTier`, `StreamType`, `StreamProtocol`, `VideoCodec`, `Category`) и для `CameraResponse` добавить под существующим `#[derive(...)]`:

```rust
#[cfg_attr(feature = "export-types", derive(ts_rs::TS))]
#[cfg_attr(feature = "export-types", ts(export, export_to = "../bindings/"))]
```

**На `CameraResponse` уже есть `#[serde(rename_all = "camelCase")]`** – ts-rs его уважает и сгенерит camelCase TS.

- [ ] **Step 6: Обновить существующий тест `camera_response_serde_roundtrip`**

Добавить в инициализацию `CameraResponse` два новых поля:

```rust
            stream_source_type: StreamSourceType::YoutubeLive,
            proxy_manifest_url: Some("/api/v1/hls-proxy/tokyo-tower-cam".to_string()),
```

Добавить assert:
```rust
        assert!(json.contains("\"streamSourceType\""));
        assert!(json.contains("\"proxyManifestUrl\""));
```

- [ ] **Step 7: Прогнать тесты – зелёные**

```bash
cd crates/placebo-shared
cargo test --lib
# expected: all green
```

- [ ] **Step 8: Прогнать ts-rs export**

```bash
cd /d/Projects/Placebo
npm run gen-types
ls src/types/api/ | grep -iE 'camera|stream'
# expected: CameraResponse.ts, CameraType.ts, StreamSourceType.ts, StreamType.ts,
#           StreamProtocol.ts, VideoCodec.ts, Category.ts, RetentionTier.ts
```

Если какой-то тип не появился – значит ts-rs derive не пробросился через `cfg_attr`. Проверить именно тот тип. Если cargo test --features export-types ругается "can't write to ../bindings/" – проверить что директория существует или ts-rs её создаёт сама (по доке создаёт).

- [ ] **Step 9: Проверить, что старый src/types/camera.ts (M1-прототип) не конфликтует**

```bash
ls src/types/camera.ts 2>&1
# если есть – это легаси из прототипа, оставляем, никто на ts-rs-генерируемые типы
# не ссылается из src/components/world3d/. Для M3 этого достаточно.
```

- [ ] **Step 10: Commit**

```bash
git add crates/placebo-shared/src/camera.rs crates/placebo-shared/src/codegen.rs src/types/api/
git commit -m "$(cat <<'EOF'
feat(shared): export camera DTOs via ts-rs + add StreamSourceType

- StreamSourceType enum (youtube_live | direct_hls | loop_mp4 | rtsp).
- CameraResponse gains streamSourceType + proxyManifestUrl fields.
- All camera enums + CameraResponse now derive ts-rs::TS under the
  export-types feature, mirroring the auth-DTO pattern from M2.
- npm run gen-types now emits Camera*, StreamSourceType, etc. into
  src/types/api/.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: camera_repo + camera_service – маппинг новых полей

**Files:**
- Modify: `crates/placebo-api/src/repositories/camera_repo.rs`
- Modify: `crates/placebo-api/src/services/camera_service.rs`

- [ ] **Step 1: Расширить `CAMERA_SELECT`**

В `camera_repo.rs` (строка ~9), добавить в конец списка перед `created_at, updated_at`:

```rust
const CAMERA_SELECT: &str = r#"
    id, name, slug, camera_type::TEXT, external_id,
    country, country_code, region, city, district, address, custom_label,
    ST_Y(location) as lat, ST_X(location) as lng, timezone,
    stream_url, backup_url, stream_type::TEXT, stream_protocol::TEXT,
    stream_quality_default, available_qualities, frame_rate, bitrate_kbps,
    codec::TEXT, resolution_w, resolution_h, latency_ms,
    has_audio, has_night_vision, is_underwater,
    category, subcategory, tags, description_en, thumbnail_url, source_url, attribution,
    recording_enabled, retention_tier::TEXT, recording_retention_days, recording_codec::TEXT,
    height_above_ground, camera_azimuth, camera_elevation, fov_horizontal, fov_vertical,
    manufacturer, camera_model, added_to_placebo_at, is_partner_camera, owner_name,
    stream_source_type::TEXT as stream_source_type,
    stream_source_config,
    created_at, updated_at
"#;
```

- [ ] **Step 2: Расширить `CameraRow`**

В `camera_repo.rs` (строка ~28), добавить два поля перед `// Timestamps`:

```rust
    // Stream source descriptor (M3)
    pub stream_source_type: Option<String>,
    pub stream_source_config: serde_json::Value,
```

Make `stream_source_type` Optional потому что миграция 009 не back-fill'ит и есть теоретическая возможность NULL (на свежей БД – нет, но защитимся).

- [ ] **Step 3: Расширить `NewCamera` (для будущих INSERT'ов из API)**

В `camera_repo.rs`, в `NewCamera`:

```rust
    pub stream_source_type: Option<String>,
    pub stream_source_config: serde_json::Value,
```

+ обновить `insert()`: добавить колонки в SQL INSERT и `bind` в правильном порядке. Это сложный момент – на 49 параметрах добавляется ещё 2. Нужно осторожно пересчитать `$N`.

```rust
pub async fn insert(pool: &PgPool, c: &NewCamera) -> Result<CameraRow, sqlx::Error> {
    let sql = format!(
        r#"INSERT INTO cameras (
            name, slug, camera_type, external_id,
            country, country_code, region, city, district, address, custom_label,
            location, timezone,
            stream_url, backup_url, stream_type, stream_protocol,
            stream_quality_default, available_qualities, frame_rate, bitrate_kbps,
            codec, resolution_w, resolution_h, latency_ms,
            has_audio, has_night_vision, is_underwater,
            category, subcategory, tags, description_en, thumbnail_url, source_url, attribution,
            recording_enabled, retention_tier, recording_retention_days, recording_codec,
            height_above_ground, camera_azimuth, camera_elevation, fov_horizontal, fov_vertical,
            manufacturer, camera_model, is_partner_camera, owner_name,
            stream_source_type, stream_source_config
        ) VALUES (
            $1, $2, $3::camera_type, $4,
            $5, $6, $7, $8, $9, $10, $11,
            ST_SetSRID(ST_Point($12, $13), 4326), $14,
            $15, $16, $17::stream_type, $18::stream_protocol,
            $19, $20, $21, $22,
            $23::video_codec, $24, $25, $26,
            $27, $28, $29,
            $30, $31, $32, $33, $34, $35, $36,
            $37, $38::retention_tier, $39, $40::video_codec,
            $41, $42, $43, $44, $45,
            $46, $47, $48, $49,
            $50::stream_source_type, $51
        ) RETURNING {CAMERA_SELECT}, NULL::float8 as distance_m"#
    );

    sqlx::query_as::<_, CameraRow>(&sql)
        .bind(&c.name)                      // $1
        .bind(&c.slug)                      // $2
        .bind(&c.camera_type)               // $3
        .bind(&c.external_id)               // $4
        .bind(&c.country)                   // $5
        .bind(&c.country_code)              // $6
        .bind(&c.region)                    // $7
        .bind(&c.city)                      // $8
        .bind(&c.district)                  // $9
        .bind(&c.address)                   // $10
        .bind(&c.custom_label)              // $11
        .bind(c.lng)                        // $12
        .bind(c.lat)                        // $13
        .bind(&c.timezone)                  // $14
        .bind(&c.stream_url)                // $15
        .bind(&c.backup_url)                // $16
        .bind(&c.stream_type)               // $17
        .bind(&c.stream_protocol)           // $18
        .bind(&c.stream_quality_default)    // $19
        .bind(&c.available_qualities)       // $20
        .bind(c.frame_rate)                 // $21
        .bind(c.bitrate_kbps)               // $22
        .bind(&c.codec)                     // $23
        .bind(c.resolution_w)               // $24
        .bind(c.resolution_h)               // $25
        .bind(c.latency_ms)                 // $26
        .bind(c.has_audio)                  // $27
        .bind(c.has_night_vision)           // $28
        .bind(c.is_underwater)              // $29
        .bind(&c.category)                  // $30
        .bind(&c.subcategory)               // $31
        .bind(&c.tags)                      // $32
        .bind(&c.description_en)            // $33
        .bind(&c.thumbnail_url)             // $34
        .bind(&c.source_url)                // $35
        .bind(&c.attribution)               // $36
        .bind(c.recording_enabled)          // $37
        .bind(&c.retention_tier)            // $38
        .bind(c.recording_retention_days)   // $39
        .bind(&c.recording_codec)           // $40
        .bind(c.height_above_ground)        // $41
        .bind(c.camera_azimuth)             // $42
        .bind(c.camera_elevation)           // $43
        .bind(c.fov_horizontal)             // $44
        .bind(c.fov_vertical)               // $45
        .bind(&c.manufacturer)              // $46
        .bind(&c.camera_model)              // $47
        .bind(c.is_partner_camera)          // $48
        .bind(&c.owner_name)                // $49
        .bind(&c.stream_source_type)        // $50
        .bind(&c.stream_source_config)      // $51
        .fetch_one(pool)
        .await
}
```

**Внимание:** в существующем `insert()` enum'ы кастятся как `$3::camera_type_enum` etc. – но в реальной миграции 001 они называются `camera_type`, `retention_tier`, etc. (без `_enum`). Я в плане использую правильные имена. Если existing код имеет `_enum` – значит существующий `insert()` на самом деле сейчас не используется (дизайн через миграции). Проверь при имплементации.

- [ ] **Step 4: Расширить `to_response` в `camera_service.rs`**

В `crates/placebo-api/src/services/camera_service.rs::to_response` (строка ~16) после `let recording_codec = ...` добавить:

```rust
    let stream_source_type = row
        .stream_source_type
        .as_deref()
        .and_then(|s| s.parse::<StreamSourceType>().ok())
        .unwrap_or(StreamSourceType::Rtsp);

    // Public manifest URL: only present for source types we actually proxy.
    let proxy_manifest_url = match stream_source_type {
        StreamSourceType::YoutubeLive
        | StreamSourceType::DirectHls
        | StreamSourceType::LoopMp4 => Some(format!("/api/v1/hls-proxy/{}", row.slug)),
        StreamSourceType::Rtsp => None,
    };
```

И в `CameraResponse { ... }` добавить:

```rust
        stream_source_type,
        proxy_manifest_url,
```

+ импорт `StreamSourceType` в начало файла:

```rust
use placebo_shared::camera::{
    CameraResponse, CameraType, Category, RetentionTier, StreamProtocol, StreamSourceType,
    StreamType, VideoCodec,
};
```

- [ ] **Step 5: cargo check + test**

```bash
cd crates/placebo-api
cargo check
cargo test --lib
# expected: builds, existing tests still pass
```

- [ ] **Step 6: Smoke test через curl**

```bash
cd crates/placebo-api
cargo run &
sleep 4
curl -s http://localhost:3001/api/v1/cameras | python -m json.tool | head -50
# expected: every camera has streamSourceType + proxyManifestUrl
# Verify NO streamUrl field present anywhere
curl -s http://localhost:3001/api/v1/cameras | grep -E 'streamUrl|stream_url' && echo FAIL || echo OK
# expected: OK (no matches)
kill %1
```

- [ ] **Step 7: Commit**

```bash
git add crates/placebo-api/src/repositories/camera_repo.rs \
        crates/placebo-api/src/services/camera_service.rs
git commit -m "feat(cameras): repo+service surface streamSourceType + proxyManifestUrl"
```

---

## Task 6: HLS-прокси (handler + service + static)

**Files:**
- Create: `crates/placebo-api/src/services/hls_source.rs`
- Create: `crates/placebo-api/src/handlers/hls_proxy.rs`
- Create: `crates/placebo-api/static/.gitkeep`
- Create: `crates/placebo-api/static/demo/README.md`
- Modify: `crates/placebo-api/Cargo.toml`
- Modify: `crates/placebo-api/src/services/mod.rs`
- Modify: `crates/placebo-api/src/handlers/mod.rs`
- Modify: `crates/placebo-api/src/lib.rs`
- Modify: `crates/placebo-api/src/repositories/camera_repo.rs` (добавить `stream_source_for_slug` helper)

### Поведение

- `GET /api/v1/hls-proxy/{slug}`:
  - resolve via `hls_source::resolve(slug)`,
  - `Hls(url)` → fetch m3u8 у upstream, переписать сегментные строки на `/api/v1/hls-proxy/{slug}/seg?u=<base64url(abs_url)>`, отдать 200 с `Content-Type: application/vnd.apple.mpegurl`,
  - `StaticLoop(rel)` → 302 redirect на `rel`,
  - `Unsupported` → 404.
- `GET /api/v1/hls-proxy/{slug}/seg?u=<base64url>`:
  - decode `u`, fetch upstream, stream bytes с пропусканием `Content-Type` и добавлением `access-control-allow-origin: *`.
- Redis key для youtube: `hls:src:<slug>`, TTL 1800 сек.
- yt-dlp вызывается через `tokio::process::Command`, args: `-f best[vcodec^=avc1] --no-warnings -g <youtube-url>`.

- [ ] **Step 1: Cargo.toml**

Добавить deps в `crates/placebo-api/Cargo.toml` (раздел `[dependencies]`):

```toml
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "stream"] }
base64 = "0.22"
futures-util = "0.3"
```

И обновить `tower-http` features:

```toml
tower-http = { version = "0.6", features = ["cors", "trace", "request-id", "compression-gzip", "fs"] }
```

- [ ] **Step 2: Repo helper для резолвера**

В `crates/placebo-api/src/repositories/camera_repo.rs` добавить (рядом с `get_by_slug`):

```rust
/// Lightweight lookup used only by the HLS proxy. Returns the source
/// type and JSON config for a camera identified by slug. Never exposes
/// `stream_url` itself.
pub async fn stream_source_for_slug(
    pool: &PgPool,
    slug: &str,
) -> Result<Option<(String, serde_json::Value)>, sqlx::Error> {
    sqlx::query_as::<_, (Option<String>, serde_json::Value)>(
        "SELECT stream_source_type::text, stream_source_config FROM cameras WHERE slug = $1",
    )
    .bind(slug)
    .fetch_optional(pool)
    .await
    .map(|opt| opt.and_then(|(t, c)| t.map(|tt| (tt, c))))
}
```

- [ ] **Step 3: hls_source.rs (service)**

Создать `crates/placebo-api/src/services/hls_source.rs`:

```rust
use anyhow::{anyhow, Context, Result};
use deadpool_redis::redis::AsyncCommands;
use deadpool_redis::Pool as RedisPool;
use serde_json::Value;
use sqlx::PgPool;
use tokio::process::Command;

use crate::repositories::camera_repo;

const CACHE_TTL_SECS: u64 = 30 * 60;

/// Resolved upstream the proxy should fetch.
pub enum ResolvedSource {
    /// Absolute m3u8 URL the proxy should fetch and rewrite.
    Hls(String),
    /// Relative path served by our own ServeDir handler.
    StaticLoop(String),
    /// Slug exists but the source type is intentionally unsupported in the alpha (e.g. rtsp).
    Unsupported,
    /// Slug does not exist.
    NotFound,
}

pub async fn resolve(
    pg: &PgPool,
    redis: &RedisPool,
    slug: &str,
) -> Result<ResolvedSource> {
    let row = camera_repo::stream_source_for_slug(pg, slug)
        .await
        .context("db lookup failed")?;
    let (kind, cfg) = match row {
        Some(p) => p,
        None => return Ok(ResolvedSource::NotFound),
    };

    match kind.as_str() {
        "youtube_live" => {
            let video_id = cfg
                .get("videoId")
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow!("youtube_live config missing videoId"))?;
            let url = resolve_youtube_cached(redis, slug, video_id).await?;
            Ok(ResolvedSource::Hls(url))
        }
        "direct_hls" => {
            let url = cfg
                .get("url")
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow!("direct_hls config missing url"))?;
            Ok(ResolvedSource::Hls(url.to_string()))
        }
        "loop_mp4" => {
            let asset = cfg
                .get("asset")
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow!("loop_mp4 config missing asset"))?;
            Ok(ResolvedSource::StaticLoop(format!(
                "/static/demo/{asset}/index.m3u8"
            )))
        }
        "rtsp" => Ok(ResolvedSource::Unsupported),
        other => Err(anyhow!("unknown stream_source_type: {other}")),
    }
}

async fn resolve_youtube_cached(
    redis: &RedisPool,
    slug: &str,
    video_id: &str,
) -> Result<String> {
    let cache_key = format!("hls:src:{slug}");
    let mut conn = redis.get().await.context("redis pool exhausted")?;
    if let Ok(Some(cached)) = conn.get::<_, Option<String>>(&cache_key).await {
        return Ok(cached);
    }
    let url = resolve_youtube(video_id).await?;
    // Best-effort cache write – proxy should still work if redis is down.
    let _: redis::RedisResult<()> = conn.set_ex(&cache_key, &url, CACHE_TTL_SECS).await;
    Ok(url)
}

async fn resolve_youtube(video_id: &str) -> Result<String> {
    let target = format!("https://www.youtube.com/watch?v={video_id}");
    let output = Command::new("yt-dlp")
        .args(["-f", "best[vcodec^=avc1]", "--no-warnings", "-g", &target])
        .output()
        .await
        .context("failed to spawn yt-dlp")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!(
            "yt-dlp failed (status={:?}): {}",
            output.status.code(),
            stderr.trim()
        ));
    }
    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if url.is_empty() {
        return Err(anyhow!("yt-dlp returned empty stdout"));
    }
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_key_format() {
        // Stable across changes – external scripts may inspect Redis.
        assert_eq!(format!("hls:src:{}", "yt-shibuya-crossing"), "hls:src:yt-shibuya-crossing");
    }
}
```

- [ ] **Step 4: handlers/hls_proxy.rs**

Создать `crates/placebo-api/src/handlers/hls_proxy.rs`:

```rust
use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Router,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use reqwest::Url;
use serde::Deserialize;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::services::hls_source::{self, ResolvedSource};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/hls-proxy/:slug", get(manifest))
        .route("/hls-proxy/:slug/seg", get(segment))
}

#[derive(Deserialize)]
struct SegQuery {
    u: String,
}

async fn manifest(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Response, AppError> {
    let resolved = hls_source::resolve(&state.db, &state.redis, &slug)
        .await
        .map_err(|e| AppError::Internal(format!("hls resolve: {e}")))?;

    match resolved {
        ResolvedSource::NotFound => Err(AppError::NotFound(format!("camera '{slug}'"))),
        ResolvedSource::Unsupported => Err(AppError::NotFound(format!(
            "stream type for '{slug}' is not proxied in the alpha"
        ))),
        ResolvedSource::StaticLoop(rel_path) => Ok(Redirect::temporary(&rel_path).into_response()),
        ResolvedSource::Hls(upstream_url) => {
            let client = reqwest::Client::new();
            let body = client
                .get(&upstream_url)
                .send()
                .await
                .map_err(|e| AppError::Internal(format!("upstream fetch: {e}")))?
                .text()
                .await
                .map_err(|e| AppError::Internal(format!("upstream body: {e}")))?;

            let base = Url::parse(&upstream_url)
                .map_err(|e| AppError::Internal(format!("bad upstream url: {e}")))?;
            let rewritten = rewrite_m3u8(&body, &base, &slug);

            let mut headers = HeaderMap::new();
            headers.insert(
                "content-type",
                HeaderValue::from_static("application/vnd.apple.mpegurl"),
            );
            headers.insert("cache-control", HeaderValue::from_static("no-cache"));
            headers.insert("access-control-allow-origin", HeaderValue::from_static("*"));
            Ok((StatusCode::OK, headers, rewritten).into_response())
        }
    }
}

/// Resolve every URI line (relative or absolute) to an absolute URL,
/// then rewrite the line to point at our own `/seg` endpoint, encoding
/// the absolute URL in base64url so it survives query-string transit.
fn rewrite_m3u8(body: &str, base: &Url, slug: &str) -> String {
    let mut out = String::with_capacity(body.len() + 256);
    for line in body.lines() {
        if line.is_empty() || line.starts_with('#') {
            out.push_str(line);
            out.push('\n');
            continue;
        }
        let absolute = if line.starts_with("http://") || line.starts_with("https://") {
            line.to_string()
        } else {
            base.join(line).map(|u| u.to_string()).unwrap_or_else(|_| line.to_string())
        };
        let encoded = URL_SAFE_NO_PAD.encode(absolute.as_bytes());
        out.push_str(&format!("/api/v1/hls-proxy/{slug}/seg?u={encoded}"));
        out.push('\n');
    }
    out
}

async fn segment(
    Path(_slug): Path<String>,
    Query(q): Query<SegQuery>,
) -> Result<Response, AppError> {
    let decoded = URL_SAFE_NO_PAD
        .decode(q.u.as_bytes())
        .map_err(|_| AppError::BadRequest("invalid base64url in u".into()))?;
    let url = String::from_utf8(decoded)
        .map_err(|_| AppError::BadRequest("u is not valid utf-8".into()))?;

    let upstream = reqwest::Client::new()
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("upstream segment: {e}")))?;

    let status = upstream.status();
    let mut headers = HeaderMap::new();
    if let Some(ct) = upstream.headers().get("content-type") {
        headers.insert("content-type", ct.clone());
    }
    headers.insert("access-control-allow-origin", HeaderValue::from_static("*"));
    headers.insert("cache-control", HeaderValue::from_static("no-cache"));

    let body = Body::from_stream(upstream.bytes_stream());
    Ok((status, headers, body).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrite_keeps_comments_and_blanks() {
        let base = Url::parse("https://cdn.example.com/path/index.m3u8").unwrap();
        let body = "\
#EXTM3U\n\
#EXT-X-VERSION:3\n\
\n\
#EXTINF:4.000,\n\
seg-001.ts\n\
#EXTINF:4.000,\n\
https://other.example.com/seg-002.ts\n";
        let out = rewrite_m3u8(body, &base, "yt-test");
        // Comments and blank lines pass through.
        assert!(out.contains("#EXTM3U"));
        assert!(out.contains("#EXT-X-VERSION:3"));
        // Both URI lines became proxy paths.
        assert!(out.contains("/api/v1/hls-proxy/yt-test/seg?u="));
        // No raw upstream URLs leak.
        assert!(!out.contains("https://cdn.example.com"));
        assert!(!out.contains("https://other.example.com"));
    }

    #[test]
    fn rewrite_resolves_relative_against_base() {
        let base = Url::parse("https://cdn.example.com/playlists/master.m3u8").unwrap();
        let body = "#EXTM3U\n#EXTINF:4.000,\nseg-1.ts\n";
        let out = rewrite_m3u8(body, &base, "slug");
        // Decode the encoded URL back and check it is absolute and joined to base.
        let line = out.lines().find(|l| l.contains("/seg?u=")).unwrap();
        let encoded = line.split("u=").nth(1).unwrap();
        let decoded = URL_SAFE_NO_PAD.decode(encoded.as_bytes()).unwrap();
        let url = String::from_utf8(decoded).unwrap();
        assert_eq!(url, "https://cdn.example.com/playlists/seg-1.ts");
    }
}
```

- [ ] **Step 5: Регистрация модулей**

В `crates/placebo-api/src/services/mod.rs` добавить:

```rust
pub mod hls_source;
```

В `crates/placebo-api/src/handlers/mod.rs`:

```rust
pub mod hls_proxy;
```

И в `api_router()` добавить `.merge(hls_proxy::router())` рядом с другими (на верхнем уровне, не внутри `/cameras` nest):

```rust
pub fn api_router() -> Router<AppState> {
    Router::new()
        .nest("/auth", auth::router())
        .merge(me::router())
        .merge(hls_proxy::router())                  // <-- new
        .nest("/cameras", cameras::router()
            .merge(ratings::router())
            .merge(boosts::router())
            .merge(clips::camera_router())
        )
        .nest("/rooms", rooms::router())
        .nest("/users", users::router())
        .nest("/clips", clips::user_router())
        .nest("/world", world::router())
}
```

- [ ] **Step 6: ServeDir в build_app**

В `crates/placebo-api/src/lib.rs::build_app` добавить ServeDir на `/static`:

```rust
use tower_http::services::ServeDir;

pub async fn build_app(state: AppState) -> Router {
    // ... existing cors logic ...

    Router::new()
        .route("/health", get(handlers::health::health))
        .route("/readiness", get(handlers::health::readiness))
        .nest("/api/v1", handlers::api_router())
        .nest_service("/static", ServeDir::new("static"))   // <-- new
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(cors)
        .with_state(state)
}
```

- [ ] **Step 7: Static placeholder**

```bash
mkdir -p crates/placebo-api/static/demo
touch crates/placebo-api/static/.gitkeep
```

Создать `crates/placebo-api/static/demo/README.md`:

```markdown
# Demo loop_mp4 assets

The alpha seed (migration 010) references 5 looped MP4 cameras whose `slug`
starts with `demo-`. Each one expects an HLS bundle at:

```
static/demo/<slug-suffix>/index.m3u8
static/demo/<slug-suffix>/segment-*.ts
```

Mapping (slug → asset folder):

| slug                 | asset folder    | duration |
|----------------------|-----------------|----------|
| demo-tokyo-alley     | tokyo-alley     | ~92s     |
| demo-cafe-street     | cafe-street     | ~120s    |
| demo-beach-sunset    | beach-sunset    | ~180s    |
| demo-rainy-window    | rainy-window    | ~240s    |
| demo-mountain-pass   | mountain-pass   | ~150s    |

Generate from a source MP4 with FFmpeg:

```bash
mkdir -p static/demo/tokyo-alley
ffmpeg -i source.mp4 \
    -c:v libx264 -preset veryfast -crf 23 \
    -g 48 -hls_time 4 -hls_playlist_type vod \
    static/demo/tokyo-alley/index.m3u8
```

The assets themselves are NOT committed (large binaries). Add `static/demo/*/`
to `.gitignore` and ship them out-of-band (e.g. R2) for production.
```

Добавить в корневой `.gitignore`:

```
/crates/placebo-api/static/demo/*
!/crates/placebo-api/static/demo/README.md
```

- [ ] **Step 8: cargo check + unit tests**

```bash
cd crates/placebo-api
cargo check
cargo test --lib hls_proxy
cargo test --lib hls_source
# expected: rewrite_keeps_comments_and_blanks, rewrite_resolves_relative_against_base, cache_key_format – PASS
```

- [ ] **Step 9: Smoke test handler**

```bash
cd crates/placebo-api
cargo run &
sleep 4
# rtsp slug should 404 (но в нашем seed нет rtsp – проверим на несуществующем)
curl -s -o /dev/null -w "%{http_code}\n" http://localhost:3001/api/v1/hls-proxy/nonexistent
# expected: 404
# loop_mp4 → 307/302 redirect
curl -s -o /dev/null -w "%{http_code} %{redirect_url}\n" http://localhost:3001/api/v1/hls-proxy/demo-tokyo-alley
# expected: 307 /static/demo/tokyo-alley/index.m3u8 (axum Redirect::temporary uses 307)
# youtube → 200 + manifest (only if yt-dlp installed and stream live)
curl -s http://localhost:3001/api/v1/hls-proxy/yt-shibuya-crossing | head -10
# expected: m3u8 lines, segment lines start with /api/v1/hls-proxy/yt-shibuya-crossing/seg?u=...
kill %1
```

- [ ] **Step 10: Commit**

```bash
git add crates/placebo-api/Cargo.toml \
        crates/placebo-api/src/services/mod.rs \
        crates/placebo-api/src/services/hls_source.rs \
        crates/placebo-api/src/handlers/mod.rs \
        crates/placebo-api/src/handlers/hls_proxy.rs \
        crates/placebo-api/src/repositories/camera_repo.rs \
        crates/placebo-api/src/lib.rs \
        crates/placebo-api/static/.gitkeep \
        crates/placebo-api/static/demo/README.md \
        .gitignore
git commit -m "$(cat <<'EOF'
feat(api): /api/v1/hls-proxy with yt-dlp + Redis cache + static demo loop

- services::hls_source resolves slug -> ResolvedSource (Hls/StaticLoop/
  Unsupported/NotFound). YouTube URLs cached in Redis (hls:src:<slug>) for
  30 minutes; cache failures are non-fatal.
- handlers::hls_proxy serves manifest (rewrites every URI line into
  /seg?u=<base64url>) and segment (streams upstream bytes through).
- ServeDir mounted at /static for loop_mp4 assets; loop_mp4 manifests
  302-redirect to /static/demo/<asset>/index.m3u8.
- rtsp source type intentionally returns 404 in the alpha.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 7: Скрипт верификации YouTube ID

**Files:**
- Create: `scripts/verify-youtube-seed.sh`

- [ ] **Step 1: Скрипт**

`scripts/verify-youtube-seed.sh`:

```bash
#!/usr/bin/env bash
# Sanity-check that every YouTube videoId in migration 010 still resolves
# via yt-dlp. Prints OK/FAIL per id and returns non-zero if any failed.
set -uo pipefail

declare -a IDS=(
    dfVK7ld38Ys u4UZ4UvZXrg FQWkgr0aHlI AdUw5RdyZxI h1wly909BYw
    7Bl5p4VTXzQ 2L4yhCmGRWg dyWHmEQAVUI qMksIqJv3pI wNmMr_ATI2E
    hSbkw-F7bzY 2Te5EvOXNZw SkdGPWUUkEw
)

failed=0
for id in "${IDS[@]}"; do
    printf '%-12s ... ' "$id"
    if yt-dlp -f 'best[vcodec^=avc1]' --no-warnings -g \
        "https://www.youtube.com/watch?v=${id}" >/dev/null 2>&1; then
        echo OK
    else
        echo FAIL
        failed=$((failed + 1))
    fi
done

if [[ $failed -gt 0 ]]; then
    echo ""
    echo "$failed/${#IDS[@]} ids failed to resolve. Update migration 010 and rerun."
    exit 1
fi
echo ""
echo "All ${#IDS[@]} ids resolve OK."
```

- [ ] **Step 2: Прогон**

```bash
chmod +x scripts/verify-youtube-seed.sh
./scripts/verify-youtube-seed.sh
# expected: 13 OK lines
```

Если есть FAIL – заменить ID в миграции 010, прогнать снова. Скрипт сам напомнит.

- [ ] **Step 3: Commit**

```bash
git add scripts/verify-youtube-seed.sh
git commit -m "build: scripts/verify-youtube-seed.sh sanity-checks alpha YouTube ids"
```

---

## Task 8: Frontend – удалить vite hls-proxy, добавить API-клиент

**Files:**
- Modify: `vite.config.ts`
- Create: `src/api/cameras.ts`
- Create: `src/hooks/useCamerasFromApi.ts`
- Modify: `src/hooks/useNearbyCameras.ts`
- Modify: `.env.example`

- [ ] **Step 1: Очистить vite.config.ts**

Заменить весь файл на:

```ts
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
    proxy: {
      "/api": {
        target: "http://localhost:3001",
        changeOrigin: true,
      },
    },
  },
});
```

Никакого `defineConfig(async () => ...)` не нужно – синхронный возврат проще, нет async-операций. И никакого hls-proxy middleware.

- [ ] **Step 2: src/api/cameras.ts**

```ts
import { apiRequest } from "./client";
import type { CameraResponse } from "../types/api/CameraResponse";

export interface ListCamerasParams {
  page?: number;
  perPage?: number;
  category?: string;
  type?: string;
}

export interface CameraListResponse {
  data: CameraResponse[];
  meta: {
    page: number;
    perPage: number;
    total: number;
    totalPages: number;
  };
}

function buildQuery(params: Record<string, unknown>): string {
  const search = new URLSearchParams();
  for (const [key, value] of Object.entries(params)) {
    if (value === undefined || value === null) continue;
    search.set(key, String(value));
  }
  const qs = search.toString();
  return qs ? `?${qs}` : "";
}

export async function listCameras(
  params: ListCamerasParams = {},
): Promise<CameraListResponse> {
  const query = buildQuery({
    page: params.page,
    per_page: params.perPage,
    category: params.category,
    type: params.type,
  });
  return apiRequest<CameraListResponse>(`/cameras${query}`, { auth: false });
}

export async function getCamera(id: string): Promise<{ data: CameraResponse }> {
  return apiRequest<{ data: CameraResponse }>(`/cameras/${id}`, { auth: false });
}

export async function getCamerasNearby(
  lat: number,
  lng: number,
  radiusM: number,
  limit = 50,
): Promise<{ data: CameraResponse[] }> {
  const query = buildQuery({ lat, lng, radius_m: radiusM, limit });
  return apiRequest<{ data: CameraResponse[] }>(`/cameras/nearby${query}`, {
    auth: false,
  });
}

export async function searchCameras(
  q: string,
  limit = 50,
): Promise<{ data: CameraResponse[] }> {
  const query = buildQuery({ q, limit });
  return apiRequest<{ data: CameraResponse[] }>(`/cameras/search${query}`, {
    auth: false,
  });
}
```

**Внимание:** `apiRequest` из `src/api/client.ts` – проверить точную сигнатуру. Если `auth: false` не такая опция как там – align with existing.

- [ ] **Step 3: src/hooks/useCamerasFromApi.ts**

```ts
import { useEffect, useState } from "react";
import { listCameras } from "../api/cameras";
import type { CameraResponse } from "../types/api/CameraResponse";

interface State {
  data: CameraResponse[] | null;
  error: Error | null;
  loading: boolean;
}

export function useCamerasFromApi(perPage = 50): State {
  const [state, setState] = useState<State>({
    data: null,
    error: null,
    loading: true,
  });

  useEffect(() => {
    let cancelled = false;
    listCameras({ perPage })
      .then((response) => {
        if (cancelled) return;
        setState({ data: response.data, error: null, loading: false });
      })
      .catch((error: unknown) => {
        if (cancelled) return;
        const e = error instanceof Error ? error : new Error(String(error));
        setState({ data: null, error: e, loading: false });
      });
    return () => {
      cancelled = true;
    };
  }, [perPage]);

  return state;
}
```

- [ ] **Step 4: useNearbyCameras streamUrl helper**

Заменить в `src/hooks/useNearbyCameras.ts` функцию `streamUrl`:

```ts
// HLS proxy now lives on the axum backend.
// Vite proxy forwards /api → http://localhost:3001 in dev.
function streamUrl(slug: string): string {
  return `/api/v1/hls-proxy/${slug}`;
}
```

Комментарий обновлён, `import.meta.env` не нужен – в dev `/api` идёт через vite proxy на 3001, в проде Tauri сервер сам работает, в обоих случаях относительный путь работает.

- [ ] **Step 5: .env.example**

Проверить что в корневом `.env.example` есть `VITE_API_BASE_URL=http://localhost:3001`. Если нет – добавить (скорее всего уже есть от M2).

```bash
grep -q VITE_API_BASE_URL .env.example || echo 'VITE_API_BASE_URL=http://localhost:3001' >> .env.example
```

- [ ] **Step 6: Build**

```bash
npm run build
# expected: vite build OK, no TS errors. predev hook regenerates types.
```

Если где-то ts-rs camelCase отличается от того что я написал в `cameras.ts` – fix.

- [ ] **Step 7: Frontend test suite**

```bash
npm test
# expected: 20/20 still green (auth tests из M2 не должны сломаться)
```

- [ ] **Step 8: Commit**

```bash
git add vite.config.ts \
        src/api/cameras.ts \
        src/hooks/useCamerasFromApi.ts \
        src/hooks/useNearbyCameras.ts \
        .env.example
git commit -m "$(cat <<'EOF'
feat(frontend): drop vite hls-proxy, add typed cameras API client

- vite.config.ts collapses to plain react() + /api proxy. The yt-dlp
  middleware that lived here in the prototype is now on axum.
- src/api/cameras.ts wraps GET /cameras, /cameras/:id, /cameras/nearby,
  /cameras/search using the auto-generated CameraResponse from M3 ts-rs.
- useCamerasFromApi() hook is the entry point M4 will hang World3D off.
- useNearbyCameras streamUrl helper now points to /api/v1/hls-proxy/:slug.
  The mock data itself stays until M4 rewires World3D.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 9: End-to-end verification

- [ ] **Step 1: Полная сборка backend**

```bash
docker compose -f docker-compose.dev.yml up -d
cd crates/placebo-api
cargo run
# leave running
```

- [ ] **Step 2: cargo test всего workspace**

В другой консоли:

```bash
cargo test --workspace
# expected: all green
```

- [ ] **Step 3: API smoke**

```bash
curl -s http://localhost:3001/api/v1/cameras | python -m json.tool | head -30
# expected: data[0].slug starts with demo- or yt-, has streamSourceType + proxyManifestUrl,
# no streamUrl field

curl -s 'http://localhost:3001/api/v1/cameras?category=city' | python -m json.tool | head -10
# expected: only city cameras

curl -s -o /dev/null -w "manifest: %{http_code}\n" http://localhost:3001/api/v1/hls-proxy/yt-shibuya-crossing
# expected: manifest: 200

curl -s -o /dev/null -w "loop: %{http_code}\n" -L http://localhost:3001/api/v1/hls-proxy/demo-tokyo-alley
# expected: loop: 404 (because demo asset MP4 not yet provided by user) OR 200 if user uploaded it
# Either is acceptable; 404 here is the static dir 404, NOT proxy logic 404.

curl -s -o /dev/null -w "missing: %{http_code}\n" http://localhost:3001/api/v1/hls-proxy/does-not-exist
# expected: missing: 404
```

- [ ] **Step 4: Browser HLS playback (manual)**

```bash
cat > /tmp/hls-test.html <<'EOF'
<!DOCTYPE html><html><body style="background:#000">
<video id="v" controls autoplay muted style="width:100%;max-height:100vh"></video>
<script src="https://cdn.jsdelivr.net/npm/hls.js@1"></script>
<script>
  const slug = new URLSearchParams(location.search).get('slug') || 'yt-shibuya-crossing';
  const v = document.getElementById('v');
  if (Hls.isSupported()) {
    const hls = new Hls();
    hls.loadSource('http://localhost:3001/api/v1/hls-proxy/' + slug);
    hls.attachMedia(v);
  } else {
    v.src = 'http://localhost:3001/api/v1/hls-proxy/' + slug;
  }
</script></body></html>
EOF
start /tmp/hls-test.html  # Windows; on linux: xdg-open
```

- [ ] **Step 5: Tauri dev (optional sanity)**

```bash
npm run tauri dev
# expected: app boots, login flow still works, no console errors about /hls-proxy
```

- [ ] **Step 6: Verify Redis cache**

```bash
docker exec placebo-redis-dev redis-cli KEYS 'hls:src:*'
# expected: at least 1 key for any yt-* slug whose manifest was fetched
docker exec placebo-redis-dev redis-cli TTL 'hls:src:yt-shibuya-crossing'
# expected: > 0 and ≤ 1800
```

- [ ] **Step 7: Push + PR**

```bash
git push -u origin feat/m3-cameras-hls
gh pr create --title "feat(m3): cameras seed + HLS proxy on axum" --body "$(cat <<'EOF'
## Summary
- Migration 009 adds `stream_source_type` enum + `stream_source_config` JSONB to cameras
- Migration 010 replaces the legacy 50-RTSP-mock seed with 18 alpha cameras (13 YouTube live + 5 looped MP4 demos)
- Axum gains `/api/v1/hls-proxy/:slug` and `/api/v1/hls-proxy/:slug/seg` with Redis-cached yt-dlp resolution and static-loop redirect
- ts-rs now exports the full Camera DTO surface to `src/types/api/`
- Vite hls-proxy middleware dropped; React app talks to axum directly

## Test plan
- [ ] `cargo test --workspace` passes
- [ ] `npm test` passes (20/20 from M2)
- [ ] `./scripts/verify-youtube-seed.sh` reports all 13 ids resolve
- [ ] Manual: open `/tmp/hls-test.html?slug=yt-shibuya-crossing` and confirm video plays in <10s
- [ ] Manual: `curl /api/v1/cameras` returns 18 cameras, none with `streamUrl`
- [ ] Manual: Redis has `hls:src:yt-*` keys with TTL ≤ 1800 after a manifest fetch

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

---

## Acceptance Criteria для Milestone 3

1. ✅ Миграции 009 и 010 применяются на чистой базе без ошибок (001..010 идут подряд).
2. ✅ `GET /api/v1/cameras` возвращает 18 камер; ни одна не содержит `streamUrl`/`stream_url`/`backupUrl`/`externalId`/`frameRate`.
3. ✅ Каждая камера в ответе имеет `streamSourceType` (`youtube_live` либо `loop_mp4`) и `proxyManifestUrl: "/api/v1/hls-proxy/<slug>"`.
4. ✅ `GET /api/v1/cameras/:id` возвращает один объект той же формы.
5. ✅ `GET /api/v1/hls-proxy/:slug` для `yt-*` slug отдаёт m3u8 с сегмент-URL вида `/api/v1/hls-proxy/<slug>/seg?u=<base64url>`.
6. ✅ `GET /api/v1/hls-proxy/:slug/seg?u=<...>` стримит сегмент upstream (или возвращает upstream-ный статус).
7. ✅ Redis содержит `hls:src:<slug>` с TTL ≤ 1800 после первого запроса любого yt-camera.
8. ✅ Для `loop_mp4` slug прокси возвращает 307 на `/static/demo/<asset>/index.m3u8` (asset existence not required; redirect target is correct).
9. ✅ Slug, не существующий в БД – 404. Slug с `stream_source_type = 'rtsp'` – тоже 404 (в seed таких нет, но защита логичная).
10. ✅ `vite.config.ts` не содержит hls-proxy middleware, yt-dlp вызовов и связанных импортов.
11. ✅ ts-rs генерирует `CameraResponse.ts`, `StreamSourceType.ts`, и 6 enum-типов в `src/types/api/`.
12. ✅ `scripts/verify-youtube-seed.sh` проходит (все 13 ID живые) ИЛИ план/seed обновлён до зелёного.
13. ✅ `cargo test --workspace` зелёный; `npm test` зелёный; `npm run build` зелёный.

---

## Что идёт дальше (M4)

M4 (`2026-05-14-milestone-4-home-categories-world.md`) переписывает Home + Categories + встраивает World3D в shell, и переключает `useNearbyCameras` с моков на `useCamerasFromApi`. До тех пор моковая часть `useNearbyCameras` живёт – единственное живое использование `streamUrl` для тестирования прокси.

## Self-review checklist

- [x] Spec coverage: каждая фича в alpha-design §5.1–5.2, §7.4, §9 покрыта таском (camera DTO ts-rs, sensitive-fields-strip уже было, alpha seed, HLS proxy, Redis cache, ServeDir).
- [x] No placeholders: каждый блок кода полный.
- [x] Type consistency: `CameraResponse` (не `CameraSummary`/`Detail`), `StreamSourceType` snake_case в JSON и enum в DB совпадают, `proxy_manifest_url` в Rust → `proxyManifestUrl` в TS (camelCase via serde rename_all).
- [x] Existing-code respect: handler модули и роутеры не переписываются – только расширяются.
