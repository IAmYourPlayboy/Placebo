-- ╔══════════════════════════════════════════════════════════════╗
-- ║  Placebo — 3D Buildings View                                ║
-- ║  Запускать ПОСЛЕ osm2pgsql импорта                         ║
-- ║  Создаёт 3D-геометрию зданий для pg2b3dm                  ║
-- ╚══════════════════════════════════════════════════════════════╝

-- Вью buildings_3d — то что pg2b3dm будет конвертировать в 3D Tiles.
--
-- Логика высоты зданий:
--   1. Если есть тег height → используем его
--   2. Если есть тег building:levels → levels × 3.0м
--   3. Если нет ничего → дефолт 10м (примерно 3 этажа)
--
-- ST_Extrude создаёт 3D-объект из 2D-полигона.
-- Но PostGIS не имеет ST_Extrude из коробки,
-- поэтому используем ST_Translate + ST_Force3D для создания
-- PolygonZ с высотой, а pg2b3dm сам экструдирует.

CREATE OR REPLACE VIEW buildings_3d AS
SELECT
    osm_id,
    -- Название здания (если есть)
    COALESCE(name, '') AS name,
    -- Тип здания
    COALESCE(building, 'yes') AS building_type,
    -- Количество этажей
    COALESCE(
        "building:levels"::INT,
        CASE
            WHEN building IN ('house', 'residential', 'detached', 'semidetached_house') THEN 2
            WHEN building IN ('apartments', 'dormitory') THEN 5
            WHEN building IN ('commercial', 'office') THEN 8
            WHEN building IN ('industrial', 'warehouse') THEN 3
            WHEN building IN ('church', 'cathedral', 'mosque', 'temple') THEN 4
            WHEN building IN ('school', 'university') THEN 3
            WHEN building IN ('hospital') THEN 6
            WHEN building IN ('skyscraper') THEN 40
            ELSE 3
        END
    ) AS levels,
    -- Высота в метрах
    COALESCE(
        parse_height(height),
        parse_height("building:height"),
        COALESCE("building:levels"::FLOAT, 3.0) * 3.0,
        10.0
    ) AS height,
    -- Минимальная высота (для зданий на постаменте)
    COALESCE(
        parse_height(min_height),
        parse_height("building:min_height"),
        COALESCE("building:min_level"::FLOAT, 0.0) * 3.0,
        0.0
    ) AS min_height,
    -- Цвет здания (если задан в OSM)
    COALESCE("building:colour", "building:color",
        CASE building
            WHEN 'commercial' THEN '#8899aa'
            WHEN 'industrial' THEN '#778877'
            WHEN 'residential' THEN '#ccbb99'
            WHEN 'church' THEN '#ddddcc'
            ELSE '#bbbbaa'
        END
    ) AS color,
    -- Материал крыши
    COALESCE("roof:shape", 'flat') AS roof_shape,
    -- Геометрия — 2D полигон в EPSG:4326 (lat/lng)
    -- pg2b3dm ожидает PolygonZ, добавляем высоту как Z-координату
    ST_Force3D(
        ST_SetSRID(way, 4326)
    ) AS geom
FROM planet_osm_polygon
WHERE
    building IS NOT NULL
    AND building != 'no'
    AND ST_IsValid(way)
    AND ST_Area(way) > 10     -- фильтр: слишком маленькие полигоны (<10 м²)
;

-- Индекс для пространственных запросов
-- (если нужно генерировать тайлы для определённого bbox)
CREATE INDEX IF NOT EXISTS idx_buildings_geom
    ON planet_osm_polygon USING GIST(way)
    WHERE building IS NOT NULL AND building != 'no';
