<script setup lang="ts">
import { nextTick, ref, watch } from 'vue';
import { copy } from '../../shared/i18n';
import type { Session } from '../../shared/ipc/types';
const props = defineProps<{
  session: Session | null;
  busy: boolean;
  cancelSession: (id: string) => Promise<boolean>;
}>();
const modal = ref<HTMLDialogElement | null>(null);
const heading = ref<HTMLHeadingElement | null>(null);
const expectedId = ref('');
const failed = ref(false);
function close() {
  if (props.busy) return;
  modal.value?.close();
  expectedId.value = '';
}
async function open() {
  if (!props.session || props.busy) return;
  expectedId.value = props.session.id;
  failed.value = false;
  await nextTick();
  modal.value?.showModal();
  heading.value?.focus({ preventScroll: true });
}
async function confirm() {
  if (!expectedId.value || props.busy) return;
  failed.value = false;
  try {
    const saved = await props.cancelSession(expectedId.value);
    if (saved) {
      modal.value?.close();
      expectedId.value = '';
    } else failed.value = true;
  } catch {
    failed.value = true;
  }
}
watch(
  () => props.session?.id,
  (id) => {
    if (expectedId.value && id !== expectedId.value) {
      modal.value?.close();
      expectedId.value = '';
    }
  },
);
defineExpose({ open });
</script>
<template>
  <dialog
    ref="modal"
    class="m-auto w-72 max-w-[90vw] rounded-xl border border-line bg-surface p-5 text-left text-ink shadow-lg backdrop:bg-black/20"
    aria-labelledby="cancel-session-title"
    aria-describedby="cancel-session-hint"
    :aria-busy="busy"
    @cancel.prevent="close"
    @keydown.esc.stop
    @mousedown.stop
  >
    <h2
      ref="heading"
      tabindex="-1"
      id="cancel-session-title"
      class="m-0 text-base font-semibold outline-none"
    >
      {{ copy.confirmCancelSession }}
    </h2>
    <p id="cancel-session-hint" class="mb-0 mt-2 text-xs leading-relaxed text-muted">
      {{ copy.discardSessionHint }}
    </p>
    <p v-if="failed" class="mb-0 mt-2 text-xs leading-relaxed text-danger" role="alert">
      {{ copy.cancelSessionError }}
    </p>
    <div class="mt-5 flex justify-end gap-2">
      <button
        class="button secondary min-w-20 !min-h-8 !px-3 !py-1.5"
        :disabled="busy"
        @click="close"
      >
        {{ copy.keepSession }}
      </button>
      <button
        class="button danger min-w-20 !min-h-8 !px-3 !py-1.5"
        :disabled="busy || !expectedId"
        @click="confirm"
      >
        {{ copy.discardSession }}
      </button>
    </div>
  </dialog>
</template>
