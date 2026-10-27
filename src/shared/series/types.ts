import type { AppLocale } from '../i18n';
export type SeriesId = 'coffee';
export type SeriesName = Readonly<Record<AppLocale, string>>;
export interface RestPalette {
  readonly ink: string;
  readonly muted: string;
  readonly selected: string;
  readonly hover: string;
  readonly line: string;
  readonly waves: readonly [string, string, string];
}
export interface RestVisual {
  readonly kind: 'liquid';
  readonly light: RestPalette;
  readonly dark: RestPalette;
  readonly waves: readonly { readonly amplitude: number; readonly period: number }[];
}
export interface SeriesDefinition {
  readonly id: SeriesId;
  readonly name: SeriesName;
  readonly summary: { readonly limit: number; readonly replace: 'shortest' };
  readonly rest: RestVisual;
}
