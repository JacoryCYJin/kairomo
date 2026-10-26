import type { SeriesDefinition, SeriesName } from '../types';
const minute = 60_000;
export const coffeeSeries = {
  id: 'coffee',
  name: { 'zh-CN': '咖啡豆', 'en-US': 'Coffee beans' },
  summary: { limit: 6, replace: 'shortest' },
  collection: { insideCapacity: 80, displayLimit: 120 },
  // Product thresholds adapted from the City → Vienna development segment, anchored at 60 min.
  tiers: [
    { from: 0, name: { 'zh-CN': '浅烘', 'en-US': 'Light' }, light: '#D8B89B', dark: '#E1C6AA' },
    {
      from: 16 * minute,
      name: { 'zh-CN': '中浅烘', 'en-US': 'Medium-light' },
      light: '#C49A76',
      dark: '#D4B18D',
    },
    {
      from: 32 * minute,
      name: { 'zh-CN': '中烘', 'en-US': 'Medium' },
      light: '#AD7C56',
      dark: '#C49972',
    },
    {
      from: 44 * minute,
      name: { 'zh-CN': '中深烘', 'en-US': 'Medium-dark' },
      light: '#8B5B3B',
      dark: '#B2825B',
    },
    {
      from: 60 * minute,
      name: { 'zh-CN': '深烘', 'en-US': 'Dark' },
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
