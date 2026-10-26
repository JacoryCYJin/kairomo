<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
import { Download, Pencil, Trash2, X, Clock3 } from 'lucide-vue-next';
import { copy, duration, locale, localTime } from '../../shared/i18n';
import { isCompletedFocus } from '../../shared/completions';
import SeriesReward from '../../shared/series/SeriesReward.vue';
import type { SeriesId } from '../../shared/series/types';
import CoffeeCupIcon from '../../shared/ui/CoffeeCupIcon.vue';
import HistoryCalendar from './HistoryCalendar.vue';
import HistoryRangeFilter from './HistoryRangeFilter.vue';
import SeriesCollection from '../../shared/series/SeriesCollection.vue';
import type { Session } from '../../shared/ipc/types';
import {
  dateInput,
  recordStartInPeriod,
  parseDateInput,
  periodFor,
  customPeriod,
  historyRecords,
  totalInPeriod,
  durationInPeriod,
  earnedBeans,
} from './history';
import type { HistoryFilter } from './history';
const props = defineProps<{
  records: Session[];
  busy: boolean;
  current: Session | null;
  checkpoint: number;
  seriesId: SeriesId;
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
const now = ref(new Date()),
  range = ref<HistoryFilter>('day');
const anchor = ref(dateInput(now.value));
const month = ref(new Date(now.value.getFullYear(), now.value.getMonth(), 1));
const from = ref(anchor.value),
  to = ref(anchor.value),
  visibleCount = ref(50);
let clockTick: ReturnType<typeof setInterval> | undefined;
onMounted(() => {
  clockTick = setInterval(() => {
    const next = new Date();
    if (anchor.value === dateInput(now.value) && dateInput(next) !== dateInput(now.value))
      selectDate(dateInput(next));
    now.value = next;
  }, 30000);
});
onUnmounted(() => clearInterval(clockTick));
const anchorDate = computed(() => parseDateInput(anchor.value) ?? now.value);
const period = computed(() =>
  range.value === 'custom'
    ? customPeriod(from.value, to.value)!
    : periodFor(range.value, anchorDate.value),
);
const sessions = computed(() =>
  props.current
    ? [props.current, ...props.records.filter((s) => s.id !== props.current?.id)]
    : props.records,
);
const filtered = computed(() => historyRecords(sessions.value, period.value, props.checkpoint));
const total = computed(() =>
  totalInPeriod(sessions.value, 'focus', period.value, props.checkpoint),
);
const beans = computed(() => earnedBeans(props.records, period.value));
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
watch([range, anchor, from, to], () => {
  visibleCount.value = 50;
});
function selectDate(value: string) {
  const date = parseDateInput(value);
  if (!date || value > dateInput(new Date())) return;
  anchor.value = value;
  range.value = 'day';
  month.value = new Date(date.getFullYear(), date.getMonth(), 1);
}
function applyCustom(start: string, end: string) {
  if (!customPeriod(start, end) || end > dateInput(now.value)) return;
  from.value = start;
  to.value = end;
  range.value = 'custom';
}
function fullDate(date: Date) {
  return date.toLocaleDateString(locale.value, { year: 'numeric', month: 'short', day: 'numeric' });
}
const periodLabel = computed(() => {
  if (range.value === 'all') return [copy.value.allHistory];
  const first = new Date(period.value.start);
  if (range.value === 'day') return [fullDate(first)];
  return [fullDate(first), fullDate(new Date(period.value.end - 1))];
});
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
    <div class="flex items-center justify-between gap-3 pt-5">
      <h1 class="text-xl font-semibold tracking-tight">{{ copy.historyTitle }}</h1>
      <button
        class="button secondary compact"
        :disabled="busy || !records.length"
        @click="emit('export')"
      >
        <Download :size="14" />{{ copy.exportAll }}
      </button>
    </div>
    <HistoryRangeFilter
      class="mt-4"
      :range="range"
      :from="from"
      :to="to"
      :today="dateInput(now)"
      @range="range = $event"
      @custom="applyCustom"
    />
    <div
      class="mb-7 grid grid-cols-[minmax(0,1fr)_minmax(200px,0.95fr)] items-stretch gap-4 border-b border-line pb-5"
    >
      <HistoryCalendar
        :month="month"
        :selected="anchor"
        :today="dateInput(now)"
        :period="period"
        :records="sessions"
        :checkpoint="checkpoint"
        :all="range === 'all'"
        @browse="month = $event"
        @select="selectDate"
        @today="selectDate(dateInput(new Date()))"
      />
      <SeriesCollection
        :records="beans"
        :series-id="seriesId"
        :total="total"
        :period-label="periodLabel"
        fit-height
      />
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
        <SeriesReward
          :series-id="seriesId"
          v-if="isCompletedFocus(s)"
          class="size-5"
          :duration="s.duration_ms"
        /><CoffeeCupIcon v-else-if="s.kind === 'break'" class="size-4 text-muted" /><Clock3
          v-else
          class="size-4 text-muted"
        />
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
