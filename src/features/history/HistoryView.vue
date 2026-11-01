<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
import { Download, Pencil, Trash2, X, Clock3 } from 'lucide-vue-next';
import MultiSelect from '../../shared/ui/MultiSelect.vue';
import SettingsSelect from '../../shared/ui/SettingsSelect.vue';
import { matchesTasks, recordsForTasks, uncategorizedTask } from './taskFilter';
import { copy, duration, locale, localTime } from '../../shared/i18n';
import { isCompletedFocus } from '../../shared/completions';
import SeriesReward from '../../shared/series/SeriesReward.vue';
import type { SeriesId } from '../../shared/series/types';
import HistoryCalendar from './HistoryCalendar.vue';
import RecordTimeField from './RecordTimeField.vue';
import HistoryRangeFilter from './HistoryRangeFilter.vue';
import HistoryTimeline from './HistoryTimeline.vue';
import RecordDetailPanel from './RecordDetailPanel.vue';
import SeriesCollection from '../../shared/series/SeriesCollection.vue';
import {
  effectiveMinutes,
  localDateTime,
  editTimestamp,
  correctionDuration,
  overlappingRecord,
} from './recordCorrection';
import type { Task, Session } from '../../shared/ipc/types';
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
  TIMELINE_MAX_SPAN,
} from './history';
import type { HistoryFilter } from './history';
const props = defineProps<{
  correctRecord: (
    id: string,
    start: number | null,
    end: number | null,
    taskId: string | null,
  ) => Promise<string | null>;
  records: Session[];
  tasks?: Task[];
  busy: boolean;
  current: Session | null;
  checkpoint: number;
  seriesId: SeriesId;
}>();
const emit = defineEmits<{
  export: [];
  remove: [string];
}>();
const selectedTasks = ref<string[]>([]);
const historyView = ref<'list' | 'timeline'>('list');
const timelineSpan = ref(TIMELINE_MAX_SPAN);
const detail = ref<Session | null>(null);
const detailOpen = ref(false);
const detailLeaving = ref(false);
const listActionsVisible = computed(() => !detailOpen.value && !detailLeaving.value);
const viewOptions = computed(() => [
  { value: 'list' as const, title: copy.value.viewList, path: 'M4 6h16M4 12h16M4 18h16' },
  {
    value: 'timeline' as const,
    title: copy.value.viewTimeline,
    path: 'M3 12h18M6 7h5v10H6zM14 10h4v4h-4z',
  },
]);
const viewIndex = computed(() => (historyView.value === 'list' ? 0 : 1));
const taskOptions = computed(() => [
  ...(props.tasks ?? []).map((task) => ({ value: task.id, label: task.name })),
  { value: uncategorizedTask, label: copy.value.uncategorized },
]);
watch(
  () => taskOptions.value.map((option) => option.value).join('\0'),
  () => {
    const available = new Set(taskOptions.value.map((option) => option.value));
    selectedTasks.value = selectedTasks.value.filter((id) => available.has(id));
  },
);
const taskRecords = computed(() => recordsForTasks(props.records, selectedTasks.value));
const taskName = (id?: string | null) =>
  props.tasks?.find((task) => task.id === id)?.name ?? copy.value.uncategorized;
function recordName(session: Session) {
  return session.title || (session.task_id ? taskName(session.task_id) : copy.value.untitled);
}
function recordMeta(session: Session) {
  return `${recordTime(session)}${
    session.title && session.task_id ? ` · ${taskName(session.task_id)}` : ''
  }`;
}
function openDetail(session: Session) {
  detail.value = session;
  detailOpen.value = true;
}
function fromTimeline(session: Session) {
  historyView.value = 'list';
  openDetail(session);
}
function navigateView(event: KeyboardEvent, current: number) {
  if (event.key !== 'ArrowRight' && event.key !== 'ArrowLeft') return;
  event.preventDefault();
  historyView.value = current === 0 ? 'timeline' : 'list';
  (event.currentTarget as HTMLElement).parentElement
    ?.querySelectorAll<HTMLButtonElement>('[role="radio"]')
    [viewIndex.value]?.focus();
}
const editTask = ref<string | null>(null);
function removeTaskFilter(id: string) {
  selectedTasks.value = selectedTasks.value.filter((value) => value !== id);
}
const editing = ref<Session | null>(null),
  deleting = ref<Session | null>(null);
