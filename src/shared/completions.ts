import type { Session } from './ipc/types';
export function isCompletedFocus(record: Session): boolean {
  return record.kind === 'focus' && record.ended_at_ms !== null;
}
