CREATE INDEX IF NOT EXISTS idx_line_highway
  ON planet_osm_line (highway) WHERE highway IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_polygon_building
  ON planet_osm_polygon (building) WHERE building IS NOT NULL AND building != 'no';
CREATE INDEX IF NOT EXISTS idx_polygon_water
  ON planet_osm_polygon (water) WHERE water IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_polygon_leisure
  ON planet_osm_polygon (leisure) WHERE leisure IS NOT NULL;

-- Spatial indexes for bbox tile queries (critical for performance)
CREATE INDEX IF NOT EXISTS idx_line_way_gist
  ON planet_osm_line USING GIST (way);
CREATE INDEX IF NOT EXISTS idx_polygon_way_gist
  ON planet_osm_polygon USING GIST (way);
