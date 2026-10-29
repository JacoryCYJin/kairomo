<script setup lang="ts">
import { computed } from 'vue';
import { defaultSeriesId } from './catalog';
import { componentsFor } from './renderers';
import type { SeriesId } from './types';
const props = withDefaults(defineProps<{ seriesId?: SeriesId; duration: number }>(), {
  seriesId: defaultSeriesId,
});
const component = computed(() => componentsFor(props.seriesId).reward);
</script>
<template><component :is="component" :duration="duration" /></template>