const editStart = ref(''),
  editEnd = ref(''),
  saveError = ref(''),
  saving = ref(false);
const editedBounds = computed(() => {
  const s = editing.value;
  return s && s.ended_at_ms !== null
    ? {
        start: editTimestamp(editStart.value, s.started_at_ms),
        end: editTimestamp(editEnd.value, s.ended_at_ms),
      }
    : { start: null, end: null };
});
const editPreview = computed(() => {
  const s = editing.value;
  const { start, end } = editedBounds.value;
  if (!s || start === null || end === null || end <= start)
    return { error: copy.value.recordTimeInvalid, duration: null };
  const changed = start !== s.started_at_ms || end !== s.ended_at_ms;
  const effective = correctionDuration(s, start, end);
  if (!changed) return { error: '', duration: effective };
  if (end > Date.now()) return { error: copy.value.recordTimeFuture, duration: effective };
  if (effective <= 0 || effective > 86400000)
    return { error: copy.value.recordNoEffective, duration: effective };
  const overlap = overlappingRecord(props.records, s.id, start, end);
  if (overlap) {
    const format = (ms: number) =>
      new Date(ms).toLocaleString(locale.value, {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
        hour12: false,
      });
    return {
      error:
        copy.value.recordOverlap +
        format(overlap.started_at_ms) +
        ' — ' +
        format(overlap.ended_at_ms!),
      duration: effective,
    };
  }
  const current = props.current;
  if (current?.kind === 'focus' && current.intervals.length && end > current.started_at_ms)
    return { error: copy.value.recordOverlapCurrent, duration: effective };
  return { error: '', duration: effective };
});
const editDurationLabel = computed(() => {
  if (editPreview.value.duration === null) return '—';
  const minutes = effectiveMinutes(editPreview.value.duration);
  if (minutes === null) return copy.value.recordUnderMinute;
  return minutes < 60
    ? copy.value.recordApproxMinutes.replace('{count}', String(minutes))
    : copy.value.recordApproxHours
        .replace('{hours}', String(Math.floor(minutes / 60)))
        .replace('{minutes}', String(minutes % 60));
});
watch([editStart, editEnd, editTask], () => {
  saveError.value = '';
});
const modal = ref<HTMLDialogElement | null>(null);
const deleteHeading = ref<HTMLHeadingElement | null>(null);
const loadTrigger = ref<HTMLDivElement | null>(null);
const listScroller = ref<HTMLDivElement | null>(null);
const historyRoot = ref<HTMLElement | null>(null);
const recordsVisible = ref(false);
const recordsExpanded = ref(false);
const coverRadius = ref(16);
function updateCover() {
  const root = historyRoot.value;
  if (!root) return;
  recordsVisible.value = root.scrollTop > 0;
  recordsExpanded.value = root.scrollTop >= root.clientHeight - 1;
  coverRadius.value = Math.min(16, Math.max(0, (root.clientHeight - root.scrollTop) / 4));
}
const calendarColumn = ref<HTMLDivElement | null>(null);
const collectionHeight = ref<number>();
let columnObserver: ResizeObserver | undefined;
let loadObserver: IntersectionObserver | undefined;
const now = ref(new Date()),
  range = ref<HistoryFilter>('day');
const anchor = ref(dateInput(now.value));
const month = ref(new Date(now.value.getFullYear(), now.value.getMonth(), 1));
const from = ref(anchor.value),
  to = ref(anchor.value),
  visibleCount = ref(50);
