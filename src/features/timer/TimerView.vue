<script setup lang="ts">
import { computed, ref } from 'vue';
import { Pause, Play, SkipForward, CircleX } from 'lucide-vue-next';
import CompletionDots from './CompletionDots.vue';
import CancelSessionDialog from './CancelSessionDialog.vue';
import CoffeeCupIcon from '../../shared/ui/CoffeeCupIcon.vue';
import { clock, copy } from '../../shared/i18n';
import type { Action, Session, Snapshot } from '../../shared/ipc/types';
const props = defineProps<{
  state: Snapshot;
  busy: boolean;
  records: Session[];
  cancelSession: (id: string) => Promise<boolean>;
}>();
const cancellation = ref<InstanceType<typeof CancelSessionDialog> | null>(null);
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
  <section class="flex h-full flex-col items-center justify-center px-5 py-8 text-center">
    <div
      class="relative whitespace-nowrap text-timer font-medium tabular-nums"
      :class="state.phase === 'paused' ? 'text-muted' : 'text-ink'"
      aria-live="off"
    >
      <span v-if="over" class="mr-1 align-[9px] text-timer-sign tracking-normal">+</span
      >{{ clock(display) }}
    </div>
    <div class="mt-2 h-6">
      <CompletionDots :records="records" :series-id="state.settings.visual_series" />
    </div>
    <div class="mt-2.5 grid grid-cols-[32px_56px_32px] items-center justify-center gap-7">
      <button
        v-if="current"
        class="icon-button pointer-events-none col-start-1 row-start-1 !size-8 opacity-0 focus-visible:pointer-events-auto focus-visible:opacity-100 disabled:opacity-0 group-hover:pointer-events-auto group-hover:opacity-100 group-hover:disabled:opacity-50"
        :disabled="busy"
        :aria-label="copy.cancelSession"
        :title="copy.cancelSession"
        @click="cancellation?.open()"
      >
        <CircleX :size="18" />
      </button>
      <button
        class="col-start-2 row-start-1 flex size-14 items-center justify-center rounded-full bg-selected text-ink enabled:hover:bg-line-strong"
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
        class="icon-button pointer-events-none col-start-3 row-start-1 !size-8 opacity-0 focus-visible:pointer-events-auto focus-visible:opacity-100 disabled:opacity-0 group-hover:pointer-events-auto group-hover:opacity-100 group-hover:disabled:opacity-50"
        :disabled="busy"
        :aria-label="isBreak ? copy.nextFocus : copy.takeBreak"
        :title="isBreak ? copy.nextFocus : copy.takeBreak"
        @click="emit('action', 'switch')"
      >
        <SkipForward v-if="isBreak" :size="17" /><CoffeeCupIcon v-else class="size-[18px]" />
      </button>
    </div>
    <p
      v-if="interruption"
      class="absolute inset-x-5 bottom-2 text-xs leading-snug text-muted"
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
