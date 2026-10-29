import test from 'node:test';
import assert from 'node:assert/strict';
import { withUpdateDeadline } from './updateDeadline.ts';

test('a missing native response times out and a retry can complete', async (t) => {
  t.mock.timers.enable({ apis: ['setTimeout'] });
  const pending = withUpdateDeadline(() => new Promise(() => {}));
  const rejected = assert.rejects(pending, /update_check_timeout/);
  t.mock.timers.tick(60_000);
  await rejected;
  assert.equal(await withUpdateDeadline(async () => 'current'), 'current');
});

test('a late response cannot overwrite the next check result', async (t) => {
  t.mock.timers.enable({ apis: ['setTimeout'] });
  let resolveOld;
  const old = new Promise((resolve) => {
    resolveOld = resolve;
  });
  let result;
  const first = withUpdateDeadline(() => old).then(
    (value) => {
      result = value;
    },
    () => {
      result = 'retry';
    },
  );
  t.mock.timers.tick(60_000);
  await first;
  result = await withUpdateDeadline(async () => 'current');
  resolveOld('available');
  await Promise.resolve();
  assert.equal(result, 'current');
});

test('native errors and synchronous failures are returned without waiting for the deadline', async () => {
  await assert.rejects(
    withUpdateDeadline(async () => {
      throw Error('network');
    }),
    /network/,
  );
  await assert.rejects(
    withUpdateDeadline(() => {
      throw Error('invoke');
    }),
    /invoke/,
  );
});
