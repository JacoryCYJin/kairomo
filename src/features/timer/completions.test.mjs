import assert from 'node:assert/strict';
import test from 'node:test';
import { completionDots } from './completions.ts';

function session(id, minutes, ended, kind = 'focus') {
  return {
    id,
    kind,
    title: id,
    started_at_ms: ended - minutes * 60000,
    ended_at_ms: ended,
    target_ms: 50 * 60000,
    duration_ms: minutes * 60000,
    intervals: [],
    corrected: false,
  };
}
const ids = (records) => completionDots(records).map((record) => record.id);

test('only finished focus sessions become dots, including saved short sessions', () => {
  const current = { ...session('active', 50, 3000), ended_at_ms: null };
  assert.deepEqual(
    ids([session('short', 0.1, 1000), session('rest', 10, 2000, 'break'), current]),
    ['short'],
  );
});

test('a new shorter completion still replaces the shortest, not the oldest or newest', () => {
  const records = [6, 1, 5, 2, 4, 3].map((minutes, index) =>
    session(String(index), minutes, (index + 1) * 1_000_000),
  );
  records.push(session('shorter', 0.5, 7_000_000));
  assert.deepEqual(ids(records), ['0', '2', '3', '4', '5', 'shorter']);
  records.push(session('shortest', 0.25, 8_000_000));
  assert.deepEqual(ids(records), ['0', '2', '3', '4', '5', 'shortest']);
});

test('equal shortest durations replace the earliest completed one', () => {
  const records = [1, 2, 1, 3, 4, 5].map((minutes, index) =>
    session(String(index), minutes, (index + 1) * 1_000_000),
  );
  records.push(session('new', 8, 7_000_000));
  assert.deepEqual(ids(records), ['1', '2', '3', '4', '5', 'new']);
});

test('reopening from reverse-ordered history reconstructs the same dots without mutating input', () => {
  const records = Array.from({ length: 12 }, (_, index) =>
    session(String(index), index + 1, (index + 1) * 1_000_000),
  );
  const reverse = records.slice().reverse();
  const before = JSON.stringify(reverse);
  assert.deepEqual(ids(reverse), ['6', '7', '8', '9', '10', '11']);
  assert.deepEqual(ids(JSON.parse(JSON.stringify(reverse))), ids(records));
  assert.equal(JSON.stringify(reverse), before);
});

test('history corrections and deletions are reflected when dots are rebuilt', () => {
  const records = Array.from({ length: 7 }, (_, index) =>
    session(String(index), index + 1, (index + 1) * 1_000_000),
  );
  assert.deepEqual(ids(records), ['1', '2', '3', '4', '5', '6']);
  records[0] = { ...records[0], duration_ms: 120 * 60000, corrected: true };
  assert.deepEqual(ids(records), ['0', '2', '3', '4', '5', '6']);
  assert.deepEqual(ids(records.filter((record) => record.id !== '6')), [
    '0',
    '1',
    '2',
    '3',
    '4',
    '5',
  ]);
});
