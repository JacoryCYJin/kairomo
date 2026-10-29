<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { Check, RefreshCw } from 'lucide-vue-next';
import { copy } from '../../shared/i18n';
import { desktop } from '../../shared/ipc/client';
import { updates } from '../../shared/ipc/updates';
import type { ReleaseInfo, UpdateCheck, UpdateProgress } from '../../shared/ipc/updates';
const info = ref<ReleaseInfo | null>(null);
const result = ref<UpdateCheck | null>(null);
const progress = ref<UpdateProgress | null>(null);
const showResult = ref(false);
let resultTimer: ReturnType<typeof setTimeout> | undefined;
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
  clearTimeout(resultTimer);
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
const development = computed(() => info.value?.channel === 'dev');
const buttonLabel = computed(() => {
  if (progress.value?.stage === 'installing') return copy.value.updateInstall;
  if (progress.value) {
    const percent = progress.value.total
      ? Math.floor((progress.value.downloaded / progress.value.total) * 100)
      : null;
    return copy.value.updateDownload + (percent === null ? '' : ` ${percent}%`);
  }
  if (busy.value) return copy.value.checkUpdates;
  if (failed.value) return copy.value.updateRetry;
  if (showResult.value && result.value?.status === 'current') return copy.value.updateUpToDate;
  if (showResult.value && result.value?.status === 'unpublished') return copy.value.updateNone;
  return copy.value.checkUpdates;
});
async function check() {
  if (busy.value || development.value) return;
  clearTimeout(resultTimer);
  showResult.value = false;
  busy.value = true;
  failed.value = false;
  progress.value = null;
  result.value = null;
  try {
    result.value = await updates.check();
    if (!disposed) {
      showResult.value = true;
      resultTimer = setTimeout(() => {
        showResult.value = false;
      }, 3000);
    }
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
  <section class="mt-4" :aria-label="copy.softwareUpdates" :aria-busy="busy">
    <div class="flex h-9 items-center justify-between gap-3 text-xs">
      <p class="min-w-0 text-muted">
        {{ copy.version }} <span class="tabular-nums">{{ info?.version ?? '—' }}</span>
      </p>
      <span v-if="development" class="shrink-0 text-muted" :title="copy.updateDevelopment">
        {{ copy.updateDev }}
      </span>
      <button
        v-else
        type="button"
        class="inline-flex h-9 w-40 shrink-0 items-center justify-end gap-2 whitespace-nowrap text-xs disabled:cursor-default"
        :class="failed ? 'text-danger' : 'text-muted'"
        :disabled="!desktop || busy || !info"
        :aria-label="copy.checkUpdates"
        :title="failed ? copy.updateFailed : copy.checkUpdates"
        @click="check"
      >
        <Check v-if="showResult && result?.status === 'current' && !busy" :size="14" />
        <RefreshCw v-else :size="14" :class="busy ? 'animate-spin' : ''" />
        {{ buttonLabel }}
      </button>
    </div>
    <p class="sr-only" role="status">{{ message }}</p>
    <div
      v-if="result?.status === 'available'"
      class="mt-3 rounded-lg border border-line bg-surface-muted p-3"
    >
      <p class="text-sm font-medium">{{ copy.updateAvailable }} {{ result.version }}</p>
      <p v-if="result.notes" class="mt-2 whitespace-pre-wrap text-xs leading-relaxed text-muted">
        {{ result.notes }}
      </p>
      <p class="mt-2 text-xs leading-relaxed text-faint">{{ copy.updateInstallHint }}</p>
      <p v-if="progress" class="mt-2 text-xs leading-relaxed text-muted" role="status">
        {{ message }}
      </p>
      <div class="mt-3 flex justify-end">
        <button type="button" class="button primary compact" :disabled="busy" @click="install">
          {{ copy.installUpdate }}
        </button>
      </div>
    </div>
  </section>
</template>
