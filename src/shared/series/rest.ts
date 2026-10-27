import { seriesFor } from './catalog.ts';

/** Local variables keep a rest theme out of other windows and focus sessions. */
export function restThemeStyle(seriesId: string, dark: boolean): Record<string, string> {
  const visual = seriesFor(seriesId).rest;
  const palette = dark ? visual.dark : visual.light;
  return {
    '--ink': palette.ink,
    '--muted': palette.muted,
    '--selected': palette.selected,
    '--hover': palette.hover,
    '--line-strong': palette.hover,
    '--line': palette.line,
  };
}

export function remainingRestFraction(duration: number, target: number): number {
  if (!Number.isFinite(duration) || !Number.isFinite(target) || target <= 0) return 0;
  return Math.max(0, Math.min(1, 1 - duration / target));
}
