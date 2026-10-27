<script setup lang="ts">
import { computed } from 'vue';
import { ChevronLeft, ChevronRight } from 'lucide-vue-next';
import { copy, locale, duration, formatBeanCount } from '../../shared/i18n';
import { dateInput, monthGrid, periodFor, totalInPeriod } from './history';
import { isCompletedFocus } from '../../shared/completions';
import type { Period } from './history';
import type { Session } from '../../shared/ipc/types';
const props = defineProps<{
  month: Date;
  selected: string;
  today: string;
  period: Period;
  records: Session[];
  checkpoint: number;
  all: boolean;
}>();
const emit = defineEmits<{ browse: [Date]; select: [string]; today: [] }>();
const days = computed(() => monthGrid(props.month));
const heading = computed(() =>
  props.month.toLocaleDateString(locale.value, { year: 'numeric', month: 'long' }),
);
const weekdays = computed(() =>
  Array.from({ length: 7 }, (_, i) =>
    new Date(2026, 9, 26 + i).toLocaleDateString(locale.value, { weekday: 'narrow' }),
  ),
);
const counts = computed(() => {
  const map = new Map<string, number>();
  for (const s of props.records)
    if (isCompletedFocus(s)) {
      const key = dateInput(new Date(s.ended_at_ms!));
      map.set(key, (map.get(key) ?? 0) + 1);
    }
  return map;
});
function browse(delta: number) {
  emit('browse', new Date(props.month.getFullYear(), props.month.getMonth() + delta, 1));
}
function inRange(day: Date) {
  return !props.all && day.getTime() >= props.period.start && day.getTime() < props.period.end;
}
function description(day: Date) {
  const key = dateInput(day),
    beans = counts.value.get(key) ?? 0;
  return `${day.toLocaleDateString(locale.value)} · ${duration(totalInPeriod(props.records, 'focus', periodFor('day', day), props.checkpoint))} · ${formatBeanCount(beans)}`;
}
</script>
<template>
  <section :aria-label="copy.calendar" class="min-w-0">
    <div class="mb-4 flex items-center justify-between gap-1">
      <span class="whitespace-nowrap text-sm font-medium">{{ heading }}</span>
      <div class="flex shrink-0 items-center gap-0.5">
        <button class="icon-button !size-7" :aria-label="copy.previousMonth" @click="browse(-1)">
          <ChevronLeft :size="15" />
        </button>
        <button class="icon-button !size-7" :aria-label="copy.nextMonth" @click="browse(1)">
          <ChevronRight :size="15" />
        </button>
        <button class="text-button ml-1 whitespace-nowrap" @click="emit('today')">
          {{ copy.backToday }}
        </button>
      </div>
    </div>
    <div class="grid grid-cols-7 gap-y-1" role="group" :aria-label="heading">
      <span v-for="(label, i) in weekdays" :key="i" class="mb-1 text-center text-xs text-faint">{{
        label
      }}</span>
      <button
        v-for="day in days"
        :key="dateInput(day)"
        class="relative flex h-9 min-w-0 flex-col items-center justify-center rounded-lg text-xs tabular-nums enabled:hover:bg-hover disabled:opacity-30"
        :class="
          dateInput(day) === selected
            ? 'bg-selected font-semibold text-ink'
            : day.getMonth() !== month.getMonth()
              ? 'text-faint'
              : inRange(day)
                ? 'bg-surface-muted text-ink'
                : 'text-muted'
        "
        :disabled="dateInput(day) > today"
        :aria-label="description(day)"
        :title="description(day)"
        :aria-pressed="dateInput(day) === selected"
        :aria-current="dateInput(day) === today ? 'date' : undefined"
        @click="emit('select', dateInput(day))"
      >
        <span
          :class="
            dateInput(day) === today ? 'underline decoration-line-strong underline-offset-2' : ''
          "
          >{{ day.getDate() }}</span
        >
        <span
          class="mt-0.5 h-1 w-1 rounded-full"
          :class="counts.get(dateInput(day)) ? 'bg-muted' : 'bg-transparent'"
          aria-hidden="true"
        ></span>
      </button>
    </div>
  </section>
</template>
