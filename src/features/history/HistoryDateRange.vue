<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue';
import { ChevronLeft, ChevronRight, ChevronsLeft, ChevronsRight } from 'lucide-vue-next';
import { copy, locale } from '../../shared/i18n';
import { dateInput, monthGrid, parseDateInput } from './history';
const props = defineProps<{ from: string; to: string; today: string }>();
const emit = defineEmits<{ 'update:from': [string]; 'update:to': [string] }>();
type Field = 'from' | 'to';
const active = ref<Field | null>(null);
const root = ref<HTMLDivElement | null>(null);
const fromButton = ref<HTMLButtonElement | null>(null);
const toButton = ref<HTMLButtonElement | null>(null);
const month = ref(new Date());
const focused = ref('');
const rows = computed(() => {
  const days = monthGrid(month.value);
  return Array.from({ length: 6 }, (_, i) => days.slice(i * 7, i * 7 + 7));
});
const heading = computed(() =>
  month.value.toLocaleDateString(locale.value, { year: 'numeric', month: 'long' }),
);
const weekdays = computed(() =>
  Array.from({ length: 7 }, (_, i) => ({
    short: new Date(2026, 9, 26 + i).toLocaleDateString(locale.value, { weekday: 'narrow' }),
    full: new Date(2026, 9, 26 + i).toLocaleDateString(locale.value, { weekday: 'long' }),
  })),
);
const minimum = computed(() => (active.value === 'to' ? props.from : ''));
const selected = computed(() => (active.value === 'from' ? props.from : props.to));
function label(value: string) {
  const date = parseDateInput(value);
  return (
    date?.toLocaleDateString(locale.value, {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
    }) ?? value
  );
}
function disabled(day: Date) {
  const key = dateInput(day);
  return key > props.today || (!!minimum.value && key < minimum.value);
}
function close(restore = true) {
  const button = active.value === 'from' ? fromButton.value : toButton.value;
  active.value = null;
  if (restore) button?.focus({ preventScroll: true });
}
async function focusDay(date: Date) {
  let key = dateInput(date);
  if (key > props.today) key = props.today;
  if (minimum.value && key < minimum.value) key = minimum.value;
  const day = parseDateInput(key)!;
  focused.value = key;
  month.value = new Date(day.getFullYear(), day.getMonth(), 1);
  await nextTick();
  root.value?.querySelector<HTMLButtonElement>(`[data-date="${key}"]`)?.focus();
}
async function open(field: Field) {
  if (active.value === field) return close();
  active.value = field;
  await focusDay(parseDateInput(props[field]) ?? parseDateInput(props.today)!);
}
function choose(day: Date) {
  if (!active.value || disabled(day)) return;
  if (active.value === 'from') emit('update:from', dateInput(day));
  else emit('update:to', dateInput(day));
  close();
}
function browse(delta: number) {
  const target = new Date(month.value.getFullYear(), month.value.getMonth() + delta, 1);
  const max = parseDateInput(props.today)!;
  const min = parseDateInput(minimum.value);
  if (target > max) target.setFullYear(max.getFullYear(), max.getMonth(), 1);
  if (min && target < new Date(min.getFullYear(), min.getMonth(), 1))
    target.setFullYear(min.getFullYear(), min.getMonth(), 1);
  month.value = target;
  focused.value = dateInput(min && target < min ? min : target);
}
const previousDisabled = computed(
  () =>
    !!minimum.value &&
    dateInput(new Date(month.value.getFullYear(), month.value.getMonth(), 1)).slice(0, 7) <=
      minimum.value.slice(0, 7),
);
const nextDisabled = computed(() => dateInput(month.value).slice(0, 7) >= props.today.slice(0, 7));
function navigate(event: KeyboardEvent, day: Date) {
  const next = new Date(day);
  const offsets: Record<string, number> = {
    ArrowLeft: -1,
    ArrowRight: 1,
    ArrowUp: -7,
    ArrowDown: 7,
  };
  if (event.key in offsets) next.setDate(next.getDate() + offsets[event.key]);
  else if (event.key === 'Home') next.setDate(next.getDate() - ((next.getDay() + 6) % 7));
  else if (event.key === 'End') next.setDate(next.getDate() + 6 - ((next.getDay() + 6) % 7));
  else if (event.key === 'PageUp' || event.key === 'PageDown') {
    const delta = (event.key === 'PageUp' ? -1 : 1) * (event.shiftKey ? 12 : 1);
    next.setDate(1);
    next.setMonth(next.getMonth() + delta);
    const last = new Date(next.getFullYear(), next.getMonth() + 1, 0).getDate();
    next.setDate(Math.min(day.getDate(), last));
  } else return;
  event.preventDefault();
  void focusDay(next);
}
function outside(event: PointerEvent) {
  if (active.value && event.target instanceof Node && !root.value?.contains(event.target))
    close(false);
}
function focusOut(event: FocusEvent) {
  if (
    active.value &&
    event.relatedTarget instanceof Node &&
    !root.value?.contains(event.relatedTarget)
  )
    close(false);
}
onMounted(() => document.addEventListener('pointerdown', outside));
onUnmounted(() => document.removeEventListener('pointerdown', outside));
</script>
<template>
  <div ref="root" class="relative min-w-0" @focusout="focusOut" @keydown.esc.stop.prevent="close()">
    <div class="grid min-w-0 grid-cols-2 gap-2">
      <div class="min-w-0">
        <label id="range-from-label" class="text-xs text-muted">{{ copy.rangeStart }}</label>
        <button
          ref="fromButton"
          type="button"
          class="mt-1 block h-8 w-full rounded-lg border border-line bg-surface-muted px-2 text-left text-xs tabular-nums hover:bg-hover"
          aria-labelledby="range-from-label range-from-value"
          aria-haspopup="dialog"
          :aria-expanded="active === 'from'"
          @click="open('from')"
        >
          <span id="range-from-value">{{ label(from) }}</span>
        </button>
      </div>
      <div class="min-w-0">
        <label id="range-to-label" class="text-xs text-muted">{{ copy.rangeEnd }}</label>
        <button
          ref="toButton"
          type="button"
          class="mt-1 block h-8 w-full rounded-lg border border-line bg-surface-muted px-2 text-left text-xs tabular-nums hover:bg-hover"
          aria-labelledby="range-to-label range-to-value"
          aria-haspopup="dialog"
          :aria-expanded="active === 'to'"
          @click="open('to')"
        >
          <span id="range-to-value">{{ label(to) }}</span>
        </button>
      </div>
    </div>
    <div
      v-if="active"
      class="absolute inset-x-0 top-full z-20 mt-2 rounded-xl border border-line bg-surface p-2 shadow-dialog"
      role="dialog"
      :aria-label="active === 'from' ? copy.rangeStart : copy.rangeEnd"
    >
      <div class="mb-2 flex items-center justify-between gap-0.5">
        <div class="flex">
          <button
            class="icon-button !size-6"
            type="button"
            :disabled="previousDisabled"
            :aria-label="copy.previousYear"
            @click="browse(-12)"
          >
            <ChevronsLeft :size="13" />
          </button>
          <button
            class="icon-button !size-6"
            type="button"
            :disabled="previousDisabled"
            :aria-label="copy.previousMonth"
            @click="browse(-1)"
          >
            <ChevronLeft :size="13" />
          </button>
        </div>
        <span class="whitespace-nowrap text-xs font-medium" aria-live="polite">{{ heading }}</span>
        <div class="flex">
          <button
            class="icon-button !size-6"
            type="button"
            :disabled="nextDisabled"
            :aria-label="copy.nextMonth"
            @click="browse(1)"
          >
            <ChevronRight :size="13" />
          </button>
          <button
            class="icon-button !size-6"
            type="button"
            :disabled="nextDisabled"
            :aria-label="copy.nextYear"
            @click="browse(12)"
          >
            <ChevronsRight :size="13" />
          </button>
        </div>
      </div>
      <div role="grid" :aria-label="heading">
        <div class="grid grid-cols-7" role="row">
          <span
            v-for="(day, i) in weekdays"
            :key="i"
            role="columnheader"
            :aria-label="day.full"
            class="pb-1 text-center text-xs text-faint"
            >{{ day.short }}</span
          >
        </div>
        <div v-for="(week, i) in rows" :key="i" class="grid grid-cols-7" role="row">
          <button
            v-for="day in week"
            :key="dateInput(day)"
            type="button"
            role="gridcell"
            class="flex h-7 min-w-0 items-center justify-center rounded-md text-xs tabular-nums enabled:hover:bg-hover disabled:opacity-25"
            :class="
              dateInput(day) === selected
                ? 'bg-selected font-semibold text-ink'
                : day.getMonth() === month.getMonth()
                  ? 'text-ink'
                  : 'text-faint'
            "
            :data-date="dateInput(day)"
            :tabindex="dateInput(day) === focused ? 0 : -1"
            :aria-label="
              day.toLocaleDateString(locale, { year: 'numeric', month: 'long', day: 'numeric' })
            "
            :aria-selected="dateInput(day) === selected"
            :aria-current="dateInput(day) === today ? 'date' : undefined"
            :disabled="disabled(day)"
            @click="choose(day)"
            @keydown="navigate($event, day)"
          >
            {{ day.getDate() }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
