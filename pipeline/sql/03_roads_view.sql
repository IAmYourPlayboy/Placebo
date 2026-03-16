CREATE OR REPLACE VIEW roads_view AS
SELECT
  osm_id,
  highway,
  name,
  way,
  CASE highway
    WHEN 'motorway' THEN 15
    WHEN 'trunk' THEN 14
    WHEN 'primary' THEN 12
    WHEN 'secondary' THEN 9
    WHEN 'tertiary' THEN 7
    WHEN 'residential' THEN 6
    WHEN 'unclassified' THEN 5
    WHEN 'service' THEN 4
    WHEN 'footway' THEN 2
    WHEN 'cycleway' THEN 2
    WHEN 'steps' THEN 1.5
    WHEN 'pedestrian' THEN 3
    WHEN 'path' THEN 1.5
    ELSE 5
  END AS width_meters
FROM planet_osm_line
WHERE highway IS NOT NULL
  AND highway NOT IN ('proposed', 'construction', 'abandoned', 'platform');
