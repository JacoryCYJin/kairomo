import { phraseAttribution } from '../../shared/phrases.ts';
import type { Snapshot } from '../../shared/ipc/types';
export type TimerHeight = 284 | 302 | 320;
/** Content selects one of three fixed layouts; text length never changes window size. */
export function timerLayout(state: Snapshot) {
  const config = state.settings.phrases;
  const entries = config?.enabled ? config.entries.filter((entry) => entry.text.trim()) : [];
  const id = config?.mode === 'fixed' ? config.selected_id : state.phrase_selection?.entry_id;
  const phrase = entries.find((entry) => entry.id === id) ?? entries[0];
  const attribution = phraseAttribution(phrase);
  const height: TimerHeight = !phrase ? 284 : attribution ? 320 : 302;
  return { phrase, attribution, height, controlsTop: height - 121 };
}

/** Same slow horizontal velocity for every sentence, with a pause at both ends. */
export function marqueeTiming(overflow: number) {
  const travel = (Math.max(0, overflow) / 22) * 1000;
  const hold = 2200;
  const duration = travel + hold * 2;
  return { duration, start: hold / duration, end: (hold + travel) / duration };
}
