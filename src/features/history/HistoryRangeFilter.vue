<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { copy } from '../../shared/i18n';
import { customPeriod } from './history';
import type { HistoryFilter } from './history';
import HistoryDateRange from './HistoryDateRange.vue';
const props = defineProps<{ range: HistoryFilter; from: string; to: string; today: string }>();
const emit = defineEmits<{ range: [HistoryFilter]; custom: [from: string, to: string] }>();
const draftFrom = ref(props.from),
  draftTo = ref(props.to);
const options = computed(
  () =>
    [
      { value: 'day', label: copy.value.rangeDayShort, title: copy.value.selectedDay },
      { value: 'week', label: copy.value.rangeWeekShort, title: copy.value.selectedWeek },
      { value: 'month', label: copy.value.rangeMonthShort, title: copy.value.selectedMonth },
      { value: 'year', label: copy.value.rangeYearShort, title: copy.value.selectedYear },
      { value: 'custom', label: copy.value.rangeCustomShort, title: copy.value.customRange },
    ] as const,
);
const index = computed(() =>
  Math.max(
    0,
    options.value.findIndex((o) => o.value === props.range),
  ),
);
const invalid = computed(
  () => !customPeriod(draftFrom.value, draftTo.value) || draftTo.value > props.today,
);
function resetDraft() {
  draftFrom.value = props.from;
  draftTo.value = props.to;
}
watch(() => [props.from, props.to, props.range], resetDraft);
watch([draftFrom, draftTo], ([from, to]) => {
  if (props.range === 'custom' && !invalid.value && (from !== props.from || to !== props.to))
    emit('custom', from, to);
});
function navigate(event: KeyboardEvent, current: number) {
  let next: number;
  if (event.key === 'ArrowRight' || event.key === 'ArrowDown')
    next = (current + 1) % options.value.length;
  else if (event.key === 'ArrowLeft' || event.key === 'ArrowUp')
    next = (current + options.value.length - 1) % options.value.length;
  else if (event.key === 'Home') next = 0;
  else if (event.key === 'End') next = options.value.length - 1;
  else return;
  event.preventDefault();
  emit('range', options.value[next]!.value);
  (event.currentTarget as HTMLElement).parentElement
    ?.querySelectorAll<HTMLButtonElement>('[role="radio"]')
    [next]?.focus();
}
</script>
<template>
  <div class="mb-5 min-w-0" data-history-range>
    <div class="flex min-w-0 items-center gap-2">
      <span
        id="history-range-label"
        class="shrink-0 whitespace-nowrap text-xs text-muted"
        :title="copy.selectedRange"
        >{{ copy.rangeLabelShort }}</span
      >
      <div
        class="relative grid h-8 min-w-0 flex-1 grid-cols-[repeat(4,minmax(0,1fr))_minmax(0,1.8fr)] items-stretch rounded-lg bg-surface-muted p-0.5"
        role="radiogroup"
        aria-labelledby="history-range-label"
      >
        <span
          aria-hidden="true"
          class="pointer-events-none absolute inset-y-0.5 rounded-md border border-line bg-surface shadow-sm transition-[left,width] duration-150 motion-reduce:transition-none"
          :style="{
            left: 'calc(2px + (100% - 4px) * ' + index + ' / 5.8)',
            width: 'calc((100% - 4px) * ' + (index === 4 ? 1.8 : 1) + ' / 5.8)',
          }"
        />
        <button
          v-for="(option, i) in options"
          :key="option.value"
          type="button"
          role="radio"
          :aria-checked="range === option.value"
          :aria-label="option.title"
          :title="option.title"
          :tabindex="range === option.value ? 0 : -1"
          class="relative z-10 min-w-0 whitespace-nowrap rounded-md text-xs"
          :class="range === option.value ? 'font-medium text-ink' : 'text-muted hover:text-ink'"
          @click="emit('range', option.value)"
          @keydown="navigate($event, i)"
        >
          {{ option.label }}
        </button>
      </div>
    </div>
    <div v-if="range === 'custom'" class="mt-3 min-w-0" data-history-custom>
      <HistoryDateRange v-model:from="draftFrom" v-model:to="draftTo" :today="today" />
      <p v-if="invalid" class="mt-2 text-xs text-danger" role="alert">{{ copy.invalidRange }}</p>
    </div>
  </div>
</template>
