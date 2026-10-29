<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, shallowRef, watch } from 'vue';
import CoffeeReward from './CoffeeReward.vue';
import type { TimerVisualProps } from '../contracts';
import { formatBeanCount } from '../../i18n';
import { api, desktop } from '../../ipc/client';
import { periodCompletions } from '../period';
import type { PileBean } from './beanPile';
import { pileController, type PileRequest } from './pileController';
import { periodPileSeed } from './pileSeed';
const props = defineProps<TimerVisualProps>();
const emit = defineEmits<{ prepared: [ready: boolean]; failed: [] }>();
const now = ref(new Date());
const periods = computed(() =>
  (['day', 'week'] as const).map((range) => {
    const completed = periodCompletions(props.records, range, now.value);
    return {
      range,
      completed,
      seed: props.state.bean_random_seed
        ? periodPileSeed(props.state.bean_random_seed, range, now.value)
        : null,
      ids: completed.map((record) => record.id),
    };
  }),
);
const selected = computed(() =>
  periods.value.find((period) => period.range === (props.state.settings.bean_range ?? 'day'))!,
);
const completed = computed(() => selected.value.completed);
const byId = computed(() => new Map(completed.value.map((session) => [session.id, session])));
const requests = computed(() =>
  periods.value
    .filter((period) => period.seed !== null)
    .map((period) => ({ seed: period.seed!, ids: period.ids })),
);
const poses = shallowRef<PileBean[]>([]);
const calculating = ref(false);
let timeout: ReturnType<typeof setTimeout> | undefined;
let unlisten: (() => void) | undefined;
let disposed = false;
const controller = pileController(
  () => new Worker(new URL('./beanPile.worker.ts', import.meta.url), { type: 'module' }),
  () => {
    const period = selected.value;
    const result = period.seed
      ? controller.read(period as PileRequest)
      : { poses: [], calculating: false };
    poses.value = result.poses;
    calculating.value = result.calculating;
    emit('prepared', !!period.seed && !result.calculating);
  },
  (seed) => {
    if (seed === selected.value.seed) emit('failed');
  },
);
function reconcile() {
  if (disposed) return;
  const paused = document.hidden && !props.warmingUp;
  if (paused) controller.suspend();
  controller.prepare(requests.value);
  if (!paused) controller.resume();
}
function dateChanged() {
  if (disposed) return;
  clearTimeout(timeout);
  now.value = new Date();
  const date = now.value;
  timeout = setTimeout(
    dateChanged,
    Math.max(
      1,
      new Date(date.getFullYear(), date.getMonth(), date.getDate() + 1).getTime() - date.getTime(),
    ),
  );
  reconcile();
}
function visible() {
  dateChanged();
}
watch(
  [
    () => JSON.stringify(requests.value),
    () => props.state.settings.bean_range,
    () => props.warmingUp,
  ],
  reconcile,
);
onMounted(async () => {
  dateChanged();
  window.addEventListener('focus', dateChanged);
  document.addEventListener('visibilitychange', visible);
  if (desktop) {
    const stop = await api.onTimerOpened(dateChanged);
    if (disposed) stop();
    else unlisten = stop;
  }
});
onUnmounted(() => {
  disposed = true;
  controller.dispose();
  clearTimeout(timeout);
  unlisten?.();
  window.removeEventListener('focus', dateChanged);
  document.removeEventListener('visibilitychange', visible);
});
</script>
<template>
  <div
    class="pointer-events-none absolute inset-x-0 bottom-0 z-0 h-[73px] overflow-hidden rounded-[inherit]"
    role="img"
    :aria-label="formatBeanCount(completed.length)"
    :aria-busy="calculating"
  >
    <div
      v-for="pose in poses"
      :key="`shadow-${pose.id}`"
      class="absolute h-1 rounded-[50%] bg-[color-mix(in_srgb,var(--ink)_7%,transparent)] blur-[1px]"
      :style="{
        left: `${pose.x / 3.6}%`,
        bottom: `${pose.shadowY - 3}px`,
        width: `${pose.size * 0.55}px`,
        transform: 'translateX(-50%)',
      }"
    />
    <div
      v-for="pose in poses"
      :key="pose.id"
      class="absolute"
      :style="{
        left: `${pose.x / 3.6}%`,
        bottom: `${pose.y - pose.size / 2}px`,
        width: `${pose.size}px`,
        height: `${pose.size}px`,
        transform: `translateX(-50%) rotate(${pose.angle}deg) scaleY(${pose.squash})`,
        filter: 'drop-shadow(0 1px 0.7px rgb(0 0 0 / 20%))',
      }"
    >
      <CoffeeReward :duration="byId.get(pose.id)?.duration_ms ?? 0" class="size-full" />
    </div>
  </div>
</template>
