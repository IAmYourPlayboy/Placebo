# Placebo — Architecture Decision Record (ADR)
> Версия 1.0 · Март 2026 · Статус: ACTIVE

---

## 0. Главный принцип

> **Никакой зависимости от чужих API которые могут закрыться или поднять цену.**
> Всё своё: сервера, медиасервер, хранилище, БД.

---

## 1. Общая картина системы

```
┌─────────────────────────────────────────────────────────────────────┐
│  Пользователь                                                        │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  Tauri Desktop App  (React 18 + TypeScript + Rust backend)  │   │
│  │                                                              │   │
│  │  Local SQLite  ←── только личные данные пользователя        │   │
│  │  (bookmarks, watch history, preferences, offline cache)      │   │
│  └───────────────────────────┬──────────────────────────────── ┘   │
└──────────────────────────────│──────────────────────────────────────┘
                               │ HTTPS / WebSocket
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│  Placebo API Server  (axum · Rust · Hetzner VPS)                    │
│                                                                      │
│  REST + WebSocket API                                                │
│  OpenAPI spec  →  auto-generates TypeScript client types            │
│                                                                      │
│  ┌──────────────┐  ┌─────────────┐  ┌──────────────────────────┐   │
│  │  PostgreSQL   │  │    Redis    │  │   Object Storage         │   │
│  │  + PostGIS    │  │  (real-time)│  │   (recordings, thumbs)   │   │
│  │               │  │             │  │                          │   │
│  │  cameras      │  │ viewers_now │  │  /recordings/{cam}/{day} │   │
│  │  users        │  │ online set  │  │  /thumbnails/{cam}.jpg   │   │
│  │  rooms        │  │ sessions    │  │  /clips/{clip_id}.mp4    │   │
│  │  recordings   │  │ rate limits │  │                          │   │
│  │  communities  │  └─────────────┘  └──────────────────────────┘   │
│  └──────────────┘                                                    │
└─────────────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│  Media Ingest Layer  (FFmpeg daemons)                                │
│                                                                      │
│  Camera RTSP ──► FFmpeg ──► HLS segments ──► Hot Storage (SSD)     │
│                                          └──► Ring buffer 24h       │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 2. Уровни данных — что где живёт

### 2.1 Локальный SQLite (в Tauri-приложении)

Хранит **только то что принадлежит конкретному пользователю** и не нужно синхронизировать между устройствами в реальном времени:

```
user_preferences     — тема, язык, уведомления
bookmarked_cameras   — сохранённые камеры
watch_history        — последние просмотренные (офлайн кеш)
downloaded_clips     — скачанные клипы
app_cache            — thumbnail cache, последний запрос к API
auth_token           — JWT, хранится зашифровано
```

**Почему не весь каталог камер локально?**
При 1M камерах это ~500MB только на метаданные. Обновлять их у всех пользователей — проблема. API запрос за 50ms → намного лучше.

### 2.2 PostgreSQL + PostGIS (на сервере)

Всё общее — видно всем пользователям:

```
cameras              — каталог камер (основа всего)
users                — аккаунты
rooms                — совместные просмотры
recording_segments   — метаданные записей (не сами файлы)
clip_requests        — запросы на нарезку клипов
communities          — клубы, сообщества
```

**Почему PostgreSQL а не MongoDB/другое?**
- PostGIS: геопространственные запросы — "найди камеры в радиусе 50км" в одну SQL строку
- ACID: важно для биллинга (подписки, клипы)
- sqlx: compile-time проверка запросов, уже используем

### 2.3 Redis (на сервере)

Данные которые меняются каждую секунду:

```
camera:viewers:{id}    → Integer      — viewers_now
cameras:online         → Set<uuid>    — какие камеры сейчас онлайн
user:session:{token}   → JSON         — сессия пользователя
rate_limit:{ip}        → Integer      — защита от DDoS
recording:active       → Set<uuid>    — какие камеры сейчас пишутся
```

Почему не в PostgreSQL? UPDATE viewers_now для 10K камер каждые 5 секунд убьёт БД. Redis atomicIncrBy = ~100K операций/сек.

### 2.4 Object Storage (на сервере или Hetzner Storagebox)

Бинарные файлы — видео, картинки:

```
/recordings/{camera_id}/{YYYY-MM-DD}/{HH}/{seg_000001.ts}
/thumbnails/{camera_id}.jpg
/clips/{clip_id}.mp4
/avatars/{user_id}.jpg
```

---

## 3. "Связующий инструмент" — API слой

### Выбор: axum (Rust) + utoipa (OpenAPI) + openapi-typescript

Цепочка типобезопасности от БД до UI:

```
PostgreSQL schema
    ↓
