import type { Session } from '../../shared/ipc/types';
export function localDateTime(ms: number): string {
  const d = new Date(ms);
  const pad = (n: number) => String(n).padStart(2, '0');
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
}
export function editTimestamp(value: string, original: number): number | null {
  if (value === localDateTime(original)) return original;
  if (!/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}$/.test(value)) return null;
  const ms = new Date(value + ':00').getTime();
  return Number.isFinite(ms) && localDateTime(ms) === value ? ms : null;
}
export function correctionDuration(record: Session, start: number, end: number): number {
  if (start === record.started_at_ms && end === record.ended_at_ms) return record.duration_ms;
  const oldEnd = record.ended_at_ms!;
  let total =
    Math.max(0, Math.min(end, record.started_at_ms) - start) +
    Math.max(0, end - Math.max(start, oldEnd));
  for (const interval of record.intervals) {
    const stop = interval.ended_at_ms!;
    const a = Math.max(start, interval.started_at_ms),
      b = Math.min(end, stop);
    if (a < b) {
      const width = BigInt(stop - interval.started_at_ms),
        effective = BigInt(interval.duration_ms);
      total += Number(
        (BigInt(b - interval.started_at_ms) * effective) / width -
          (BigInt(a - interval.started_at_ms) * effective) / width,
      );
    } else if (stop === interval.started_at_ms && start <= stop && stop < end)
      total += interval.duration_ms;
  }
  return total;
}
export function overlappingRecord(
  records: Session[],
  id: string,
  start: number,
  end: number,
): Session | undefined {
  return records.find(
    (s) =>
      s.id !== id &&
      s.kind === 'focus' &&
      s.ended_at_ms !== null &&
      start < s.ended_at_ms &&
      s.started_at_ms < end,
  );
}

export function effectiveMinutes(ms: number): number | null {
  return ms < 60000 ? null : Math.round(ms / 60000);
}
