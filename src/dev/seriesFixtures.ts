import type { Session } from '../shared/ipc/types';
export const previewCounts = [20, 40, 80, 110, 120] as const;
/** Synthetic sessions stay in memory; this module never uses IPC or user storage. */
export function seriesFixtures(count: number, now = Date.now()): Session[] {
  const minutes = [10, 25, 40, 50, 70];
  return Array.from({ length: Math.max(0, Math.min(500, Math.floor(count))) }, (_, index) => {
    const duration = minutes[index % minutes.length] * 60_000;
    const end = now - (count - index) * 90 * 60_000;
    return {
      id: `preview-${index}`,
      title: `Demo ${index + 1}`,
      kind: 'focus',
      started_at_ms: end - duration,
      ended_at_ms: end,
      duration_ms: duration,
      target_ms: 50 * 60_000,
      corrected: false,
      intervals: [{ started_at_ms: end - duration, ended_at_ms: end, duration_ms: duration }],
    };
  });
}
