import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
/** Initiate OS window movement; callers exclude interactive controls. */
export function startWindowDrag(): Promise<void> {
  return getCurrentWebviewWindow().startDragging();
}
export function windowTitle(): Promise<string> {
  return getCurrentWebviewWindow().title();
}
