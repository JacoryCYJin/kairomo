<script setup lang="ts">
import { computed } from 'vue';
import { Pencil, Trash2 } from 'lucide-vue-next';
import SeriesReward from '../../shared/series/SeriesReward.vue';
import { copy, duration, durationParts } from '../../shared/i18n';
import type { SeriesId } from '../../shared/series/types';
import type { Session } from '../../shared/ipc/types';
const props = defineProps<{
  record: Session;
  seriesId: SeriesId;
  name: string;
  time: string;
  effectiveMs: number;
  busy: boolean;
  isCurrent: boolean;
}>();
const emit = defineEmits<{ collapse: []; edit: [Session]; remove: [Session] }>();
const hero = computed(() => {
  const parts = durationParts(props.effectiveMs);
  return { figure: parts.value, unit: parts.unit };
});
// 本轮完整时长只在和周期内有效时长不同时才出现，避免重复同一份信息。
const note = computed(() =>
  props.record.duration_ms === props.effectiveMs
    ? ''
    : `${copy.value.sessionTotal} ${duration(props.record.duration_ms)}`,
);
</script>
<template>
  <div
    class="relative flex flex-col items-center justify-center rounded-xl bg-surface-muted px-4 py-5"
  >
    <button
      type="button"
      class="icon-button absolute right-1.5 top-1.5"
      :aria-label="copy.collapseDetail"
      :title="copy.collapseDetail"
      @click="emit('collapse')"
    >
      <svg
        width="16"
        height="16"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="1.8"
        aria-hidden="true"
      >
        <path d="M10 6l6 6-6 6M5 5v14" />
      </svg>
    </button>
    <SeriesReward :series-id="seriesId" class="size-7" :duration="record.duration_ms" />
    <div class="mt-2.5 min-w-0 max-w-full">
      <strong class="block truncate text-base font-semibold tracking-tight">{{ name }}</strong>
    </div>
    <span class="mt-1 flex items-center gap-1.5 text-xs tabular-nums text-muted">
      {{ time }}
      <span v-if="record.corrected" class="whitespace-nowrap text-faint"
        >· {{ copy.corrected }}</span
      >
    </span>
    <div class="mt-4 flex items-baseline leading-none">
      <span class="text-3xl font-semibold tabular-nums tracking-tight">{{ hero.figure }}</span
      ><span class="ml-1 text-sm text-muted">{{ hero.unit }}</span>
    </div>
    <span class="mt-1.5 text-xs text-muted">{{ copy.recordEffective }}</span>
    <span v-if="note" class="mt-2.5 text-center text-xs text-faint">{{ note }}</span>
    <div v-if="!isCurrent" class="mt-3.5 flex gap-2">
      <button class="button secondary compact" :disabled="busy" @click="emit('edit', record)">
        <Pencil :size="14" />{{ copy.edit }}
      </button>
      <button class="button secondary compact" :disabled="busy" @click="emit('remove', record)">
        <Trash2 :size="14" />{{ copy.remove }}
      </button>
    </div>
  </div>
</template>
