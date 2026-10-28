/** Calendar boundaries use local dates so DST, month and year changes stay correct. */
export function localWeekStart(date: Date): Date {
  return new Date(date.getFullYear(), date.getMonth(), date.getDate() - ((date.getDay() + 6) % 7));
}
export function beanPeriod(range: 'day' | 'week', now: Date): { start: number; end: number } {
  const start =
    range === 'week'
      ? localWeekStart(now)
      : new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const end = new Date(
    start.getFullYear(),
    start.getMonth(),
    start.getDate() + (range === 'week' ? 7 : 1),
  );
  return { start: start.getTime(), end: end.getTime() };
}
