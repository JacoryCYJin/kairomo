<script setup lang="ts">
import { computed } from 'vue';
import { defaultSeriesId } from './catalog';
import { componentsFor } from './renderers';
import type { SeriesId } from './types';
import type { Session } from '../ipc/types';
const props = withDefaults(
  defineProps<{
    seriesId?: SeriesId;
    records: Session[];
    completedRecords?: Session[];
    total: number;
    fitHeight?: boolean;
  }>(),
  { seriesId: defaultSeriesId, fitHeight: false, completedRecords: undefined },
);
const component = computed(() => componentsFor(props.seriesId).collection);
</script>
<template>
  <component
    :is="component"
    :records="records"
    :completed-records="completedRecords"
    :total="total"
    :fit-height="fitHeight"
  />
</template>
