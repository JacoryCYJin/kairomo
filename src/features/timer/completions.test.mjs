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
const ids = (records, now = new Date(0)) =>
  completionDots(records, 'coffee', now).map((record) => record.id);

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

const at = (day, hour = 0, minute = 0) => new Date(2026, 9, day, hour, minute).getTime();

test('only completions within the current local day appear, including exact midnight', () => {
  const records = [
    session('yesterday', 25, at(26, 23, 59)),
    session('midnight', 25, at(27)),
    session('today', 40, at(27, 12)),
    session('tomorrow', 50, at(28)),
  ];
  assert.deepEqual(ids(records, new Date(at(27, 15))), ['midnight', 'today']);
});

test('a new day clears yesterday beans without deleting any records', () => {
  const records = Array.from({ length: 8 }, (_, index) =>
    session(String(index), 25, at(26, 10, index)),
  );
  assert.equal(ids(records, new Date(at(26, 23, 59))).length, 6);
  assert.deepEqual(ids(records, new Date(at(27))), []);
  assert.equal(records.length, 8);
});

test('a focus spanning midnight earns its bean on the day it finishes', () => {
  const record = session('overnight', 40, at(27, 0, 20));
  assert.deepEqual(ids([record], new Date(at(26, 23, 59))), []);
  assert.deepEqual(ids([record], new Date(at(27, 1))), ['overnight']);
});

test('today excludes rest and ongoing focus and applies the six-bean replacement independently', () => {
  const records = [
    session('old-long', 120, at(26, 22)),
    ...Array.from({ length: 7 }, (_, index) =>
      session(String(index), index + 1, at(27, 10, index)),
    ),
    session('break', 20, at(27, 12), 'break'),
    { ...session('active', 50, at(27, 13)), ended_at_ms: null },
  ];
  assert.deepEqual(ids(records, new Date(at(27, 14))), ['1', '2', '3', '4', '5', '6']);
});
