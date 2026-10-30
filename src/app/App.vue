<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
import { ChartNoAxesColumnIncreasing, SlidersHorizontal, X } from 'lucide-vue-next';
import TimerView from '../features/timer/TimerView.vue';
import { timerLayout } from '../features/timer/timerLayout';
import HistoryView from '../features/history/HistoryView.vue';
import SettingsView from '../features/settings/SettingsView.vue';
import { copy } from '../shared/i18n';
import { api, desktop } from '../shared/ipc/client';
import { syncAppearance } from '../shared/ipc/appearance';
import { startWindowDrag, windowTitle } from '../shared/ipc/window';
import { useKairomo } from './useKairomo';
import { popupStartup } from './popupStartup';
import { updates } from '../shared/ipc/updates';
import { dialogConfirmStyle, seriesThemeStyle } from '../shared/series/rest';
const auxiliary = new URLSearchParams(window.location.search).has('view');
const view = ref<'history' | 'settings'>('settings');
const shell = ref<HTMLElement | null>(null);
const title = ref('Kairomo');
const development = ref(false);
const identityReady = ref(!desktop || auxiliary);
const startupReleased = ref(!desktop || auxiliary);
const visualPrepared = ref(false);
const visualFailed = ref(false);
const k = useKairomo();
const { state, records, ready, busy, error, notice } = k;
const popupHeight = computed(() => timerLayout(state.value).height);
let pendingResize = Promise.resolve();
function syncPopupSize() {
  if (auxiliary || !desktop || !ready.value) return;
  const height = popupHeight.value;
  pendingResize = pendingResize
    .catch(() => {})
    .then(async () => {
      try {
        await api.resizePopup(height);
      } catch {
        error.value = copy.value.timerLayoutError;
      }
    });
}
watch([ready, popupHeight], syncPopupSize);
const media = window.matchMedia('(prefers-color-scheme: dark)');
const systemDark = ref(media.matches);
const dark = computed(
  () =>
    state.value.settings.appearance === 'dark' ||
    (state.value.settings.appearance === 'system' && systemDark.value),
);
const resting = computed(() => !auxiliary && state.value.current?.kind === 'break');
const visualSeries = computed(() => state.value.settings.visual_series);
const restStyle = computed(() => seriesThemeStyle(visualSeries.value, dark.value, resting.value));
const confirmStyle = computed(() => dialogConfirmStyle(visualSeries.value, dark.value));
watch(
  dark,
  (value) => {
    document.documentElement.dataset.theme = value ? 'dark' : 'light';
  },
  { immediate: true },
);
let pendingAppearance = Promise.resolve();
watch([ready, () => state.value.settings.appearance, dark, resting, visualSeries], () => {
  if (ready.value)
    pendingAppearance = syncAppearance(
      state.value.settings.appearance,
      getComputedStyle(document.documentElement).getPropertyValue('--paper').trim(),
    ).catch(() => {
      error.value = copy.value.appearanceError;
    });
});

const startup = popupStartup(
  async () => {
    await pendingResize;
    await pendingAppearance;
    await nextTick();
  },
  () => api.popupPrepared(),
  () => {
    startupReleased.value = true;
  },
  () => {
    error.value = copy.value.windowError;
  },
);
function themeVisualFailed() {
  visualFailed.value = true;
  error.value = copy.value.error;
}
watch(
  [
    ready,
    visualPrepared,
    visualFailed,
    resting,
    identityReady,
    popupHeight,
    dark,
    () => state.value.settings.appearance,
    error,
  ],
  () => {
    if (auxiliary || !desktop || startupReleased.value) return;
    // A failed load displays its error and retry control, rather than trapping a hidden window.
    const prepared =
      !!error.value ||
      (ready.value && identityReady.value && (visualPrepared.value || visualFailed.value));
    void startup.update(prepared);
  },
  { flush: 'post' },
);

