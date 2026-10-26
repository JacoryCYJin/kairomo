export type SeriesId = 'coffee';
export type SeriesName = Readonly<Record<'zh-CN' | 'en-US', string>>;
export interface SeriesDefinition {
  readonly id: SeriesId;
  readonly name: SeriesName;
  readonly summary: { readonly limit: number; readonly replace: 'shortest' };
}
