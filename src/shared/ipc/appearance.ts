import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { invoke } from '@tauri-apps/api/core';
import { desktop } from './client';
import type { Appearance } from './types';
let pending: Promise<void> = Promise.resolve();
/** Keep native chrome and the backing surface consistent with the persisted preference. */
export function syncAppearance(appearance: Appearance, background: string): Promise<void> {
  if (!desktop) return Promise.resolve();
  // Preserve request order when the user quickly switches between focus and rest.
  pending = pending
    .catch(() => {})
    .then(async () => {
      const window = getCurrentWebviewWindow();
      await window.setTheme(appearance === 'system' ? null : appearance);
      await window.setBackgroundColor(background);
      await invoke<void>('sync_window_appearance', { appearance });
    });
  return pending;
}
