import test from 'node:test';
import assert from 'node:assert/strict';
import {
  localDateTime,
  editTimestamp,
  correctionDuration,
  overlappingRecord,
  effectiveMinutes,
} from './recordCorrection.ts';
const record = {
  id: 'one',
  kind: 'focus',
  started_at_ms: 1000,
  ended_at_ms: 11000,
  duration_ms: 6000,
  intervals: [
    { started_at_ms: 1000, ended_at_ms: 4000, duration_ms: 3000 },
    { started_at_ms: 8000, ended_at_ms: 11000, duration_ms: 3000 },
  ],
};
test('boundary edits preserve the inner pause and trim or extend effective portions', () => {
  assert.equal(correctionDuration(record, 2000, 12000), 6000);
  assert.equal(correctionDuration(record, 2000, 10000), 4000);
  assert.equal(correctionDuration(record, 5000, 7000), 0);
  assert.equal(correctionDuration(record, 0, 12000), 8000);
  const legacy = {
    ...record,
    duration_ms: 4234,
    intervals: [{ ...record.intervals[0], duration_ms: 1234 }, record.intervals[1]],
  };
  assert.equal(correctionDuration(legacy, 1000, 11000), 4234);
  assert.equal(correctionDuration(legacy, 2000, 11000), 3823);
});
test('local input roundtrips unchanged milliseconds, accepts cross-day changes and rejects normalized impossible dates', () => {
  const original = new Date(2026, 9, 31, 23, 59, 59, 501).getTime();
  assert.equal(editTimestamp(localDateTime(original), original), original);
  assert.equal(editTimestamp('2026-11-01T00:10', original), new Date(2026, 10, 1, 0, 10).getTime());
  assert.equal(editTimestamp('2026-02-30T10:00', original), null);
  assert.equal(editTimestamp('', original), null);
  assert.equal(localDateTime(original), '2026-10-31T23:59');
  assert.equal(editTimestamp('2026-11-01T00:10:01', original), null);
  assert.equal(effectiveMinutes(61501), 1);
  assert.equal(effectiveMinutes(104764), 2);
  assert.equal(effectiveMinutes(59999), null);
});
test('overlap ignores filters and self, includes containment and pauses, but permits adjacent bounds', () => {
  assert.equal(overlappingRecord([record], 'other', 4000, 8000), record);
  assert.equal(overlappingRecord([record], 'other', 11000, 12000), undefined);
  assert.equal(overlappingRecord([record], 'one', 1000, 11000), undefined);
  assert.equal(overlappingRecord([record], 'other', 0, 12000), record);
});
