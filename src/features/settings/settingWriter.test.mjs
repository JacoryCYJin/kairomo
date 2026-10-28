import test from 'node:test';
import assert from 'node:assert/strict';
import { settingWriter } from './settingWriter.ts';
test('rapid edits are serialized and merged against the latest saved settings', async () => {
  let release;
  const barrier = new Promise((resolve) => {
    release = resolve;
  });
  let settings = { minutes: 50, range: 'day', phrase: 'old' };
  const applied = [],
    calls = [];
  const write = settingWriter(
    (s) => applied.push({ ...s }),
    () => assert.fail('unexpected failure'),
  );
  const first = write(async () => {
    calls.push('minutes');
    await barrier;
    settings = { ...settings, minutes: 25 };
    return settings;
  });
  const second = write(async () => {
    calls.push('range');
    settings = { ...settings, range: 'week' };
    return settings;
  });
  const third = write(async () => {
    calls.push('phrase');
    settings = { ...settings, phrase: 'new' };
    return settings;
  });
  await Promise.resolve();
  assert.deepEqual(calls, ['minutes']);
  release();
  await Promise.all([first, second, third]);
  assert.deepEqual(calls, ['minutes', 'range', 'phrase']);
  assert.deepEqual(applied.at(-1), { minutes: 25, range: 'week', phrase: 'new' });
});
test('a failed write reports an error without blocking the next change', async () => {
  const applied = [];
  let failures = 0;
  const write = settingWriter(
    (s) => applied.push(s),
    () => failures++,
  );
  await Promise.all([
    write(async () => {
      throw Error('disk unavailable');
    }),
    write(async () => 'recovered'),
  ]);
  assert.equal(failures, 1);
  assert.deepEqual(applied, ['recovered']);
});