let clockTick: ReturnType<typeof setInterval> | undefined;
onMounted(() => {
  if (calendarColumn.value) {
    columnObserver = new ResizeObserver(() => {
      const column = calendarColumn.value;
      if (!column) return;
      const custom = column.querySelector<HTMLElement>('[data-history-custom]');
      const extra = custom
        ? custom.getBoundingClientRect().height + parseFloat(getComputedStyle(custom).marginTop)
        : 0;
      collectionHeight.value = column.getBoundingClientRect().height - extra;
    });
    columnObserver.observe(calendarColumn.value);
  }
  clockTick = setInterval(() => {
    const next = new Date();
    if (anchor.value === dateInput(now.value) && dateInput(next) !== dateInput(now.value))
      selectDate(dateInput(next));
    now.value = next;
  }, 30000);
});
onUnmounted(() => {
  columnObserver?.disconnect();
  clearInterval(clockTick);
  loadObserver?.disconnect();
});
const anchorDate = computed(() => parseDateInput(anchor.value) ?? now.value);
const period = computed(() =>
  range.value === 'custom'
    ? customPeriod(from.value, to.value)!
    : periodFor(range.value, range.value === 'month' ? month.value : anchorDate.value),
);
watch(period, (next, previous) => {
  if (range.value !== 'month' || (next.start === previous.start && next.end === previous.end))
    return;
  visibleCount.value = 50;
  detailOpen.value = false;
  if (listScroller.value) listScroller.value.scrollTop = 0;
});
const sessions = computed(() =>
  props.current && matchesTasks(props.current, selectedTasks.value)
    ? [props.current, ...taskRecords.value.filter((s) => s.id !== props.current?.id)]
    : taskRecords.value,
);
const filtered = computed(() =>
  historyRecords(taskRecords.value, period.value, props.checkpoint).filter(isCompletedFocus),
);
const total = computed(() =>
  totalInPeriod(sessions.value, 'focus', period.value, props.checkpoint),
);
const completedRecords = computed(() => earnedBeans(taskRecords.value, period.value));
const groups = computed(() => {
  const result: { date: string; label: string; records: Session[] }[] = [];
  for (const session of filtered.value.slice(0, visibleCount.value)) {
    const date = dateInput(new Date(recordStartInPeriod(session, period.value, props.checkpoint)));
    let group = result.find((g) => g.date === date);
    if (!group) {
      group = { date, label: fullDate(parseDateInput(date)!), records: [] };
      result.push(group);
    }
    group.records.push(session);
  }
  return result.sort((a, b) => b.date.localeCompare(a.date));
});
watch([range, anchor, from, to, selectedTasks], () => {
  visibleCount.value = 50;
  detailOpen.value = false;
});
watch(detailOpen, async (open) => {
  if (!open) return;
  await nextTick();
  const list = listScroller.value;
  const record = list?.querySelector<HTMLElement>(`[data-record-id="${detail.value?.id}"]`);
  if (!list || !record) return;
  const bounds = list.getBoundingClientRect();
  const row = record.getBoundingClientRect();
  // Keep selection visible inside the list without moving the covering layer.
  if (row.top < bounds.top) list.scrollTop += row.top - bounds.top;
  else if (row.bottom > bounds.bottom) list.scrollTop += row.bottom - bounds.bottom;
});
watch(
  [loadTrigger, visibleCount],
  ([trigger]) => {
    loadObserver?.disconnect();
    if (!trigger) return;
    const root =
      trigger.closest<HTMLElement>('[data-history-list]') ??
      trigger.closest<HTMLElement>('.details-content');
    loadObserver = new IntersectionObserver(
      (entries) => {
        if (entries.some((entry) => entry.isIntersecting))
          visibleCount.value = Math.min(filtered.value.length, visibleCount.value + 50);
      },
      { root, rootMargin: '300px 0px' },
    );
    loadObserver.observe(trigger);
  },
  { flush: 'post' },
);
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
  if (range.value === 'all') return copy.value.allHistory;
  const first = new Date(period.value.start);
  if (range.value === 'day') return fullDate(first);
  const compact = (date: Date) =>
    date.toLocaleDateString(locale.value, { year: 'numeric', month: '2-digit', day: '2-digit' });
  return `${compact(first)} – ${compact(new Date(period.value.end - 1))}`;
});
function recordTime(session: Session) {
  const start = new Date(session.started_at_ms);
  const end = session.ended_at_ms === null ? null : new Date(session.ended_at_ms);
  const days = end
    ? Math.round(
        (Date.UTC(end.getFullYear(), end.getMonth(), end.getDate()) -
          Date.UTC(start.getFullYear(), start.getMonth(), start.getDate())) /
          86400000,
      )
    : 0;
  const offset = days === 1 ? copy.value.nextDay : days > 1 ? `+${days}${copy.value.days}` : '';
  return `${localTime(start.getTime())} — ${end ? `${offset ? offset + ' ' : ''}${localTime(end.getTime())}` : copy.value.ongoing}`;
}
async function edit(s: Session) {
  editing.value = s;
  deleting.value = null;
  editTask.value = s.task_id ?? null;
  editStart.value = localDateTime(s.started_at_ms);
  editEnd.value = localDateTime(s.ended_at_ms!);
  saveError.value = '';
  await nextTick();
  modal.value?.showModal();
}
async function askDelete(s: Session) {
  deleting.value = s;
  editing.value = null;
  await nextTick();
  modal.value?.showModal();
  deleteHeading.value?.focus({ preventScroll: true });
}
function close() {
  if (saving.value) return;
  modal.value?.close();
  editing.value = null;
  deleting.value = null;
}
async function submit() {
  if (saving.value || props.busy || !editing.value) return;
  if (editPreview.value.error) {
    saveError.value = editPreview.value.error;
    return;
  }
  const record = editing.value;
  const { start, end } = editedBounds.value;
  const changed = start !== record.started_at_ms || end !== record.ended_at_ms;
  saving.value = true;
  try {
    const failure = await props.correctRecord(
      record.id,
      changed ? start : null,
      changed ? end : null,
      editTask.value,
    );
    if (failure) saveError.value = failure;
    else {
      saving.value = false;
      close();
    }
  } catch {
    saveError.value = copy.value.error;
  } finally {
    saving.value = false;
  }
}
</script>
<template>
  <section
    ref="historyRoot"
    class="record-list-scroll h-full overflow-y-auto overscroll-none"
    tabindex="0"
    :aria-label="copy.historyTitle"
    data-history-viewport
    @scroll="updateCover"
  >
    <div class="relative h-[200%]" data-history-track>
      <div
        class="sticky top-0 mx-auto flex h-1/2 w-full max-w-[820px] flex-col px-7"
        :inert="recordsExpanded"
        data-history-summary
      >
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
        <div
          class="mt-4 grid grid-cols-[minmax(0,1fr)_minmax(200px,0.95fr)] items-stretch gap-4 pb-5"
        >
          <div ref="calendarColumn" class="flex min-w-0 flex-col" data-history-calendar-column>
            <HistoryRangeFilter
              :range="range"
              :from="from"
              :to="to"
              :today="dateInput(now)"
              @range="range = $event"
              @custom="applyCustom"
            />
            <HistoryCalendar
              :month="month"
              :selected="range === 'month' ? dateInput(month) : anchor"
              :today="dateInput(now)"
              :period="period"
              :records="sessions"
              :checkpoint="checkpoint"
              :all="range === 'all'"
              @browse="month = $event"
              @select="selectDate"
              @today="selectDate(dateInput(new Date()))"
            />
          </div>
          <div
            class="flex min-w-0 flex-col items-center"
            :style="
              collectionHeight === undefined ? undefined : { height: `${collectionHeight}px` }
            "
            data-history-collection-column
          >
            <SeriesCollection
              class="!h-auto min-h-0 w-full flex-1"
              :records="filtered"
              :completed-records="completedRecords"
              :series-id="seriesId"
              :total="total"
              fit-height
            />
            <p
              class="mt-3 shrink-0 whitespace-nowrap text-xs leading-relaxed text-faint"
              data-history-period
            >
              {{ periodLabel }}
            </p>
          </div>
        </div>
      </div>
      <div
        class="history-sheet absolute inset-x-0 top-1/2 z-10 flex h-1/2 flex-col py-5"
        :style="{
          borderTopLeftRadius: `${coverRadius}px`,
          borderTopRightRadius: `${coverRadius}px`,
        }"
        :class="{ 'shadow-none': !recordsVisible }"
        :inert="!recordsVisible"
        data-history-cover
      >
        <div class="mx-auto flex h-full min-h-0 w-full max-w-[820px] flex-col px-7">
          <div
            class="mb-2 grid min-w-0 grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-2 shrink-0"
            data-task-header
          >
            <div class="flex min-w-0 items-center gap-2">
              <span class="eyebrow">{{ copy.historyEntries }}</span>
              <div
                class="relative grid h-7 shrink-0 grid-cols-2 rounded-lg bg-surface-muted p-0.5"
                role="radiogroup"
                :aria-label="copy.historyEntries"
                data-history-view
              >
                <span
                  aria-hidden="true"
                  class="pointer-events-none absolute inset-y-0.5 rounded-md border border-line bg-surface shadow-sm transition-[left] duration-150 motion-reduce:transition-none"
                  :style="{
                    left: 'calc(2px + (100% - 4px) * ' + viewIndex + ' / 2)',
                    width: 'calc((100% - 4px) / 2)',
                  }"
                />
                <button
                  v-for="option in viewOptions"
                  :key="option.value"
                  type="button"
                  role="radio"
                  :aria-checked="historyView === option.value"
                  :aria-label="option.title"
                  :title="option.title"
                  :tabindex="historyView === option.value ? 0 : -1"
                  class="relative z-10 flex items-center justify-center rounded-md px-2 py-1"
                  :class="historyView === option.value ? 'text-ink' : 'text-muted hover:text-ink'"
                  @click="historyView = option.value"
                  @keydown="navigateView($event, option.value === 'list' ? 0 : 1)"
                >
                  <svg
                    width="14"
                    height="14"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.8"
                    aria-hidden="true"
                  >
                    <path :d="option.path" />
                  </svg>
                </button>
              </div>
            </div>
            <div
              class="record-list-scroll flex min-w-0 flex-nowrap items-center gap-2 overflow-x-auto overflow-y-hidden"
              data-selected-task-tags
            >
              <span
                v-for="id in selectedTasks"
                :key="id"
                class="inline-flex min-w-0 max-w-full shrink-0 items-center gap-1 rounded-md border border-line bg-surface-muted px-2 py-1 text-xs text-ink"
                ><span
                  class="truncate"
                  :title="id === uncategorizedTask ? copy.uncategorized : taskName(id)"
                  >{{ id === uncategorizedTask ? copy.uncategorized : taskName(id) }}</span
                ><button
                  type="button"
                  class="shrink-0 rounded text-muted hover:text-ink"
                  :aria-label="
                    copy.removeTaskFilter +
                    (id === uncategorizedTask ? copy.uncategorized : taskName(id))
                  "
                  @click="removeTaskFilter(id)"
                >
                  <X :size="12" /></button
              ></span>
            </div>
            <div class="flex shrink-0 items-center gap-3">
              <MultiSelect
                id="history-tasks"
                :label="copy.filterTasks"
                :selected="selectedTasks"
                :options="taskOptions"
                :disabled="busy"
                @update:selected="selectedTasks = $event"
              /><span class="text-xs text-faint tabular-nums">{{
                filtered.length.toString().padStart(2, '0')
              }}</span>
            </div>
          </div>
          <div v-if="!filtered.length" class="py-14 text-center text-faint">
            <Clock3 class="mx-auto" :size="30" stroke-width="1" />
            <h2 class="mb-2 mt-4 text-sm font-medium text-muted">{{ copy.noPeriodRecords }}</h2>
          </div>
          <div
            v-if="historyView === 'timeline'"
            class="record-list-scroll min-h-0 flex-1"
            :class="recordsExpanded ? 'overflow-y-auto' : 'overflow-y-hidden'"
          >
            <HistoryTimeline
              v-model:span="timelineSpan"
              :groups="groups"
              :name-of="recordName"
              :time-of="recordTime"
              :checkpoint="checkpoint"
              :series-id="seriesId"
              @select="fromTimeline"
            />
          </div>
          <div v-else class="relative min-h-0 flex-1 overflow-hidden" data-history-records>
            <div
              class="h-full min-w-0 overflow-hidden transition-[width] duration-[240ms] ease-out motion-reduce:transition-none"
              :style="{ width: detailOpen ? 'calc(100% - max(266px, 44%))' : '100%' }"
              data-history-list-column
            >
              <div
                ref="listScroller"
                class="record-list-scroll h-full overflow-x-hidden px-1 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-muted"
                :class="recordsExpanded ? 'overflow-y-auto' : 'overflow-y-hidden'"
                tabindex="0"
                role="region"
                :aria-label="copy.historyEntries"
                data-history-list
              >
                <section v-for="(group, index) in groups" :key="group.date" class="history-day">
                  <h2
                    class="mb-1 text-xs font-medium text-muted"
                    :class="index === 0 ? 'mt-0' : 'mt-5'"
                  >
                    {{ group.label }}
                  </h2>
                  <div
                    v-for="s in group.records"
                    :key="s.id"
                    :data-record-id="s.id"
                    tabindex="0"
                    class="group flex cursor-pointer items-center gap-2 rounded-lg py-3 pl-2 pr-[9px] hover:bg-surface-muted wide:gap-2.5"
                    :class="{ 'bg-surface-muted': detailOpen && detail?.id === s.id }"
                    @click="openDetail(s)"
                    @keydown.enter.self="openDetail(s)"
                  >
                    <SeriesReward :series-id="seriesId" class="size-5" :duration="s.duration_ms" />
                    <div class="min-w-0 flex-1">
                      <strong class="block truncate text-sm font-normal">{{
                        recordName(s)
                      }}</strong>
                      <span
                        class="mt-1.5 flex min-w-0 items-center gap-1.5 text-xs text-faint"
                        :title="recordMeta(s)"
                      >
                        <span class="shrink-0">{{ recordTime(s) }}</span>
                        <span v-if="s.corrected" class="shrink-0 whitespace-nowrap"
                          >· {{ copy.corrected }}</span
                        >
                        <span v-if="s.title && s.task_id" class="truncate"
                          >· {{ taskName(s.task_id) }}</span
                        >
                      </span>
                    </div>
                    <span class="hidden shrink-0 text-xs text-muted wide:inline">{{
                      copy.focus
                    }}</span>
                    <!-- 32px targets around 14px icons leave 9px on each side. -->
                    <div class="flex shrink-0 items-center" data-record-actions>
                      <strong
                        class="w-14 shrink-0 text-right text-xs font-medium tabular-nums wide:w-16"
                        :class="listActionsVisible ? 'mr-[9px]' : 'mr-0'"
                        :title="copy.sessionTotal + ' · ' + duration(s.duration_ms)"
                        >{{ duration(durationInPeriod(s, period, checkpoint)) }}</strong
                      ><button
                        class="icon-button opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 [@media(hover:none)]:opacity-100"
                        :disabled="busy"
                        v-if="listActionsVisible && s.id !== current?.id"
                        :aria-label="copy.edit"
                        :title="copy.edit"
                        @click.stop="edit(s)"
                      >
                        <Pencil :size="14" /></button
                      ><button
                        class="icon-button opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 [@media(hover:none)]:opacity-100"
                        :disabled="busy"
                        v-if="listActionsVisible && s.id !== current?.id"
                        :aria-label="copy.remove"
                        :title="copy.remove"
                        @click.stop="askDelete(s)"
                      >
                        <Trash2 :size="14" />
                      </button>
                    </div>
                  </div>
                </section>
                <div
                  v-if="filtered.length > visibleCount"
                  ref="loadTrigger"
                  class="h-px"
                  aria-hidden="true"
                />
              </div>
            </div>
            <Transition
              enter-active-class="transition-transform duration-[240ms] ease-out motion-reduce:transition-none"
              enter-from-class="translate-x-[calc(100%+16px)]"
              enter-to-class="translate-x-0"
              leave-active-class="pointer-events-none transition-transform duration-[240ms] ease-out motion-reduce:transition-none"
              leave-from-class="translate-x-0"
              leave-to-class="translate-x-[calc(100%+16px)]"
              @before-enter="detailLeaving = false"
              @before-leave="detailLeaving = true"
              @after-leave="detailLeaving = false"
              @leave-cancelled="detailLeaving = false"
            >
              <RecordDetailPanel
                v-if="detailOpen && detail"
                class="!absolute inset-y-0 right-0 h-full"
                style="width: calc(max(266px, 44%) - 16px)"
                data-history-detail
                :record="detail"
                :series-id="seriesId"
                :name="recordName(detail)"
                :time="recordTime(detail)"
                :effective-ms="durationInPeriod(detail, period, checkpoint)"
                :busy="busy"
                :is-current="detail.id === current?.id"
                @collapse="detailOpen = false"
                @edit="edit"
                @remove="askDelete"
              />
            </Transition>
          </div>
        </div>
      </div>
    </div>
    <dialog
      ref="modal"
      class="modal"
      :class="editing ? '!w-80 !rounded-xl !p-5 !shadow-lg' : undefined"
      :aria-labelledby="deleting ? 'delete-record-title' : undefined"
      @cancel="saving ? $event.preventDefault() : close()"
    >
      <form v-if="editing" @submit.prevent="submit">
        <div class="modal-heading !mb-3">
          <h2 class="!text-base !leading-7">{{ copy.edit }}</h2>
          <button
            type="button"
            class="icon-button group enabled:hover:!bg-transparent"
            :aria-label="copy.cancel"
            @click="close"
          >
            <span class="flex size-6 items-center justify-center rounded-md group-hover:bg-hover">
              <X :size="14" />
            </span>
          </button>
        </div>
        <div class="flex min-h-9 items-center justify-between gap-3">
          <span class="text-xs text-muted">{{ copy.taskCategory }}</span
          ><SettingsSelect
            id="record-task"
            :label="copy.taskCategory"
            :model-value="editTask ?? ''"
            :options="[
              { value: '', label: copy.uncategorized },
              ...(tasks ?? []).map((task) => ({ value: task.id, label: task.name })),
            ]"
            :disabled="busy || saving"
            @update:model-value="editTask = $event || null"
          />
        </div>
        <RecordTimeField
          id="correction-start"
          :label="copy.recordStart"
          v-model="editStart"
          :disabled="busy || saving"
        />
        <p
          v-if="editPreview.error && editedBounds.start === null"
          role="alert"
          class="!my-1 !text-danger"
        >
          {{ editPreview.error }}
        </p>
        <RecordTimeField
          id="correction-end"
          :label="copy.recordEnd"
          v-model="editEnd"
          :disabled="busy || saving"
        />
        <p
          v-if="saveError || (editPreview.error && editedBounds.start !== null)"
          role="alert"
          class="!my-1 !text-danger"
        >
          {{ saveError || editPreview.error }}
        </p>
        <div class="mt-4 flex items-center justify-between gap-3 border-t border-line pt-4">
          <div class="flex flex-col gap-1 text-xs">
            <span class="text-muted">{{ copy.recordEffective }}</span>
            <output class="text-sm font-medium tabular-nums">{{ editDurationLabel }}</output>
          </div>
          <div class="flex shrink-0 gap-2">
            <button
              type="button"
              class="button h-8 min-h-8 px-3 py-1.5 text-muted enabled:hover:bg-hover"
              :disabled="saving"
              @click="close"
            >
              {{ copy.cancel }}
            </button>
            <button
              class="button primary h-8 min-h-8 px-3 py-1.5"
              type="submit"
              :disabled="busy || saving || !!editPreview.error"
            >
              {{ copy.save }}
            </button>
          </div>
        </div>
      </form>
      <div v-if="deleting">
        <h2 ref="deleteHeading" id="delete-record-title" tabindex="-1" class="outline-none">
          {{ copy.confirmDelete }}
        </h2>
        <p>{{ copy.deleteHint }}</p>
        <div class="modal-actions">
          <button class="button secondary" @click="close">{{ copy.cancel }}</button
          ><button
            class="button dialog-confirm"
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
