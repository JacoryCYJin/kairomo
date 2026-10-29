import manifest from '../../../theme-kits/catalog.json' with { type: 'json' };
import type { SeriesDefinition, SeriesId } from './types';
export const defaultSeriesId = manifest.default as SeriesId;
export const availableSeries: readonly SeriesDefinition[] = Object.entries(manifest.themes).map(
  ([id, theme]) => ({ ...theme, id: id as SeriesId }),
);
export function seriesFor(id: string | undefined): SeriesDefinition {
  return (
    availableSeries.find((series) => series.id === id) ??
    availableSeries.find((series) => series.id === defaultSeriesId)!
  );
}
