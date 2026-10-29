<script setup lang="ts">
import { availableSeries } from '../../shared/series/catalog';
import type { SeriesId } from '../../shared/series/types';
import type { Settings, ThemeOptionsChange } from '../../shared/ipc/types';
import { componentsFor } from '../../shared/series/renderers';
import { copy, locale } from '../../shared/i18n';
import SettingsSelect from '../../shared/ui/SettingsSelect.vue';
defineProps<{ settings: Settings; busy: boolean }>();
const emit = defineEmits<{ theme: [SeriesId]; change: [ThemeOptionsChange] }>();
</script>
<template>
  <section class="mt-4" :aria-label="copy.themeSettings">
    <h2 class="mb-2.5 text-sm font-medium">{{ copy.themeSettings }}</h2>
    <div class="rounded-xl border border-line bg-surface">
      <div class="flex min-h-12 items-center justify-between gap-4 px-3.5 py-2">
        <label for="visual-series" class="text-xs">{{ copy.visualSeries }}</label>
        <SettingsSelect
          id="visual-series"
          :model-value="settings.visual_series"
          :label="copy.visualSeries"
          :options="
            availableSeries.map((series) => ({
              value: series.id,
              label: series.name[locale],
            }))
          "
          :disabled="busy"
          @update:model-value="emit('theme', $event as SeriesId)"
        />
      </div>
      <component
        :is="componentsFor(settings.visual_series).settings"
        :key="settings.visual_series"
        :settings="settings"
        :busy="busy"
        @change="emit('change', { theme_id: settings.visual_series, values: $event })"
      />
    </div>
  </section>
</template>
