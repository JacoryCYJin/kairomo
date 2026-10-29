import type { SeriesDefinition, SeriesName } from '../types';
import manifest from '../../../../theme-kits/catalog.json' with { type: 'json' };
const metadata = manifest.themes.coffee;
const minute = 60_000;
const coffeeRest = {
  light: { ...metadata.rest.light, waves: metadata.rest.light.layers },
  dark: { ...metadata.rest.dark, waves: metadata.rest.dark.layers },
  waves: [
    { amplitude: 6, period: 9 },
    { amplitude: 9, period: 7 },
    { amplitude: 12, period: 5 },
  ],
};
export const coffeeSeries = {
  id: 'coffee',
  name: metadata.name,
  collection: { insideCapacity: 80, displayLimit: 120 },
  rest: coffeeRest,
  // Product thresholds adapted from the City → Vienna development segment, anchored at 60 min.
  tiers: [
    {
      from: 0,
      name: { 'zh-CN': '浅烘', 'zh-TW': '淺焙', 'en-US': 'Light', 'ja-JP': '浅煎り' },
      light: '#D8B89B',
      dark: '#E1C6AA',
    },
    {
      from: 16 * minute,
      name: { 'zh-CN': '中浅烘', 'zh-TW': '中淺焙', 'en-US': 'Medium-light', 'ja-JP': '中浅煎り' },
      light: '#C49A76',
      dark: '#D4B18D',
    },
    {
      from: 32 * minute,
      name: { 'zh-CN': '中烘', 'zh-TW': '中焙', 'en-US': 'Medium', 'ja-JP': '中煎り' },
      light: '#AD7C56',
      dark: '#C49972',
    },
    {
      from: 44 * minute,
      name: { 'zh-CN': '中深烘', 'zh-TW': '中深焙', 'en-US': 'Medium-dark', 'ja-JP': '中深煎り' },
      light: '#8B5B3B',
      dark: '#B2825B',
    },
    {
      from: 60 * minute,
      name: { 'zh-CN': '深烘', 'zh-TW': '深焙', 'en-US': 'Dark', 'ja-JP': '深煎り' },
      light: '#6B4028',
      dark: '#A36F4A',
    },
  ] satisfies { from: number; name: SeriesName; light: string; dark: string }[],
} as const satisfies SeriesDefinition & Record<string, unknown>;
export function coffeeTier(duration: number) {
  const value = Number.isFinite(duration) ? Math.max(0, duration) : 0;
  for (let i = coffeeSeries.tiers.length - 1; i >= 0; i--)
    if (value >= coffeeSeries.tiers[i].from) return coffeeSeries.tiers[i];
  return coffeeSeries.tiers[0];
}
