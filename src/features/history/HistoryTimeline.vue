<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
import { copy, localTime } from '../../shared/i18n';
import { taskInk } from '../../shared/taskColor';
import { seriesFor } from '../../shared/series/catalog';
import type { SeriesId } from '../../shared/series/types';
import {
  TIMELINE_MAX_SPAN,
  TIMELINE_MIN_SPAN,
  daySpan,
  timelineScale,
  timelineScrollLeft,
  timelineStepMs,
  timelineTicks,
  timelineWindow,
  zoomPosition,
  zoomSpan,
} from './history';
import type { Session } from '../../shared/ipc/types';
const props = defineProps<{
  groups: { date: string; label: string; records: Session[] }[];
  nameOf: (record: Session) => string;
  colorOf: (record: Session) => string | null;
  timeOf: (record: Session) => string;
  checkpoint: number;
  span: number;
  seriesId: SeriesId;
}>();
const emit = defineEmits<{ select: [Session]; 'update:span': [number] }>();
const DAY_MS = 86400000;
const viewport = ref<HTMLDivElement | null>(null);
const viewportWidth = ref(0);
const scrollLeft = ref(0);
let frame = 0;
let observer: ResizeObserver | undefined;
const scale = computed(() => timelineScale(props.span));
const reference = computed(() => {
  const date = props.groups[0]?.date;
  if (!date) return null;
  const [year, month, day] = date.split('-').map(Number);
  const start = new Date(year!, month! - 1, day!).getTime();
  return { start, end: start + DAY_MS };
});
const view = computed(() => {
  const day = reference.value;
  if (!day) return { start: 0, end: 0 };
  return timelineWindow(day.start, DAY_MS, scale.value, scrollLeft.value, viewportWidth.value);
});
const ticks = computed(() => {
  const day = reference.value;
  if (!day) return [];
  return timelineTicks(
    day.start,
    view.value.start,
    view.value.end,
    timelineStepMs(DAY_MS / scale.value),
  );
});
const axisLabels = computed(() => {
  const day = reference.value;
  if (!day) return [];
  const labels = ticks.value.map((at) => ({
    at,
    label:
      props.span >= TIMELINE_MAX_SPAN
        ? String(new Date(at).getHours()).padStart(2, '0')
        : localTime(at),
  }));
  if (view.value.end >= day.end - 1) labels.push({ at: day.end, label: '24' });
  return labels;
});
const gridTicks = computed(() => {
  const day = reference.value;
  return day ? ticks.value.filter((at) => at > day.start) : [];
});
const spanLabel = computed(() => {
  const total = Math.max(1, Math.round(props.span));
  if (total < 60) return `${total}${copy.value.minShort}`;
  const hours = Math.floor(total / 60);
  const rest = total % 60;
  return rest === 0
    ? `${hours}${copy.value.hourShort}`
    : `${hours}${copy.value.hourShort} ${rest}${copy.value.minShort}`;
});
const sliderPosition = computed(() => zoomPosition(props.span));
const sliderStyle = computed(() => {
  const palette = seriesFor(props.seriesId).rest;
  const light = palette.light.layers;
  const dark = palette.dark.layers;
  return {
    '--zoom-fill': `${sliderPosition.value * 100}%`,
    '--zoom-from-light': light[0] ?? palette.light.line,
    '--zoom-to-light': light[light.length - 1] ?? palette.light.ink,
    '--zoom-from-dark': dark[dark.length - 1] ?? palette.dark.ink,
    '--zoom-to-dark': dark[0] ?? palette.dark.line,
  };
});
const recordAnchor = computed(() => {
  const day = reference.value;
  const records = props.groups[0]?.records;
  if (!day || !records?.length) return null;
  let first = Infinity;
  let last = -Infinity;
  for (const record of records) {
    first = Math.min(first, record.started_at_ms);
    last = Math.max(last, record.ended_at_ms ?? props.checkpoint);
  }
  return Math.min(Math.max((first + last) / 2, day.start), day.end);
});
function recordStyle(day: string, record: Session) {
  const [year, month, date] = day.split('-').map(Number);
  const part = daySpan(
    record.started_at_ms,
    record.ended_at_ms ?? props.checkpoint,
    new Date(year!, month! - 1, date!),
  );
  const style = { left: `${part.left}%`, width: `${part.width}%`, minWidth: '6px' };
  const color = props.colorOf(record);
  return color ? { ...style, backgroundColor: color, color: taskInk(color) } : style;
}
function description(record: Session) {
  return `${props.nameOf(record)} · ${props.timeOf(record)}`;
}
function position(at: number) {
  const day = reference.value;
  return day ? `${((at - day.start) / (day.end - day.start)) * 100}%` : '0%';
}
function align(at: number) {
  const day = reference.value;
  const ratio = day ? (at - day.start) / (day.end - day.start) : 0;
  return ratio <= 0 ? 'none' : ratio >= 0.96 ? 'translateX(-100%)' : 'translateX(-50%)';
}
function updateScroll() {
  const element = viewport.value;
  if (!element) return;
  scrollLeft.value = element.scrollLeft;
  viewportWidth.value = element.clientWidth;
}
function onScroll() {
  if (frame) return;
  frame = requestAnimationFrame(() => {
    frame = 0;
    updateScroll();
  });
}
function scrollTo(anchor: number, span: number) {
  const day = reference.value;
  const element = viewport.value;
  if (!day || !element) return;
  element.scrollLeft = timelineScrollLeft(
    day.start,
    DAY_MS,
    timelineScale(span),
    element.clientWidth,
    anchor,
  );
  updateScroll();
}
function anchorTime() {
  const day = reference.value;
  if (!day) return 0;
  if (props.span >= TIMELINE_MAX_SPAN && recordAnchor.value !== null) return recordAnchor.value;
  return view.value.start + (view.value.end - view.value.start) / 2;
}
async function setSpan(next: number) {
  const target = Math.min(TIMELINE_MAX_SPAN, Math.max(TIMELINE_MIN_SPAN, next));
  if (Math.abs(target - props.span) < 1e-9) return;
  const anchor = anchorTime();
  emit('update:span', target);
  await nextTick();
  scrollTo(anchor, target);
}
function onZoomInput(event: Event) {
  const value = Number((event.target as HTMLInputElement).value);
  if (Number.isFinite(value)) void setSpan(zoomSpan(value));
}
watch(
  () => props.groups[0]?.date,
  async () => {
    if (props.span >= TIMELINE_MAX_SPAN) return;
    await nextTick();
    if (recordAnchor.value !== null) scrollTo(recordAnchor.value, props.span);
  },
);
onMounted(async () => {
  const element = viewport.value;
  if (element) {
    viewportWidth.value = element.clientWidth;
    observer = new ResizeObserver(() => {
      const width = element.clientWidth;
      if (width === viewportWidth.value) return;
      const center = view.value.start + (view.value.end - view.value.start) / 2;
      viewportWidth.value = width;
      if (props.span >= TIMELINE_MAX_SPAN) {
        element.scrollLeft = 0;
        scrollLeft.value = 0;
        return;
      }
      const day = reference.value;
      if (day) {
        element.scrollLeft = timelineScrollLeft(day.start, DAY_MS, scale.value, width, center);
        scrollLeft.value = element.scrollLeft;
      }
    });
    observer.observe(element);
  }
  await nextTick();
  if (props.span < TIMELINE_MAX_SPAN && recordAnchor.value !== null)
    scrollTo(recordAnchor.value, props.span);
  else updateScroll();
});
onUnmounted(() => {
  observer?.disconnect();
  if (frame) cancelAnimationFrame(frame);
});
</script>
<template>
  <div>
    <div class="mt-5 flex items-center justify-between gap-3">
      <span class="eyebrow">{{ copy.timelineZoom }}</span>
      <div class="flex items-center gap-2">
        <input
          type="range"
          class="zoom-slider"
          :style="sliderStyle"
          min="0"
          max="1"
          step="any"
          :value="sliderPosition"
          :aria-label="copy.timelineZoom"
          :aria-valuetext="spanLabel"
          @input="onZoomInput"
        />
        <span class="w-24 shrink-0 text-right text-xs tabular-nums text-muted">{{
          spanLabel
        }}</span>
      </div>
    </div>
    <div
      ref="viewport"
      class="record-list-scroll overflow-x-auto overscroll-x-contain focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-muted"
      tabindex="0"
      role="region"
      :aria-label="copy.timelineScrollHint"
      @scroll.passive="onScroll"
    >
      <div class="min-w-full" :style="{ width: `${scale * 100}%` }">
        <section v-for="group in groups" :key="group.date">
          <h2
            class="sticky left-0 z-10 mb-2 mt-5 w-fit bg-paper pr-2 text-xs font-medium text-muted"
          >
            {{ group.label }}
          </h2>
          <div class="relative h-4 text-xs tabular-nums text-faint" aria-hidden="true">
            <span
              v-for="label in axisLabels"
              :key="label.at"
              class="absolute top-0 whitespace-nowrap"
              :style="{ left: position(label.at), transform: align(label.at) }"
              >{{ label.label }}</span
            >
          </div>
          <div class="relative mt-1 h-9 overflow-hidden rounded-lg bg-surface-muted">
            <span
              v-for="line in gridTicks"
              :key="line"
              class="absolute inset-y-0 w-px bg-line-strong opacity-60"
              :style="{ left: position(line) }"
              aria-hidden="true"
            />
            <button
              v-for="record in group.records"
              :key="record.id"
              type="button"
              class="absolute inset-y-1 flex items-center overflow-hidden whitespace-nowrap rounded-md px-1.5 text-xs"
              :class="colorOf(record) ? '' : 'border border-line-strong bg-surface'"
              :style="recordStyle(group.date, record)"
              :title="description(record)"
              :aria-label="description(record)"
              @click="emit('select', record)"
            >
              <span class="truncate">{{ nameOf(record) }}</span>
            </button>
          </div>
        </section>
      </div>
    </div>
  </div>
</template>
