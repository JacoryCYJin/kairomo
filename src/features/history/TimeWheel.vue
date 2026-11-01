<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from 'vue';
const props = defineProps<{ value: number; count: number; label: string }>();
const emit = defineEmits<{ change: [number] }>();
const track = ref<HTMLDivElement>();
const position = ref(0);
const step = 32;
let quiet: ReturnType<typeof setTimeout> | undefined;
let clickReset: ReturnType<typeof setTimeout> | undefined;
let animation = 0,
  paintFrame = 0,
  writeFrame = 0;
let animating = false,
  dirty = false,
  disposed = false,
  suppressClick = false;
let target: number | undefined;
let offset: number | undefined;
let pointer: { id: number; y: number; offset: number; dragging: boolean } | undefined;
const clamp = (n: number) => Math.max(0, Math.min(props.count - 1, Number.isFinite(n) ? n : 0));
function paint() {
  if (track.value) position.value = clamp(track.value.scrollTop / step);
}
function cancel() {
  clearTimeout(quiet);
  cancelAnimationFrame(animation);
  animation = 0;
  animating = false;
  target = undefined;
}
function begin() {
  cancel();
  dirty = true;
}
function write() {
  if (track.value && offset !== undefined) {
    track.value.scrollTop = offset;
    paint();
  }
}
function queue(value: number) {
  offset = Math.max(0, Math.min((props.count - 1) * step, value));
  if (!writeFrame)
    writeFrame = requestAnimationFrame(() => {
      writeFrame = 0;
      write();
    });
}
function publish(index: number) {
  dirty = false;
  if (index !== props.value) emit('change', index);
}
function align(value: number) {
  if (!track.value || disposed) return;
  const index = clamp(value);
  cancel();
  cancelAnimationFrame(writeFrame);
  writeFrame = 0;
  offset = undefined;
  const element = track.value,
    from = element.scrollTop,
    to = index * step;
  const finish = () => {
    element.scrollTop = to;
    paint();
    animation = 0;
    animating = false;
    target = undefined;
    publish(index);
  };
  if (Math.abs(to - from) < 0.5 || window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
    finish();
    return;
  }
  animating = true;
  target = index;
  const duration = Math.min(220, 140 + Math.abs(to - from) * 0.3);
  let started: number | undefined;
  function frame(now: number) {
    if (disposed) return;
    started ??= now;
    const progress = Math.min(1, (now - started) / duration);
    element.scrollTop = from + (to - from) * (1 - Math.pow(1 - progress, 3));
    paint();
    if (progress < 1) animation = requestAnimationFrame(frame);
    else finish();
  }
  animation = requestAnimationFrame(frame);
}
function settle() {
  if (dirty && !pointer && !animating) align(Math.round((track.value?.scrollTop ?? 0) / step));
}
function schedule() {
  clearTimeout(quiet);
  if (dirty && !pointer && !animating) quiet = setTimeout(settle, 170);
}
function scrolled() {
  if (!paintFrame)
    paintFrame = requestAnimationFrame(() => {
      paintFrame = 0;
      paint();
    });
  schedule();
}
function wheel(event: WheelEvent) {
  if (event.ctrlKey) return;
  // Keep native vertical scrolling and trackpad momentum. Only the final alignment is animated.
  event.stopPropagation();
  begin();
  schedule();
}
function down(event: PointerEvent) {
  if (event.pointerType === 'touch') {
    begin();
    return;
  }
  if (event.pointerType !== 'mouse' || event.button !== 0 || !track.value) return;
  begin();
  pointer = {
    id: event.pointerId,
    y: event.clientY,
    offset: track.value.scrollTop,
    dragging: false,
  };
}
function move(event: PointerEvent) {
  if (!pointer || pointer.id !== event.pointerId || !track.value) return;
  const delta = event.clientY - pointer.y;
  if (!pointer.dragging && Math.abs(delta) < 3) return;
  if (!pointer.dragging) {
    pointer.dragging = true;
    track.value.setPointerCapture(event.pointerId);
  }
  event.preventDefault();
  queue(pointer.offset - delta);
}
function up(event: PointerEvent) {
  if (event.pointerType === 'touch') {
    schedule();
    return;
  }
  if (!pointer || pointer.id !== event.pointerId) return;
  const dragged = pointer.dragging;
  pointer = undefined;
  if (track.value?.hasPointerCapture(event.pointerId))
    track.value.releasePointerCapture(event.pointerId);
  if (dragged) {
    cancelAnimationFrame(writeFrame);
    writeFrame = 0;
    write();
    offset = undefined;
    suppressClick = true;
    clearTimeout(clickReset);
    clickReset = setTimeout(() => {
      suppressClick = false;
    }, 0);
    settle();
  } else schedule();
}
function choose(index: number) {
  if (suppressClick) return;
  begin();
  align(index);
}
function key(event: KeyboardEvent) {
  let value = target ?? Math.round(position.value);
  if (event.key === 'ArrowDown') value++;
  else if (event.key === 'ArrowUp') value--;
  else if (event.key === 'Home') value = 0;
  else if (event.key === 'End') value = props.count - 1;
  else return;
  event.preventDefault();
  begin();
  align(value);
}
function flush() {
  const changed = dirty;
  const index = clamp(target ?? Math.round((offset ?? track.value?.scrollTop ?? 0) / step));
  cancel();
  cancelAnimationFrame(writeFrame);
  writeFrame = 0;
  offset = undefined;
  if (track.value) track.value.scrollTop = index * step;
  paint();
  dirty = false;
  return { value: index, changed };
}
function sync() {
  if (!dirty && !animating && track.value) {
    track.value.scrollTop = clamp(props.value) * step;
    paint();
  }
}
watch(() => props.value, sync);
onMounted(sync);
onUnmounted(() => {
  disposed = true;
  cancel();
  clearTimeout(clickReset);
  cancelAnimationFrame(paintFrame);
  cancelAnimationFrame(writeFrame);
});
defineExpose({ flush });
</script>
<template>
  <div
    ref="track"
    role="spinbutton"
    :aria-label="label"
    :aria-valuemin="0"
    :aria-valuemax="count - 1"
    :aria-valuenow="Math.round(position)"
    :aria-valuetext="String(Math.round(position)).padStart(2, '0')"
    tabindex="0"
    class="relative z-10 h-40 select-none overflow-y-auto overscroll-contain py-16 text-center text-sm font-medium tabular-nums outline-none [scrollbar-width:none] [touch-action:pan-y] [mask-image:linear-gradient(to_bottom,transparent,black_30%,black_70%,transparent)] focus-visible:outline focus-visible:outline-1 focus-visible:outline-line [&::-webkit-scrollbar]:hidden"
    @scroll="scrolled"
    @wheel="wheel"
    @pointerdown="down"
    @pointermove="move"
    @pointerup="up"
    @pointercancel="up"
    @keydown="key"
  >
    <div
      v-for="i in count"
      :key="i"
      aria-hidden="true"
      class="flex h-8 shrink-0 items-center justify-center text-ink"
      :style="{ opacity: Math.max(0.15, 1 - Math.abs(i - 1 - position) * 0.35) }"
      @click="choose(i - 1)"
    >
      {{ String(i - 1).padStart(2, '0') }}
    </div>
  </div>
</template>
