import test from 'node:test';
import assert from 'node:assert/strict';
import {
  collectionProgress,
  focusProgress,
  routePosition,
  ascentRoute,
  routeSegmentMs,
} from './progress.ts';
test('collection route uses eight completed half-hours and remains saturated', () => {
  assert.equal(collectionProgress(0), 0);
  assert.equal(collectionProgress(routeSegmentMs - 1), 0);
  assert.equal(collectionProgress(routeSegmentMs), 1 / 8);
  assert.equal(collectionProgress(routeSegmentMs * 8), 1);
  assert.equal(collectionProgress(routeSegmentMs * 10000), 1);
  assert.equal(collectionProgress(-1), 0);
  assert.equal(collectionProgress(NaN), 0);
  assert.deepEqual(routePosition(0), { x: ascentRoute[0][0], y: ascentRoute[0][1] });
  assert.deepEqual(routePosition(1), { x: ascentRoute[8][0], y: ascentRoute[8][1] });
});
test('one session route clamps reminder overtime and does not use historical totals', () => {
  assert.equal(focusProgress(25 * 60000, 50 * 60000), 0.5);
  assert.equal(focusProgress(90 * 60000, 50 * 60000), 1);
  assert.equal(focusProgress(1, 0), 0);
  assert.equal(focusProgress(-1, 60000), 0);
  assert.equal(focusProgress(Infinity, 60000), 0);
  assert.equal(focusProgress(1, NaN), 0);
});
