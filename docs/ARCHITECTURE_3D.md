# Placebo 3D World — Architecture Decision Record
> Версия 1.0 · Март 2026 · Статус: ACTIVE

---

## 0. Концепция

3D-мир — это **дефолтная среда просмотра камеры**, а не отдельный режим.
Пользователь открывает камеру → видит видеопоток на весь экран →
поворачивает мышкой → обнаруживает что стоит в 3D-городе.

Ключевая идея: видео камеры натянуто на плоскость перед "глазами",
а вокруг — 3D-модель города из OpenStreetMap.

---

## 1. Стек 3D-модуля

| Задача | Библиотека | Версия |
|---|---|---|
| 3D-движок | `three` | ^0.170.0 |
| React-обёртка | `@react-three/fiber` | ^8.17.0 |
| Хелперы | `@react-three/drei` | ^9.114.0 |
| Загрузка 3D Tiles | `3d-tiles-renderer` | ^0.4.0 |
| HLS-потоки | `hls.js` (через drei) | ^1.5.0 |
| Post-processing | `@react-three/postprocessing` | ^2.16.0 |

Итого: ~390KB gzipped. Для десктопного Tauri-приложения — ничтожно.

---

## 2. Пайплайн генерации 3D Tiles

### 2.1 Обзор

```
Geofabrik (OSM PBF)
    ↓ wget
PostGIS (osm2pgsql)
    ↓ SQL: extrude buildings
pg2b3dm (PostGIS → 3D Tiles)
    ↓ статические файлы
Cloudflare R2 / Hetzner (хостинг)
    ↓ HTTPS
3d-tiles-renderer (клиент)
```

### 2.2 Компоненты пайплайна

**osm2pgsql** — импортирует OSM PBF в PostGIS. Создаёт таблицы
с полигонами зданий, дорогами, land use.

**PostGIS** — хранит геометрию. SQL-запросом экструдируем 2D-полигоны
зданий в 3D: берём тег `building:height` из OSM, если нет — дефолт 10м.

**pg2b3dm** (.NET, Geodan) — читает 3D-геометрию из PostGIS,
генерирует 3D Tiles (tileset.json + .b3dm файлы) с автоматическим LOD.

### 2.3 Структура выходных данных

```
tiles/
├── tokyo/
│   ├── tileset.json        — корневой файл, описывает bounding volume и LOD-дерево
│   ├── tile_0_0_0.b3dm     — Batched 3D Model (glTF + метаданные зданий)
│   ├── tile_1_0_0.b3dm
│   └── ...
├── moscow/
│   ├── tileset.json
│   └── ...
└── index.json              — реестр доступных городов
```

### 2.4 Размер данных

| Город | Площадь | Кол-во зданий (OSM) | Размер 3D Tiles |
|---|---|---|---|
| Tokyo | ~2000 км² | ~1.2M | ~800MB |
| Moscow | ~2500 км² | ~600K | ~400MB |
| NYC | ~780 км² | ~1M | ~600MB |
| Mumbai | ~600 км² | ~300K | ~200MB |
| Helsinki | ~715 км² | ~100K | ~80MB |
| **Итого 5 городов** | | **~3.2M** | **~2.1GB** |

Cloudflare R2: 2.1GB × $0.015/GB = **$0.03/мес** за хранение.
Egress бесплатный. Итого: практически бесплатно.

---

## 3. Архитектура фронтенда

### 3.1 Структура компонентов

```
src/components/world3d/
├── WorldScene.tsx           — R3F Canvas + освещение + post-processing
├── BuildingsLayer.tsx       — загрузка 3D Tiles, wireframe логика
├── CameraMarker3D.tsx       — 3D-маркер соседней камеры
├── CameraFrustum.tsx        — конус видимости камеры
├── VideoPlane.tsx           — плоскость с видеотекстурой (HLS)
├── GroundPlane.tsx          — земля/дороги
├── WireframeShader.tsx      — шейдер тёмных контуров зданий
├── NavigationControls.tsx   — FPS-подобное управление (WASD + мышь)
├── CameraMinimap.tsx        — 2D мини-карта в углу (HTML overlay)
├── SkyDome.tsx              — небо (день/ночь по timezone)
└── TransitionFlight.tsx     — анимация перелёта между камерами
```

### 3.2 Дерево рендеринга

