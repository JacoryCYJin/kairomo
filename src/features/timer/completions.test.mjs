import assert from 'node:assert/strict';
import test from 'node:test';
import { todayCompletions } from './completions.ts';
const at = (day, hour = 0) => new Date(2026, 9, day, hour).getTime();
const session = (id, end, kind = 'focus') => ({
  id,
  kind,
  title: '',
  ended_at_ms: end,
  started_at_ms: (end ?? at(28)) - 60000,
  duration_ms: 60000,
});
test('all today completions survive, beyond the former six and history jar limit', () => {
  const records = Array.from({ length: 150 }, (_, i) => session(String(i), at(28, 10) + i));
  const before = JSON.stringify(records);
  assert.equal(todayCompletions(records, new Date(at(28, 12))).length, 150);
  assert.deepEqual(todayCompletions(records.toReversed(), new Date(at(28, 12))), records);
  assert.equal(JSON.stringify(records), before);
});
test('local completion date owns beans, midnight resets only display', () => {
  const overnight = session('overnight', at(28));
  const records = [
    session('old', at(27, 23)),
    overnight,
    session('break', at(28, 1), 'break'),
    session('ongoing', null),
  ];
  assert.deepEqual(
    todayCompletions(records, new Date(at(28, 2))).map((s) => s.id),
    ['overnight'],
  );
  assert.equal(todayCompletions(records, new Date(at(29))).length, 0);
  assert.equal(records.length, 4);
});
test('deleted records disappear and saved legacy short records retain their reward', () => {
  const record = { ...session('legacy', at(28, 3)), duration_ms: 1000 };
  assert.equal(todayCompletions([record], new Date(at(28, 4))).length, 1);
  assert.equal(todayCompletions([], new Date(at(28, 4))).length, 0);
});