let disposed = false;
let unlisten: (() => void) | undefined;
function resetTimerFocus() {
  if (auxiliary) return;
  void nextTick(() => {
    if (!disposed && !document.querySelector('dialog[open]'))
      shell.value?.focus({ preventScroll: true });
  });
}
function applyTheme() {
  systemDark.value = media.matches;
}
function drag(event: MouseEvent) {
  if (
    !desktop ||
    event.button !== 0 ||
    event.detail !== 1 ||
    document.querySelector('dialog[open]')
  )
    return;
  const target = event.target;
  if (auxiliary && (!(target instanceof Element) || !target.closest('[data-toolbar]'))) return;
  if (
    !(target instanceof Element) ||
    target.closest('button, input, select, textarea, a, [role="button"], [contenteditable="true"]')
  )
    return;
  event.preventDefault();
  void startWindowDrag().catch(() => {
    error.value = copy.value.windowError;
  });
}
function hide(event: KeyboardEvent) {
  if (
    event.key === 'Escape' &&
    !event.defaultPrevented &&
    !document.querySelector('dialog[open]') &&
    !auxiliary &&
    desktop
  )
    void hideTimer();
}
async function hideTimer() {
  try {
    await api.hidePopup();
  } catch {
    error.value = copy.value.error;
  }
}
async function open(view: 'history' | 'settings') {
  try {
    await api.openDetails(view);
  } catch {
    error.value = copy.value.error;
  }
}
onMounted(async () => {
  if (auxiliary) document.documentElement.classList.add('details-window');
  if (!auxiliary && desktop) {
    const info = await updates.info().catch(() => null);
    if (!disposed) {
      development.value = info?.channel === 'dev';
      identityReady.value = true;
    }
  }
  if (auxiliary && desktop) title.value = await windowTitle().catch(() => 'Kairomo');
  applyTheme();
  media.addEventListener('change', applyTheme);
  window.addEventListener('keydown', hide);
  resetTimerFocus();
  if (!auxiliary && desktop) {
    const stop = await api.onTimerOpened(() => {
      resetTimerFocus();
      syncPopupSize();
    });
    if (disposed) stop();
    else unlisten = stop;
    resetTimerFocus();
  }
  if (auxiliary && desktop) {
    const stop = await api.onView((next) => {
      view.value = next;
      notice.value = '';
    });
    view.value = await api.detailsView();
    if (disposed) stop();
    else unlisten = stop;
  }
});
onUnmounted(() => {
  if (auxiliary) document.documentElement.classList.remove('details-window');
  disposed = true;
  startup.dispose();
  unlisten?.();
  media.removeEventListener('change', applyTheme);
  window.removeEventListener('keydown', hide);
});
</script>
<template>
  <main
    ref="shell"
    :class="
      auxiliary
        ? 'details-shell flex h-screen flex-col overflow-hidden bg-paper'
        : 'timer-shell group relative h-screen select-none overflow-auto bg-paper focus:outline-none'
    "
    :tabindex="auxiliary ? undefined : -1"
    :style="[restStyle, confirmStyle]"
    @mousedown="drag"
  >
    <header
      v-if="!auxiliary"
      class="absolute inset-x-0 top-0 z-10 flex h-12 items-center justify-between px-3.5 pt-2.5"
    >
      <span
        v-if="development"
        class="pointer-events-none absolute left-1/2 -translate-x-1/2 text-xs font-medium tracking-widest text-muted"
        >DEV</span
      >
      <button
        class="icon-button pointer-events-none opacity-0 focus-visible:pointer-events-auto focus-visible:opacity-100 group-hover:pointer-events-auto group-hover:opacity-100"
        :aria-label="copy.hideTimer"
        :title="copy.hideTimer"
        @click="hideTimer"
      >
        <X :size="15" />
      </button>
      <div class="flex gap-0.5">
        <button
          class="icon-button pointer-events-none opacity-0 focus-visible:pointer-events-auto focus-visible:opacity-100 group-hover:pointer-events-auto group-hover:opacity-100"
          :aria-label="copy.history"
          :title="copy.history"
          @click="open('history')"
        >
          <ChartNoAxesColumnIncreasing :size="17" />
        </button>
        <button
          class="icon-button pointer-events-none opacity-0 focus-visible:pointer-events-auto focus-visible:opacity-100 group-hover:pointer-events-auto group-hover:opacity-100"
          :aria-label="copy.settings"
          :title="copy.settings"
          @click="open('settings')"
        >
          <SlidersHorizontal :size="17" />
        </button>
      </div>
    </header>
    <header
      v-else
      data-toolbar
      class="z-10 flex h-12 shrink-0 select-none items-center justify-between gap-4 border-b border-line bg-paper pr-4 pt-px"
      :class="desktop ? 'pl-[102px]' : 'pl-7'"
    >
      <span class="truncate text-sm font-semibold">{{ title }}</span>
      <nav
        class="flex shrink-0 gap-0.5 rounded-lg bg-surface-muted p-0.5"
        :aria-label="copy.navigation"
      >
        <button
          class="rounded-md px-3 py-1 text-xs"
          :aria-pressed="view === 'history'"
          :class="view === 'history' ? 'bg-selected text-ink' : 'text-muted'"
          @click="
            view = 'history';
            notice = '';
          "
        >
          {{ copy.history }}
        </button>
        <button
          class="rounded-md px-3 py-1 text-xs"
          :aria-pressed="view === 'settings'"
          :class="view === 'settings' ? 'bg-selected text-ink' : 'text-muted'"
          @click="
            view = 'settings';
            notice = '';
          "
        >
          {{ copy.settings }}
        </button>
      </nav>
    </header>
    <div v-if="!desktop" class="banner">{{ copy.previewHint }}</div>
    <div v-else-if="error" class="banner error" role="alert">
      {{ error
      }}<button class="text-button" :disabled="busy" @click="k.connect">{{ copy.retry }}</button>
    </div>
    <div
      v-if="notice && (auxiliary || notice === copy.notificationError)"
      class="banner"
      role="status"
    >
      {{ notice
      }}<button class="icon-button dismiss" :aria-label="copy.dismiss" @click="notice = ''">
        <X :size="14" />
      </button>
    </div>
    <TimerView
      v-if="!auxiliary"
      :state="state"
      :warming-up="!startupReleased"
      @prepared="visualPrepared = $event"
      @visual-failed="themeVisualFailed"
      :dark="dark"
      :records="records"
      :cancel-session="k.cancelSession"
      :busy="busy || !ready"
      @action="k.act($event)"
    />
    <div
      v-else
      class="details-content min-h-0 flex-1 overflow-x-hidden overflow-y-auto overscroll-none"
    >
      <div class="mx-auto max-w-[820px] px-7" :class="view === 'history' ? 'pb-3' : 'pb-6'">
        <HistoryView
          v-if="view === 'history'"
          :records="records"
          :busy="busy || !ready"
          :current="state.current"
          :checkpoint="state.checkpoint_at_ms"
          :series-id="state.settings.visual_series"
          @export="k.exportRecords"
          @remove="k.remove"
          @correct="k.correct"
        />
        <SettingsView
          v-else
          :settings="state.settings"
          :clear-records="k.clearRecords"
          :busy="busy || !ready"
          @save="k.configure"
          @preference="k.preferences"
          @phrases="k.phrases"
          @theme-options="k.themeOptions"
          @bean-range="k.beanRange"
        />
      </div>
    </div>
  </main>
</template>
