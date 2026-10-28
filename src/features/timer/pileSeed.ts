import { beanPeriod } from '../../shared/localCalendar.ts';
import type { BeanRange } from '../../shared/ipc/types';
/** Use local calendar identity, not UTC timestamps or a new random value on each open. */
export function periodPileSeed(base: string, range: BeanRange, now: Date): string {
  const start = new Date(beanPeriod(range, now).start);
  const day = [start.getFullYear(), start.getMonth() + 1, start.getDate()].join('-');
  return JSON.stringify(['coffee-pile-v2', base, range, day]);
}
