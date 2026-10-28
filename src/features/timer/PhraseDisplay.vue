<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
import type { Snapshot } from '../../shared/ipc/types';
import { copy } from '../../shared/i18n';
import { api, desktop } from '../../shared/ipc/client';
import { marqueeTiming, timerLayout } from './timerLayout';
const props = defineProps<{ state: Snapshot }>();
const layout = computed(() => timerLayout(props.state));
const chosen = computed(() => layout.value.phrase?.text ?? '');
const attribution = computed(() => layout.value.attribution);
const viewport = ref<HTMLElement | null>(null);
const text = ref<HTMLElement | null>(null);
const overflowing = ref(false);
const reduceMotion = window.matchMedia('(prefers-reduced-motion: reduce)');
let observer: ResizeObserver | undefined;
let animation: Animation | undefined;
let unlisten: (() => void) | undefined;
let disposed = false;
function measure() {
  animation?.cancel();
  animation = undefined;
  if (disposed || !viewport.value || !text.value) return;
  const overflow = Math.max(0, text.value.scrollWidth - viewport.value.clientWidth);
  overflowing.value = overflow > 1;
  if (overflow <= 1 || reduceMotion.matches) return;
  const timing = marqueeTiming(overflow);
  animation = text.value.animate(
    [
      { transform: 'translateX(0)', offset: 0 },
      { transform: 'translateX(0)', offset: timing.start },
      { transform: `translateX(-${overflow}px)`, offset: timing.end },
      { transform: `translateX(-${overflow}px)`, offset: 1 },
    ],
    { duration: timing.duration, iterations: Infinity, direction: 'alternate', easing: 'linear' },
  );
  if (document.hidden) animation.pause();
}
async function restart() {
  await nextTick();
  if (!disposed) measure();
}
function visibility() {
  if (document.hidden) animation?.pause();
  else animation?.play();
}
watch(chosen, restart);
onMounted(async () => {
  observer = new ResizeObserver(measure);
  if (viewport.value) observer.observe(viewport.value);
  if (text.value) observer.observe(text.value);
  measure();
  void document.fonts.ready.then(() => {
    if (!disposed) measure();
  });
  document.fonts.addEventListener('loadingdone', measure);
  document.addEventListener('visibilitychange', visibility);
  reduceMotion.addEventListener('change', measure);
  if (desktop) {
    const stop = await api.onTimerOpened(() => {
      void restart();
    });
    if (disposed) stop();
    else unlisten = stop;
  }
});
onUnmounted(() => {
  disposed = true;
  animation?.cancel();
  observer?.disconnect();
  unlisten?.();
  document.fonts.removeEventListener('loadingdone', measure);
  document.removeEventListener('visibilitychange', visibility);
  reduceMotion.removeEventListener('change', measure);
});
</script>
<template>
  <div
    class="pointer-events-none flex w-full flex-col gap-1"
    role="note"
    :aria-label="copy.customPhrases"
  >
    <div
      ref="viewport"
      class="h-[18px] w-full overflow-hidden"
      :style="
        overflowing
          ? {
              maskImage:
                'linear-gradient(to right,transparent,#000 5px,#000 calc(100% - 5px),transparent)',
            }
          : undefined
      "
    >
      <p
        ref="text"
        class="w-max max-w-none whitespace-nowrap text-xs leading-[18px] text-muted"
        :class="overflowing ? 'mx-0' : 'mx-auto'"
        aria-live="off"
      >
        {{ chosen }}
      </p>
    </div>
    <p v-if="attribution" class="w-full truncate text-xs leading-[14px] text-faint">
      {{ attribution }}
    </p>
  </div>
</template>
