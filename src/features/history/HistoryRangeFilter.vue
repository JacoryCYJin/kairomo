<script setup lang="ts">
import { ref, watch } from 'vue';
import { copy } from '../../shared/i18n';
import { customPeriod } from './history';
import type { HistoryFilter } from './history';
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
function apply() {
  invalid.value = !customPeriod(draftFrom.value, draftTo.value) || draftTo.value > props.today;
  if (!invalid.value) emit('custom', draftFrom.value, draftTo.value);
}
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
    <form v-if="range === 'custom'" class="mt-3" novalidate @submit.prevent="apply">
      <div class="flex flex-wrap items-end gap-2">
        <label class="min-w-0 text-xs text-muted"
          >{{ copy.rangeStart
          }}<input
            v-model="draftFrom"
            class="mt-1 block w-36 rounded-lg border border-line bg-surface-muted px-2 py-1.5 text-xs"
            type="date"
            :max="today"
            required
        /></label>
        <label class="min-w-0 text-xs text-muted"
          >{{ copy.rangeEnd
          }}<input
            v-model="draftTo"
            class="mt-1 block w-36 rounded-lg border border-line bg-surface-muted px-2 py-1.5 text-xs"
            type="date"
            :min="draftFrom"
            :max="today"
            required
        /></label>
        <button class="button secondary compact" type="submit">{{ copy.applyRange }}</button>
      </div>
      <p v-if="invalid" class="mt-2 text-xs text-danger" role="alert">{{ copy.invalidRange }}</p>
    </form>
  </div>
</template>
