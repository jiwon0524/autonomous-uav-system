import test from 'node:test';
import assert from 'node:assert/strict';
import { incidentArea } from '../src/mapGeometry.ts';
import { toLonLat } from 'ol/proj.js';
import { getDistance } from 'ol/sphere.js';

test('incident circles have the requested surface radius', () => {
  for (const center of [[126.978, 37.5665], [129.0756, 35.1796]]) {
    for (const radius of [50, 100, 2000]) {
      const polygon = incidentArea(center, radius);
      assert.ok(polygon);
      for (const point of polygon.getCoordinates()[0]) {
        assert.ok(Math.abs(getDistance(center, toLonLat(point)) - radius) < 0.01);
      }
    }
  }
});

test('rejects invalid or out-of-bounds incident areas', () => {
  for (const [center, radius] of [[[0,0],0], [[0,0],2001], [[0,0],NaN], [[181,0],50], [[0,86],50], [[NaN,0],50], [[0],50]]) {
    assert.equal(incidentArea(center,radius),null);
  }
});
