# PLACEBO — ПОЛНЫЙ КОНТЕКСТ ДЛЯ CLAUDE CODE
# Дата переноса: 15 марта 2026
# Автор: IAmYourPlayboy
# Модель-источник: Claude Opus 4.6 (claude.ai Pro)

---

## 0. КАК РАБОТАТЬ СО МНОЙ

### Стиль общения
- Два разработчика, один старший
- Русский язык всегда (кроме технических терминов, переменных, команд)
- НЕ используй длинное тире (—), заменяй на (–) или не используй
- Коротко и по делу в технических вещах
- Развёрнуто там где нужны объяснения архитектуры или бизнес-логики
- Умеренные эмодзи (✅ 🔲 ⚠️), не для красоты
- Инициатива приветствуется: видишь лучший путь – говори
- Будь честен и несогласен там где думаешь что я ошибаюсь
- Без капитанства – не объясняй очевидное
- Если можно сказать в одном предложении – говори в одном

### Технические правила
- ВСЕГДА используй скиллы (см. раздел "Скиллы" ниже)
- Не используй заглушки/placeholder код – только реальный рабочий код
- Задавай себе вопрос "что изменится для конечного пользователя?" перед каждым решением
- Делай тесты к коду
- Не ограничивайся по времени ответа
- Если не уверен в чём-то – говори прямо

---

## 1. ЧТО ТАКОЕ PLACEBO

Placebo – десктопное приложение для **совместного просмотра** видео и камер мира.
Главная идея: ощущение компании когда ты один.

**Уникальная ценность**: комбинация живых видеокамер всего мира + социальный слой
(чат, голосовой созвон, совместный просмотр). Никто не сделал это раньше.

**Целевые рынки**: СНГ + международный (фокус на Индии и Финляндии помимо России).

**Платформа**: Windows (primary), macOS (secondary). Мобиль – не в планах сейчас.

---

## 2. ТЕКУЩАЯ СТАДИЯ

### Что уже сделано
- ✅ Мобильный HTML-прототип (калибровка, не продукт)
- ✅ Desktop HTML-прототип (16 экранов, 1920×1080)
- ✅ Tauri v2 + React 18 скелет с базовыми экранами
- ✅ SQLite БД камер с 50 seed-камерами (JSON + JSON Schema)
- ✅ Desktop layout (левый сайдбар 224px)
- ✅ CamerasScreen с SVG картой мира и карточками камер
- ✅ Архитектурные решения по хранению, инфраструктуре, типам камер
- ✅ 8 Tauri IPC команд для камер (Rust)
- ✅ 25 Rust тестов (camera + recording)
- ✅ **3D World архитектура полностью спроектирована** (этот чат)
- ✅ **3D World фронтенд-код написан** (7 компонентов + хук + типы)
- ✅ **Pipeline OSM → 3D Tiles спроектирован и написан** (Docker + скрипты)

### Что НЕ сделано
- 🔲 Установка npm-зависимостей для 3D (three, @react-three/fiber, drei, и т.д.)
- 🔲 Интеграция WorldScene в WatchRoomScreen
- 🔲 Генерация реальных 3D Tiles для Токио (нужен Docker)
- 🔲 Подключение 3d-tiles-renderer (раскомментировать код в BuildingsLayer.tsx)
- 🔲 API-сервер (axum/Rust)
- 🔲 Авторизация (Better Auth)
- 🔲 Медиасервер (FFmpeg → HLS)
- 🔲 WatchRoom адаптирован под desktop (частично – 3D заменяет старый плеер)
- 🔲 FriendsScreen
- 🔲 ElevenLabs перевод речи

---

## 3. ТЕХНИЧЕСКИЙ СТЕК

### Клиент (D:\Placebo)
```
Tauri v2.0.0          – десктопная обёртка
React 18.3.1          – UI
TypeScript 5.5.3      – типизация
Vite 5.4.8            – bundler
Rust (edition 2021)   – Tauri backend/IPC
```

### 3D World (НОВОЕ – из этого чата)
```
three ^0.170.0                  – 3D движок
@react-three/fiber ^8.17.0     – React обёртка для Three.js
@react-three/drei ^9.114.0     – хелперы (Billboard, Html, useVideoTexture)
@react-three/postprocessing     – bloom, outline (пока не используется)
3d-tiles-renderer ^0.4.0       – NASA загрузчик 3D Tiles (пока не установлен)
hls.js ^1.5.0                  – HLS потоки для видеотекстур
```

