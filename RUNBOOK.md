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
- PostgreSQL 17 (с PostGIS + OSM данные Канто)
- Redis
- Axum API (порт 3001)
- Vite dev server (порт 1420)

**Что НЕ нужно:**
- Docker / go2rtc – заменён на HLS proxy в `vite.config.ts`

### Шаги

```bash
# 1. Проверить что PostgreSQL и Redis запущены
pg_isready
redis-cli ping

# 2. Запустить axum API (из корня проекта)
cd placebo-api && cargo run &
# Ждём "listening on 0.0.0.0:3001"

# 3. Запустить Vite dev server
npm run dev
# Откроется на http://localhost:1420

# 4. В браузере: Главная → клик на "Онлайн карта мира"
# 3D мир загрузится, видео появится через 3–8 сек (буферизация HLS)
```

### Как работает видео

```
Браузер (hls.js) → /hls-proxy?src=shibuya-crossing (Vite middleware)
                    ↓
              yt-dlp получает HLS URL от YouTube (кеш 30 мин)
                    ↓
              Fetch m3u8 с YouTube CDN
                    ↓
              Перезапись URL сегментов через наш прокси (решает CORS)
                    ↓
              Браузер качает сегменты через /hls-proxy?src=...&seg=...
```

### Ключевые файлы

| Файл | Роль |
|------|------|
| `vite.config.ts` | HLS proxy middleware (yt-dlp + CORS) |
| `src/hooks/useNearbyCameras.ts` | Mock камеры с `/hls-proxy` URL |
| `src/screens/World3DScreen.tsx` | Стартовая камера + `streamUrl()` |
| `src/components/world3d/CameraFrustum.tsx` | VideoPlane + hls.js логика |
| `src/hooks/useCityTiles.ts` | Загрузка OSM тайлов (дороги, здания, вода, парки) |

### Частые проблемы

| Симптом | Причина | Решение |
|---------|---------|---------|
| Белые плоскости вместо видео | Запросы идут на go2rtc (порт 1984) | Убрать `VITE_GO2RTC_URL` из `.env` и `launch.json` |
| Белый экран при входе в 3D | R3F компоненты крашатся | Проверить маппинг API→frontend в `useCityTiles.ts` (coords→points, width_meters→width) |
| `stream not found` в консоли | YouTube ID не в словаре | Добавить ID в `YOUTUBE_IDS` в `vite.config.ts` |
| Сегменты 403/410 | Истёк YouTube HLS URL | Перезапустить Vite (сбросит кеш yt-dlp) |
| Тайлы 502 / ERR_ABORTED | Axum API не запущен | `cd placebo-api && cargo run` |

### Требования к системе

- `yt-dlp` установлен и в PATH
- `hls.js` в зависимостях (`npm install`)
- Порт 1420 свободен (Vite)
- Порт 3001 свободен (axum API)

---

## 2. Добавление новой YouTube камеры

```bash
# 1. Найти YouTube Live stream ID (из URL: youtube.com/live/XXXXXXXXXXX)

# 2. Добавить в vite.config.ts → YOUTUBE_IDS:
#    'my-camera-slug': 'XXXXXXXXXXX',

# 3. Добавить mock камеру в src/hooks/useNearbyCameras.ts → MOCK_CAMERAS:
#    makeMock('my-camera-slug', 'Camera Name', lat, lng, 'city', height,
#      { azimuth: 180, elevation: -20, fovHorizontal: 80, fovVertical: 50 },
#      streamUrl('my-camera-slug')),

# 4. HMR подхватит – перезагрузка не нужна
```

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
