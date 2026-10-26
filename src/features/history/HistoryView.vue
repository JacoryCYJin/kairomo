<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
import {
  Download,
  Pencil,
  Trash2,
  ArrowUpRight,
  X,
  Clock3,
  ChevronLeft,
  ChevronRight,
} from 'lucide-vue-next';
import { copy, duration, locale, localTime } from '../../shared/i18n';
import type { Session } from '../../shared/ipc/types';
import {
  dateInput,
  recordStartInPeriod,
  parseDateInput,
  periodFor,
  shiftPeriod,
  historyRecords,
  historyBuckets,
  totalInPeriod,
  durationInPeriod,
} from './history';
import type { HistoryRange, HistoryBucket } from './history';
const props = defineProps<{
  records: Session[];
  busy: boolean;
  current: Session | null;
  checkpoint: number;
}>();
const emit = defineEmits<{
  export: [];
  remove: [string];
  correct: [id: string, title: string, minutes: number];
}>();
const editing = ref<Session | null>(null),
  deleting = ref<Session | null>(null);
const editTitle = ref(''),
  editMinutes = ref(1),
  invalid = ref(false);
const modal = ref<HTMLDialogElement | null>(null);
const range = ref<HistoryRange>('day');
const anchor = ref(dateInput(new Date()));
const now = ref(new Date());
const visibleCount = ref(50);
let clockTick: ReturnType<typeof setInterval> | undefined;
onMounted(() => {
  clockTick = setInterval(() => {
    const next = new Date();
    if (anchor.value === dateInput(now.value)) anchor.value = dateInput(next);
    now.value = next;
  }, 30000);
});
onUnmounted(() => {
  clearInterval(clockTick);
});
const anchorDate = computed(() => parseDateInput(anchor.value) ?? now.value);
const period = computed(() => periodFor(range.value, anchorDate.value));
const sessions = computed(() =>
  props.current
    ? [props.current, ...props.records.filter((s) => s.id !== props.current?.id)]
    : props.records,
);
const filtered = computed(() => historyRecords(sessions.value, period.value, props.checkpoint));
const total = computed(() =>
  totalInPeriod(sessions.value, 'focus', period.value, props.checkpoint),
);
const restTotal = computed(() =>
  totalInPeriod(sessions.value, 'break', period.value, props.checkpoint),
);
const buckets = computed(() =>
  historyBuckets(sessions.value, range.value, anchorDate.value, props.checkpoint),
);
const max = computed(() => Math.max(60000, ...buckets.value.map((b) => b.value)));
const isCurrent = computed(
  () => range.value === 'all' || period.value.start === periodFor(range.value, now.value).start,
);
const groups = computed(() => {
  const result: { date: string; records: Session[] }[] = [];
  for (const session of filtered.value.slice(0, visibleCount.value)) {
    const date = dateInput(new Date(recordStartInPeriod(session, period.value, props.checkpoint)));
    let group = result.find((g) => g.date === date);
    if (!group) {
      group = { date, records: [] };
      result.push(group);
    }
    group.records.push(session);
  }
  return result.sort((a, b) => b.date.localeCompare(a.date));
});
watch([range, anchor], () => {
  visibleCount.value = 50;
});
function selectRange(next: HistoryRange) {
  range.value = next;
  anchor.value = dateInput(now.value);
}
function move(direction: number) {
  const next = shiftPeriod(range.value, anchorDate.value, direction);
  if (periodFor(range.value, next).start <= periodFor(range.value, now.value).start)
    anchor.value = dateInput(next);
}
function chooseDate(event: Event) {
  const value = (event.target as HTMLInputElement).value;
  const date = parseDateInput(value);
  if (date && value <= dateInput(now.value)) anchor.value = value;
  else (event.target as HTMLInputElement).value = anchor.value;
}
function fullDate(date: Date) {
  return date.toLocaleDateString(locale.value, { year: 'numeric', month: 'short', day: 'numeric' });
}
const periodLabel = computed(() => {
  if (range.value === 'all') return copy.value.allHistory;
  const start = new Date(period.value.start);
  if (range.value === 'month')
    return start.toLocaleDateString(locale.value, { year: 'numeric', month: 'long' });
  return range.value === 'day'
    ? fullDate(start)
    : fullDate(start) + ' – ' + fullDate(new Date(period.value.end - 1));
});
function bucketLabel(bucket: HistoryBucket) {
  if (bucket.unit === 'hour')
    return bucket.date.toLocaleTimeString(locale.value, { hour: '2-digit', hour12: false });
  if (bucket.unit === 'year') return String(bucket.date.getFullYear());
  if (bucket.unit === 'month')
    return bucket.date.toLocaleDateString(locale.value, { year: '2-digit', month: 'numeric' });
  if (range.value === 'week')
    return bucket.date.toLocaleDateString(locale.value, { weekday: 'narrow' });
  return String(bucket.date.getDate());
}
function showBucketLabel(index: number) {
  return index % Math.ceil(buckets.value.length / 7) === 0 || index === buckets.value.length - 1;
}
function recordTime(session: Session) {
  const start = new Date(session.started_at_ms);
  const end = session.ended_at_ms === null ? null : new Date(session.ended_at_ms);
  return (
    fullDate(start) +
    ' ' +
    localTime(start.getTime()) +
    ' — ' +
    (end
      ? (dateInput(start) === dateInput(end) ? '' : fullDate(end) + ' ') + localTime(end.getTime())
      : copy.value.ongoing)
  );
}
async function edit(s: Session) {
  editing.value = s;
  deleting.value = null;
  editTitle.value = s.title;
  editMinutes.value = Math.max(0.01, Math.round(s.duration_ms / 600) / 100);
  invalid.value = false;
  await nextTick();
  modal.value?.showModal();
}
async function askDelete(s: Session) {
  deleting.value = s;
  editing.value = null;
  await nextTick();
  modal.value?.showModal();
}
function close() {
  modal.value?.close();
  editing.value = null;
  deleting.value = null;
}
function submit() {
  if (!Number.isFinite(editMinutes.value) || editMinutes.value < 0.01 || editMinutes.value > 1440) {
    invalid.value = true;
    return;
  }
  if (editing.value) emit('correct', editing.value.id, editTitle.value, editMinutes.value);
  close();
}
</script>
<template>
  <section class="pb-6">
    <div class="pt-5">
      <div class="flex items-center justify-between gap-3">
        <h1 class="mb-2 text-xl font-semibold tracking-tight">{{ copy.historyTitle }}</h1>
        <button
          class="button secondary compact"
          :disabled="busy || !records.length"
          @click="emit('export')"
        >
          <Download :size="14" />{{ copy.exportAll }}
        </button>
      </div>
    </div>
    <div
      class="my-4 inline-flex gap-1 rounded-xl bg-surface-muted p-1"
      role="group"
      :aria-label="copy.historyRange"
    >
      <button
        v-for="option in ['day', 'week', 'month', 'all'] as const"
        :key="option"
        class="rounded-lg px-3 py-1.5 text-xs"
        :class="range === option ? 'bg-surface text-ink' : 'text-muted'"
        :aria-pressed="range === option"
        @click="selectRange(option)"
      >
        {{
          { day: copy.today, week: copy.thisWeek, month: copy.thisMonth, all: copy.allHistory }[
            option
          ]
        }}
      </button>
    </div>
    <div class="flex min-h-8 flex-wrap items-center justify-between gap-2">
      <span class="text-xs text-muted">{{ periodLabel }}</span>
      <div v-if="range !== 'all'" class="flex items-center gap-1">
        <button
          class="icon-button"
          :aria-label="copy.previousPeriod"
          :title="copy.previousPeriod"
          @click="move(-1)"
        >
          <ChevronLeft :size="16" />
        </button>
        <input
          class="w-32 min-w-0 rounded-lg border border-line bg-surface-muted px-1.5 py-1 text-xs"
          type="date"
          :value="anchor"
          :max="dateInput(now)"
          :aria-label="copy.chooseDate"
          @change="chooseDate"
        />
        <button
          class="icon-button"
          :disabled="isCurrent"
          :aria-label="copy.nextPeriod"
          :title="copy.nextPeriod"
          @click="move(1)"
        >
          <ChevronRight :size="16" />
        </button>
        <button v-if="!isCurrent" class="text-button" @click="anchor = dateInput(now)">
          {{ copy.currentPeriod }}
        </button>
      </div>
    </div>
    <div class="mb-7 mt-4 flex gap-4 rounded-2xl bg-surface-muted p-4 wide:gap-7 wide:p-5">
      <div class="flex w-28 shrink-0 flex-col justify-center gap-3 wide:w-36">
        <span class="eyebrow">{{ copy.total }}</span
        ><strong class="text-2xl font-medium tracking-tight tabular-nums">{{
          duration(total)
        }}</strong
        ><span class="flex items-center gap-2 text-xs"
          >{{ filtered.length }} {{ copy.recordCount }} <ArrowUpRight :size="13"
        /></span>
        <span class="text-xs text-muted">{{ copy.breakLegend }} · {{ duration(restTotal) }}</span>
      </div>
      <div class="min-w-0 flex-1">
        <div class="mb-3 flex items-center justify-between text-xs text-muted">
          <span class="eyebrow">{{ range === 'all' ? copy.allHistory : copy.periodFocus }}</span
          ><span>{{ copy.focusLegend }}</span>
        </div>
        <div class="flex h-28 justify-between" :class="buckets.length > 14 ? 'gap-1' : 'gap-3'">
          <div
            v-for="(day, index) in buckets"
            :key="day.start"
            class="flex min-w-0 flex-1 flex-col items-center gap-1.5 text-xs text-muted"
            :title="`${fullDate(day.date)}${day.unit === 'hour' ? ' ' + bucketLabel(day) : ''}: ${duration(day.value)}`"
          >
            <span v-if="buckets.length <= 14" class="text-xs tabular-nums">{{
              Math.floor(day.value / 60000) || '—'
            }}</span>
            <div class="flex w-full max-w-5 flex-1 items-end">
              <div
                class="min-h-0.5 w-full rounded-sm bg-faint opacity-60"
                :style="{ height: day.value ? `${Math.max(3, (day.value / max) * 100)}%` : '2px' }"
              ></div>
            </div>
            <span class="whitespace-nowrap" :class="{ invisible: !showBucketLabel(index) }">{{
              bucketLabel(day)
            }}</span>
          </div>
        </div>
      </div>
    </div>
    <div class="mb-2 flex items-center justify-between">
      <span class="eyebrow">{{ copy.historyEntries }}</span
      ><span class="text-xs text-faint tabular-nums">{{
        filtered.length.toString().padStart(2, '0')
      }}</span>
    </div>
    <div v-if="!filtered.length" class="py-14 text-center text-faint">
      <Clock3 class="mx-auto" :size="30" stroke-width="1" />
      <h2 class="mb-2 mt-4 text-sm font-medium text-muted">{{ copy.noPeriodRecords }}</h2>
    </div>
    <section v-for="group in groups" :key="group.date" class="history-day">
      <h2 class="mb-1 mt-5 text-xs font-medium text-muted">
        {{ fullDate(parseDateInput(group.date)!) }}
      </h2>
      <div
        v-for="s in group.records"
        :key="s.id"
        class="group -mx-2 flex items-center gap-2 rounded-lg px-2 py-3 hover:bg-surface-muted wide:gap-2.5"
      >
        <span class="record-dot" :class="{ rest: s.kind === 'break' }"></span>
        <div class="min-w-0 flex-1">
          <strong class="block truncate text-sm font-normal">{{ s.title || copy.untitled }}</strong
          ><span class="mt-1.5 block text-xs text-faint"
            >{{ recordTime(s) }}<span v-if="s.corrected"> · {{ copy.corrected }}</span></span
          >
        </div>
        <span class="hidden shrink-0 text-xs text-muted wide:inline">{{
          s.kind === 'focus' ? copy.focus : copy.resting
        }}</span
        ><strong
          class="mr-1 w-14 shrink-0 text-right text-xs font-medium tabular-nums wide:w-16"
          :title="copy.sessionTotal + ' · ' + duration(s.duration_ms)"
          >{{ duration(durationInPeriod(s, period, checkpoint)) }}</strong
        ><button
          class="icon-button opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 [@media(hover:none)]:opacity-100"
          :disabled="busy"
          v-if="s.id !== current?.id"
          :aria-label="copy.edit"
          :title="copy.edit"
          @click="edit(s)"
        >
          <Pencil :size="14" /></button
        ><button
          class="icon-button opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 [@media(hover:none)]:opacity-100"
          :disabled="busy"
          v-if="s.id !== current?.id"
          :aria-label="copy.remove"
          :title="copy.remove"
          @click="askDelete(s)"
        >
          <Trash2 :size="14" />
        </button>
      </div>
    </section>
    <div v-if="filtered.length > visibleCount" class="mt-5 flex justify-center">
      <button class="button secondary compact" @click="visibleCount += 50">
        {{ copy.loadMore }}
      </button>
    </div>
    <dialog ref="modal" class="modal" @cancel="close">
      <form v-if="editing" @submit.prevent="submit">
        <div class="modal-heading">
          <h2>{{ copy.edit }}</h2>
          <button type="button" class="icon-button" :aria-label="copy.cancel" @click="close">
            <X :size="18" />
          </button>
        </div>
        <label>{{ copy.title }}<input v-model="editTitle" maxlength="160" autofocus /></label
        ><label
          >{{ copy.duration
          }}<input
            v-model.number="editMinutes"
            type="number"
            min="0.01"
            max="1440"
            step="0.01"
            required
        /></label>
        <p v-if="invalid" role="alert">{{ copy.invalidDuration }}</p>
        <div class="modal-actions">
          <button type="button" class="button secondary" @click="close">{{ copy.cancel }}</button
          ><button class="button primary" type="submit">{{ copy.save }}</button>
        </div>
      </form>
      <div v-if="deleting">
        <h2>{{ copy.confirmDelete }}</h2>
        <p>{{ copy.deleteHint }}</p>
        <div class="modal-actions">
          <button class="button secondary" @click="close">{{ copy.cancel }}</button
          ><button
            class="button danger"
            @click="
              emit('remove', deleting.id);
              close();
            "
          >
            {{ copy.remove }}
          </button>
        </div>
      </div>
    </dialog>
  </section>
</template>
