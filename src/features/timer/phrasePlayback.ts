/** Leave time to read each segment; no visual controls or continuous motion. */
export function phraseReadingMs(text: string): number {
  const units = [...new Intl.Segmenter(undefined, { granularity: 'grapheme' }).segment(text)]
    .length;
  return Math.max(8_000, Math.min(25_000, units * 220));
}
