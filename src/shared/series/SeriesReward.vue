<script setup lang="ts">
import { computed } from 'vue';
import { seriesFor } from './catalog';
import { seriesRenderers } from './renderers';
import type { SeriesId } from './types';
const props = withDefaults(defineProps<{ seriesId?: SeriesId; duration: number }>(), {
  seriesId: 'coffee',
});
const component = computed(() => seriesRenderers[seriesFor(props.seriesId).id].reward);
</script>
<template><component :is="component" :duration="duration" /></template>
