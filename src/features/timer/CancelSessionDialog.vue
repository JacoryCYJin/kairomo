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
const keep = ref<HTMLButtonElement | null>(null);
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
  keep.value?.focus();
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
    class="modal"
    aria-labelledby="cancel-session-title"
    aria-describedby="cancel-session-hint"
    :aria-busy="busy"
    @cancel.prevent="close"
    @keydown.esc.stop
    @mousedown.stop
  >
    <h2 id="cancel-session-title">{{ copy.confirmCancelSession }}</h2>
    <p id="cancel-session-hint">{{ copy.discardSessionHint }}</p>
    <p v-if="failed" class="!text-danger" role="alert">{{ copy.cancelSessionError }}</p>
    <div class="modal-actions">
      <button ref="keep" class="button secondary" :disabled="busy" @click="close">
        {{ copy.keepSession }}
      </button>
      <button class="button danger" :disabled="busy || !expectedId" @click="confirm">
        {{ copy.discardSession }}
      </button>
    </div>
  </dialog>
</template>
