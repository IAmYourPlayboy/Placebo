#!/usr/bin/env bash
# ╔══════════════════════════════════════════════════════════════╗
# ║  Placebo — генерация 3D Tiles для города                    ║
# ║                                                              ║
# ║  Использование:                                              ║
# ║    ./generate-tiles.sh tokyo                                 ║
# ║    ./generate-tiles.sh moscow                                ║
# ║    ./generate-tiles.sh all                                   ║
# ║                                                              ║
# ║  Требования:                                                 ║
# ║    - Docker + Docker Compose                                 ║
# ║    - ~5GB свободного места на диск                          ║
# ║    - ~30 минут на город (зависит от размера)                ║
# ╚══════════════════════════════════════════════════════════════╝

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR/.."

# ── Конфигурация городов ────────────────────────────────────────
# Формат: "slug|geofabrik_url|bbox_для_pg2b3dm"
# bbox = min_lng,min_lat,max_lng,max_lat
declare -A CITIES=(
    ["tokyo"]="kanto|https://download.geofabrik.de/asia/japan/kanto-latest.osm.pbf|139.5,35.5,140.0,35.85"
    ["moscow"]="central-fed-district|https://download.geofabrik.de/russia/central-fed-district-latest.osm.pbf|37.3,55.55,37.9,55.95"
    ["nyc"]="new-york|https://download.geofabrik.de/north-america/us/new-york-latest.osm.pbf|-74.1,40.6,-73.7,40.9"
    ["mumbai"]="india|https://download.geofabrik.de/asia/india-latest.osm.pbf|72.75,18.85,72.99,19.28"
    ["helsinki"]="finland|https://download.geofabrik.de/europe/finland-latest.osm.pbf|24.8,60.1,25.2,60.3"
    ["london"]="england|https://download.geofabrik.de/europe/great-britain/england-latest.osm.pbf|-0.3,51.4,0.05,51.6"
    ["paris"]="ile-de-france|https://download.geofabrik.de/europe/france/ile-de-france-latest.osm.pbf|2.2,48.8,2.5,48.92"
    ["dubai"]="gcc-states|https://download.geofabrik.de/asia/gcc-states-latest.osm.pbf|55.1,25.05,55.4,25.35"
)

# Цвета для вывода
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log()  { echo -e "${GREEN}[✓]${NC} $1"; }
warn() { echo -e "${YELLOW}[!]${NC} $1"; }
err()  { echo -e "${RED}[✗]${NC} $1"; exit 1; }

# ── Параметры PostGIS ───────────────────────────────────────────
PG_HOST="localhost"
PG_PORT="5433"
PG_DB="placebo_tiles"
PG_USER="placebo"
PG_PASS="placebo_dev_only"
PG_CONN="postgresql://${PG_USER}:${PG_PASS}@${PG_HOST}:${PG_PORT}/${PG_DB}"

# ── Функции ─────────────────────────────────────────────────────

ensure_docker() {
    if ! docker compose ps --services --filter "status=running" | grep -q postgis; then
        log "Запускаю PostGIS..."
        docker compose up -d postgis
        sleep 5
        # Ждём пока PostGIS будет ready
        for i in $(seq 1 30); do
            if docker compose exec -T postgis pg_isready -U "$PG_USER" -d "$PG_DB" &>/dev/null; then
                log "PostGIS готов"
                return
            fi
            sleep 1
        done
        err "PostGIS не запустился за 30 секунд"
    fi
}

download_pbf() {
    local city="$1"
    local url="$2"
    local filename="data/$(basename "$url")"

    mkdir -p data

    if [ -f "$filename" ]; then
        warn "PBF уже скачан: $filename (пропускаю)"
        echo "$filename"
        return
    fi

    log "Скачиваю OSM данные для $city..."
    log "URL: $url"
    wget -q --show-progress -O "$filename" "$url"
    log "Скачано: $filename ($(du -h "$filename" | cut -f1))"
    echo "$filename"
}

