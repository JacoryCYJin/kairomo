import test from 'node:test';
import assert from 'node:assert/strict';
import { phraseReadingMs } from './phrasePlayback.ts';
import { cachedPile, rememberPile } from '../../shared/series/coffee/pileCache.ts';
test('segments receive reading time without changing the stored sentence', () => {
  assert.equal(phraseReadingMs('短句'), 8000);
  assert.ok(phraseReadingMs('a'.repeat(60)) > 8000);
  assert.equal(phraseReadingMs('a'.repeat(1000)), 25000);
  assert.equal(phraseReadingMs('👨‍👩‍👧‍👦'.repeat(20)), 8000);
});
test('pile cache reuses exact ordered identities and evicts older results', () => {
  const layout = { seed: 'test-period', poses: [{ id: 'a' }], evaluatedIds: ['a'], capped: false };
  rememberPile(layout);
  assert.equal(cachedPile(['a'], 'test-period'), layout);
  assert.equal(cachedPile(['b'], 'test-period'), undefined);
  for (let i = 0; i < 5; i++)
    rememberPile({ seed: 'test-period', poses: [], evaluatedIds: [String(i)], capped: false });
  assert.equal(cachedPile(['a'], 'test-period'), undefined);
});

test('capped cache skips extra records but invalidates a changed supported prefix', () => {
  const layout = {
    seed: 'test-period',
    poses: [{ id: 'first' }],
    evaluatedIds: ['first', 'overflow'],
    capped: true,
  };
  rememberPile(layout);
  assert.equal(cachedPile(['first', 'overflow', 'later'], 'test-period'), layout);
  assert.equal(cachedPile(['first', 'overflow', 'later', 'new'], 'test-period'), layout);
  assert.equal(cachedPile(['first'], 'test-period'), undefined);
  assert.equal(cachedPile(['overflow', 'later'], 'test-period'), undefined);
  assert.equal(cachedPile(['first', 'changed'], 'test-period'), undefined);
});

test('period seeds isolate cached layouts even for an identical record prefix', () => {
  const day = {
    seed: 'day-A',
    poses: [{ id: 'a' }],
    evaluatedIds: ['a', 'overflow'],
    capped: true,
  };
  const week = { ...day, seed: 'week-A', poses: [{ id: 'a', x: 12 }] };
  rememberPile(day);
  rememberPile(week);
  assert.equal(cachedPile(['a', 'overflow', 'later'], 'day-A'), day);
  assert.equal(cachedPile(['a', 'overflow', 'later'], 'week-A'), week);
  assert.equal(cachedPile(['a', 'overflow'], 'day-B'), undefined);
});
