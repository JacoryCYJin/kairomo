<script setup lang="ts">
import { computed } from 'vue';
import SeriesReward from '../../shared/series/SeriesReward.vue';
import type { SeriesId } from '../../shared/series/types';
import { clock, copy } from '../../shared/i18n';
import type { Session } from '../../shared/ipc/types';
import { completionDots } from './completions';
const props = defineProps<{ records: Session[]; seriesId: SeriesId }>();
const dots = computed(() => completionDots(props.records, props.seriesId));
const description = (record: Session) =>
  `${copy.value.completedFocus} · ${clock(record.duration_ms)}`;
</script>
<template>
  <ol
    v-if="dots.length"
    class="flex items-center justify-center gap-1.5"
    :aria-label="copy.completedFocusHistory"
  >
    <li
      v-for="record in dots"
      :key="record.id"
      class="size-6"
      role="img"
      :aria-label="description(record)"
      :title="description(record)"
    >
      <SeriesReward :series-id="seriesId" :duration="record.duration_ms" class="size-full" />
    </li>
  </ol>
</template>
