<script setup lang="ts">
import { computed, ref } from 'vue';
import { copy } from '../../shared/i18n';
import { hexToHsv, hsvToHex } from '../../shared/taskColor';
const props = defineProps<{ taken: string[]; disabled: boolean; initial?: string | null }>();
const emit = defineEmits<{ add: [string]; cancel: [] }>();
const initial = hexToHsv(props.initial ?? '#3e63dd');
const hue = ref(initial.h);
const saturation = ref(initial.s);
const value = ref(initial.v);
const area = ref<HTMLDivElement | null>(null);
const hex = computed(() => hsvToHex(hue.value, saturation.value, value.value));
const taken = computed(() => props.taken.includes(hex.value));
function pick(event: PointerEvent) {
  const element = area.value;
  if (!element) return;
  const bounds = element.getBoundingClientRect();
  saturation.value = Math.min(1, Math.max(0, (event.clientX - bounds.left) / bounds.width));
  value.value = Math.min(1, Math.max(0, 1 - (event.clientY - bounds.top) / bounds.height));
}
function start(event: PointerEvent) {
  area.value?.setPointerCapture(event.pointerId);
  pick(event);
}
function move(event: PointerEvent) {
  if (area.value?.hasPointerCapture(event.pointerId)) pick(event);
}
function stop(event: PointerEvent) {
  area.value?.releasePointerCapture(event.pointerId);
}
</script>
<template>
  <div class="mt-3 rounded-xl border border-line p-3">
    <div
      ref="area"
      class="relative h-28 w-full cursor-crosshair touch-none overflow-hidden rounded-lg"
      :style="{ backgroundColor: `hsl(${hue} 100% 50%)` }"
      @pointerdown.prevent="start"
      @pointermove="move"
      @pointerup="stop"
      @pointercancel="stop"
    >
      <span
        aria-hidden="true"
        class="pointer-events-none absolute inset-0"
        :style="{ background: 'linear-gradient(to right, #fff, rgb(255 255 255 / 0))' }"
      />
      <span
        aria-hidden="true"
        class="pointer-events-none absolute inset-0"
        :style="{ background: 'linear-gradient(to top, #000, rgb(0 0 0 / 0))' }"
      />
      <span
        aria-hidden="true"
        class="pointer-events-none absolute size-3 -translate-x-1/2 -translate-y-1/2 rounded-full border-2 border-white shadow"
        :style="{ left: `${saturation * 100}%`, top: `${(1 - value) * 100}%` }"
      />
    </div>
    <input
      v-model.number="hue"
      type="range"
      class="hue-slider mt-3"
      min="0"
      max="360"
      step="1"
      :aria-label="copy.categoryColorHue"
    />
    <div class="mt-3 flex items-center gap-2">
      <span
        class="size-7 shrink-0 rounded-full border border-line-strong"
        :style="{ backgroundColor: hex }"
      />
      <span class="text-xs tabular-nums text-muted">{{ hex }}</span>
      <div class="ml-auto flex gap-2">
        <button
          type="button"
          class="button secondary compact"
          :disabled="disabled"
          @click="emit('cancel')"
        >
          {{ copy.cancel }}</button
        ><button
          type="button"
          class="button dialog-confirm compact"
          :disabled="disabled || taken"
          @click="emit('add', hex)"
        >
          {{ copy.categoryColorAdd }}
        </button>
      </div>
    </div>
  </div>
</template>
