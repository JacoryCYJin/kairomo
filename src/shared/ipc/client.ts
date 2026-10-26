import { invoke, isTauri } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { save } from '@tauri-apps/plugin-dialog';
import type { Action, Session, Settings, Snapshot } from './types';
export const desktop = isTauri();
export const api = {
  onTimerOpened: (handler: () => void) => listen('timer:opened', handler),
  detailsView: () => invoke<'history' | 'settings'>('details_view'),
  openDetails: (view: 'history' | 'settings') => invoke<void>('open_details', { view }),
  hidePopup: () => invoke<void>('hide_popup'),
  onView: (handler: (view: 'history' | 'settings') => void) =>
    listen<'history' | 'settings'>('details:view', (e) => handler(e.payload)),
  snapshot: () => invoke<Snapshot>('snapshot'),
  sessions: () => invoke<Session[]>('sessions'),
  action: (action: Action, title: string) => invoke<Snapshot>('timer_action', { action, title }),
  settings: (settings: Settings) => invoke<Snapshot>('save_settings', { settings }),
  delete: (id: string) => invoke<void>('delete_session', { id }),
  correct: (id: string, title: string, minutes: number) =>
    invoke<void>('correct_session', { id, title, minutes }),
  async export() {
    const path = await save({
      defaultPath: 'kairomo-sessions.csv',
      filters: [{ name: 'CSV', extensions: ['csv'] }],
    });
    if (!path) return false;
    await invoke('export_csv', { path });
    return true;
  },
  onSnapshot: (handler: (s: Snapshot) => void) =>
    listen<Snapshot>('timer:changed', (e) => handler(e.payload)),
  onError: (handler: () => void) => listen('runtime:error', handler),
  onNotificationError: (handler: () => void) => listen('notification:error', handler),
};
