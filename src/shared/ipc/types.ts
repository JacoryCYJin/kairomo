import type { SeriesId } from '../series/types';
export type Kind = 'focus' | 'break';
export type Phase = 'idle' | 'running' | 'paused';
export type Appearance = 'system' | 'light' | 'dark';
export interface Settings {
  focus_minutes: number;
  break_minutes: number;
  locale: 'zh-CN' | 'en-US';
  notifications: boolean;
  appearance: Appearance;
  visual_series: SeriesId;
}
export type Preferences = Pick<Settings, 'locale' | 'appearance' | 'visual_series'>;
export interface Interval {
  started_at_ms: number;
  ended_at_ms: number | null;
  duration_ms: number;
}
export interface Session {
  id: string;
  kind: Kind;
  title: string;
  started_at_ms: number;
  ended_at_ms: number | null;
  target_ms: number;
  duration_ms: number;
  intervals: Interval[];
  corrected: boolean;
}
export interface Snapshot {
  revision: number;
  phase: Phase;
  current: Session | null;
  settings: Settings;
  needs_recovery: boolean;
  pause_reason: string | null;
  checkpoint_at_ms: number;
  target_notified: boolean;
}
export type Action = 'start' | 'pause' | 'resume' | 'switch' | 'finish';
