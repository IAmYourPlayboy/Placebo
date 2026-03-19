CREATE OR REPLACE VIEW parks_view AS
SELECT osm_id,
       COALESCE(leisure, "natural", landuse) AS park_type,
       name, way
FROM planet_osm_polygon
WHERE leisure IN ('park', 'garden', 'playground', 'pitch')
   OR "natural" IN ('wood', 'scrub', 'grassland')
   OR landuse IN ('grass', 'forest', 'recreation_ground', 'cemetery', 'meadow');