```
<WatchRoomScreen>
  <WorldScene>                          ← R3F Canvas
    <SkyDome timezone={camera.timezone} />
    <Lights />
    <BuildingsLayer                     ← 3D Tiles загрузка
      center={[camera.lat, camera.lng]}
      activeCamera={camera}
    />
    <GroundPlane />
    <CameraFrustum camera={camera}>     ← конус видимости
      <VideoPlane                        ← видео в конусе
        streamUrl={camera.hlsUrl}
      />
    </CameraFrustum>
    {nearbyCameras.map(cam =>           ← маркеры соседних камер
      <CameraMarker3D
        key={cam.id}
        camera={cam}
        onClick={navigateTo}
      />
    )}
    <NavigationControls />
    <TransitionFlight target={...} />
  </WorldScene>
  <CameraMinimap />                     ← HTML overlay поверх 3D
  <ChatPanel />                         ← существующий чат
  <ControlsOverlay />                   ← UI управления
</WatchRoomScreen>
```

### 3.3 Координатная система

3D Tiles используют ECEF (Earth-Centered, Earth-Fixed) координаты.
Для street-level рендера конвертируем в локальную систему:

```
Центр (0,0,0) = позиция активной камеры
X = восток (метры от центра)
Y = высота (метры над землёй)
Z = север (метры от центра)

Конвертация lat/lng → XZ (метры):
  dx = (lng2 - lng1) × cos(lat) × 111320
  dz = (lat2 - lat1) × 111320
```

### 3.4 Состояния компонента

```
LOADING:
  → Загружаются 3D Tiles для текущей локации
  → Показывается скелет: сетка на земле + loading spinner
  → Видео уже играет на VideoPlane

READY:
  → Здания загружены, wireframe применён
  → Пользователь может вращать камеру
  → Маркеры соседних камер видны

TRANSITIONING:
  → Летим к другой камере (Bezier animation)
  → Подгружаются тайлы новой зоны
  → Старые тайлы fade-out

ERROR:
  → 3D Tiles недоступны
  → Fallback на обычный 2D-плеер (без 3D-окружения)
```

---

## 4. Wireframe-эффект

Здания делятся на две группы:

**В поле зрения камеры (frustum)**:
- Рендерятся с полным цветом/текстурой
- Полупрозрачная подсветка (#E8345A, 10% opacity)

**Вне поля зрения (всё остальное)**:
- Custom ShaderMaterial
- Цвет рёбер: rgba(30, 35, 50, 0.6)
- Заливка: rgba(15, 18, 28, 0.15)
- Эффект "тёмного города" как в Watch Dogs / Deus Ex

Определение "в поле зрения" — Frustum Culling:
- Строим усечённую пирамиду из fov_horizontal, fov_vertical, azimuth, elevation
- Для каждого здания проверяем пересечение bounding box с пирамидой
- Оптимизация: проверка по тайлам (bounding volume), не по зданиям

---

## 5. Управление

### Desktop (мышь + клавиатура):
- **Мышь**: вращение камеры (как FPS-игра, pointer lock)
- **Колёсико**: zoom in/out
- **WASD / стрелки**: перемещение (fly mode)
- **Клик на маркер**: перелёт к камере
- **ESC**: выход из pointer lock (показать UI)
- **Пробел**: вернуться к "прямому" виду (видео на весь экран)

### Дефолтное состояние:
- Камера смотрит прямо на видеоплоскость
- Pointer lock НЕ активен
- Пользователь видит обычный плеер
- При зажатии ПКМ или при drag — начинает вращаться вид

---

## 6. Производительность

### Budget:
- Целевой FPS: 60 на GPU уровня GTX 1060 / RX 580
- Макс. треугольники в кадре: 500K
- Макс. видеотекстуры одновременно: 1 (активная камера)
- Макс. thumbnail-текстуры: 10 (ближайшие камеры)

### LOD-стратегия:
- 0–200м от центра: полная детализация зданий
- 200м–500м: средняя детализация (LOD1)
- 500м–1км: низкая детализация (LOD0)
- >1км: не загружать

### Оптимизации:
- Instanced rendering для одинаковых зданий
- Frustum culling (встроен в Three.js)
- Occlusion culling (встроен в 3d-tiles-renderer)
- Lazy load: тайлы грузятся по мере перемещения
- Adaptive quality: если FPS < 30, снижаем LOD multiplier

---

## 7. Зависимости от других модулей

| Зависимость | Нужна для | Блокирует? |
|---|---|---|
| Медиасервер (FFmpeg) | Live HLS потоки в видеотекстуре | Нет — используем thumbnail как заглушку |
| axum API | Список камер в радиусе | Нет — используем seed-данные |
| PostGIS | Генерация 3D Tiles | Да — но одноразово, потом статические файлы |
| Cloudflare R2 | Хостинг тайлов | Нет — для разработки локальный HTTP-сервер |

**Вывод**: для разработки 3D-мира блокеров нет. Всё можно делать локально.
