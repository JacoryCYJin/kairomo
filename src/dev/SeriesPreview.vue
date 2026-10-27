<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import SeriesCollection from '../shared/series/SeriesCollection.vue';
import SettingsView from '../features/settings/SettingsView.vue';
import HistoryView from '../features/history/HistoryView.vue';
import { locale } from '../shared/i18n';
import type { Settings } from '../shared/ipc/types';
import { previewCounts, seriesFixtures } from './seriesFixtures';
const query = new URLSearchParams(location.search);
const count = ref(Number(query.get('count') || 20));
const view = ref(query.has('gallery') ? 'gallery' : query.get('panel') || 'collection');
const dark = ref(query.get('theme') === 'dark');
const settings = ref<Settings>({
  focus_minutes: 50,
  break_minutes: 10,
  locale: 'zh-CN',
  notifications: false,
  focus_mode: 'manual',
  appearance: 'light',
  visual_series: 'coffee',
});
watch(dark, (value) => (document.documentElement.dataset.theme = value ? 'dark' : 'light'), {
  immediate: true,
});
watch(
  () => settings.value.locale,
  (value) => (locale.value = value),
  { immediate: true },
);
const records = computed(() => seriesFixtures(count.value));
const total = (n: number) => seriesFixtures(n).reduce((sum, s) => sum + s.duration_ms, 0);
const caption = computed(() =>
  locale.value === 'zh-CN'
    ? '测试数据 · 不写入学习记录'
    : 'Test data · never saved to your history',
);
</script>
<template>
  <main class="mx-auto max-w-[1200px] p-6">
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
        <SeriesCollection series-id="coffee" :records="seriesFixtures(n)" :total="total(n)" />
      </section>
    </div>
    <div
      v-else-if="view === 'collection'"
      class="mx-auto w-full max-w-[400px]"
      :data-preview-count="count"
    >
      <SeriesCollection series-id="coffee" :records="records" :total="total(count)" />
    </div>
    <div v-else class="mx-auto max-w-[620px]">
      <HistoryView
        v-if="view === 'history'"
        :records="records"
        :busy="false"
        :current="null"
        :checkpoint="Date.now()"
        series-id="coffee"
      />
      <SettingsView
        v-else
        :settings="settings"
        :busy="false"
        @save="settings = $event"
        @preference="settings = { ...settings, ...$event }"
      />
    </div>
  </main>
</template>
