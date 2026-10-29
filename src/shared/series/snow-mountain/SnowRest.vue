<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import type { TimerVisualProps } from '../contracts';
import SnowScene from './SnowScene.vue';
import { prepareSnowAssets, snowAssets } from './assets';
defineProps<TimerVisualProps>();
const emit = defineEmits<{ prepared: [boolean]; failed: [] }>();
let disposed = false;
onMounted(() => {
  emit('prepared', false);
  void prepareSnowAssets([snowAssets.rest]).then(
    () => {
      if (!disposed) emit('prepared', true);
    },
    () => {
      if (!disposed) emit('failed');
    },
  );
});
onUnmounted(() => {
  disposed = true;
});
</script>
<template>
  <div class="relative h-full w-full" aria-hidden="true">
    <SnowScene mode="rest" :dark="dark" class="absolute inset-x-0 bottom-0 block w-full" />
  </div>
</template>