sqlx Rust types  (compile-time: запрос не совпадает со схемой — ошибка сборки)
    ↓
axum handler returns typed structs
    ↓
utoipa генерирует openapi.json автоматически из Rust аннотаций
    ↓
openapi-typescript генерирует TypeScript типы из openapi.json
    ↓
React frontend имеет полную типобезопасность без ручного написания типов
```

**Почему не Supabase/PocketBase/Hasura?**

| Инструмент | Проблема для Placebo |
|---|---|
| Supabase | Vendor lock-in, $25+/мес, нельзя мигрировать дёшево |
| PocketBase | SQLite внутри — не масштабируется для 1M камер |
| Hasura | Ещё один сервис, GraphQL overhead для простых запросов |
| **axum + sqlx** | ✅ Наш Rust, полный контроль, 0 зависимостей |

**Почему не tRPC?**
tRPC — TypeScript-only. Наш бэкенд на Rust. Не работает.

**Почему не gRPC/Protobuf?**
Слишком сложно для текущей стадии, Protobuf в браузере требует дополнительной инфраструктуры.

---

## 4. Записи с камер — реальная архитектура

### ⚠️ Важная поправка: .mp3 vs .mp4

**.mp3 — это аудиоформат без видео.** Для записей и клипов с видеокамер нужен **.mp4** (контейнер H.264/H.265 видео + AAC аудио). Если камера без микрофона — тогда silent .mp4.

### 4.1 Почему "записывать все камеры постоянно" невозможно при 1M+

```
Расчёт на 1M камер:
  1 камера @ 360p H.264 ≈ 100 kbps = ~45 MB/час
  1M камер × 45 MB × 24ч = 1.08 Петабайт В ДЕНЬ

  Hetzner Storagebox 20TB = €22/мес
  Нужно: 1080 TB / 20 = 54 хранилища × €22 = €1188/мес только за 1 день хранения
  
  Вывод: физически и финансово невозможно записывать всё постоянно.
```

### 4.2 Умная стратегия записей (Smart Recording)

```
Tier 0 — "Витрина" (всегда пишутся):
  - Топ-1000 камер по среднесуточному трафику
  - Все партнёрские камеры
  - "Избранные" камеры редакции Placebo
  Хранение: 7 дней warm + 30 дней cold

Tier 1 — "Demand" (пишутся когда смотрят):
  - Запись стартует когда открывает первый зритель
  - Запись идёт + 24 часа после последнего зрителя
  - Хранение: 48 часов
  
Tier 2 — "Pro пользователи":
  - Pro-подписчик нажимает "Записывать эту камеру"
  - Запись идёт 30 дней для этого пользователя
  - Хранится в его персональном хранилище
  
Tier 3 — "Остальные":
  - Никакой записи по умолчанию
  - Только live просмотр
```

### 4.3 Хранилище по уровням (Tiered Storage)

```
HOT tier (0–24ч):
  Формат: HLS + .ts сегменты, 10 сек каждый
  Кодек: H.264 (быстрое кодирование, NVIDIA NVENC 60+ FPS)
  Хранение: SSD/NVMe на медиасервере
  Размер: 100 MB/час @ 720p H.264

  Пример 1000 камер: 1000 × 100 MB × 24h = ~2.4 TB
  Цена: 2x Hetzner CX32 (NVMe SSD) = €20/мес ✅

