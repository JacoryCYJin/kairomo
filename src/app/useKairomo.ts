import { computed, onMounted, onUnmounted, ref } from 'vue';
import { api, desktop } from '../shared/ipc/client';
import type { Action, Preferences, Session, Settings, Snapshot } from '../shared/ipc/types';
import { copy, locale } from '../shared/i18n';
const defaults: Snapshot = {
  revision: 0,
  phase: 'idle',
  current: null,
  settings: {
    focus_minutes: 50,
    break_minutes: 10,
    locale: 'zh-CN',
    notifications: true,
    appearance: 'system',
    visual_series: 'coffee',
  },
  needs_recovery: false,
  pause_reason: null,
  checkpoint_at_ms: 0,
  target_notified: false,
};
export function useKairomo() {
  const state = ref<Snapshot>(defaults);
  const records = ref<Session[]>([]);
  const ready = ref(false),
    busy = ref(false),
    error = ref(''),
    notice = ref('');
  const unlisten: (() => void)[] = [];
  let disposed = false;
  function apply(s: Snapshot) {
    if (s.revision < state.value.revision) return;
    state.value = s;
    locale.value = s.settings.locale;
    document.documentElement.lang = s.settings.locale;
  }
  async function refresh() {
    records.value = await api.sessions();
  }
  async function connect() {
    if (!desktop || busy.value) return;
    busy.value = true;
    error.value = '';
    try {
      apply(await api.snapshot());
      await refresh();
      ready.value = true;
    } catch {
      ready.value = false;
      error.value = copy.value.error;
    } finally {
      busy.value = false;
    }
  }
  async function operate(operation: () => Promise<void>) {
    if (!ready.value || busy.value) return;
    busy.value = true;
    error.value = '';
    notice.value = '';
    try {
      await operation();
    } catch {
      error.value = copy.value.error;
    } finally {
      busy.value = false;
    }
  }
  async function act(action: Action, title = '') {
    await operate(async () => {
      apply(await api.action(action, title));
      await refresh();
    });
  }
  async function cancelSession(expectedId: string): Promise<boolean> {
    if (!ready.value || busy.value) return false;
    let cancelled = false;
    await operate(async () => {
      apply(await api.cancelSession(expectedId));
      cancelled = true;
      await refresh();
    });
    return cancelled;
  }
  async function configure(settings: Settings) {
    await operate(async () => {
      apply(await api.settings(settings));
      notice.value = copy.value.saved;
    });
  }
  async function preferences(preferences: Partial<Preferences>) {
    await operate(async () => {
      apply(await api.preferences(preferences));
    });
  }
  async function remove(id: string) {
    await operate(async () => {
      await api.delete(id);
      await refresh();
    });
  }
  async function correct(id: string, title: string, minutes: number) {
    await operate(async () => {
      await api.correct(id, title, minutes);
      await refresh();
    });
  }
  async function exportRecords() {
    await operate(async () => {
      if (await api.export()) notice.value = copy.value.exported;
    });
  }
  onMounted(async () => {
    if (!desktop) return;
    try {
      const subscriptions = await Promise.all([
        api.onSnapshot((s) => {
          const changed = state.value.current?.id !== s.current?.id;
          apply(s);
          if (changed)
            void refresh().catch(() => {
              error.value = copy.value.error;
            });
        }),
        api.onError(() => {
          error.value = copy.value.error;
        }),
        api.onNotificationError(() => {
          notice.value = copy.value.notificationError;
        }),
      ]);
      if (disposed) {
        subscriptions.forEach((fn) => fn());
        return;
      }
      unlisten.push(...subscriptions);
      await connect();
    } catch {
      error.value = copy.value.error;
    }
  });
  onUnmounted(() => {
    disposed = true;
    unlisten.forEach((fn) => fn());
  });
  const all = computed(() =>
    state.value.current ? [state.value.current, ...records.value] : records.value,
  );
  // Split intervals at local midnight; midnight-spanning sessions retain accurate day attribution.
  function dayTotal(kind: 'focus' | 'break', day: Date) {
    const start = new Date(day.getFullYear(), day.getMonth(), day.getDate()).getTime();
    const end = new Date(day.getFullYear(), day.getMonth(), day.getDate() + 1).getTime();
    return all.value
      .filter((s) => s.kind === kind)
      .reduce(
        (sum, s) =>
          sum +
          s.intervals.reduce((sub, i) => {
            const stop = i.ended_at_ms ?? state.value.checkpoint_at_ms;
            if (stop <= i.started_at_ms)
              return sub + (i.started_at_ms >= start && i.started_at_ms < end ? i.duration_ms : 0);
            const overlap = Math.max(0, Math.min(stop, end) - Math.max(i.started_at_ms, start));
            return sub + (i.duration_ms * overlap) / (stop - i.started_at_ms);
          }, 0),
        0,
      );
  }
  const todayFocus = computed(() => dayTotal('focus', new Date()));
  const todayBreak = computed(() => dayTotal('break', new Date()));
  const todayCount = computed(
    () =>
      all.value.filter(
        (s) =>
          s.kind === 'focus' &&
          new Date(s.started_at_ms).toDateString() === new Date().toDateString(),
      ).length,
  );
  return {
    state,
    records,
    ready,
    busy,
    error,
    notice,
    connect,
    act,
    cancelSession,
    configure,
    preferences,
    remove,
    correct,
    exportRecords,
    todayFocus,
    todayBreak,
    todayCount,
  };
}
