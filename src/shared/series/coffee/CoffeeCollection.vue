<script setup lang="ts">
import { computed } from 'vue';
import CoffeeReward from './CoffeeReward.vue';
import { coffeeSeries, coffeeTier } from './definition';
import { useCoffeeLayout } from './useCoffeeLayout';
import { SCENE_WIDTH, SCENE_HEIGHT } from './physics';
import { clock, copy, duration, locale } from '../../i18n';
import type { Session } from '../../ipc/types';
const props = defineProps<{ records: Session[]; total: number; periodLabel: string }>();
const visible = computed(() => props.records.slice(0, coffeeSeries.collection.displayLimit));
const byId = computed(() => new Map(visible.value.map((record) => [record.id, record])));
const { layout } = useCoffeeLayout(() => visible.value.map((record) => record.id));
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
    :aria-label="copy.coffeeJar"
    :data-layout-ready="!!layout"
    :data-visible-beans="layout?.poses.length ?? 0"
  >
    <svg
      class="block w-full max-w-[400px] shrink-0 overflow-hidden"
      :viewBox="`0 0 ${SCENE_WIDTH} ${SCENE_HEIGHT}`"
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
    <strong class="mt-3 text-xl font-medium tracking-tight tabular-nums">{{
      duration(total)
    }}</strong>
    <span class="mt-1 text-center text-xs leading-relaxed text-faint">{{ periodLabel }}</span>
  </section>
</template>
