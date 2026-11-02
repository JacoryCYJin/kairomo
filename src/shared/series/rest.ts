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
function mixHex(from: string, to: string, ratio: number) {
  const parse = (hex: string) =>
    [0, 2, 4].map((index) => parseInt(hex.slice(1).slice(index, index + 2), 16));
  const [r1, g1, b1] = parse(from);
  const [r2, g2, b2] = parse(to);
  return (
    '#' +
    [r1! + (r2! - r1!) * ratio, g1! + (g2! - g1!) * ratio, b1! + (b2! - b1!) * ratio]
      .map((channel) => Math.round(channel).toString(16).padStart(2, '0'))
      .join('')
  );
}

export function dotColor(id: string, dark: boolean, intensity: number) {
  const palette = seriesFor(id).rest[dark ? 'dark' : 'light'];
  const layers = dark ? [...palette.layers].reverse() : palette.layers;
  if (!layers.length) return palette.ink;
  if (layers.length === 1) return layers[0]!;
  const position = Math.min(1, Math.max(0, intensity)) * (layers.length - 1);
  const lower = Math.floor(position);
  const upper = Math.min(layers.length - 1, lower + 1);
  return mixHex(layers[lower] ?? palette.ink, layers[upper] ?? palette.ink, position - lower);
}

export function dialogConfirmStyle(id: string, dark: boolean): Record<string, string> {
  const palette = seriesFor(id).dialogConfirm[dark ? 'dark' : 'light'];
  return {
    '--dialog-confirm': palette.background,
    '--dialog-confirm-ink': palette.ink,
  };
}
/** Compatibility helper for callers rendering a rest-only preview. */
export function restThemeStyle(id: string, dark: boolean): Record<string, string> {
  return seriesThemeStyle(id, dark, true)!;
}

export function remainingRestFraction(duration: number, target: number): number {
  if (!Number.isFinite(duration) || !Number.isFinite(target) || target <= 0) return 0;
  return Math.max(0, Math.min(1, 1 - duration / target));
}
