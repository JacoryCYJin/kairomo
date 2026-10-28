import type { Phrase } from './ipc/types';
export function phraseAttribution(phrase: Phrase | undefined): string {
  if (!phrase) return '';
  const author = phrase.author?.trim() ?? '';
  const title = phrase.source_title?.trim() ?? '';
  return author + (title ? `《${title}》` : '');
}
export function matchesPhrase(phrase: Phrase, query: string): boolean {
  const words = query.trim().toLocaleLowerCase().split(/\s+/u);
  const text = [phrase.text, phrase.author, phrase.source_title]
    .filter(Boolean)
    .join(' ')
    .toLocaleLowerCase();
  return words.every((word) => text.includes(word));
}
