export type SettingsPage =
  'settings' | 'phrase-library' | 'phrase-editor' | 'task-manager' | 'shortcut-manager';
const parent: Partial<Record<SettingsPage, SettingsPage>> = {
  'phrase-library': 'settings',
  'phrase-editor': 'phrase-library',
  'task-manager': 'settings',
  'shortcut-manager': 'settings',
};
/** Remember each parent independently; opening a child always starts at its top. */
export function settingsNavigation() {
  let current: SettingsPage = 'settings';
  const positions = new Map<SettingsPage, number>();
  return {
    navigate(next: SettingsPage, scrollTop: number) {
      if (next === current) return { page: next, scrollTop };
      positions.set(current, scrollTop);
      const returning = parent[current] === next;
      current = next;
      if (!returning) positions.set(next, 0);
      return { page: next, scrollTop: returning ? (positions.get(next) ?? 0) : 0 };
    },
  };
}
