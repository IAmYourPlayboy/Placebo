# Placebo – Runbook

> **ЗАПРЕЩЕНО редактировать этот файл без явного разрешения пользователя (IAmYourPlayboy).**
> Нейронка: НЕ изменяй, НЕ перезаписывай, НЕ удаляй содержимое этого файла.
> Только пользователь может попросить внести правки. Без его слов – не трогать.

Пошаговые инструкции для типичных сценариев. Обновляется по мере развития проекта.

---

## 0. Требования перед запуском

Пользователь должен **вручную** запустить:

1. **Docker Desktop** – открыть приложение, дождаться зелёного индикатора "Docker is running"
2. **PostgreSQL 17** – если не через Docker, проверить: `pg_isready`
3. **Redis** – если не через Docker, проверить: `redis-cli ping`

Без Docker Desktop контейнеры (если используются) не поднимутся.

---

## 1. Запуск 3D мира с видеотрансляциями

**Что нужно запущено:**
- PostgreSQL 17 + PostGIS (с OSM данными нужного региона)
- Redis
- Axum API (порт 3001) – HLS-прокси теперь живёт здесь
- Vite dev server (порт 1420) либо Tauri dev
- `yt-dlp` в `PATH` (нужен axum-прокси для разрешения youtube_live)

**Что НЕ нужно:**
- go2rtc – никогда не использовался в M3+.
- vite hls-proxy middleware – удалён в M3, всё проксируется через axum.

### Шаги

```bash
# 1. Поднять postgres+redis (если ещё не подняты)
docker compose -f docker-compose.dev.yml up -d

# 2. Запустить axum API
cd crates/placebo-api && cargo run
# Ждём "listening on 0.0.0.0:3001"
#   Миграции 001..010 накатятся автоматически.

# 3. В отдельном окне – Vite dev (или `npm run tauri dev`)
npm run dev
# Открыть http://localhost:1420

# 4. Login → Категории → "Онлайн карта мира".
#   3D мир грузится, маркеры камер из /api/v1/cameras (18 шт).
#   На клик маркера появляется CameraDetailPanel; HLS-плоскость
#   подтягивает поток через GET /api/v1/hls-proxy/<slug>.
```

### Как работает видео (M3+)

```
Браузер (hls.js) → GET /api/v1/hls-proxy/<slug>          (axum handler)
                    ↓
  resolve <slug> → stream_source_type из БД
   ├── youtube_live → yt-dlp -g <videoId> (Redis cache 30 мин)
   ├── direct_hls   → URL из stream_source_config.url
   └── loop_mp4     → 307 redirect на /static/demo/<asset>/index.m3u8
                    ↓
  m3u8 переписывается: каждая ссылка на сегмент =
     /api/v1/hls-proxy/<slug>/seg?u=<base64url(absolute-segment-url)>
                    ↓
  Браузер запрашивает /seg → axum стримит сегмент upstream
```

### Ключевые файлы

| Файл | Роль |
|------|------|
| `crates/placebo-api/src/handlers/hls_proxy.rs` | manifest() + segment() handlers |
| `crates/placebo-api/src/services/hls_source.rs` | resolve(slug) + Redis-кеш yt-dlp |
| `crates/placebo-api/migrations/010_seed_alpha_cameras.sql` | Альфа-сид 13 youtube + 5 demo |
| `src/hooks/useCamerasFromApi.ts` | Загрузка камер из /api/v1/cameras |
| `src/api/camera3d.ts` | Адаптер `CameraResponse → Camera3D` |
| `src/screens/world/World3DScreen.tsx` | Активная камера + URL `/world/:slug` |
| `src/components/world3d/CameraFrustum.tsx` | VideoPlane + hls.js (без изменений с M2) |
| `src/hooks/useCityTiles.ts` | OSM-тайлы дорог/зданий/воды/парков |

### Частые проблемы

