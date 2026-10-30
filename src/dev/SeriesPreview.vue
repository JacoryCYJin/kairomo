<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import TimerView from '../features/timer/TimerView.vue';
import ThemeCanvas from '../shared/series/ThemeCanvas.vue';
import CanvasFixture from './CanvasFixture.vue';
import { timerLayout } from '../features/timer/timerLayout';
import { availableSeries, defaultSeriesId } from '../shared/series/catalog';
import { dialogConfirmStyle } from '../shared/series/rest';
import SeriesCollection from '../shared/series/SeriesCollection.vue';
import SettingsView from '../features/settings/SettingsView.vue';
import HistoryView from '../features/history/HistoryView.vue';
import { locale } from '../shared/i18n';
import type { Settings, Snapshot, ThemeOptionsChange } from '../shared/ipc/types';
import { previewCounts, seriesFixtures } from './seriesFixtures';
const query = new URLSearchParams(location.search);
const count = ref(Number(query.get('count') || 20));
const view = ref(query.has('gallery') ? 'gallery' : query.get('panel') || 'collection');
const dark = ref(query.get('theme') === 'dark');
const settings = ref<Settings>({
  phrases: { enabled: false, mode: 'fixed', entries: [], selected_id: null },
  bean_range: 'day',
  focus_minutes: 50,
  break_minutes: 10,
  locale: 'zh-CN',
  notifications: false,
  focus_mode: 'manual',
  appearance: 'light',
  visual_series:
    availableSeries.find((kit) => kit.id === query.get('series'))?.id ?? defaultSeriesId,
});
watch(dark, (value) => (document.documentElement.dataset.theme = value ? 'dark' : 'light'), {
  immediate: true,
});
watch(
  () => settings.value.locale,
  (value) => (locale.value = value),
  { immediate: true },
);
const previewState = computed<Snapshot>(() => {
  const kind = query.get('kind') === 'break' ? 'break' : 'focus';
  const phase =
    query.get('phase') === 'paused' ? 'paused' : query.get('phase') === 'idle' ? 'idle' : 'running';
  const target =
    (kind === 'break' ? settings.value.break_minutes : settings.value.focus_minutes) * 60000;
  const elapsed = Number(query.get('elapsed') ?? 25) * 60000;
  const phrases = query.get('phrase');
  return {
    bean_random_seed: 'preview-only',
    revision: 0,
    phase,
    settings: {
      ...settings.value,
      phrases: {
        enabled: !!phrases,
        mode: 'fixed',
        selected_id: 'preview',
        entries: phrases
          ? [
              {
                id: 'preview',
                text: '先完成今天最重要的一件事。',
                ...(phrases === 'author' ? { author: 'Demo', source_title: 'Preview' } : {}),
              },
            ]
          : [],
      },
    },
    current:
      phase === 'idle'
        ? null
        : {
            id: 'preview-current',
            kind,
            title: '',
            started_at_ms: Date.now() - elapsed,
            ended_at_ms: null,
            target_ms: target,
            focus_mode: 'manual',
            duration_ms: elapsed,
            intervals: [
              { started_at_ms: Date.now() - elapsed, ended_at_ms: null, duration_ms: elapsed },
            ],
            corrected: false,
          },
    needs_recovery: false,
    pause_reason: null,
    checkpoint_at_ms: Date.now(),
    target_notified: elapsed >= target,
  };
});
function saveThemeOptions(change: ThemeOptionsChange) {
  if (change.theme_id === 'coffee' && change.values.range)
    settings.value.bean_range = change.values.range as Settings['bean_range'];
  else
    settings.value.theme_options = {
      ...settings.value.theme_options,
      [change.theme_id]: { ...settings.value.theme_options?.[change.theme_id], ...change.values },
    };
}
const records = computed(() => seriesFixtures(count.value));
const total = (n: number) => seriesFixtures(n).reduce((sum, s) => sum + s.duration_ms, 0);
const caption = computed(() =>
  locale.value === 'zh-CN'
    ? '测试数据 · 不写入学习记录'
    : 'Test data · never saved to your history',
);
</script>
<template>
  <main
    class="mx-auto max-w-[1200px] p-6"
    :style="dialogConfirmStyle(settings.visual_series, dark)"
    :class="{ 'proof-still': query.get('motion') === 'still' }"
  >
    <header class="mb-5 flex flex-wrap items-center gap-3 border-b border-line pb-4 text-xs">
      <strong>{{ caption }}</strong>
      <label
        >{{ locale === 'zh-CN' ? '数量' : 'Count' }}
        <select v-model.number="count" class="rounded-lg border border-line bg-surface px-2 py-1">
          <option v-for="n in previewCounts" :key="n" :value="n">{{ n }}</option>
        </select></label
      >
      <select
        v-model="view"
        aria-label="Preview"
        class="rounded-lg border border-line bg-surface px-2 py-1"
      >
        <option value="collection">Collection</option>
        <option value="gallery">Gallery</option>
        <option value="history">History</option>
        <option value="settings">Settings</option>
        <option value="timer">Timer</option>
        <option value="canvas">Canvas test</option>
      </select>
      <select
        v-model="settings.visual_series"
        aria-label="Theme"
        class="rounded-lg border border-line bg-surface px-2 py-1"
      >
        <option v-for="kit in availableSeries" :key="kit.id" :value="kit.id">
          {{ kit.name[locale] }}
        </option>
      </select>
      <label
        ><input v-model="dark" type="checkbox" /> {{ locale === 'zh-CN' ? '深色' : 'Dark' }}</label
      >
      <select
        v-model="settings.locale"
        aria-label="Language"
        class="rounded-lg border border-line bg-surface px-2 py-1"
      >
        <option value="zh-CN">中文</option>
        <option value="en-US">English</option>
      </select>
    </header>
    <div v-if="view === 'gallery'" class="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
      <section v-for="n in previewCounts" :key="n" class="min-w-0" :data-preview-count="n">
        <h2 class="mb-2 text-center text-sm font-medium">
          {{ n }} {{ locale === 'zh-CN' ? '颗' : 'beans' }}
        </h2>
        <SeriesCollection
          :series-id="settings.visual_series"
          :records="seriesFixtures(n)"
          :total="total(n)"
        />
      </section>
    </div>
    <div
      v-else-if="view === 'collection'"
      class="mx-auto w-full max-w-[400px]"
      :data-preview-count="count"
    >
      <SeriesCollection
        :series-id="settings.visual_series"
        :records="records"
        :total="total(count)"
      />
    </div>
    <div
      v-else-if="view === 'canvas'"
      class="relative mx-auto w-[360px] overflow-hidden rounded-xl border border-line bg-paper"
      :style="{ height: timerLayout(previewState).height + 'px' }"
    >
      <ThemeCanvas :visual="CanvasFixture" kind="focus" :state="previewState" :records="records" />
      <button
        class="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 rounded-lg border border-line p-2"
        data-canvas-hit-test
      >
        Canvas test
      </button>
    </div>
    <div
      v-else-if="view === 'timer'"
      class="mx-auto w-[360px] overflow-hidden rounded-xl border border-line bg-paper"
      :style="{ height: timerLayout(previewState).height + 'px' }"
    >
      <TimerView
        :state="previewState"
        :records="records"
        :busy="false"
        :dark="dark"
        :cancel-session="async () => false"
        :warming-up="true"
      />
    </div>
    <div v-else class="mx-auto max-w-[620px]">
      <HistoryView
        v-if="view === 'history'"
        :records="records"
        :busy="false"
        :current="null"
        :checkpoint="Date.now()"
        :series-id="settings.visual_series"
      />
      <SettingsView
        v-else
        :settings="settings"
        :busy="false"
        :clear-records="
          async () => {
            count = 0;
            return true;
          }
        "
        @save="settings = { ...settings, ...$event }"
        @preference="settings = { ...settings, ...$event }"
        @phrases="settings = { ...settings, phrases: $event }"
        @bean-range="settings = { ...settings, bean_range: $event }"
        @theme-options="saveThemeOptions($event)"
      />
    </div>
  </main>
</template>

<style scoped>
.proof-still :deep(*) {
  animation: none !important;
  transition: none !important;
}
</style>