import_osm() {
    local pbf_file="$1"
    local city="$2"

    log "Импортирую OSM данные в PostGIS ($city)..."
    log "Это может занять 5-30 минут в зависимости от размера региона"

    # osm2pgsql импорт
    # -S default.style — стандартный стиль (включает building, height, etc.)
    # -C 4096 — 4GB кеша (увеличь если много RAM)
    # --slim — необходим для обновлений, но мы пока не обновляем
    docker compose run --rm osm2pgsql \
        osm2pgsql \
        --create \
        --slim \
        -H postgis \
        -d "$PG_DB" \
        -U "$PG_USER" \
        -C 4096 \
        -S /styles/default.style \
        "/data/$(basename "$pbf_file")"

    log "Импорт завершён"
}

create_3d_view() {
    log "Создаю 3D-вью зданий..."

    # Выполняем SQL для создания buildings_3d вью
    docker compose exec -T postgis \
        psql -U "$PG_USER" -d "$PG_DB" \
        -f /docker-entrypoint-initdb.d/02_buildings_view.sql

    # Проверяем количество зданий
    local count
    count=$(docker compose exec -T postgis \
        psql -U "$PG_USER" -d "$PG_DB" -t -A \
        -c "SELECT COUNT(*) FROM buildings_3d;")

    log "Найдено зданий: $count"
}

generate_tiles() {
    local city="$1"
    local bbox="$2"

    mkdir -p "output/$city"

    log "Генерирую 3D Tiles для $city (bbox: $bbox)..."
    log "Это может занять 10-60 минут"

    # pg2b3dm конвертация
    # --lodcolumn height — LOD основан на высоте (высокие здания грузятся первыми)
    # --geometricerrors — уровни LOD: 500м, 200м, 50м, 10м
    docker compose run --rm pg2b3dm \
        pg2b3dm \
        -h postgis \
        -U "$PG_USER" \
        -d "$PG_DB" \
        -p 5432 \
        -t buildings_3d \
        -c geom \
        --idcolumn osm_id \
        --lodcolumn height \
        --geometricerrors 500,200,50,10 \
        -o "/output/$city"

    local tile_count
    tile_count=$(find "output/$city" -name "*.b3dm" | wc -l)
    local total_size
    total_size=$(du -sh "output/$city" | cut -f1)

    log "Готово! $city: $tile_count тайлов, $total_size"
}

create_city_index() {
    log "Создаю index.json (реестр городов)..."

    local index_json="output/index.json"
    echo '{"cities":[' > "$index_json"

    local first=true
    for dir in output/*/; do
        [ -f "$dir/tileset.json" ] || continue
        local city_name
        city_name=$(basename "$dir")

        if [ "$first" = true ]; then
            first=false
        else
            echo ',' >> "$index_json"
        fi

        echo "  {\"slug\":\"$city_name\",\"tileset\":\"/$city_name/tileset.json\"}" >> "$index_json"
    done

    echo ']}' >> "$index_json"
    log "index.json создан"
}

process_city() {
    local city="$1"

    if [ -z "${CITIES[$city]+x}" ]; then
        err "Неизвестный город: $city. Доступные: ${!CITIES[*]}"
    fi

    IFS='|' read -r region url bbox <<< "${CITIES[$city]}"

    log "═══════════════════════════════════════"
    log "Обработка: $city"
    log "═══════════════════════════════════════"

    ensure_docker

    local pbf_file
    pbf_file=$(download_pbf "$city" "$url")

    import_osm "$pbf_file" "$city"
    create_3d_view
    generate_tiles "$city" "$bbox"
}

# ── Main ────────────────────────────────────────────────────────

CITY="${1:-}"

if [ -z "$CITY" ]; then
    echo "Использование: $0 <город|all>"
    echo ""
    echo "Доступные города:"
    for c in "${!CITIES[@]}"; do
        echo "  - $c"
    done
    echo "  - all (все города)"
    exit 0
fi

if [ "$CITY" = "all" ]; then
    for c in "${!CITIES[@]}"; do
        process_city "$c"
    done
else
    process_city "$CITY"
fi

create_city_index

log ""
log "═══════════════════════════════════════"
log "Всё готово!"
log ""
log "Для запуска локального тайл-сервера:"
log "  docker compose up -d tile-server"
log ""
log "Тайлы доступны по адресу:"
log "  http://localhost:8090/<город>/tileset.json"
log "═══════════════════════════════════════"
