import assert from 'node:assert/strict';
import test from 'node:test';
import { recordsForTasks, matchesTasks, uncategorizedTask } from './taskFilter.ts';
const records = [
  { id: 'a', task_id: 'math' },
  { id: 'b', task_id: 'physics' },
  { id: 'old' },
  { id: 'empty', task_id: null },
];
test('multiple task filters form a union without duplicating sessions', () => {
  assert.deepEqual(
    recordsForTasks(records, ['math', 'physics', 'math']).map((s) => s.id),
    ['a', 'b'],
  );
  assert.deepEqual(
    recordsForTasks(records, ['math', uncategorizedTask]).map((s) => s.id),
    ['a', 'old', 'empty'],
  );
});
test('removing the last task filter restores all records, including legacy unclassified ones', () => {
  assert.deepEqual(recordsForTasks(records, []), records);
  assert.equal(matchesTasks({ task_id: 'physics' }, ['math']), false);
  assert.equal(matchesTasks(null, ['math']), false);
});
