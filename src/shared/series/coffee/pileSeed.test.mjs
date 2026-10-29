import test from 'node:test';
import assert from 'node:assert/strict';
import { periodPileSeed } from './pileSeed.ts';
const at = (y, m, d, h = 0, min = 0) => new Date(y, m - 1, d, h, min);
test('daily seed is stable within a local day, including DST, and changes at midnight', () => {
  for (const [y, m, d] of [
    [2026, 3, 8],
    [2026, 11, 1],
    [2026, 12, 31],
  ]) {
    const first = periodPileSeed('saved-base', 'day', at(y, m, d));
    assert.equal(first, periodPileSeed('saved-base', 'day', at(y, m, d, 23, 59)));
    assert.notEqual(first, periodPileSeed('saved-base', 'day', at(y, m, d + 1)));
  }
});
test('weekly seed spans Monday through Sunday and month/year transitions', () => {
  const monday = periodPileSeed('saved-base', 'week', at(2026, 12, 28));
  assert.equal(monday, periodPileSeed('saved-base', 'week', at(2027, 1, 3, 23, 59)));
  assert.notEqual(monday, periodPileSeed('saved-base', 'week', at(2027, 1, 4)));
  assert.equal(
    periodPileSeed('saved-base', 'week', at(2026, 10, 28)),
    periodPileSeed('saved-base', 'week', at(2026, 11, 1)),
  );
});
test('reopening and range roundtrips recover the seed; independent bases and scopes differ', () => {
  const base = JSON.parse(JSON.stringify({ bean_random_seed: 'saved-base' })).bean_random_seed;
  const date = at(2026, 10, 26, 12);
  const day = periodPileSeed(base, 'day', date);
  const week = periodPileSeed(base, 'week', date);
  assert.notEqual(day, week);
  assert.equal(day, periodPileSeed(base, 'day', date));
  assert.equal(week, periodPileSeed(base, 'week', date));
  assert.notEqual(day, periodPileSeed('another-saved-base', 'day', date));
});
