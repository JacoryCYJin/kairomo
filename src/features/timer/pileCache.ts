import type { PileLayout } from './beanPile';
const cache = new Map<string, PileLayout>();
export function cachedPile(ids: readonly string[], seed: string) {
  for (const layout of [...cache.values()].reverse()) {
    if (layout.seed !== seed) continue;
    const prefix = layout.evaluatedIds;
    if (
      (layout.capped ? ids.length >= prefix.length : ids.length === prefix.length) &&
      prefix.every((id, index) => ids[index] === id)
    )
      return layout;
  }
  return undefined;
}
export function rememberPile(layout: PileLayout) {
  const key = JSON.stringify([layout.seed, layout.evaluatedIds]);
  cache.delete(key);
  cache.set(key, layout);
  while (cache.size > 4) cache.delete(cache.keys().next().value!);
}

/** An unchanged ordered prefix is a complete, supported old pile, safe while new beans settle. */
export function cachedPilePrefix(ids: readonly string[], seed: string) {
  let best: PileLayout | undefined;
  for (const layout of cache.values()) {
    if (
      layout.seed === seed &&
      layout.evaluatedIds.length <= ids.length &&
      layout.evaluatedIds.every((id, index) => ids[index] === id) &&
      (!best || layout.poses.length > best.poses.length)
    )
      best = layout;
  }
  return best;
}
