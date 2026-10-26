import assert from 'node:assert/strict';
import test from 'node:test';
import { coffeeSeries, coffeeTier } from './definition.ts';
import { availableSeries, seriesFor } from '../catalog.ts';
test('five roast tiers use exact effective-time boundaries and saturate at 60 minutes', () => {
  assert.equal(coffeeTier(-1), coffeeSeries.tiers[0]);
  for (let i = 1; i < 5; i++) {
    const threshold = coffeeSeries.tiers[i].from;
    assert.equal(coffeeTier(threshold - 1), coffeeSeries.tiers[i - 1]);
    assert.equal(coffeeTier(threshold), coffeeSeries.tiers[i]);
  }
  for (const [minutes, tier] of [
    [1, 0],
    [25, 1],
    [40, 2],
    [50, 3],
    [60, 4],
    [120, 4],
  ])
    assert.equal(coffeeTier(minutes * 60000), coffeeSeries.tiers[tier]);
  assert.equal(coffeeTier(NaN), coffeeSeries.tiers[0]);
  assert.equal(new Set(coffeeSeries.tiers.map((t) => t.light)).size, 5);
  assert.equal(new Set(coffeeSeries.tiers.map((t) => t.dark)).size, 5);
});
test('catalog contains only implemented series and unknown identifiers fall back to coffee', () => {
  assert.deepEqual(
    availableSeries.map((s) => s.id),
    ['coffee'],
  );
  assert.equal(seriesFor('money'), coffeeSeries);
  assert.equal(seriesFor(undefined), coffeeSeries);
  assert.equal(coffeeSeries.summary.limit, 6);
  assert.equal(coffeeSeries.collection.insideCapacity, 80);
  assert.equal(coffeeSeries.collection.displayLimit, 120);
});
