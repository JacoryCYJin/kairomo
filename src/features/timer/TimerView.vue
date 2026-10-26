<script setup lang="ts">
import { computed } from 'vue';
import { Pause, Play, Coffee, Square } from 'lucide-vue-next';
import { clock, copy } from '../../shared/i18n';
import type { Action, Snapshot } from '../../shared/ipc/types';
const props = defineProps<{ state: Snapshot; busy: boolean }>();
const emit = defineEmits<{ action: [Action] }>();
const current = computed(() => props.state.current);
const isBreak = computed(() => current.value?.kind === 'break');
const elapsed = computed(() => current.value?.duration_ms ?? 0);
const target = computed(
  () => current.value?.target_ms ?? props.state.settings.focus_minutes * 60000,
);
const over = computed(() => !!current.value && elapsed.value >= target.value);
const display = computed(() =>
  over.value ? elapsed.value - target.value : target.value - elapsed.value,
);
const label = computed(() =>
  props.state.phase === 'paused'
    ? copy.value.paused
    : isBreak.value
      ? copy.value.breakLegend
      : copy.value.focus,
);
const primaryLabel = computed(() =>
  props.state.phase === 'idle'
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
  <section class="px-5 pb-4 pt-2.5 text-center" :class="{ resting: isBreak }">
    <div class="text-sm font-normal leading-5">{{ label }}</div>
    <div
      class="relative mt-1 whitespace-nowrap text-timer font-light tabular-nums"
      :class="state.phase === 'paused' ? 'text-muted' : 'text-ink'"
      aria-live="off"
    >
      <span v-if="over" class="mr-1 align-[9px] text-timer-sign tracking-normal">+</span
      >{{ clock(display) }}
    </div>
    <div class="mt-4 flex items-center justify-center gap-7">
      <button
        v-if="current"
        class="icon-button !size-8"
        :disabled="busy"
        :aria-label="copy.finish"
        :title="copy.finish"
        @click="emit('action', 'finish')"
      >
        <Square :size="16" />
      </button>
      <button
        class="flex size-14 items-center justify-center rounded-full bg-selected text-ink enabled:hover:bg-line-strong"
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
        v-if="current"
        class="icon-button !size-8"
        :disabled="busy"
        :aria-label="isBreak ? copy.nextFocus : copy.takeBreak"
        :title="isBreak ? copy.nextFocus : copy.takeBreak"
        @click="emit('action', 'switch')"
      >
        <Play v-if="isBreak" :size="17" /><Coffee v-else :size="18" />
      </button>
    </div>
    <p v-if="interruption" class="mt-2 text-xs leading-snug text-muted" role="status">
      {{ interruption }}
    </p>
  </section>
</template>
