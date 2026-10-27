import type { SeriesDefinition, SeriesName } from '../types';
const minute = 60_000;
export const coffeeSeries = {
  id: 'coffee',
  name: { 'zh-CN': '咖啡豆', 'en-US': 'Coffee beans', 'ja-JP': 'コーヒー豆' },
  summary: { limit: 6, replace: 'shortest' },
  collection: { insideCapacity: 80, displayLimit: 120 },
  rest: {
    kind: 'liquid',
    light: {
      ink: '#3D2518',
      muted: '#43291C',
      selected: '#EED5BA',
      hover: '#E3BF98',
      line: '#D1A57B',
      waves: ['#D8B18A', '#C89B72', '#BF8D63'],
    },
    dark: {
      ink: '#E7C7A6',
      muted: '#D8B897',
      selected: '#51392C',
      hover: '#634634',
      line: '#79563F',
      waves: ['#302119', '#3B2A21', '#493329'],
    },
    waves: [
      { amplitude: 6, period: 9 },
      { amplitude: 9, period: 7 },
      { amplitude: 12, period: 5 },
    ],
  },
  // Product thresholds adapted from the City → Vienna development segment, anchored at 60 min.
  tiers: [
    {
      from: 0,
      name: { 'zh-CN': '浅烘', 'en-US': 'Light', 'ja-JP': '浅煎り' },
      light: '#D8B89B',
      dark: '#E1C6AA',
    },
    {
      from: 16 * minute,
      name: { 'zh-CN': '中浅烘', 'en-US': 'Medium-light', 'ja-JP': '中浅煎り' },
      light: '#C49A76',
      dark: '#D4B18D',
    },
    {
      from: 32 * minute,
      name: { 'zh-CN': '中烘', 'en-US': 'Medium', 'ja-JP': '中煎り' },
      light: '#AD7C56',
      dark: '#C49972',
    },
    {
      from: 44 * minute,
      name: { 'zh-CN': '中深烘', 'en-US': 'Medium-dark', 'ja-JP': '中深煎り' },
      light: '#8B5B3B',
      dark: '#B2825B',
    },
    {
      from: 60 * minute,
      name: { 'zh-CN': '深烘', 'en-US': 'Dark', 'ja-JP': '深煎り' },
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
