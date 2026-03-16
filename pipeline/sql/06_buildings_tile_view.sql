CREATE OR REPLACE VIEW buildings_tile_view AS
SELECT
  osm_id,
  way,
  CASE
    WHEN height ~ '^\d+(\.\d+)?$' THEN height::float
    WHEN "building:levels" ~ '^\d+$' THEN "building:levels"::int * 3.0
    ELSE 9.0
  END AS height_meters,
  name
FROM planet_osm_polygon
WHERE building IS NOT NULL
  AND building NOT IN ('no', 'entrance')
  AND ST_Area(way::geography) > 10;
