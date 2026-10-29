<script setup lang="ts">
import { onMounted } from 'vue';
import type { TimerVisualProps } from '../shared/series/contracts';
defineProps<TimerVisualProps>();
const emit = defineEmits<{ prepared: [boolean]; failed: [] }>();
onMounted(() => emit('prepared', true));
</script>
<template>
  <div class="relative h-full w-full" aria-hidden="true">
    <span class="absolute left-2 top-2 size-3 bg-accent" data-canvas-top-probe />
    <span class="absolute bottom-2 right-2 size-3 bg-rest" data-canvas-bottom-probe />
  </div>
</template>
