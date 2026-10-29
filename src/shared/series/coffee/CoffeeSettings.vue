<script setup lang="ts">
import type { ThemeOptionValue } from '../../ipc/types';
import { themeOptions } from '../options';
import type { ThemeSettingsProps } from '../contracts';
import { copy } from '../../i18n';
import SettingsSelect from '../../ui/SettingsSelect.vue';
defineProps<ThemeSettingsProps>();
const emit = defineEmits<{ change: [Record<string, ThemeOptionValue>] }>();
</script>
<template>
  <div class="mx-3.5 flex min-h-12 items-center justify-between gap-4 border-t border-line py-2">
    <label for="bean-range" class="text-xs">{{ copy.beanRange }}</label>
    <SettingsSelect
      id="bean-range"
      :model-value="String(themeOptions(settings).range)"
      :label="copy.beanRange"
      :options="[
        { value: 'day', label: copy.beanDay },
        { value: 'week', label: copy.thisWeek },
      ]"
      :disabled="busy"
      @update:model-value="emit('change', { range: $event })"
    />
  </div>
</template>
