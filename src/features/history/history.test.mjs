import assert from 'node:assert/strict';
import test from 'node:test';
import {
  dateInput,
  recordStartInPeriod,
  parseDateInput,
  periodFor,
  shiftPeriod,
  durationInPeriod,
  historyRecords,
  totalInPeriod,
  historyBuckets,
} from './history.ts';

const at = (y, m, d, h = 0, min = 0) => new Date(y, m - 1, d, h, min).getTime();
function session(id, start, end, kind = 'focus', intervals) {
  const parts = intervals ?? [{ started_at_ms: start, ended_at_ms: end, duration_ms: end - start }];
  return {
    id,
    kind,
    title: id,
    started_at_ms: start,
    ended_at_ms: end,
    target_ms: 3000000,
    duration_ms: parts.reduce((n, i) => n + i.duration_ms, 0),
    intervals: parts,
    corrected: false,
  };
}

test('local date input rejects impossible dates and roundtrips without UTC shifts', () => {
  assert.equal(dateInput(parseDateInput('2026-01-01')), '2026-01-01');
  assert.equal(parseDateInput('2026-02-30'), null);
  assert.equal(parseDateInput('2026-2-3'), null);
  assert.equal(dateInput(parseDateInput('2024-02-29')), '2024-02-29');
});

test('weeks start on Monday and span month/year boundaries', () => {
  assert.deepEqual(periodFor('week', new Date(2027, 0, 3)), {
    start: at(2026, 12, 28),
    end: at(2027, 1, 4),
  });
  assert.deepEqual(periodFor('week', new Date(2027, 0, 4)), {
    start: at(2027, 1, 4),
    end: at(2027, 1, 11),
  });
});

test('month navigation from the 31st cannot skip February; leap month has 29 days', () => {
  assert.equal(dateInput(shiftPeriod('month', new Date(2026, 2, 31), -1)), '2026-02-01');
  assert.equal(dateInput(shiftPeriod('month', new Date(2026, 11, 31), 1)), '2027-01-01');
  assert.equal(historyBuckets([], 'month', new Date(2024, 1, 1), 0).length, 29);
});

test('a midnight-spanning session is split across dates, not lost or counted twice', () => {
  const s = session('overnight', at(2026, 10, 25, 23, 40), at(2026, 10, 26, 0, 20));
  const first = periodFor('day', new Date(2026, 9, 25));
  const second = periodFor('day', new Date(2026, 9, 26));
  assert.equal(durationInPeriod(s, first, 0), 20 * 60000);
  assert.equal(durationInPeriod(s, second, 0), 20 * 60000);
  assert.equal(historyRecords([s], first, 0).length, 1);
  assert.equal(historyRecords([s], second, 0).length, 1);
});

test('cross-month/year intervals and corrected durations retain their proportional allocation', () => {
  const s = session('corrected', at(2026, 12, 31, 23, 30), at(2027, 1, 1, 0, 30));
  s.duration_ms = 120 * 60000;
  s.intervals[0].duration_ms = s.duration_ms;
  s.corrected = true;
  assert.equal(durationInPeriod(s, periodFor('month', new Date(2026, 11, 1)), 0), 60 * 60000);
  assert.equal(durationInPeriod(s, periodFor('month', new Date(2027, 0, 1)), 0), 60 * 60000);
});

test('pauses do not contribute; a paused-only date has no effective session', () => {
  const s = session('paused', at(2026, 10, 24, 23), at(2026, 10, 26, 1), 'focus', [
    {
      started_at_ms: at(2026, 10, 24, 23),
      ended_at_ms: at(2026, 10, 24, 23, 30),
      duration_ms: 30 * 60000,
    },
    {
      started_at_ms: at(2026, 10, 26, 0, 30),
      ended_at_ms: at(2026, 10, 26, 1),
      duration_ms: 30 * 60000,
    },
  ]);
  assert.equal(historyRecords([s], periodFor('day', new Date(2026, 9, 25)), 0).length, 0);
  assert.equal(totalInPeriod([s], 'focus', periodFor('all', new Date()), 0), 60 * 60000);
});

