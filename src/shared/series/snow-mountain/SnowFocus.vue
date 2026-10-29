<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue';
import type { TimerVisualProps } from '../contracts';
import SnowScene from './SnowScene.vue';
import { focusProgress } from './progress';
import { prepareSnowAssets, snowAssets } from './assets';
const props = defineProps<TimerVisualProps>();
const emit = defineEmits<{ prepared: [boolean]; failed: [] }>();
let disposed = false;
const progress = computed(() =>
  props.state.current?.kind === 'focus'
    ? focusProgress(props.state.current.duration_ms, props.state.current.target_ms)
    : 0,
);
onMounted(() => {
  emit('prepared', false);
  void prepareSnowAssets([snowAssets.focus, snowAssets.climber]).then(
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
    <SnowScene
      mode="focus"
      :progress="progress"
      :dark="dark"
      class="absolute inset-x-0 bottom-0 block w-full"
    />
  </div>
</template>
