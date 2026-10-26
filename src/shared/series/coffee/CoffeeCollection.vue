<script setup lang="ts">
import { computed } from 'vue';
import CoffeeReward from './CoffeeReward.vue';
import { coffeeSeries, coffeeTier } from './definition';
import { useCoffeeLayout } from './useCoffeeLayout';
import { clock, copy, duration, locale } from '../../i18n';
import type { Session } from '../../ipc/types';
const props = withDefaults(
  defineProps<{
    records: Session[];
    total: number;
    periodLabel: string | string[];
    fitHeight?: boolean;
  }>(),
  { fitHeight: false },
);
const visible = computed(() => props.records.slice(0, coffeeSeries.collection.displayLimit));
const byId = computed(() => new Map(visible.value.map((record) => [record.id, record])));
const { layout } = useCoffeeLayout(() => visible.value.map((record) => record.id));
const periodLines = computed(() =>
  typeof props.periodLabel === 'string' ? [props.periodLabel] : props.periodLabel,
);
const viewBox = computed(() => {
  const poses = layout.value?.poses ?? [];
  // Frame the jar and every visible bean, not unused space reserved for a full
  // spill. Keep a complete bean-sized margin so nothing is cropped at any angle.
  const left = Math.min(166, ...poses.map((pose) => pose.x - 14)) - 10;
  const right = Math.max(314, ...poses.map((pose) => pose.x + 14)) + 10;
  const top = Math.min(30, ...poses.map((pose) => pose.y - 14)) - 10;
  const bottom = Math.max(246, ...poses.map((pose) => pose.y + 14)) + 10;
  return `${left} ${top} ${right - left} ${bottom - top}`;
});
const bodyPoses = computed(() => layout.value?.poses.filter((pose) => pose.layer === 'body') ?? []);
const frontPoses = computed(() =>
  (layout.value?.poses.filter((pose) => pose.layer === 'front') ?? []).sort((a, b) => a.y - b.y),
);
const jar =
  'M176 40V55Q176 62 172 70Q170 76 170 85V204Q170 220 186 226Q240 240 294 226Q310 220 310 204V85Q310 76 308 70Q304 62 304 55V40';
function description(id: string) {
  const record = byId.value.get(id)!;
  return `${record.title || copy.value.untitled} · ${clock(record.duration_ms)} · ${coffeeTier(record.duration_ms).name[locale.value]}`;
}
</script>
<template>
  <section
    class="flex min-w-0 flex-col items-center"
    :class="fitHeight ? 'h-full min-h-0' : ''"
    :aria-label="copy.coffeeJar"
    :data-layout-ready="!!layout"
    :data-visible-beans="layout?.poses.length ?? 0"
  >
    <div class="relative w-full" :class="fitHeight ? 'min-h-0 flex-1' : 'h-56 max-w-[400px]'">
      <svg
        class="absolute inset-0 block size-full"
        :viewBox="viewBox"
        preserveAspectRatio="xMidYMid meet"
        role="img"
        :aria-label="`${records.length} ${copy.beanCount}`"
        :aria-busy="!layout"
      >
        <path :d="jar" class="fill-surface-muted" />
        <path
          d="M176 40A64 6 0 0 1 304 40"
          class="stroke-line-strong"
          fill="none"
          stroke-width="1.4"
          stroke-linecap="round"
        />
        <path d="M186 226Q240 212 294 226" class="stroke-line" fill="none" stroke-width="1" />
        <g
          v-for="pose in bodyPoses"
          :key="pose.id"
          :transform="`translate(${pose.x} ${pose.y}) rotate(${(pose.angle * 180) / Math.PI})`"
          :data-bean-id="pose.id"
          data-bean-layer="body"
        >
          <title>{{ description(pose.id) }}</title>
          <CoffeeReward
            :duration="byId.get(pose.id)!.duration_ms"
            x="-14"
            y="-14"
            width="28"
            height="28"
          />
        </g>
        <path
          :d="jar"
          class="stroke-line-strong"
          fill="none"
          stroke-width="1.4"
          stroke-linejoin="round"
          stroke-linecap="round"
        />
        <path
          d="M176 40A64 6 0 0 0 304 40"
          class="stroke-line-strong"
          fill="none"
          stroke-width="1.4"
        />
        <g
          v-for="pose in frontPoses"
          :key="pose.id"
          :transform="`translate(${pose.x} ${pose.y}) rotate(${(pose.angle * 180) / Math.PI})`"
          :data-bean-id="pose.id"
          data-bean-layer="front"
        >
          <title>{{ description(pose.id) }}</title>
          <CoffeeReward
            :duration="byId.get(pose.id)!.duration_ms"
            x="-14"
            y="-14"
            width="28"
            height="28"
          />
        </g>
      </svg>
    </div>
    <strong class="mt-3 shrink-0 text-xl font-medium tracking-tight tabular-nums">{{
      duration(total)
    }}</strong>
    <p
      class="mt-1 flex min-h-10 shrink-0 flex-col justify-center text-center text-xs leading-relaxed text-faint"
      :aria-label="periodLines.join(' – ')"
    >
      <span v-for="(line, index) in periodLines" :key="index" class="block whitespace-nowrap"
        >{{ index ? '– ' : '' }}{{ line }}</span
      >
    </p>
  </section>
</template>
