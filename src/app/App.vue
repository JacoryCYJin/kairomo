<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
import { ChartNoAxesColumnIncreasing, SlidersHorizontal, X } from 'lucide-vue-next';
import TimerView from '../features/timer/TimerView.vue';
import HistoryView from '../features/history/HistoryView.vue';
import SettingsView from '../features/settings/SettingsView.vue';
import { copy } from '../shared/i18n';
import { api, desktop } from '../shared/ipc/client';
import { syncAppearance } from '../shared/ipc/appearance';
import { startWindowDrag } from '../shared/ipc/window';
import { useKairomo } from './useKairomo';
const auxiliary = new URLSearchParams(window.location.search).has('view');
const view = ref<'history' | 'settings'>('settings');
const shell = ref<HTMLElement | null>(null);
const k = useKairomo();
const { state, records, ready, busy, error, notice } = k;
const media = window.matchMedia('(prefers-color-scheme: dark)');
const systemDark = ref(media.matches);
const dark = computed(
  () =>
    state.value.settings.appearance === 'dark' ||
    (state.value.settings.appearance === 'system' && systemDark.value),
);
watch(
  dark,
  (value) => {
    document.documentElement.dataset.theme = value ? 'dark' : 'light';
  },
  { immediate: true },
);
watch([ready, () => state.value.settings.appearance, dark], () => {
  if (ready.value)
    void syncAppearance(
      state.value.settings.appearance,
      getComputedStyle(document.documentElement).getPropertyValue('--paper').trim(),
    ).catch(() => {
      error.value = copy.value.appearanceError;
    });
});
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
    auxiliary ||
    !desktop ||
    event.button !== 0 ||
    event.detail !== 1 ||
    document.querySelector('dialog[open]')
  )
    return;
  const target = event.target;
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
  applyTheme();
  media.addEventListener('change', applyTheme);
  window.addEventListener('keydown', hide);
  resetTimerFocus();
  if (!auxiliary && desktop) {
    const stop = await api.onTimerOpened(resetTimerFocus);
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
  disposed = true;
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
        ? 'details-shell mx-auto max-w-[820px]'
        : 'timer-shell group relative h-screen select-none overflow-auto focus:outline-none'
    "
    :tabindex="auxiliary ? undefined : -1"
    @mousedown="drag"
  >
    <header
      v-if="!auxiliary"
      class="absolute inset-x-0 top-0 z-10 flex h-12 items-center justify-between px-3.5 pt-2.5"
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
    <nav
      v-else
      class="sticky top-0 z-10 flex justify-center gap-1 border-b border-line bg-paper px-5 py-3.5"
      :aria-label="copy.navigation"
    >
      <button
        class="rounded-lg px-5 py-2 text-xs"
        :class="view === 'history' ? 'bg-selected text-ink' : 'text-muted'"
        @click="
          view = 'history';
          notice = '';
        "
      >
        {{ copy.history }}
      </button>
      <button
        class="rounded-lg px-5 py-2 text-xs"
        :class="view === 'settings' ? 'bg-selected text-ink' : 'text-muted'"
        @click="
          view = 'settings';
          notice = '';
        "
      >
        {{ copy.settings }}
      </button>
    </nav>
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
      :records="records"
      :cancel-session="k.cancelSession"
      :busy="busy || !ready"
      @action="k.act($event)"
    />
    <div v-else class="px-7 pb-6">
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
        :busy="busy || !ready"
        @save="k.configure"
        @preference="k.preferences"
      />
    </div>
  </main>
</template>