### Зависимости Rust (src-tauri/Cargo.toml)
```
tauri 2.0.0, tauri-plugin-opener
serde 1.0, serde_json 1.0
sqlx 0.7 (sqlite), tokio 1 (full)
thiserror 1.0, anyhow 1.0
uuid 1.6 (v4), chrono 0.4 (serde)
```

### Планируемый бэкенд (НЕ написан)
```
axum (Rust)           – HTTP/WebSocket API
PostgreSQL 15+        – основная БД
PostGIS               – геозапросы
Redis                 – real-time (viewers, sessions, rate limits)
Cloudflare R2         – объектное хранилище
Better Auth           – авторизация
FFmpeg                – RTSP→HLS транскодирование
```

---

## 4. СТРУКТУРА ПРОЕКТА

```
D:\Placebo\
├── package.json
├── vite.config.ts
├── tsconfig.json
├── index.html
├── data/
│   └── cameras-seed.json           – 50 реальных камер (JSON + JSON Schema)
├── docs/
│   ├── ARCHITECTURE.md             – общие архитектурные решения
│   └── ARCHITECTURE_3D.md          – 🆕 архитектура 3D-мира
├── pipeline/                        – 🆕 генерация 3D Tiles из OSM
│   ├── docker-compose.yml          – PostGIS + osm2pgsql + pg2b3dm + nginx
│   ├── nginx.conf                  – CORS для тайл-сервера
│   ├── scripts/generate-tiles.sh   – полный цикл генерации
│   ├── sql/01_init.sql             – PostGIS расширения
│   ├── sql/02_buildings_view.sql   – SQL вью для 3D-экструзии
│   └── styles/default.style        – osm2pgsql стиль импорта
├── src/
│   ├── App.tsx                     – root, routing, desktop layout
│   ├── App.css                     – все стили (design system)
│   ├── main.tsx                    – точка входа
│   ├── components/
│   │   ├── Sidebar.tsx             – левый сайдбар 224px
│   │   ├── BottomNav.tsx           – скрыт на десктопе
│   │   ├── Icons.tsx               – SVG иконки Apple SF-style
│   │   └── world3d/               – 🆕 3D-мир
│   │       ├── index.ts
│   │       ├── WorldScene.tsx       – корневой R3F Canvas
│   │       ├── BuildingsLayer.tsx   – 3D Tiles + wireframe шейдер
│   │       ├── CameraMarker3D.tsx   – 3D маркер камеры
│   │       ├── CameraFrustum.tsx    – конус видимости + видеоплоскость
│   │       ├── Environment.tsx      – GroundPlane, SkyDome, WorldLights
│   │       └── NavigationControls.tsx – FPS управление (WASD+мышь)
│   ├── hooks/
│   │   └── useNearbyCameras.ts     – 🆕 камеры в радиусе (mock данные)
│   ├── screens/
│   │   ├── HomeScreen.tsx
│   │   ├── ExploreScreen.tsx
│   │   ├── CamerasScreen.tsx
│   │   ├── ProfileScreen.tsx
│   │   ├── WatchRoomScreen.tsx      – ⚠️ нужна интеграция с WorldScene
│   │   ├── FriendsScreen.tsx        – заглушка
│   │   └── CreateScreen.tsx         – заглушка
│   ├── services/
│   │   └── cameras.ts              – Tauri IPC вызовы
│   └── types/
│       ├── camera.ts               – TypeScript типы камер
│       └── world3d.ts              – 🆕 типы 3D-мира
└── src-tauri/
    ├── tauri.conf.json             – конфиг (1280×800, min 900×600)
    ├── Cargo.toml
    ├── build.rs
    ├── migrations/
    │   ├── 001_cameras.sql         – схема камер
    │   └── 002_full_schema.sql     – recording_segments, clip_requests, ratings
    ├── src/
    │   ├── main.rs                 – Tauri entry point
    │   ├── lib.rs                  – регистрация команд, DB init
    │   ├── db/
    │   │   ├── mod.rs              – init() + run_migrations()
    │   │   ├── camera.rs           – Camera model + 14 тестов
    │   │   ├── recording.rs        – RecordingSegment + ClipRequest + 11 тестов
    │   │   └── seed.rs             – 50 камер (Rust structs)
    │   └── commands/
    │       ├── mod.rs
    │       └── camera.rs           – 8 Tauri IPC команд
```