WARM tier (1–7 дней):
  Пересобирается из hot сегментов ночью
  Кодек: H.265/HEVC (NVIDIA NVENC, Intel QSV)
  Экономия: 40–50% vs H.264
  Размер: ~50 MB/час @ 720p H.265

  Пример топ-500 камер, 7 дней: 500 × 50 MB × 168h = ~4.2 TB
  Цена: Hetzner Storagebox 10TB = €11.41/мес ✅

COLD tier (8–30 дней):
  Пересобирается из warm
  Кодек: H.265 с пониженным битрейтом (CRF 28)
  Размер: ~20 MB/час
  
  Пример топ-100 камер, 30 дней: 100 × 20 MB × 720h = ~1.4 TB
  Цена: Hetzner Storagebox 2TB = €3.45/мес ✅

ARCHIVE (30+ дней):
  Только "highlights" — моменты с высоким трафиком (>100 зрителей)
  Кодек: AV1 (медленное batch-кодирование ночью, -60% vs H.264)
  Хранение: объектное хранилище
```

### 4.4 Codec Decision Matrix

| Кодек | Скорость encode | Сжатие vs H.264 | Поддержка | Когда использовать |
|---|---|---|---|---|
| **H.264** | ⚡⚡⚡⚡ очень быстро | baseline | 100% устройств | Hot tier, live стриминг |
| **H.265/HEVC** | ⚡⚡⚡ быстро (NVENC) | -40–50% | 95% устройств | Warm tier, клипы |
| **AV1** | ⚡ медленно | -55–65% | 70% устройств | Archive, batch overnight |
| **VP9** | ⚡⚡ средне | -40% | 85% устройств | Альтернатива H.265 |

**Итоговый выбор:**
- Live stream → H.264 HLS
- Записи hot → H.264 HLS segments  
- Recordings warm/cold → H.265
- Archive → AV1 (batch encoding ночью)
- Клипы для скачивания → H.265 MP4

### 4.5 Нарезка клипов (Clip Pipeline)

```
Пользователь выбирает: камера X, с 14:30 до 14:37

API запрос:
  POST /api/cameras/{id}/clips
  { "start": "2025-01-15T14:30:00Z", "end": "2025-01-15T14:37:00Z" }

Сервер:
  1. Проверяет: записи за этот период существуют?
  2. Находит нужные .ts сегменты в recording_segments таблице
  3. Ставит задачу в очередь (Redis queue)
  
FFmpeg worker:
  ffmpeg -i "concat:seg1.ts|seg2.ts|..." \
         -ss 00:00:15 -to 00:07:00 \
         -c:v libx265 -crf 22 \
         -c:a aac -b:a 128k \
         -movflags +faststart \
         /clips/{clip_id}.mp4

API отдаёт:
  { "clip_id": "uuid", "status": "processing", "eta_seconds": 30 }

Когда готово:
  { "clip_id": "uuid", "status": "ready",
    "download_url": "https://cdn.placebo.app/clips/{id}.mp4",
    "expires_at": "2025-01-22T14:37:00Z" }
