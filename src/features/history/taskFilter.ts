import type { Session } from '../../shared/ipc/types';
export const uncategorizedTask = '__uncategorized__';
export function matchesTasks(session: Session | null, selected: readonly string[]): boolean {
  return !selected.length || (!!session && selected.includes(session.task_id ?? uncategorizedTask));
}
export function recordsForTasks(
  records: readonly Session[],
  selected: readonly string[],
): Session[] {
  return records.filter((session) => matchesTasks(session, selected));
}