---

## 5. ДИЗАЙН-СИСТЕМА

### Цвета
```css
--accent:    #E8345A    /* розово-красный, основной акцент */
--bg:        #FFFFFF    /* фон */
--bg-2:      #F5F5F7    /* вторичный фон */
--bg-3:      #EBEBEB    /* третичный фон */
--t1:        #0F0F0F    /* основной текст */
--t2:        #444444    /* вторичный текст */
--t3:        #999999    /* третичный текст */
--border:    #E8E8EC    /* границы */
--sidebar-w: 224px      /* ширина сайдбара */
```

### 3D-мир цвета
```
Wireframe здания (вне FOV):  рёбра #1e2332, заливка #0f121c
Маркер камеры (city):        #E8345A (accent)
Маркер камеры (traffic):     #FF9500
Маркер камеры (nature):      #34C759
Фон 3D-сцены:               #0a0a0f
Туман:                       #0a0a1a (500м–2000м)
```

### Типографика
```
Шрифт: Nunito (Google Fonts)
Веса: 400, 600, 700, 800, 900
```

### Анимации
```
Clash Royale spring для карточек:
  crCardPop: scale(0.55)→scale(1.06)→scale(0.97)→scale(1)
  Stagger: 9 элементов с задержкой 0.05s

3D маркеры камер: пульсация scale 1.0 → 1.1 (sin wave, 2Hz)
3D hover на маркер: scale 1.5, emissiveIntensity 2.0
```

### Emoji
Twemoji через CDN. В коде используй Unicode – Twemoji заменяет автоматически.

---

## 6. 3D-МИР PLACEBO (КЛЮЧЕВОЙ КОНТЕКСТ ЭТОГО ЧАТА)

### 6.1 Концепция

3D-мир – это **дефолтная среда просмотра камеры**, НЕ отдельный режим.

Пользователь открывает камеру → видит видеопоток на весь экран →
поворачивает мышкой (ПКМ drag) → обнаруживает что стоит в 3D-городе.

Видео камеры натянуто на плоскость перед "глазами", а вокруг – 3D-модель
города из OpenStreetMap. Здания вне поля зрения камеры показаны wireframe
(тёмные контуры).

### 6.2 Стек 3D

| Задача | Библиотека |
|---|---|
| 3D-движок | three + @react-three/fiber |
| Хелперы | @react-three/drei |
| 3D здания | 3d-tiles-renderer (NASA AMMOS) |
| HLS потоки | hls.js (через drei useVideoTexture) |
| Post-processing | @react-three/postprocessing |

### 6.3 Источник 3D-зданий: свой хостинг

**Решение принято**: НЕ используем Cesium ion. Сразу свой хостинг.

Пайплайн:
```
Geofabrik (OSM PBF) → osm2pgsql → PostGIS → pg2b3dm → 3D Tiles → Cloudflare R2
```

8 городов преднастроено: Tokyo, Moscow, NYC, Mumbai, Helsinki, London, Paris, Dubai.

Размер всех 5 основных городов: ~2.1GB.
Стоимость хранения на R2: ~$0.03/мес. Egress бесплатный.

### 6.4 Компоненты 3D-мира (написаны, не интегрированы)

| Компонент | Файл | Статус |
|---|---|---|
| Корневой Canvas | WorldScene.tsx | ✅ написан |
| 3D здания | BuildingsLayer.tsx | ✅ написан (mock + заготовка для 3D Tiles) |
| Маркеры камер | CameraMarker3D.tsx | ✅ написан |
| Конус видимости | CameraFrustum.tsx | ✅ написан |
| Земля/Небо/Свет | Environment.tsx | ✅ написан |
| FPS управление | NavigationControls.tsx | ✅ написан |
| Хук камер | useNearbyCameras.ts | ✅ написан (mock данные) |
| Типы | world3d.ts | ✅ написан |

### 6.5 Управление в 3D

| Действие | Клавиша/мышь |
|---|---|
| Вращение | ПКМ + drag |
| Перемещение | WASD / стрелки |
| Вверх/вниз | E / Q |
| Zoom | Колёсико |
| Сброс вида | Пробел |
| Перелёт к камере | Клик на маркер |

### 6.6 Координатная система 3D

```
Центр (0,0,0) = позиция активной камеры
X = восток (метры от центра)
Y = высота (метры над землёй)
Z = север (метры от центра)

Конвертация lat/lng → XZ:
  dx = (lng2 - lng1) × cos(lat) × 111320
  dz = (lat2 - lat1) × 111320
```

