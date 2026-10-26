import { coffeeSeries } from './coffee/definition.ts';
import type { SeriesDefinition, SeriesId } from './types';
export const availableSeries: readonly SeriesDefinition[] = [coffeeSeries];
export function seriesFor(id: string | undefined): SeriesDefinition {
  return availableSeries.find((series) => series.id === id) ?? coffeeSeries;
}
export const defaultSeriesId: SeriesId = 'coffee';