test('running intervals use the authoritative checkpoint; focus and rest stay separate', () => {
  const start = at(2026, 10, 26, 10);
  const s = session('live', start, null, 'focus', [
    { started_at_ms: start, ended_at_ms: null, duration_ms: 600000 },
  ]);
  const rest = session('rest', start, start + 300000, 'break');
  const period = periodFor('day', new Date(start));
  assert.equal(totalInPeriod([s, rest], 'focus', period, start + 600000), 600000);
  assert.equal(totalInPeriod([s, rest], 'break', period, start + 600000), 300000);
});

test('period end is exclusive; zero-duration sessions remain accessible', () => {
  const boundary = at(2026, 10, 26);
  const s = session('zero', boundary, boundary);
  assert.equal(historyRecords([s], periodFor('day', new Date(2026, 9, 25)), 0).length, 0);
  assert.equal(historyRecords([s], periodFor('day', new Date(2026, 9, 26)), 0).length, 1);
});

test('all history includes old dates and more than a display batch without mutating source data', () => {
  const data = Array.from({ length: 125 }, (_, i) =>
    session(String(i), at(2020, 1, i + 1), at(2020, 1, i + 1, 1)),
  );
  const sorted = historyRecords(data, periodFor('all', new Date(2026, 9, 26)), 0);
  assert.equal(sorted.length, 125);
  assert.equal(sorted[0].id, '124');
  assert.equal(data[0].id, '0');
  assert.equal(totalInPeriod(data, 'focus', periodFor('all', new Date()), 0), 125 * 3600000);
});

test('all-history chart includes every year, while shorter history uses monthly buckets', () => {
  const old = session('old', at(2020, 1, 1), at(2020, 1, 1, 1));
  const recent = session('recent', at(2026, 10, 26), at(2026, 10, 26, 1));
  const years = historyBuckets([old, recent], 'all', new Date(), 0);
  assert.equal(years.length, 7);
  assert.ok(years.every((b) => b.unit === 'year'));
  assert.equal(
    years.reduce((sum, b) => sum + b.value, 0),
    7200000,
  );
  assert.equal(historyBuckets([recent], 'all', new Date(), 0)[0].unit, 'month');
  assert.deepEqual(historyBuckets([], 'all', new Date(), 0), []);
});

test('chart sum matches range total across pauses and day boundaries', () => {
  const s = session('night', at(2026, 10, 25, 23, 30), at(2026, 10, 26, 1, 30));
  for (const range of ['day', 'week', 'month', 'all']) {
    const anchor = new Date(2026, 9, 26);
    const buckets = historyBuckets([s], range, anchor, 0);
    assert.equal(
      buckets.reduce((sum, b) => sum + b.value, 0),
      totalInPeriod([s], 'focus', periodFor(range, anchor), 0),
    );
  }
});

test('local-day buckets respect daylight saving instead of assuming 24-hour days', () => {
  const spring = periodFor('day', new Date(2026, 2, 8));
  const fall = periodFor('day', new Date(2026, 10, 1));
  const springHours = historyBuckets([], 'day', new Date(2026, 2, 8), 0);
  const fallHours = historyBuckets([], 'day', new Date(2026, 10, 1), 0);
  assert.equal(
    springHours.reduce((sum, b) => sum + b.end - b.start, 0),
    spring.end - spring.start,
  );
  assert.equal(
    fallHours.reduce((sum, b) => sum + b.end - b.start, 0),
    fall.end - fall.start,
  );
  if (process.env.TZ === 'America/New_York') {
    assert.equal(springHours.length, 23);
    assert.equal(fallHours.length, 25);
  }
});

test('date grouping uses the first effective interval in the selected period after a long pause', () => {
  const s = session('pause', at(2026, 10, 24, 23), at(2026, 10, 28, 1), 'focus', [
    {
      started_at_ms: at(2026, 10, 24, 23),
      ended_at_ms: at(2026, 10, 24, 23, 30),
      duration_ms: 1800000,
    },
    {
      started_at_ms: at(2026, 10, 28, 0, 30),
      ended_at_ms: at(2026, 10, 28, 1),
      duration_ms: 1800000,
    },
  ]);
  assert.equal(
    recordStartInPeriod(s, periodFor('week', new Date(2026, 9, 26)), 0),
    at(2026, 10, 28, 0, 30),
  );
  assert.equal(recordStartInPeriod(s, periodFor('all', new Date()), 0), s.started_at_ms);
});

