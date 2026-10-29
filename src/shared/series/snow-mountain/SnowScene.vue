<script setup lang="ts">
import { computed } from 'vue';
import SnowClimber from './SnowClimber.vue';
import { snowAssets } from './assets';
import { routePosition, slopePosition } from './progress';
const props = withDefaults(
  defineProps<{ progress?: number; mode?: 'collection' | 'focus' | 'rest'; dark?: boolean }>(),
  { progress: 0, mode: 'collection', dark: false },
);
const viewport = computed(() =>
  props.mode === 'collection'
    ? '36 60 1184 1122'
    : props.mode === 'rest'
      ? '18 0 2136 710'
      : '0 0 2172 724',
);
const artwork = computed(() =>
  props.mode === 'collection' ? snowAssets.mountain : snowAssets[props.mode],
);
const position = computed(() =>
  props.mode === 'collection' ? routePosition(props.progress) : slopePosition(props.progress),
);
</script>
<template>
  <svg
    xmlns="http://www.w3.org/2000/svg"
    :viewBox="viewport"
    preserveAspectRatio="xMidYMax meet"
    class="snow-scene"
    aria-hidden="true"
    :data-snow-scene="mode"
  >
    <image
      :href="artwork"
      :width="mode === 'collection' ? 1254 : 2172"
      :height="mode === 'collection' ? 1254 : 724"
    />
    <SnowClimber
      v-if="mode !== 'rest'"
      :x="position.x - (mode === 'collection' ? 27 : 60)"
      :y="position.y - (mode === 'collection' ? 68 : 148)"
      :width="mode === 'collection' ? 56 : 124"
      :height="mode === 'collection' ? 70 : 150"
    />
  </svg>
</template>
<style scoped>
.snow-scene {
  overflow: hidden;
  image-rendering: pixelated;
}
:global(:root[data-theme='dark']) .snow-scene {
  filter: brightness(0.88) saturate(0.84);
}
</style>
