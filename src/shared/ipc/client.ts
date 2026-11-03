import { invoke, isTauri } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { save } from '@tauri-apps/plugin-dialog';
import type {
  Action,
  BeanRange,
  Phase,
  Phrases,
  Preferences,
  Session,
  Settings,
  ShortcutBinding,
  ShortcutKind,
  ShortcutMutation,
  ShortcutStatus,
  ThemeOptionsChange,
  TaskChange,
  Snapshot,
} from './types';
export const desktop = isTauri();
export const api = {
  manageTasks: (change: TaskChange) => invoke<Snapshot>('manage_tasks', { change }),
  createTask: (name: string) => invoke<Snapshot>('create_task', { name }),
  selectTask: (taskId: string | null) => invoke<Snapshot>('select_task', { taskId }),
  onTimerOpened: (handler: () => void) => listen('timer:opened', handler),
  detailsView: () => invoke<'history' | 'settings'>('details_view'),
  openDetails: (view: 'history' | 'settings') => invoke<void>('open_details', { view }),
  hidePopup: () => invoke<void>('hide_popup'),
  popupPrepared: () => invoke<void>('popup_prepared'),
  resizePopup: (height: number) => invoke<void>('resize_popup', { height }),
  onView: (handler: (view: 'history' | 'settings') => void) =>
    listen<'history' | 'settings'>('details:view', (e) => handler(e.payload)),
  snapshot: () => invoke<Snapshot>('snapshot'),
  sessions: () => invoke<Session[]>('sessions'),
  action: (action: Action, title: string, expectedSessionId: string | null, expectedPhase: Phase) =>
    invoke<Snapshot>('timer_action', { action, title, expectedSessionId, expectedPhase }),
  cancelSession: (expectedId: string) => invoke<Snapshot>('cancel_session', { expectedId }),
  settings: (settings: Settings) => invoke<Snapshot>('save_settings', { settings }),
  shortcutStatus: () => invoke<ShortcutStatus>('shortcut_status'),
  retryShortcuts: () => invoke<ShortcutMutation>('retry_shortcuts'),
  beginShortcutRecording: () => invoke<ShortcutMutation>('begin_shortcut_recording'),
  endShortcutRecording: () => invoke<ShortcutMutation>('end_shortcut_recording'),
  updateShortcut: (kind: ShortcutKind, binding: ShortcutBinding) =>
    invoke<ShortcutMutation>('update_shortcut', { kind, binding }),
  setShortcutEnabled: (kind: ShortcutKind, enabled: boolean) =>
    invoke<ShortcutMutation>('set_shortcut_enabled', { kind, enabled }),
  restoreShortcut: (kind: ShortcutKind) => invoke<ShortcutMutation>('restore_shortcut', { kind }),
  restoreAllShortcuts: () => invoke<ShortcutMutation>('restore_all_shortcuts'),
  onShortcutsChanged: (handler: (status: ShortcutStatus) => void) =>
    listen<ShortcutStatus>('shortcuts:changed', (event) => handler(event.payload)),
  themeOptions: (change: ThemeOptionsChange) =>
    invoke<Snapshot>('save_theme_options', { themeId: change.theme_id, values: change.values }),
  beanRange: (range: BeanRange) => invoke<Snapshot>('save_bean_range', { range }),
  phrases: (phrases: Phrases) => invoke<Snapshot>('save_phrases', { phrases }),
  preferences: (preferences: Partial<Preferences>) =>
    invoke<Snapshot>('save_preferences', {
      locale: preferences.locale,
      appearance: preferences.appearance,
      visualSeries: preferences.visual_series,
    }),
  delete: (id: string) => invoke<void>('delete_session', { id }),
  clearRecords: () => invoke<void>('clear_sessions'),
  onRecordsChanged: (handler: () => void) => listen('records:changed', handler),
  correct: (
    id: string,
    startedAtMs: number | null,
    endedAtMs: number | null,
    taskId: string | null,
  ) => invoke<void>('correct_session', { id, startedAtMs, endedAtMs, taskId }),
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
