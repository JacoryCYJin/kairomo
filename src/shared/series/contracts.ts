import type { Component } from 'vue';
import type { Session, Settings, Snapshot } from '../ipc/types';
/** Host-owned full-window dimensions; each theme chooses its own painted bounds. */
export interface ThemeCanvasSize {
  readonly width: number;
  readonly height: number;
}
/** Theme visuals only read state. Prepared/failed events coordinate first display. */
export interface TimerVisualProps {
  canvas?: ThemeCanvasSize;
  state: Snapshot;
  records: Session[];
  dark?: boolean;
  warmingUp?: boolean;
}
export interface CollectionProps {
  records: Session[];
  completedRecords?: Session[];
  total: number;
  fitHeight?: boolean;
}
export interface RewardProps {
  duration: number;
}
export interface ThemeSettingsProps {
  settings: Settings;
  busy: boolean;
}
/** Independent region implementations; no coffee-specific component types or geometry. */
export interface ThemeComponents {
  focus: Component<TimerVisualProps>;
  rest: Component<TimerVisualProps>;
  restIcon: Component;
  collection: Component<CollectionProps>;
  reward: Component<RewardProps>;
  settings: Component<ThemeSettingsProps>;
}
