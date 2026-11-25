import { circular } from 'ol/geom/Polygon.js';

// OpenLayers constructs this circle on the mean-radius sphere in EPSG:4326.
export function incidentArea(center: number[], radius: number) {
  if (center.length !== 2 || !center.every(Number.isFinite) || Math.abs(center[0]) > 180 || Math.abs(center[1]) > 85 || !Number.isFinite(radius) || radius <= 0 || radius > 2000) return null;
  return circular(center, radius, 96).transform('EPSG:4326', 'EPSG:3857');
}