### 6.7 Wireframe-эффект (GLSL шейдер)

Здания В поле зрения камеры → MeshStandardMaterial (#aab0bc).
Здания ВНЕ поля зрения → Custom ShaderMaterial:
- Рёбра: rgba(30, 35, 50, 0.6)
- Заливка: rgba(15, 18, 28, 0.12)
- Fade по расстоянию (прозрачнее к 1500м)

### 6.8 Поля камеры для 3D (будущая миграция, НЕ сейчас)

```
height_above_ground  REAL  – высота камеры над землёй (м)
camera_azimuth       REAL  – направление (0-360°)
camera_elevation     REAL  – вертикальный угол (-90..90°)
fov_horizontal       REAL  – горизонтальный FOV (°)
fov_vertical         REAL  – вертикальный FOV (°)
```

Дефолты: height=5м, azimuth=0°, elevation=-15°, fov_h=90°, fov_v=58°.

### 6.9 Производительность

- Целевой FPS: 60 на GTX 1060 / RX 580
- Макс треугольники: 500K
- LOD: 0–200м полный, 200–500м средний, 500–1000м низкий, >1км не грузить
- Макс видеотекстур: 1 (активная камера)
- Макс thumbnail: 10 (ближайшие)
- Adaptive quality: если FPS < 30, снижаем LOD multiplier

---

## 7. ТИПЫ КАМЕР

### Public
- Добавляются командой Placebo + краулером
- Открыты для всех
- Записи по системе тиров
- Без шифрования

### Enterprise
- B2B-подписка, закрытая сеть
- Только выбранные пользователи
- 6 месяцев хранения (по закону)
- AES-256 E2E шифрование

### Yourself (Premium)
- Личные камеры Premium-пользователей
- Можно открыть для публики
- 45 дней хранения
- AES-256 E2E
- Адрес скрыт – только custom_label

---

## 8. СИСТЕМА ЗАПИСЕЙ

### Тиры (пересчёт раз в месяц)

| Тир | Просмотры/мес | Retention | С бустами (макс) |
|---|---|---|---|
| tier1 | 1000+ | 14 дней | 365 дней |
| tier2 | 500–999 | 7 дней | 365 дней |
| tier3 | 100–499 | 5 дней | 365 дней |
| tier4 | 20–99 | 2 дня | 365 дней |
| tier5 | 0–19 | 0 (live only) | – |

### Буст-система

- Premium: 4 токена/мес (сгорают в конце месяца)
- 1 токен = +3 дня retention для любой камеры
- Потолок: 365 дней (продуктовый, не финансовый)
- Каждый буст – отдельная запись с expires_at
- Доп. бусты: +4 за 59₽, +10 за 99₽
- На странице камеры: "Поддержано N пользователями" + аватары

Стоимость бустов ≈ 10% от дохода Premium (линейно масштабируется).

### Хранение по уровням

| Уровень | Срок | Кодек |
|---|---|---|
| HOT | 0–24ч | H.264 HLS .ts |
| WARM | 1–7 дней | H.265 MP4 |
| COLD | 8–30 дней | H.265 CRF28 |
| ARCHIVE | 30+ дней | AV1 (batch ночью) |

---

## 9. ФИНАЛЬНЫЕ ПОЛЯ КАМЕРЫ

```
ИДЕНТИФИКАЦИЯ:
  id TEXT PK                    – UUID v4
  name TEXT NOT NULL            – оригинальный язык страны
  slug TEXT UNIQUE              – URL-friendly
  camera_type TEXT DEFAULT 'public'  – 'public'|'enterprise'|'yourself'
  external_id TEXT              – только в БД, не в API

ЛОКАЦИЯ:
  country, country_code, region, city, district  – все TEXT
  address TEXT                  – только для public, NULL для yourself/enterprise
  custom_label TEXT             – для yourself/enterprise (пользовательское название)
  lat REAL NOT NULL, lng REAL NOT NULL
  timezone TEXT                 – IANA (Asia/Tokyo)
  [НЕ хранить: google_maps_url и т.д. – генерировать из lat/lng на лету]

СТРИМ:
  stream_url TEXT NOT NULL      – только в БД, не в API
  backup_url TEXT
  stream_type TEXT              – 'rtsp'|'hls'|'youtube'|'dash'|'webrtc'|'mjpeg'
  stream_protocol TEXT          – 'tcp'|'udp'|'https'
  stream_quality_default TEXT
  available_qualities TEXT      – JSON ['480p','720p','1080p']
  frame_rate INTEGER            – только для FFmpeg, не в API
  bitrate_kbps INTEGER
  codec TEXT                    – 'h264'|'h265'|'av1'|'vp9'
  resolution_w INTEGER, resolution_h INTEGER
  latency_ms INTEGER            – показывать в UI

ВОЗМОЖНОСТИ:
  has_audio INTEGER DEFAULT 0
  has_night_vision INTEGER DEFAULT 0
  is_underwater INTEGER DEFAULT 0

МЕТА:
  category TEXT DEFAULT 'city'
  subcategory TEXT
  tags TEXT DEFAULT '[]'        – JSON массив
  description_en TEXT           – pivot-язык для переводов
  thumbnail_url TEXT
  source_url TEXT               – опционально
  attribution TEXT              – "Предоставлено: ..."

ЗАПИСЬ:
  recording_enabled INTEGER DEFAULT 0
  retention_tier TEXT DEFAULT 'tier5'
  recording_retention_days INTEGER DEFAULT 0
  recording_codec TEXT DEFAULT 'h264'

ПАРТНЁР/ЖЕЛЕЗО:
  manufacturer TEXT
  camera_model TEXT
  added_to_placebo_at TEXT
  is_partner_camera INTEGER DEFAULT 0
  owner_name TEXT               – опционально, с разрешения

RUNTIME (Redis/PostgreSQL, НЕ в seed):
  viewers_now, total_views, health_score, uptime_pct_30d, is_online

TIMESTAMPS:
  created_at, updated_at
```

---

## 10. ЯЗЫКОВАЯ СИСТЕМА

- 28 языков планируется
- Никнеймы хранятся в оригинале, НЕ переводятся
- name камеры – оригинальный язык страны
- description_en – pivot-язык (английский)
- Перевод: description_en → целевой язык через API (DeepL/LibreTranslate)
- Переводы кешируются в таблице camera_translations
- Логика: исходный → EN → целевой (не прямой ja→hi)
- Формат дат/времени берётся с устройства (locale системы)

---

## 11. МОНЕТИЗАЦИЯ

```
Free:
  - Просмотр public-камер без ограничений
  - Комнаты до 4 человек
  - Клипы: только скачать
  - Нет yourself-камер

Premium (199₽/мес или 1490₽/год):
  - Комнаты до 20 человек
  - 4 буст-токена/месяц
  - Yourself-камеры
  - Облако 7 GB
  - Синхронизация watch_history
  - Анимированные обои профиля
  - Голосовой созвон
  - Видеорегистратор

Enterprise (B2B):
  - Закрытая сеть камер
  - 6 месяцев хранения
  - AES-256 E2E
  - SLA
```

---

## 12. ИНФРАСТРУКТУРА

```
Cloudflare (WAF + Anycast DDoS)
    ↓
Load Balancer (Hetzner)
    ↓
2-3 API серверов (axum/Rust)
    ↓
PostgreSQL + PostGIS  |  Redis  |  Cloudflare R2

Медиа-слой:
  Cameras (RTSP) → FFmpeg workers → HLS segments → SSD (hot 24h) → R2 (warm/cold/archive)
```

Путь масштабирования: Docker Compose → Swarm → K8s (только с DevOps-командой).

---

## 13. КЛЮЧЕВЫЕ АРХИТЕКТУРНЫЕ РЕШЕНИЯ (зафиксированы, не пересматривать)

1. SQLite локально – только личные данные пользователя
2. PostgreSQL + PostGIS на сервере – всё общее
3. Redis – только real-time (viewers, sessions, rate limits)
4. Cloudflare с первого дня – R2 + WAF + CDN
5. axum (Rust) – API сервер
6. Seed данные: JSON + JSON Schema
7. RTSP URLs никогда не в API – только через медиасервер
8. Docker Compose → Swarm → K8s
9. Три типа камер: public / enterprise / yourself
10. Пять тиров хранения + буст-система (потолок 365 дней)
11. Клипы = нарезки с камер. Моменты = пользовательский контент
12. **3D-мир = дефолтная среда просмотра** (не отдельный режим)
13. **3D здания: свой хостинг** (OSM → PostGIS → pg2b3dm → R2, НЕ Cesium ion)
14. **React Three Fiber** для 3D (не CesiumJS, не MapLibre)
15. **Видео при наведении: только thumbnail** (live preview HLS – позже, не MVP)

---

## 14. GITHUB И ЛОКАЛЬНАЯ РАЗРАБОТКА

```
Репозиторий: https://github.com/IAmYourPlayboy/Placebo.git
Ветка: main
Локальный путь: D:\Placebo
IDE: VSCode с PowerShell terminal
OS: Windows (primary)

Команды:
  npm run dev          → Vite dev server
  npm run tauri dev    → Tauri + Vite (полное приложение)
  npm run tauri build  → .exe инсталлятор
  cd src-tauri && cargo test  → 25 Rust тестов

Git workflow:
  git add .
  git commit -m "feat: описание"
  git push
```

---

## 15. СКИЛЛЫ (ОБЯЗАТЕЛЬНЫ К ИСПОЛЬЗОВАНИЮ)

| Скилл | Когда использовать |
|---|---|
| `tauri` | Любая работа с Tauri |
| `tauri-architecture` | Проектирование архитектуры |
| `rust-pro` | Написание Rust кода |
| `react-guidelines` | React компоненты |
| `frontend-design` | Создание UI |
| `systematic-debugging` | ЛЮБОЙ баг (не debugger!) |
| `better-auth-best-practices` | Авторизация |
| `create-auth-skill` | Скаффолдинг auth |
| `email-and-password-best-practices` | Email auth |
| `two-factor-authentication-best-practices` | 2FA |
| `better-auth-security-best-practices` | Безопасность auth |
| `typescript-advanced-types` | Сложные TS типы |

---

## 16. ЧТО ДЕЛАТЬ ДАЛЬШЕ (в порядке приоритета)

### Немедленно (текущая задача):
1. **Установить npm-зависимости 3D** в D:\Placebo:
   ```bash
   npm install three @react-three/fiber @react-three/drei @react-three/postprocessing hls.js
   npm install -D @types/three
   ```

2. **Скопировать файлы 3D-мира** из пакета placebo-3d-world.zip:
   - `src/types/world3d.ts` → `D:\Placebo\src\types\world3d.ts`
   - `src/hooks/useNearbyCameras.ts` → `D:\Placebo\src\hooks\useNearbyCameras.ts`
   - `src/components/world3d/` → `D:\Placebo\src\components\world3d\`
   - `pipeline/` → `D:\Placebo\pipeline\`
   - `docs/ARCHITECTURE_3D.md` → `D:\Placebo\docs\ARCHITECTURE_3D.md`

3. **Интегрировать WorldScene в WatchRoomScreen** – заменить текущий mock-плеер на 3D-среду

4. **Убедиться что npm run tauri dev собирается** без ошибок

### После интеграции:
5. **Генерация 3D Tiles для Токио** (Docker):
   ```bash
   cd pipeline && docker compose up -d postgis && bash scripts/generate-tiles.sh tokyo
   ```

6. **Подключить 3d-tiles-renderer** – раскомментировать код в BuildingsLayer.tsx

7. **npm install 3d-tiles-renderer** – после генерации тайлов

### Потом (бэкенд):
8. Миграция 003 (финальная схема камер со всеми полями)
9. axum API-сервер (placebo-api crate)
10. Better Auth
11. FFmpeg медиасервер → HLS видеотекстуры в 3D

---

## 17. ИЗВЕСТНЫЕ ПРОБЛЕМЫ И ЗАМЕТКИ

- main.rs содержит дублированный код (два fn main, legacy + новый) – нужно вычистить
- Legacy команды (greet, get_public_rooms, create_room) в lib.rs – удалить когда будет API
- BottomNav.tsx скрыт через CSS display:none – можно удалить
- CamerasScreen и HomeScreen используют mock-данные – это ок для прототипа
- 3D Tiles загрузчик закомментирован в BuildingsLayer.tsx до npm install 3d-tiles-renderer
- HLS видеотекстура закомментирована в CameraFrustum.tsx до готовности медиасервера
- pipeline/scripts/generate-tiles.sh – bash-скрипт, на Windows нужен WSL или Git Bash

---

*Документ создан Claude Opus 4.6 на основе переноса контекста из claude.ai Pro.*
*Исходный контекст: ~20ч работы Sonnet + 1 сессия Opus (3D-мир).*
*Репозиторий: https://github.com/IAmYourPlayboy/Placebo.git*
