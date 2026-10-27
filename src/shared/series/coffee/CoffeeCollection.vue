<script setup lang="ts">
import { computed, useId } from 'vue';
import ExteriorBean from './ExteriorBean.vue';
import { compactOverflow } from './compactOverflow';
import CollectionBean from './CollectionBean.vue';
import { coffeeSeries, coffeeTier } from './definition';
import { useCoffeeLayout } from './useCoffeeLayout';
import { clock, copy, duration, locale, formatBeanCount } from '../../i18n';
import type { Session } from '../../ipc/types';
import { jarPath as jar } from './jarGeometry';
const props = withDefaults(
  defineProps<{
    records: Session[];
    total: number;
    fitHeight?: boolean;
  }>(),
  { fitHeight: false },
);
const visible = computed(() => props.records.slice(0, coffeeSeries.collection.displayLimit));
const byId = computed(() => new Map(visible.value.map((record) => [record.id, record])));
const { layout } = useCoffeeLayout(() => visible.value.map((record) => record.id));
const glassClip = `coffee-glass-${useId()}`;
const packing = computed(() => compactOverflow(layout.value?.poses ?? []));
const sorted = computed(() => [...packing.value.poses].sort((a, b) => a.depth - b.depth));
const insidePoses = computed(() => sorted.value.filter((pose) => pose.inside));
const backPoses = computed(() =>
  sorted.value.filter((pose) => !pose.inside && pose.layer === 'body'),
);
const frontPoses = computed(() => sorted.value.filter((pose) => pose.layer === 'front'));
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
        class="absolute inset-0 block size-full overflow-visible"
        viewBox="156 20 168 236"
        preserveAspectRatio="xMidYMid meet"
        role="img"
        :aria-label="formatBeanCount(records.length)"
        :aria-busy="!layout"
      >
        <defs>
          <clipPath :id="glassClip">
            <path :d="jar" />
            <rect x="156" y="-120" width="168" height="160" />
          </clipPath>
        </defs>
        <ellipse cx="240" cy="234" rx="65" ry="7" class="fill-ink" opacity="0.055" />
        <ellipse
          v-for="shadow in packing.shadows"
          :key="shadow.id"
          :cx="shadow.x"
          :cy="shadow.y"
          :rx="shadow.rx"
          :ry="shadow.ry"
          class="fill-ink"
          opacity="0.065"
        />
        <ExteriorBean
          v-for="pose in backPoses"
          :key="pose.id"
          :pose="pose"
          :duration="byId.get(pose.id)!.duration_ms"
          :description="description(pose.id)"
        />
        <path :d="jar" class="fill-surface-muted" />
        <path
          d="M176 40A64 6 0 0 1 304 40"
          class="stroke-line-strong"
          fill="none"
          stroke-width="1.4"
          stroke-linecap="round"
        />
        <path d="M186 226Q240 212 294 226" class="stroke-line" fill="none" stroke-width="1" />
        <g :clip-path="`url(#${glassClip})`">
          <CollectionBean
            v-for="pose in insidePoses"
            :key="pose.id"
            :pose="pose"
            :duration="byId.get(pose.id)!.duration_ms"
            :description="description(pose.id)"
          />
        </g>
        <path :d="jar" class="fill-paper" opacity="0.07" />
        <path
          d="M178 82V196Q178 211 188 215"
          class="stroke-paper"
          opacity="0.65"
          fill="none"
          stroke-width="2"
          stroke-linecap="round"
        />
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
        <ExteriorBean
          v-for="pose in frontPoses"
          :key="pose.id"
          :pose="pose"
          :duration="byId.get(pose.id)!.duration_ms"
          :description="description(pose.id)"
        />
      </svg>
    </div>
    <strong class="mt-3 shrink-0 text-xl font-medium tracking-tight tabular-nums">{{
      total === 0 ? copy.emptyCollection : duration(total)
    }}</strong>
  </section>
</template>
