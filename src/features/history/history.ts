import { localWeekStart } from '../../shared/localCalendar.ts';
import { isCompletedFocus } from '../../shared/completions.ts';
import type { Session } from '../../shared/ipc/types';

export type HistoryRange = 'day' | 'week' | 'month' | 'year' | 'all';
export type HistoryFilter = HistoryRange | 'custom';
export interface Period {
  start: number;
  end: number;
}
export interface HistoryBucket extends Period {
  date: Date;
  unit: 'hour' | 'day' | 'month' | 'year';
  value: number;
}

export function localDay(date: Date): Date {
  return new Date(date.getFullYear(), date.getMonth(), date.getDate());
}
export function dateInput(date: Date): string {
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`;
}
export function parseDateInput(value: string): Date | null {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(value)) return null;
  const [year, month, day] = value.split('-').map(Number);
  const date = new Date(year, month - 1, day);
  return dateInput(date) === value ? date : null;
}
export function periodFor(range: HistoryRange, anchor: Date): Period {
  let start = localDay(anchor);
  if (range === 'all') return { start: -Infinity, end: Infinity };
  if (range === 'week') start = localWeekStart(anchor);
  if (range === 'month') start.setDate(1);
  if (range === 'year') start.setMonth(0, 1);
  const end = new Date(start);
  if (range === 'year') end.setFullYear(end.getFullYear() + 1);
  else if (range === 'month') end.setMonth(end.getMonth() + 1);
  else end.setDate(end.getDate() + (range === 'week' ? 7 : 1));
  return { start: start.getTime(), end: end.getTime() };
}
export function shiftPeriod(range: HistoryRange, anchor: Date, direction: number): Date {
  const start = new Date(periodFor(range === 'all' ? 'day' : range, anchor).start);
  if (range === 'year') start.setFullYear(start.getFullYear() + direction);
  else if (range === 'month') start.setMonth(start.getMonth() + direction);
  else start.setDate(start.getDate() + direction * (range === 'week' ? 7 : 1));
  return start;
}

// Use saved effective durations, including corrections; wall-clock pauses never count.
export function durationInPeriod(session: Session, period: Period, checkpoint: number): number {
  return session.intervals.reduce((sum, interval) => {
    const end = interval.ended_at_ms ?? checkpoint;
    if (end <= interval.started_at_ms) {
      return (
        sum +
        (interval.started_at_ms >= period.start && interval.started_at_ms < period.end
          ? interval.duration_ms
          : 0)
      );
    }
    const overlap = Math.max(
      0,
      Math.min(end, period.end) - Math.max(interval.started_at_ms, period.start),
    );
    return sum + (interval.duration_ms * overlap) / (end - interval.started_at_ms);
  }, 0);
}
export function inPeriod(session: Session, period: Period, checkpoint: number): boolean {
  if (period.start === -Infinity && period.end === Infinity) return true;
  if (session.duration_ms === 0)
    return session.started_at_ms >= period.start && session.started_at_ms < period.end;
  return durationInPeriod(session, period, checkpoint) > 0;
}
export function recordStartInPeriod(session: Session, period: Period, checkpoint: number): number {
  if (period.start === -Infinity) return session.started_at_ms;
  const first = session.intervals.reduce((earliest, interval) => {
    const end = interval.ended_at_ms ?? checkpoint;
    if (interval.duration_ms <= 0 || end <= period.start || interval.started_at_ms >= period.end)
      return earliest;
    return Math.min(earliest, Math.max(interval.started_at_ms, period.start));
  }, Infinity);
  return Number.isFinite(first) ? first : Math.max(session.started_at_ms, period.start);
}

export function historyRecords(sessions: Session[], period: Period, checkpoint: number): Session[] {
  return sessions
    .filter(
      (session) =>
        inPeriod(session, period, checkpoint) ||
        (isCompletedFocus(session) &&
          session.ended_at_ms! >= period.start &&
          session.ended_at_ms! < period.end),
    )
    .sort((a, b) => b.started_at_ms - a.started_at_ms || b.id.localeCompare(a.id));
}
export function totalInPeriod(
  sessions: Session[],
  kind: Session['kind'],
  period: Period,
  checkpoint: number,
): number {
  return sessions
    .filter((session) => session.kind === kind)
    .reduce((sum, session) => sum + durationInPeriod(session, period, checkpoint), 0);
}

export function historyBuckets(
  sessions: Session[],
  range: HistoryRange,
  anchor: Date,
  checkpoint: number,
): HistoryBucket[] {
  const period = periodFor(range, anchor);
  let cursor = new Date(period.start);
  let limit = period.end;
  let unit: HistoryBucket['unit'] = range === 'day' ? 'hour' : range === 'year' ? 'month' : 'day';
  if (range === 'all') {
    if (!sessions.length) return [];
    const first = sessions.reduce((earliest, s) => Math.min(earliest, s.started_at_ms), Infinity);
    const last = sessions.reduce(
      (latest, s) => Math.max(latest, s.ended_at_ms ?? checkpoint),
      -Infinity,
    );
    cursor = new Date(first);
    cursor = new Date(cursor.getFullYear(), cursor.getMonth(), 1);
    const latest = new Date(last);
    unit =
      (latest.getFullYear() - cursor.getFullYear()) * 12 + latest.getMonth() - cursor.getMonth() <
      24
        ? 'month'
        : 'year';
    if (unit === 'year') cursor = new Date(cursor.getFullYear(), 0, 1);
    limit =
      unit === 'year'
        ? new Date(latest.getFullYear() + 1, 0, 1).getTime()
        : new Date(latest.getFullYear(), latest.getMonth() + 1, 1).getTime();
  }
  const buckets: HistoryBucket[] = [];
  while (cursor.getTime() < limit) {
    const end = new Date(cursor);
    if (unit === 'hour') end.setTime(end.getTime() + 3600000);
    else if (unit === 'day') end.setDate(end.getDate() + 1);
    else if (unit === 'month') end.setMonth(end.getMonth() + 1);
    else end.setFullYear(end.getFullYear() + 1);
    const bucket = { start: cursor.getTime(), end: Math.min(end.getTime(), limit) };
    buckets.push({
      ...bucket,
      date: new Date(cursor),
      unit,
      value: totalInPeriod(sessions, 'focus', bucket, checkpoint),
    });
    cursor = end;
  }
  return buckets;
}

/** Inclusive local dates, represented internally as a half-open interval. */
export function customPeriod(from: string, to: string): Period | null {
  const start = parseDateInput(from),
    last = parseDateInput(to);
  if (!start || !last || start > last) return null;
  const end = new Date(last);
  end.setDate(end.getDate() + 1);
  return { start: start.getTime(), end: end.getTime() };
}
export function monthGrid(month: Date): Date[] {
  const first = new Date(month.getFullYear(), month.getMonth(), 1);
  first.setDate(1 - ((first.getDay() + 6) % 7));
  return Array.from(
    { length: 42 },
    (_, index) => new Date(first.getFullYear(), first.getMonth(), first.getDate() + index),
  );
}
/** Each completed focus earns exactly one bean, on its local completion date. */
export function earnedBeans(records: readonly Session[], period: Period): Session[] {
  return records
    .filter(
      (s) => isCompletedFocus(s) && s.ended_at_ms! >= period.start && s.ended_at_ms! < period.end,
    )
    .sort((a, b) => a.ended_at_ms! - b.ended_at_ms! || a.id.localeCompare(b.id));
}

/** Portion of one record inside a local day, as percentages for the 24-hour timeline ruler. */
export function daySpan(
  startedAtMs: number,
  endedAtMs: number | null,
  day: Date,
): { left: number; width: number } {
  const dayStart = new Date(day.getFullYear(), day.getMonth(), day.getDate()).getTime();
  const dayEnd = dayStart + 86400000;
  const from = Math.min(Math.max(startedAtMs, dayStart), dayEnd);
  const to = Math.min(Math.max(endedAtMs ?? startedAtMs, from), dayEnd);
  return { left: ((from - dayStart) / 86400000) * 100, width: ((to - from) / 86400000) * 100 };
}

/** Continuous timeline zoom bounds, in visible minutes. */
export const TIMELINE_MIN_SPAN = 30;
export const TIMELINE_MAX_SPAN = 24 * 60;

/** Equal slider moves multiply the visible span, so zooming feels even at any scale. */
export function zoomSpan(position: number): number {
  const step = Math.min(1, Math.max(0, position));
  return TIMELINE_MAX_SPAN * Math.pow(TIMELINE_MIN_SPAN / TIMELINE_MAX_SPAN, step);
}

/** Inverse of zoomSpan, used to place the slider thumb. */
export function zoomPosition(spanMinutes: number): number {
  const span = Math.min(TIMELINE_MAX_SPAN, Math.max(TIMELINE_MIN_SPAN, spanMinutes));
  return Math.max(
    0,
    Math.log(span / TIMELINE_MAX_SPAN) / Math.log(TIMELINE_MIN_SPAN / TIMELINE_MAX_SPAN),
  );
}

/** How many times wider than the viewport the full day becomes at this span. */
export function timelineScale(spanMinutes: number): number {
  return TIMELINE_MAX_SPAN / spanMinutes;
}

/** Pick a clock step that keeps roughly six labels inside the visible window. */
export function timelineStepMs(spanMs: number): number {
  const minutes = spanMs / 60000;
  const steps = [5, 10, 15, 30, 60, 120, 180, 360];
  return (steps.find((step) => minutes / step <= 6) ?? 720) * 60000;
}

/** Ticks aligned to the local day, limited to the visible window. */
export function timelineTicks(
  dayStartMs: number,
  viewStartMs: number,
  viewEndMs: number,
  stepMs: number,
): number[] {
  if (stepMs <= 0 || viewEndMs <= viewStartMs) return [];
  const first = dayStartMs + Math.ceil((viewStartMs - dayStartMs) / stepMs) * stepMs;
  const ticks: number[] = [];
  for (let tick = first; tick < viewEndMs; tick += stepMs) ticks.push(tick);
  return ticks;
}

/** Visible part of a day for the current zoom and scroll offset. */
export function timelineWindow(
  dayStartMs: number,
  dayMs: number,
  scale: number,
  scrollLeft: number,
  viewportWidth: number,
): { start: number; end: number } {
  if (scale <= 0 || viewportWidth <= 0) return { start: dayStartMs, end: dayStartMs + dayMs };
  const contentWidth = viewportWidth * scale;
  const offset = Math.max(0, Math.min(scrollLeft, contentWidth - viewportWidth));
  const start = dayStartMs + (offset / contentWidth) * dayMs;
  return { start, end: Math.min(start + dayMs / scale, dayStartMs + dayMs) };
}

/** Scroll offset that keeps an anchor time at the center of the viewport. */
export function timelineScrollLeft(
  dayStartMs: number,
  dayMs: number,
  scale: number,
  viewportWidth: number,
  anchorMs: number,
): number {
  const contentWidth = viewportWidth * scale;
  if (contentWidth <= viewportWidth) return 0;
  const offset = ((anchorMs - dayStartMs) / dayMs) * contentWidth - viewportWidth / 2;
  return Math.max(0, Math.min(offset, contentWidth - viewportWidth));
}
