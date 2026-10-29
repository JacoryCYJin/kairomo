<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import type { Component } from 'vue';
import type { ThemeCanvasSize, TimerVisualProps } from './contracts';
const props = defineProps<
  TimerVisualProps & { visual: Component<TimerVisualProps>; kind: 'focus' | 'rest' }
>();
const emit = defineEmits<{ prepared: [boolean]; failed: [] }>();
const surface = ref<HTMLDivElement | null>(null);
const size = ref<ThemeCanvasSize>({ width: 0, height: 0 });
const visualReady = ref(false);
const ready = computed(() => visualReady.value && size.value.width > 0 && size.value.height > 0);
watch(ready, (value) => emit('prepared', value), { immediate: true });
let observer: ResizeObserver | undefined;
function measure() {
  const rect = surface.value?.getBoundingClientRect();
  if (rect) size.value = { width: rect.width, height: rect.height };
}
onMounted(() => {
  measure();
  observer = new ResizeObserver(measure);
  if (surface.value) observer.observe(surface.value);
});
onUnmounted(() => observer?.disconnect());
</script>
<template>
  <div
    ref="surface"
    class="pointer-events-none absolute inset-0 z-0 isolate overflow-hidden rounded-[inherit]"
    :data-theme-canvas="kind"
  >
    <component
      :is="props.visual"
      :state="state"
      :records="records"
      :dark="dark"
      :warming-up="warmingUp"
      :canvas="size"
      @prepared="visualReady = $event"
      @failed="
        visualReady = false;
        emit('failed');
      "
    />
  </div>
</template>
