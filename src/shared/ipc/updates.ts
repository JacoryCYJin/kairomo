import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { withUpdateDeadline } from './updateDeadline';
export interface ReleaseInfo {
  name: string;
  version: string;
  channel: 'dev' | 'beta';
}
export interface UpdateCheck {
  status: 'development' | 'unpublished' | 'current' | 'available';
  version: string | null;
  notes: string | null;
  token: string | null;
}
export interface UpdateProgress {
  stage: 'downloading' | 'installing';
  downloaded: number;
  total: number | null;
}
export const updates = {
  info: () => invoke<ReleaseInfo>('release_info'),
  check: () => withUpdateDeadline(() => invoke<UpdateCheck>('check_update')),
  install: (token: string) => invoke<void>('install_update', { token }),
  onProgress: (handler: (progress: UpdateProgress) => void) =>
    listen<UpdateProgress>('update:progress', (event) => handler(event.payload)),
};
