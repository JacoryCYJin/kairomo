import type { AppLocale } from '../i18n';
import type { SeriesId } from '../series/types';
export type Kind = 'focus' | 'break';
export type Phase = 'idle' | 'running' | 'paused';
export type Appearance = 'system' | 'light' | 'dark';
export type FocusMode = 'manual' | 'timed';
export interface Phrase {
  id: string;
  text: string;
  author?: string;
  source_title?: string;
  source_kind?: 'book' | 'poem' | 'song' | 'other' | null;
}
export interface Phrases {
  enabled: boolean;
  mode: 'fixed' | 'random';
  entries: Phrase[];
  selected_id: string | null;
}
export const emptyPhrases = (): Phrases => ({
  enabled: false,
  mode: 'fixed',
  entries: [],
  selected_id: null,
});
export type BeanRange = 'day' | 'week';
export type ThemeOptionValue = string | number | boolean;
export interface ThemeOptionsChange {
  theme_id: SeriesId;
  values: Record<string, ThemeOptionValue>;
}
export interface Settings {
  theme_options?: Partial<Record<SeriesId, Record<string, ThemeOptionValue>>>;
  bean_range: BeanRange;
  phrases: Phrases;
  focus_minutes: number;
  break_minutes: number;
  locale: AppLocale;
  notifications: boolean;
  focus_mode: FocusMode;
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
  focus_mode: FocusMode;
  duration_ms: number;
  intervals: Interval[];
  corrected: boolean;
}
export interface Snapshot {
  bean_random_seed: string | null;
  phrase_selection?: { session_id: string; entry_id: string | null } | null;
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
