<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { RefreshCw } from 'lucide-vue-next';
import { copy } from '../../shared/i18n';
import { desktop } from '../../shared/ipc/client';
import { updates } from '../../shared/ipc/updates';
import type { ReleaseInfo, UpdateCheck, UpdateProgress } from '../../shared/ipc/updates';
const info = ref<ReleaseInfo | null>(null);
const result = ref<UpdateCheck | null>(null);
const progress = ref<UpdateProgress | null>(null);
const busy = ref(false),
  failed = ref(false);
let disposed = false,
  unlisten: (() => void) | undefined;
onMounted(async () => {
  if (!desktop) return;
  try {
    info.value = await updates.info();
    const stop = await updates.onProgress((value) => {
      progress.value = value;
    });
    if (disposed) stop();
    else unlisten = stop;
  } catch {
    failed.value = true;
  }
});
onUnmounted(() => {
  disposed = true;
  unlisten?.();
});
const message = computed(() => {
  if (failed.value) return copy.value.updateFailed;
  if (progress.value?.stage === 'installing') return copy.value.updateInstalling;
  if (progress.value) {
    const percent = progress.value.total
      ? Math.floor((progress.value.downloaded / progress.value.total) * 100)
      : null;
    return copy.value.updateDownloading + (percent === null ? '' : ` ${percent}%`);
  }
  if (busy.value) return copy.value.updateChecking;
  if (result.value?.status === 'unpublished') return copy.value.updateUnpublished;
  if (result.value?.status === 'current') return copy.value.updateCurrent;
  if (info.value?.channel === 'dev' || result.value?.status === 'development')
    return copy.value.updateDevelopment;
  return '';
});
async function check() {
  busy.value = true;
  failed.value = false;
  progress.value = null;
  result.value = null;
  try {
    result.value = await updates.check();
  } catch {
    failed.value = true;
  } finally {
    busy.value = false;
  }
}
async function install() {
  if (!result.value?.token || busy.value) return;
  busy.value = true;
  failed.value = false;
  progress.value = { stage: 'downloading', downloaded: 0, total: null };
  try {
    await updates.install(result.value.token);
  } catch {
    failed.value = true;
    progress.value = null;
    result.value = null;
  } finally {
    busy.value = false;
  }
}
</script>
<template>
  <section
    class="mt-4 border-t border-line pt-3"
    :aria-label="copy.softwareUpdates"
    :aria-busy="busy"
  >
    <div class="flex items-center justify-between gap-3">
      <div>
        <h2 class="text-sm font-medium">{{ info?.name ?? copy.softwareUpdates }}</h2>
        <p v-if="info" class="mt-1 text-xs text-faint">{{ info.version }}</p>
      </div>
      <button
        type="button"
        class="button secondary compact"
        :disabled="!desktop || busy || !info || info.channel === 'dev'"
        @click="check"
      >
        <RefreshCw :size="14" :class="busy ? 'animate-spin' : ''" />{{ copy.checkUpdates }}
      </button>
    </div>
    <p
      v-if="message"
      class="mt-3 text-xs leading-relaxed"
      :class="failed ? 'text-danger' : 'text-faint'"
      role="status"
    >
      {{ message }}
    </p>
    <div
      v-if="result?.status === 'available'"
      class="mt-3 rounded-lg border border-line bg-surface-muted p-3"
    >
      <p class="text-sm font-medium">{{ copy.updateAvailable }} {{ result.version }}</p>
      <p v-if="result.notes" class="mt-2 whitespace-pre-wrap text-xs leading-relaxed text-muted">
        {{ result.notes }}
      </p>
      <p class="mt-2 text-xs leading-relaxed text-faint">{{ copy.updateInstallHint }}</p>
      <div class="mt-3 flex justify-end">
        <button type="button" class="button primary compact" :disabled="busy" @click="install">
          {{ copy.installUpdate }}
        </button>
      </div>
    </div>
  </section>
</template>
