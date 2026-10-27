<script setup lang="ts">
import { ref, watch } from 'vue';
import { copy } from '../../shared/i18n';
import { customPeriod } from './history';
import type { HistoryFilter } from './history';
import HistoryDateRange from './HistoryDateRange.vue';
const props = defineProps<{ range: HistoryFilter; from: string; to: string; today: string }>();
const emit = defineEmits<{ range: [HistoryFilter]; custom: [from: string, to: string] }>();
const draftFrom = ref(props.from),
  draftTo = ref(props.to),
  invalid = ref(false);
watch(
  () => [props.from, props.to],
  () => {
    draftFrom.value = props.from;
    draftTo.value = props.to;
    invalid.value = false;
  },
);
watch([draftFrom, draftTo], ([from, to]) => {
  if (props.range !== 'custom') return;
  invalid.value = !customPeriod(from, to) || to > props.today;
  if (!invalid.value && (from !== props.from || to !== props.to)) emit('custom', from, to);
});
</script>
<template>
  <div class="mb-5">
    <div class="flex flex-wrap items-center gap-2">
      <label for="history-range" class="text-xs text-muted">{{ copy.selectedRange }}</label>
      <select
        id="history-range"
        class="rounded-lg border border-line bg-surface-muted px-3 py-2 text-xs"
        :value="range"
        @change="emit('range', ($event.target as HTMLSelectElement).value as HistoryFilter)"
      >
        <option value="day">{{ copy.selectedDay }}</option>
        <option value="week">{{ copy.selectedWeek }}</option>
        <option value="month">{{ copy.selectedMonth }}</option>
        <option value="year">{{ copy.selectedYear }}</option>
        <option value="custom">{{ copy.customRange }}</option>
        <option value="all">{{ copy.allHistory }}</option>
      </select>
    </div>
    <div v-if="range === 'custom'" class="mt-3">
      <HistoryDateRange v-model:from="draftFrom" v-model:to="draftTo" :today="today" />
      <p v-if="invalid" class="mt-2 text-xs text-danger" role="alert">{{ copy.invalidRange }}</p>
    </div>
  </div>
</template>
