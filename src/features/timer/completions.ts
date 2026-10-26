import type { Session } from '../../shared/ipc/types';

import { seriesFor } from '../../shared/series/catalog.ts';
import type { SeriesId } from '../../shared/series/types';
import { isCompletedFocus } from '../../shared/completions.ts';

/** Replay completion order, replacing the shortest displayed session each time. */
export function completionDots(
  records: readonly Session[],
  seriesId: SeriesId = 'coffee',
): Session[] {
  const capacity = seriesFor(seriesId).summary.limit;
  const completed = records
    .filter(isCompletedFocus)
    .sort(
      (a, b) =>
        a.ended_at_ms! - b.ended_at_ms! ||
        a.started_at_ms - b.started_at_ms ||
        (a.id < b.id ? -1 : a.id > b.id ? 1 : 0),
    );
  const dots: Session[] = [];
  for (const record of completed) {
    if (dots.length === capacity) {
      let shortest = 0;
      for (let index = 1; index < dots.length; index++) {
        if (dots[index].duration_ms < dots[shortest].duration_ms) shortest = index;
      }
      dots.splice(shortest, 1);
    }
    dots.push(record);
  }
  return dots;
}
