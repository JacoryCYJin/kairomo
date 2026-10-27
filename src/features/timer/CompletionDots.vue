<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import SeriesReward from '../../shared/series/SeriesReward.vue';
import type { SeriesId } from '../../shared/series/types';
import { clock, copy } from '../../shared/i18n';
import type { Session } from '../../shared/ipc/types';
import { api, desktop } from '../../shared/ipc/client';
import { completionDots } from './completions';
const props = defineProps<{ records: Session[]; seriesId: SeriesId }>();
const now = ref(new Date());
const updateDate = () => {
  now.value = new Date();
};
let midnight: ReturnType<typeof setTimeout> | undefined;
let disposed = false;
let unlistenOpened: (() => void) | undefined;
function scheduleMidnight() {
  clearTimeout(midnight);
  updateDate();
  const date = now.value;
  const next = new Date(date.getFullYear(), date.getMonth(), date.getDate() + 1).getTime();
  midnight = setTimeout(scheduleMidnight, Math.max(1, next - date.getTime()));
}
onMounted(async () => {
  scheduleMidnight();
  window.addEventListener('focus', scheduleMidnight);
  document.addEventListener('visibilitychange', scheduleMidnight);
  if (desktop) {
    const unlisten = await api.onTimerOpened(scheduleMidnight);
    if (disposed) unlisten();
    else unlistenOpened = unlisten;
  }
});
onUnmounted(() => {
  disposed = true;
  unlistenOpened?.();
  clearTimeout(midnight);
  window.removeEventListener('focus', scheduleMidnight);
  document.removeEventListener('visibilitychange', scheduleMidnight);
});
const dots = computed(() => completionDots(props.records, props.seriesId, now.value));
const description = (record: Session) =>
  `${copy.value.completedFocus} · ${clock(record.duration_ms)}`;
</script>
<template>
  <ol
    v-if="dots.length"
    class="flex items-center justify-center gap-1.5"
    :aria-label="copy.completedFocusHistory"
  >
    <li
      v-for="record in dots"
      :key="record.id"
      class="size-6"
      role="img"
      :aria-label="description(record)"
      :title="description(record)"
    >
      <SeriesReward :series-id="seriesId" :duration="record.duration_ms" class="size-full" />
    </li>
  </ol>
</template>
