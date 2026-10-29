import { seriesFor } from './catalog.ts';
import type { SeriesDefinition, RestPalette } from './types';

/** Palette mapping belongs to the host, so visuals never mutate parent DOM or global colors. */
export function paletteStyle(palette: RestPalette): Record<string, string> {
  return {
    '--ink': palette.ink,
    '--muted': palette.muted,
    '--faint': palette.muted,
    '--selected': palette.selected,
    '--hover': palette.hover,
    '--line-strong': palette.hover,
    '--line': palette.line,
  };
}
export function visualThemeStyle(
  theme: SeriesDefinition,
  dark: boolean,
  resting: boolean,
): Record<string, string> | undefined {
  const visual = resting ? theme.rest : theme.focus;
  return visual ? paletteStyle(dark ? visual.dark : visual.light) : undefined;
}
export function seriesThemeStyle(id: string, dark: boolean, resting: boolean) {
  return visualThemeStyle(seriesFor(id), dark, resting);
}
/** Compatibility helper for callers rendering a rest-only preview. */
export function restThemeStyle(id: string, dark: boolean): Record<string, string> {
  return seriesThemeStyle(id, dark, true)!;
}

export function remainingRestFraction(duration: number, target: number): number {
  if (!Number.isFinite(duration) || !Number.isFinite(target) || target <= 0) return 0;
  return Math.max(0, Math.min(1, 1 - duration / target));
}
