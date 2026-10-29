import type { ThemeComponents } from './contracts';
/** Pure resolver also supports isolated, unregistered test kits. No runtime plugin loading. */
export function themeResolver<T extends string>(kits: Record<T, ThemeComponents>, fallback: T) {
  return (id: string | undefined): ThemeComponents =>
    Object.hasOwn(kits, id ?? '') ? kits[id as T] : kits[fallback];
}
