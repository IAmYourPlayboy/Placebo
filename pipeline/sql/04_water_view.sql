CREATE OR REPLACE VIEW water_view AS
SELECT osm_id, 'polygon' AS geom_type,
       COALESCE(water, "natural") AS water_type, name, way
FROM planet_osm_polygon
WHERE water IS NOT NULL
   OR "natural" IN ('water', 'wetland')
UNION ALL
SELECT osm_id, 'line' AS geom_type,
       waterway AS water_type, name, way
FROM planet_osm_line
WHERE waterway IN ('river', 'stream', 'canal', 'drain');