| Симптом | Причина | Решение |
|---------|---------|---------|
| `Не удалось загрузить камеры: Request failed (400)` | Axum валидирует `?per_page=...` строго; до M4 фикса был баг | Подтянуть main – фикс в `extractors/pagination.rs` (M4 PR #6) |
| `migration N was previously applied but has been modified` | CRLF/LF расхождение в `.sql` файле (Windows autocrlf) | `.gitattributes` уже фиксирует `*.sql text eol=lf`. Если уже зацепило – `docker compose down -v` + перезапуск (dev volume) |
| HLS 500 на `yt-shibuya-crossing` (RU IP) | YouTube anti-bot блокирует анонимный yt-dlp | См. CLAUDE.md §16. Решение – EU VPS (вне scope альфы) |
| Серая плоскость на `demo-*` slug | Нет MP4 в `crates/placebo-api/static/demo/<asset>/` | Сгенерить FFmpeg по инструкции в `crates/placebo-api/static/demo/README.md` |
| Тайлы 502 / ERR_ABORTED | Axum API не запущен | `cd crates/placebo-api && cargo run` |

### Требования к системе

- `yt-dlp` установлен и в `PATH` (через `pip install yt-dlp` или другим способом)
- `hls.js` уже в `package.json`
- Порты 1420 (Vite), 3001 (axum), 5432 (postgres), 6380 (redis) свободны

---

## 2. Добавление новой камеры в seed

С M3 камеры живут в SQL-миграциях, не в JSON, и URL никогда не торчит в API.

```sql
-- Новая миграция, например 011_seed_more_cameras.sql
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
) VALUES (
    'My Camera', 'my-camera-slug', 'public',
    'Country', 'CC', 'City',
    ST_SetSRID(ST_Point(lng, lat), 4326), 'IANA/Tz',
    'youtube://VIDEO_ID', 'youtube',
    'youtube_live', '{"videoId":"VIDEO_ID"}'::jsonb,
    'city', 'Description',
    1920, 1080, 'h264',
    8, 180, -15, 80, 50,
    NOW(), NOW()
);
```

После этого:

1. `cargo sqlx migrate run` (axum накатит при старте сам).
2. Перезапустить axum.
3. Опционально: добавить YouTube ID в `scripts/verify-youtube-seed.sh` и прогнать его.

В pgsql БД ставит фронт. Никаких изменений в `vite.config.ts` или mock-хуках больше не требуется – `useCamerasFromApi` подтянет новую камеру автоматически.

Для **демо-камеры** (`stream_source_type = 'loop_mp4'`) добавить MP4 + HLS-сегменты в `crates/placebo-api/static/demo/<asset>/`. Подробности – `crates/placebo-api/static/demo/README.md`.

Для **direct HLS** (`stream_source_type = 'direct_hls'`) – `stream_source_config = '{"url":"https://..."}'`.

Для **rtsp** – пока не поддерживается прокси (вернёт 404 на /hls-proxy/<slug>); ингест RTSP через FFmpeg = отдельная задача в M5+.

---

## 3. Сборка и запуск Tauri (desktop app)

```bash
# Пока не настроено – frontend only через Vite
# TODO: npm run tauri dev
```

---

## 4. Работа с базой данных

```bash
# PostgreSQL с PostGIS (OSM данные региона Канто)
psql -d placebo_dev

# Проверить что OSM данные есть:
SELECT count(*) FROM planet_osm_line;   -- дороги
SELECT count(*) FROM planet_osm_polygon; -- здания, парки, вода

# Redis (сессии, viewer counts)
redis-cli
> KEYS *

# Миграции (placebo-api)
cd placebo-api
sqlx migrate run
```

---

## 5. Чистый перезапуск (если всё сломалось)

```bash
# 1. Убить всё
pkill -f "cargo run" 2>/dev/null
kill $(lsof -ti :1420) 2>/dev/null
docker stop placebo-go2rtc 2>/dev/null

# 2. Проверить .env – не должно быть VITE_GO2RTC_URL
cat .env

# 3. Очистить Vite кеш
rm -rf node_modules/.vite

# 4. Запустить заново (см. пункт 1)
```

---

## 6. GitHub CLI (gh) – особенности доступа

`gh auth login` сам по себе **бесполезен** для PR в Placebo: токен,
который он создаёт по дефолту, не имеет прав на этот репозиторий и
padает с `Could not resolve to a Repository`.

Для `gh pr create` / `gh pr view` нужен PAT, у которого явно выставлено:

- **Resource owner**: `IAmYourPlayboy`
- **Repository access**: `Only select repositories` → `Placebo`
- **Repository permissions**: `Contents: read & write`, `Pull requests: read & write`, `Metadata: read-only`

Или классический token с scope `repo`. Просто `gh auth login` с дефолтным
fine-grained PAT даст ошибку доступа.

Если `gh` не работает – PR-описание можно писать в локальный `.md`-файл
и открывать https://github.com/IAmYourPlayboy/Placebo/pull/new/<branch>
в браузере, копируя body вручную.