```

**Срок жизни клипов:** 7 дней для Free, 30 дней для Pro, бессрочно для Pro Yearly.

---

## 5. Форматы хранения seed-данных камер

### Вариант А: Rust-структуры (текущий)

```rust
// Нынешнее состояние
const SEED_CAMERAS: &[SeedCamera] = &[
    SeedCamera { name: "Shibuya", lat: 35.6595, ... },
];
```

**Плюсы:** Компилируется, типобезопасно  
**Минусы:** Нельзя редактировать без перекомпиляции, нет тулинга для geo-данных

### Вариант Б: JSON (рекомендуется)

```json
// data/cameras.json
{ "cameras": [ { "slug": "shibuya-crossing-tokyo", "lat": 35.6595, ... } ] }
```

**Плюсы:** Читаем в любом редакторе, легко импортировать в PostgreSQL, VSCode валидирует через JSON Schema  
**Минусы:** Нет compile-time проверки

### Вариант В: TOML

```toml
# data/cameras.toml
[[cameras]]
slug = "shibuya-crossing-tokyo"
lat = 35.6595
```

**Плюсы:** Human-friendly, Rust-native  
**Минусы:** Плохо для массивов, нет экосистемы тулинга

### Вариант Г: CSV

```
slug,name,lat,lng,country_code,...
shibuya-crossing-tokyo,Shibuya Crossing,35.6595,139.7005,JP,...
```

**Плюсы:** Открывается в Excel, легко массово редактировать  
**Минусы:** Нельзя хранить массивы (tags), нет вложенности

### Вариант Д: SQL seed файл

```sql
INSERT INTO cameras (slug, name, lat, ...) VALUES
('shibuya-crossing-tokyo', 'Shibuya Crossing', 35.6595, ...);
```

**Плюсы:** Прямо в БД  
**Минусы:** Дублирование схемы, сложно редактировать

### ✅ Рекомендую: JSON + JSON Schema валидация

JSON читается везде, импортируется в PostgreSQL через `COPY` или `jsonb_populate_recordset`, легко редактировать в VSCode с автодополнением через JSON Schema.

---

## 6. Все возможные поля камеры

Ниже **ПОЛНЫЙ** список. Ты выбираешь методом исключения что оставить.

```json
{
  "_СЕКЦИЯ_ИДЕНТИФИКАЦИЯ": "─────────────────────────────",
  "id": "uuid-v4",
  "name": "Shibuya Crossing",
  "slug": "shibuya-crossing-tokyo",
  "display_name_ru": "Перекрёсток Сибуя",
  "display_name_en": "Shibuya Crossing",
  "external_id": "earthcam-shibuya-001",

  "_СЕКЦИЯ_ЛОКАЦИЯ": "─────────────────────────────",
  "country": "Japan",
  "country_code": "JP",
  "region": "Kanto",
  "prefecture_state": "Tokyo",
  "city": "Tokyo",
  "district": "Shibuya",
  "address": "2-2-1 Dogenzaka, Shibuya City",
  "lat": 35.6595,
  "lng": 139.7005,
  "altitude_m": 43,
  "timezone": "Asia/Tokyo",
  "locale": "ja-JP",
  "what3words": "///filled.count.soap",

  "_СЕКЦИЯ_СТРИМ": "─────────────────────────────",
  "stream_url": "rtsp://192.168.1.1:554/stream",
  "stream_url_hd": "rtsp://192.168.1.1:554/stream_hd",
  "backup_url": "https://youtube.com/embed/...",
  "stream_type": "rtsp",
  "stream_protocol": "tcp",
  "stream_quality_default": "720p",
  "available_qualities": ["360p", "720p", "1080p"],
  "frame_rate": 30,
  "bitrate_kbps": 2000,
  "codec": "h264",
  "resolution_w": 1280,
  "resolution_h": 720,
  "has_audio": false,
  "has_ptz": false,
  "is_360_degree": false,
  "has_night_vision": true,
  "is_underwater": false,
  "fov_degrees": 120,
  "latency_ms": 3000,

  "_СЕКЦИЯ_СТАТУС": "─────────────────────────────",
  "is_online": true,
  "last_checked": "2025-01-15T14:30:00Z",
  "last_online": "2025-01-15T14:30:00Z",
  "uptime_pct_24h": 99.8,
  "uptime_pct_7d": 98.5,
  "uptime_pct_30d": 97.2,
  "health_score": 95,
  "avg_response_ms": 120,
  "error_code": null,
  "error_message": null,

  "_СЕКЦИЯ_МЕТА": "─────────────────────────────",
  "category": "city",
  "subcategory": "intersection",
  "tags": ["iconic", "crowded", "night", "crossing"],
  "description_ru": "Самый оживлённый пешеходный переход в мире",
  "description_en": "World's busiest pedestrian crossing",
  "thumbnail_url": "https://cdn.placebo.app/thumbs/shibuya.jpg",
  "preview_gif_url": null,
  "source": "manual",
  "source_url": "https://...",
  "license": "public",
  "attribution": "Shibuya City Tokyo",
  "is_featured": true,
  "is_verified": true,
  "is_editors_pick": false,
  "content_rating": "all",
  "primary_language": "ja",
  "has_weather_data": true,
  "weather_station_id": null,

  "_СЕКЦИЯ_ЗАПИСЬ": "─────────────────────────────",
  "recording_enabled": true,
  "recording_tier": "tier0",
  "recording_retention_days": 7,
  "recording_codec": "h265",

  "_СЕКЦИЯ_ЖЕЛЕЗО_ПАРТНЁРСКИЕ": "─────────────────────────────",
  "manufacturer": "Axis",
  "camera_model": "P3245-V",
  "install_date": "2019-03-01",
  "is_partner_camera": false,
  "owner_name": null,
  "owner_contact": null,
  "monthly_cost_usd": null,

  "_СЕКЦИЯ_ВОВЛЕЧЁННОСТЬ_runtime": "─────────────────────────────",
  "viewers_now": 0,
  "viewers_peak_24h": 0,
  "total_views": 0,
  "total_watch_minutes": 0,
  "bookmark_count": 0,
  "avg_rating": 0.0,
  "rating_count": 0,
  "clip_count": 0,
  "share_count": 0,

  "_СЕКЦИЯ_ВРЕМЕННЫЕ_МЕТКИ": "─────────────────────────────",
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z",
  "verified_at": null,
  "featured_at": null
}
```

---

## 7. Масштабирование — дорожная карта серверов

```
Сейчас (0–1K камер):
  1x Hetzner CX21 (2 ядра, 4GB RAM, €4.51/мес)
  → axum API + PostgreSQL + Redis + FFmpeg всё на одном сервере
  → SQLite можно оставить для разработки

