<script setup lang="ts">
import { nextTick, onMounted, onUnmounted, ref } from 'vue';
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
const dark = ref(media.matches);
let disposed = false;
let unlisten: (() => void) | undefined;
function resetTimerFocus() {
  if (auxiliary) return;
  void nextTick(() => {
    if (!disposed) shell.value?.focus({ preventScroll: true });
  });
}
function applyTheme() {
  dark.value = media.matches;
  void syncAppearance(media.matches).catch(() => {});
}
function drag(event: MouseEvent) {
  if (auxiliary || !desktop || event.button !== 0 || event.detail !== 1) return;
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
  if (event.key === 'Escape' && !auxiliary && desktop) void api.hidePopup();
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
        : 'timer-shell relative h-screen select-none overflow-auto focus:outline-none'
    "
    :tabindex="auxiliary ? undefined : -1"
    @mousedown="drag"
  >
    <header v-if="!auxiliary" class="flex h-12 items-center justify-between pl-5 pr-3.5 pt-2.5">
      <span class="text-sm font-medium text-muted">Kairomo</span>
      <div class="flex gap-0.5">
        <button
          class="icon-button"
          :aria-label="copy.history"
          :title="copy.history"
          @click="open('history')"
        >
          <ChartNoAxesColumnIncreasing :size="17" />
        </button>
        <button
          class="icon-button"
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
    <TimerView v-if="!auxiliary" :state="state" :busy="busy || !ready" @action="k.act($event)" />
    <div v-else class="px-7 pb-6">
      <HistoryView
        v-if="view === 'history'"
        :records="records"
        :busy="busy || !ready"
        :current="state.current"
        :checkpoint="state.checkpoint_at_ms"
        @export="k.exportRecords"
        @remove="k.remove"
        @correct="k.correct"
      />
      <SettingsView
        v-else
        :settings="state.settings"
        :busy="busy || !ready"
        :dark="dark"
        @save="k.configure"
      />
    </div>
  </main>
</template>
