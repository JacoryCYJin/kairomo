import { beanPeriod } from '../../shared/localCalendar.ts';
import type { BeanRange, Session } from '../../shared/ipc/types';
import { isCompletedFocus } from '../../shared/completions.ts';
/** Completion date owns the bean. Changing days never changes the records. */
export function periodCompletions(
  records: readonly Session[],
  range: BeanRange,
  now: Date = new Date(),
): Session[] {
  const { start, end } = beanPeriod(range, now);
  return records
    .filter((s) => isCompletedFocus(s) && s.ended_at_ms! >= start && s.ended_at_ms! < end)
    .sort(
      (a, b) =>
        a.ended_at_ms! - b.ended_at_ms! ||
        a.started_at_ms - b.started_at_ms ||
        a.id.localeCompare(b.id),
    );
}

export function todayCompletions(records: readonly Session[], now = new Date()): Session[] {
  return periodCompletions(records, 'day', now);
}