Месяц 2–3 (1K–10K камер):
  1x Hetzner CX41 (4 ядра, 16GB RAM, €15.9/мес) → API + PostgreSQL
  1x Hetzner CX21 → Redis + Media ingest (FFmpeg)
  1x Hetzner Storagebox 10TB → recordings warm/cold

Месяц 4–6 (10K–100K камер):
  Добавляем регионы: EU (Frankfurt) + Asia (Singapore) + US (Ashburn)
  Каждый регион: 1x media server + 1x storage
  Централизованная PostgreSQL master + read replicas

Год 1 (100K–1M камер):
  Kubernetes кластер
  Отдельные media workers per region
  Собственный CDN или Cloudflare R2 для recordings
  TimescaleDB для recording_segments (time-series оптимизация)
```

---

## 8. Безопасность

- **Записи:** хранятся по непредсказуемым UUID-путям, нет directory listing
- **Клипы:** signed URL с истечением (presigned S3 URL или HMAC-подписанный путь)
- **API:** JWT + refresh tokens, rate limiting через Redis
- **Камеры:** RTSP stream URLs не отдаются напрямую клиенту — только через прокси
- **GDPR:** если камера снимает лица — нужно размытие или согласие владельца

---

## 9. Что строим прямо сейчас

| Приоритет | Что | Зачем |
|---|---|---|
| 🔴 | PostgreSQL схема (002_full_schema.sql) | Основа всего |
| 🔴 | cameras.json seed данные | Отвязываемся от Rust compile |
| 🔴 | recording_segments таблица | Метаданные записей |
| 🟡 | axum API сервер (заготовка) | Долговечная основа |
| 🟡 | clip_requests таблица + pipeline | Нарезка клипов |
| 🟢 | Redis интеграция | Real-time viewers |
| 🟢 | TimescaleDB migration | Time-series оптимизация |
