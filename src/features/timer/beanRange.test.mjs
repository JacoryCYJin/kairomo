import assert from 'node:assert/strict';
import test from 'node:test';
import { beanPeriod, localWeekStart } from '../../shared/localCalendar.ts';
import { periodCompletions } from './completions.ts';
const at = (y, m, d, h = 0) => new Date(y, m - 1, d, h);
const record = (id, date, kind = 'focus') => ({
  id,
  kind,
  started_at_ms: date.getTime() - 60000,
  ended_at_ms: date.getTime(),
  duration_ms: 60000,
});
test('week starts Monday and crosses months and years using local dates', () => {
  for (const date of [at(2026, 10, 28), at(2027, 1, 1), at(2026, 11, 1)]) {
    const { start, end } = beanPeriod('week', date);
    assert.equal(new Date(start).getDay(), 1);
    assert.equal(new Date(end).getDay(), 1);
    assert.equal(start, localWeekStart(date).getTime());
    assert.ok(start <= date.getTime() && end > date.getTime());
  }
  assert.equal(beanPeriod('week', at(2027, 1, 1)).start, at(2026, 12, 28).getTime());
});
test('day versus week uses completion ownership with exclusive end and never changes input', () => {
  const now = at(2026, 10, 28, 12),
    records = [
      record('last-week', at(2026, 10, 25, 23)),
      record('monday', at(2026, 10, 26)),
      record('today', at(2026, 10, 28)),
      record('next-week', at(2026, 11, 2)),
      record('break', at(2026, 10, 28), 'break'),
      { ...record('ongoing', now), ended_at_ms: null },
    ];
  const before = JSON.stringify(records);
  assert.deepEqual(
    periodCompletions(records, 'day', now).map((r) => r.id),
    ['today'],
  );
  assert.deepEqual(
    periodCompletions(records, 'week', now).map((r) => r.id),
    ['monday', 'today'],
  );
  assert.equal(JSON.stringify(records), before);
  assert.equal(periodCompletions(records, 'week', at(2026, 11, 2)).length, 1);
});
