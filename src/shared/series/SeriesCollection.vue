<script setup lang="ts">
import { computed } from 'vue';
import { seriesFor } from './catalog';
import { seriesRenderers } from './renderers';
import type { SeriesId } from './types';
import type { Session } from '../ipc/types';
const props = withDefaults(
  defineProps<{
    seriesId?: SeriesId;
    records: Session[];
    total: number;
    periodLabel: string | string[];
    fitHeight?: boolean;
  }>(),
  { seriesId: 'coffee', fitHeight: false },
);
const component = computed(() => seriesRenderers[seriesFor(props.seriesId).id].collection);
</script>
<template>
  <component
    :is="component"
    :records="records"
    :total="total"
    :period-label="periodLabel"
    :fit-height="fitHeight"
  />
</template>
