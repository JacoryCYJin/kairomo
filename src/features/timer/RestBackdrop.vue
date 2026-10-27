<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import type { RestVisual } from '../../shared/series/types';
const props = defineProps<{
  remaining: number;
  running: boolean;
  dark: boolean;
  visual: RestVisual;
}>();
const palette = computed(() => (props.dark ? props.visual.dark : props.visual.light));
const level = computed(() => (1 - props.remaining * 0.94) * 100);
// The last sip settles as it empties, avoiding a final wave that snaps away.
const settling = computed(() => Math.min(1, props.remaining / 0.08));
const surface = ref<HTMLDivElement | null>(null);
const height = ref(280);
let observer: ResizeObserver | undefined;
onMounted(() => {
  observer = new ResizeObserver(([entry]) => {
    if (entry && entry.contentRect.height > 0) height.value = entry.contentRect.height;
  });
  if (surface.value) observer.observe(surface.value);
});
onUnmounted(() => observer?.disconnect());
function liquidPath(amplitude: number) {
  return `M0 ${amplitude} C60 0 120 0 180 ${amplitude} S300 ${2 * amplitude} 360 ${amplitude} S480 0 540 ${amplitude} S660 ${2 * amplitude} 720 ${amplitude} V${height.value + amplitude} H0Z`;
}
</script>
<template>
  <div
    ref="surface"
    class="pointer-events-none absolute inset-0 -z-10 overflow-hidden"
    aria-hidden="true"
  >
    <div
      v-for="(wave, index) in visual.waves"
      v-show="remaining > 0"
      :key="index"
      class="liquid-level absolute inset-x-0 top-0 h-full"
      :style="{
        transform: `translateY(${level}%)`,
        transition: running ? 'transform 1000ms linear' : 'none',
        '--wave-amplitude': `${wave.amplitude * settling}px`,
        '--wave-period': `${wave.period}s`,
        color: palette.waves[index],
      }"
    >
      <svg
        class="liquid-wave absolute left-0 w-[200%]"
        :style="{ animationDirection: index === 1 ? 'reverse' : 'normal' }"
        :viewBox="`0 0 720 ${height + wave.amplitude * settling}`"
        preserveAspectRatio="none"
      >
        <path :d="liquidPath(wave.amplitude * settling)" fill="currentColor" />
      </svg>
    </div>
  </div>
</template>
<style scoped>
.liquid-wave {
  top: calc(-1 * var(--wave-amplitude));
  /* One closed path paints both surface and body, eliminating a raster seam. */
  height: calc(100% + var(--wave-amplitude));
  animation: liquid-drift var(--wave-period) linear infinite;
  transform-box: border-box;
}
@keyframes liquid-drift {
  from {
    transform: translateX(0);
  }
  to {
    transform: translateX(-50%);
  }
}
@media (prefers-reduced-motion: reduce) {
  .liquid-wave {
    animation: none;
  }
  .liquid-level {
    transition: none !important;
  }
}
</style>
