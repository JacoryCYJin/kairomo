import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { desktop } from './client';
/** Native backing surface follows the same system preference as the CSS palette. */
export async function syncAppearance(dark: boolean): Promise<void> {
  if (!desktop) return;
  await getCurrentWebviewWindow().setBackgroundColor(dark ? '#212121' : '#ffffff');
}
