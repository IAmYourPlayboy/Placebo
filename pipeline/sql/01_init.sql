-- ╔══════════════════════════════════════════════════════════════╗
-- ║  Placebo 3D Tiles Pipeline — PostGIS Init                  ║
-- ║  Создаёт расширения и вью для 3D-экструзии зданий         ║
-- ╚══════════════════════════════════════════════════════════════╝

-- Расширения PostGIS
CREATE EXTENSION IF NOT EXISTS postgis;
CREATE EXTENSION IF NOT EXISTS postgis_topology;

-- После импорта osm2pgsql создаст таблицу planet_osm_polygon.
-- Этот SQL создаёт вью buildings_3d которое pg2b3dm будет читать.
-- Вью создаётся ПОСЛЕ импорта данных (см. generate-tiles.sh).

-- Функция для безопасного парсинга высоты из OSM-тегов.
-- OSM хранит высоту как текст: "15", "15 m", "15m", "49.2 ft"
CREATE OR REPLACE FUNCTION parse_height(height_str TEXT)
RETURNS FLOAT AS $$
BEGIN
    IF height_str IS NULL OR height_str = '' THEN
        RETURN NULL;
    END IF;

    -- Убираем "m", "м", " m" из конца
    DECLARE
        cleaned TEXT := TRIM(regexp_replace(height_str, '\s*(m|м|meters?|метр(ов|а)?)\s*$', '', 'i'));
        val FLOAT;
    BEGIN
        -- Пробуем парсить как число
        val := cleaned::FLOAT;

        -- Если значение похоже на футы (содержит "ft" или "'" или "`")
        IF height_str ~* '(ft|feet|''|`)' THEN
            val := val * 0.3048;  -- конвертация футов в метры
        END IF;

        -- Санитарная проверка: здание не может быть <1м или >1000м
        IF val < 1 OR val > 1000 THEN
            RETURN NULL;
        END IF;

        RETURN val;
    EXCEPTION WHEN OTHERS THEN
        RETURN NULL;
    END;
END;
$$ LANGUAGE plpgsql IMMUTABLE;
