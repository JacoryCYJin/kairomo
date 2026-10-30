<script setup lang="ts">
import { nextTick, onUnmounted, ref } from 'vue';
import { CircleCheck } from 'lucide-vue-next';
import { copy } from '../../shared/i18n';
const props = defineProps<{ busy: boolean; clearRecords: () => Promise<boolean> }>();
const modal = ref<HTMLDialogElement | null>(null);
const trigger = ref<HTMLButtonElement | null>(null);
const heading = ref<HTMLHeadingElement | null>(null);
const pending = ref(false);
const failed = ref(false);
const toastVisible = ref(false);
let toastTimer: ReturnType<typeof setTimeout> | undefined;
let toastRemaining = 5000;
let toastStarted = 0;
function dismissToast() {
  clearTimeout(toastTimer);
  toastTimer = undefined;
  toastVisible.value = false;
}
function resumeToast() {
  if (!toastVisible.value || toastTimer !== undefined) return;
  toastStarted = performance.now();
  toastTimer = setTimeout(dismissToast, toastRemaining);
}
function pauseToast() {
  if (toastTimer === undefined) return;
  clearTimeout(toastTimer);
  toastTimer = undefined;
  toastRemaining = Math.max(0, toastRemaining - (performance.now() - toastStarted));
}
function showToast() {
  dismissToast();
  toastRemaining = 5000;
  toastVisible.value = true;
  resumeToast();
}
onUnmounted(dismissToast);
function open() {
  if (props.busy || pending.value || modal.value?.open) return;
  failed.value = false;
  dismissToast();
  modal.value?.showModal();
  heading.value?.focus({ preventScroll: true });
}
function close() {
  if (!pending.value) modal.value?.close();
}
async function restoreFocus() {
  await nextTick();
  trigger.value?.focus({ preventScroll: true });
}
async function confirm() {
  if (props.busy || pending.value) return;
  pending.value = true;
  failed.value = false;
  try {
    if (await props.clearRecords()) {
      modal.value?.close();
      showToast();
    } else failed.value = true;
  } catch {
    failed.value = true;
  } finally {
    pending.value = false;
  }
}
</script>
<template>
  <section class="mt-4" aria-labelledby="record-settings-title">
    <h2 id="record-settings-title" class="mb-2.5 text-sm font-medium">
      {{ copy.historySettings }}
    </h2>
    <div
      class="flex min-h-12 items-center justify-between gap-4 rounded-xl border border-line bg-surface px-3.5 py-2"
    >
      <span class="text-xs">{{ copy.completedFocusRecords }}</span>
      <button
        ref="trigger"
        type="button"
        class="button secondary compact shrink-0"
        :aria-label="copy.clearHistory"
        :disabled="busy || pending"
        @click="open"
      >
        {{ copy.clearRecordsAction }}
      </button>
    </div>
  </section>
  <dialog
    ref="modal"
    class="modal"
    role="alertdialog"
    aria-labelledby="clear-records-title"
    aria-describedby="clear-records-hint"
    :aria-busy="pending"
    @cancel.prevent="close"
    @close="restoreFocus"
    @keydown.esc.stop
    @mousedown.stop
  >
    <h2 ref="heading" id="clear-records-title" tabindex="-1" class="outline-none">
      {{ copy.confirmClearRecords }}
    </h2>
    <p id="clear-records-hint">
      {{ copy.clearRecordsWarning }}
    </p>
    <p v-if="failed" class="!text-danger" role="alert">{{ copy.clearRecordsError }}</p>
    <div class="modal-actions">
      <div class="inline-grid auto-cols-fr grid-flow-col gap-2">
        <button type="button" class="button secondary" :disabled="pending" @click="close">
          {{ copy.keepRecords }}
        </button>
        <button
          type="button"
          class="button dialog-confirm grid"
          :disabled="busy || pending"
          @click="confirm"
        >
          <span class="invisible col-start-1 row-start-1" aria-hidden="true">{{
            copy.clearRecords
          }}</span>
          <span class="col-start-1 row-start-1">{{
            pending ? copy.clearingRecords : copy.clearRecords
          }}</span>
        </button>
      </div>
    </div>
  </dialog>
  <div
    class="pointer-events-none fixed inset-x-0 top-16 z-50 flex justify-center px-4"
    role="status"
    aria-live="polite"
    aria-atomic="true"
  >
    <div
      v-if="toastVisible"
      class="pointer-events-auto flex max-w-full items-center gap-2 rounded-lg border border-transparent bg-dialog-confirm px-4 py-3 text-xs text-dialog-confirm-ink shadow-sm"
      @pointerenter="pauseToast"
      @pointerleave="resumeToast"
    >
      <CircleCheck :size="16" aria-hidden="true" />
      <span>{{ copy.recordsCleared }}</span>
    </div>
  </div>
</template>