test('year selection includes leap day and uses the next local year boundary', () => {
  assert.deepEqual(periodFor('year', new Date(2024, 6, 9)), {
    start: at(2024, 1, 1),
    end: at(2025, 1, 1),
  });
  assert.equal(historyBuckets([], 'year', new Date(2024, 0, 1), 0).length, 12);
  assert.equal(dateInput(shiftPeriod('year', new Date(2024, 1, 29), 1)), '2025-01-01');
});

test('custom dates include the whole last day and reject reversed, missing and impossible dates', async () => {
  const { customPeriod } = await import('./history.ts');
  assert.deepEqual(customPeriod('2026-10-25', '2026-10-26'), {
    start: at(2026, 10, 25),
    end: at(2026, 10, 27),
  });
  assert.deepEqual(customPeriod('2026-03-08', '2026-03-08'), {
    start: at(2026, 3, 8),
    end: at(2026, 3, 9),
  });
  for (const [from, to] of [
    ['2026-02-30', '2026-03-01'],
    ['2026-10-27', '2026-10-26'],
    ['', '2026-10-26'],
  ])
    assert.equal(customPeriod(from, to), null);
  const s = session('last-minute', at(2026, 10, 26, 23, 58), at(2026, 10, 26, 23, 59));
  assert.equal(totalInPeriod([s], 'focus', customPeriod('2026-10-25', '2026-10-26'), 0), 60000);
});

test('one completed focus earns one bean on its completion day, independently of cross-day time', async () => {
  const { earnedBeans } = await import('./history.ts');
  const s = session('midnight', at(2026, 10, 25, 23, 40), at(2026, 10, 26, 0, 20));
  const first = periodFor('day', new Date(2026, 9, 25)),
    second = periodFor('day', new Date(2026, 9, 26));
  assert.equal(earnedBeans([s], first).length, 0);
  assert.equal(earnedBeans([s], second).length, 1);
  assert.equal(totalInPeriod([s], 'focus', first, 0), 20 * 60000);
  assert.equal(totalInPeriod([s], 'focus', second, 0), 20 * 60000);
  const midnight = session('exact-midnight', at(2026, 10, 25, 23, 59), at(2026, 10, 26));
  assert.equal(earnedBeans([midnight], second).length, 1);
  assert.equal(historyRecords([midnight], second, 0).length, 1);
});

test('beans exclude ongoing focus and rest, include short tasks, and never apply the six-bean summary cap', async () => {
  const { earnedBeans } = await import('./history.ts');
  const day = periodFor('day', new Date(2026, 9, 26));
  const many = Array.from({ length: 125 }, (_, i) =>
    session(String(i), at(2026, 10, 26, 1), at(2026, 10, 26, 1, 1)),
  );
  const ongoing = session('ongoing', at(2026, 10, 26), null, 'focus', []);
  const rest = session('rest', at(2026, 10, 26), at(2026, 10, 26, 0, 10), 'break');
  const data = [...many, ongoing, rest],
    before = [...data];
  assert.equal(earnedBeans(data, day).length, 125);
  assert.deepEqual(data, before);
  assert.equal(
    earnedBeans(
      data.filter((s) => s.id !== '5'),
      day,
    ).length,
    124,
  );
});

test('month grid starts Monday, handles year boundaries and leaves selection independent of browsing', async () => {
  const { monthGrid } = await import('./history.ts');
  const grid = monthGrid(new Date(2026, 0, 31));
  assert.equal(grid.length, 42);
  assert.equal(grid[0].getDay(), 1);
  assert.equal(dateInput(grid[0]), '2025-12-29');
  assert.equal(grid.filter((d) => d.getMonth() === 0).length, 31);
  assert.equal(monthGrid(new Date(2024, 1, 29)).filter((d) => d.getMonth() === 1).length, 29);
});
