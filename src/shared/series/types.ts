import manifest from '../../../theme-kits/catalog.json' with { type: 'json' };
import type { AppLocale } from '../i18n';
export type SeriesId = keyof typeof manifest.themes;
export type SeriesName = Readonly<Record<AppLocale, string>>;
export interface RestPalette {
  readonly ink: string;
  readonly muted: string;
  readonly selected: string;
  readonly hover: string;
  readonly line: string;
  readonly background: string;
}
export interface RestVisual {
  readonly light: RestPalette;
  readonly dark: RestPalette;
}
export interface SeriesDefinition {
  readonly id: SeriesId;
  readonly name: SeriesName;
  readonly rest: RestVisual;
  readonly focus?: RestVisual;
}
