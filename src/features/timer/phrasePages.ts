/** Pack complete grapheme clusters into the available measured display area. */
export function phrasePages(text: string, fits: (text: string) => boolean): string[] {
  const chars = [...new Intl.Segmenter(undefined, { granularity: 'grapheme' }).segment(text)].map(
    (s) => s.segment,
  );
  const pages: string[] = [];
  let offset = 0;
  while (offset < chars.length) {
    let low = 1,
      high = chars.length - offset;
    while (low < high) {
      const middle = Math.ceil((low + high) / 2);
      if (fits(chars.slice(offset, offset + middle).join(''))) low = middle;
      else high = middle - 1;
    }
    let length = low;
    const punctuation = /[。！？；，、,.!?;:]/u;
    for (let i = length - 1; i >= Math.floor(length * 0.55); i--) {
      if (punctuation.test(chars[offset + i] ?? '')) {
        length = i + 1;
        break;
      }
    }
    // Prefer a word boundary, without losing whitespace or splitting an oversized word.
    const chunk = chars.slice(offset, offset + length).join('');
    const lastSpace = chunk.search(/\s+\S*$/u);
    if (offset + length < chars.length && lastSpace > chunk.length * 0.6) {
      const prefix = chunk.slice(0, lastSpace + 1);
      length = [...new Intl.Segmenter(undefined, { granularity: 'grapheme' }).segment(prefix)]
        .length;
    }
    while (length > 1 && /^[，。！？、；：,.!?;:」』）》】…]$/u.test(chars[offset + length] ?? ''))
      length--;
    pages.push(chars.slice(offset, offset + length).join(''));
    offset += length;
  }
  return pages.length ? pages : [''];
}
