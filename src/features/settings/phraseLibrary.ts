import type { Phrase } from '../../shared/ipc/types';

/** Keep the remaining entries in their original order when the chosen message moves up. */
export function selectedPhrasesFirst(entries: Phrase[], selected: string | null): Phrase[] {
  return [...entries].sort((a, b) => Number(b.id === selected) - Number(a.id === selected));
}

/** Library/editor attribution is independent of the timer's existing attribution. */
export function libraryAttribution(entry: Phrase): string {
  const author = entry.author?.trim() ?? '';
  const title = entry.source_title?.trim() ?? '';
  return author + (title ? `《${title}》` : '');
}
