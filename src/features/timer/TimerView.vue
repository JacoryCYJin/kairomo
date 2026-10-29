<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { Pause, Play, SkipForward, X } from 'lucide-vue-next';
import ThemeCanvas from '../../shared/series/ThemeCanvas.vue';
import { componentsFor } from '../../shared/series/renderers';
import PhraseDisplay from './PhraseDisplay.vue';
import CancelSessionDialog from './CancelSessionDialog.vue';
import { timerLayout } from './timerLayout';
import { seriesThemeStyle } from '../../shared/series/rest';
import { clock, copy } from '../../shared/i18n';
import type { Action, Session, Snapshot } from '../../shared/ipc/types';
const props = defineProps<{
  state: Snapshot;
  busy: boolean;
  records: Session[];
  dark?: boolean;
  warmingUp?: boolean;
  cancelSession: (id: string) => Promise<boolean>;
}>();
const cancellation = ref<InstanceType<typeof CancelSessionDialog> | null>(null);
const emit = defineEmits<{ action: [Action]; prepared: [ready: boolean]; visualFailed: [] }>();
const layout = computed(() => timerLayout(props.state));
const current = computed(() => props.state.current);
const isBreak = computed(() => current.value?.kind === 'break');
const visualSeries = computed(() => props.state.settings.visual_series);
const kit = computed(() => componentsFor(visualSeries.value));
const focusPrepared = ref(false),
  restPrepared = ref(false);
watch(visualSeries, () => {
  focusPrepared.value = false;
  restPrepared.value = false;
});
watch(
  [isBreak, focusPrepared, restPrepared],
  () => emit('prepared', isBreak.value ? restPrepared.value : focusPrepared.value),
  { immediate: true },
);
const restStyle = computed(() => seriesThemeStyle(visualSeries.value, !!props.dark, isBreak.value));
const readyToStart = computed(
  () => props.state.phase === 'paused' && current.value?.intervals.length === 0,
);
const hasStarted = computed(() => (current.value?.intervals.length ?? 0) > 0);
const elapsed = computed(() => current.value?.duration_ms ?? 0);
const target = computed(
  () => current.value?.target_ms ?? props.state.settings.focus_minutes * 60000,
);
const over = computed(() => !!current.value && elapsed.value >= target.value);
const display = computed(() =>
  over.value ? elapsed.value - target.value : target.value - elapsed.value,
);
const primaryLabel = computed(() =>
  readyToStart.value
    ? isBreak.value
      ? copy.value.startBreak
      : copy.value.start
    : props.state.phase === 'idle'
      ? copy.value.start
      : props.state.phase === 'paused'
        ? copy.value.resume
        : copy.value.pause,
);
const interruption = computed(() =>
  props.state.needs_recovery
    ? copy.value.recoveryHint
    : ['sleep', 'interrupted'].includes(props.state.pause_reason ?? '')
      ? copy.value.sleepHint
      : '',
);
</script>
<template>
  <section
    class="relative isolate h-full text-center"
    :class="isBreak ? 'bg-paper' : ''"
    :style="restStyle"
  >
    <ThemeCanvas
      v-if="isBreak"
      :visual="kit.rest"
      kind="rest"
      :key="`rest-${visualSeries}`"
      :state="state"
      :records="records"
      :dark="dark"
      :warming-up="warmingUp"
      @prepared="restPrepared = $event"
      @failed="emit('visualFailed')"
    />
    <div
      class="absolute inset-x-2.5 top-[43px] z-10 whitespace-nowrap text-timer font-medium !leading-[90px] tabular-nums"
      :class="state.phase === 'paused' && !readyToStart ? 'text-muted' : 'text-ink'"
      aria-live="off"
    >
      <span v-if="over" class="mr-1 align-[9px] text-timer-sign tracking-normal">+</span
      >{{ clock(display) }}
    </div>
    <div v-if="layout.phrase" class="absolute inset-x-[34px] top-[145px] z-10">
      <PhraseDisplay :state="state" />
    </div>
    <ThemeCanvas
      :visual="kit.focus"
      kind="focus"
      :key="`focus-${visualSeries}`"
      :state="state"
      :records="records"
      :dark="dark"
      :warming-up="warmingUp"
      v-show="!isBreak"
      @prepared="focusPrepared = $event"
      @failed="emit('visualFailed')"
    />
    <div
      class="group/controls absolute left-1/2 z-10 grid -translate-x-1/2 grid-cols-[32px_48px_32px] items-center justify-center gap-7"
      :style="{ top: `${layout.controlsTop}px` }"
    >
      <button
        v-if="hasStarted && !isBreak"
        class="icon-button pointer-events-none col-start-1 row-start-1 !size-8 translate-x-[68px] opacity-0 transition-[transform,opacity] duration-200 ease-out disabled:opacity-0 group-has-[:focus-visible]/controls:pointer-events-auto group-has-[:focus-visible]/controls:translate-x-0 group-has-[:focus-visible]/controls:opacity-100 group-hover:pointer-events-auto group-hover:translate-x-0 group-hover:opacity-100 group-hover:disabled:opacity-50"
        :disabled="busy"
        :aria-label="copy.cancelSession"
        :title="copy.cancelSession"
        @click="cancellation?.open()"
      >
        <X :size="20" />
      </button>
      <button
        class="relative z-10 col-start-2 row-start-1 flex size-12 items-center justify-center rounded-full bg-selected text-ink enabled:hover:bg-line-strong"
        :disabled="busy"
        :aria-label="primaryLabel"
        :title="primaryLabel"
        @click="
          emit(
            'action',
            state.phase === 'idle' ? 'start' : state.phase === 'paused' ? 'resume' : 'pause',
          )
        "
      >
        <Pause v-if="state.phase === 'running'" :size="25" stroke-width="1.7" /><Play
          v-else
          :size="25"
          stroke-width="1.7"
          class="ml-1"
        />
      </button>
      <button
        v-if="hasStarted"
        class="icon-button pointer-events-none col-start-3 row-start-1 !size-8 -translate-x-[68px] opacity-0 transition-[transform,opacity] duration-200 ease-out disabled:opacity-0 group-has-[:focus-visible]/controls:pointer-events-auto group-has-[:focus-visible]/controls:translate-x-0 group-has-[:focus-visible]/controls:opacity-100 group-hover:pointer-events-auto group-hover:translate-x-0 group-hover:opacity-100 group-hover:disabled:opacity-50"
        :disabled="busy"
        :aria-label="isBreak ? copy.nextFocus : copy.takeBreak"
        :title="isBreak ? copy.nextFocus : copy.takeBreak"
        @click="emit('action', 'switch')"
      >
        <SkipForward v-if="isBreak" :size="17" /><component
          :is="kit.restIcon"
          v-else
          class="size-[18px]"
        />
      </button>
    </div>
    <p
      v-if="interruption"
      class="absolute inset-x-5 bottom-2 z-20 rounded bg-paper/95 text-xs leading-snug text-muted"
      role="status"
    >
      {{ interruption }}
    </p>
    <CancelSessionDialog
      ref="cancellation"
      :session="current"
      :busy="busy"
      :cancel-session="cancelSession"
    />
  </section>
</template>
