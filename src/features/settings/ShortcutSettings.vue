<script setup lang="ts">
import { computed, onBeforeUnmount, reactive, ref, watch } from 'vue';
import { ChevronRight, Pencil, RotateCcw } from 'lucide-vue-next';
import { copy, locale } from '../../shared/i18n';
import {
  bindingEquals,
  bindingFromKeyboardEvent,
  bindingKeycaps,
  bindingText,
  defaultShortcutBindings,
  isModifierCode,
  modifierLabel,
  modifiersFromEvent,
} from '../../shared/shortcutKeys';
import type {
  Settings,
  ShortcutBinding,
  ShortcutKind,
  ShortcutModifier,
  ShortcutStatus,
} from '../../shared/ipc/types';
import SettingsSubpage from './SettingsSubpage.vue';

const props = defineProps<{
  settings: Settings;
  shortcuts?: ShortcutStatus | null;
  busy: boolean;
  retryShortcuts: () => Promise<string | null>;
  beginShortcutRecording: () => Promise<string | null>;
  endShortcutRecording: () => Promise<string | null>;
  updateShortcut: (kind: ShortcutKind, binding: ShortcutBinding) => Promise<string | null>;
  setShortcutEnabled: (kind: ShortcutKind, enabled: boolean) => Promise<string | null>;
  restoreShortcut: (kind: ShortcutKind) => Promise<string | null>;
  restoreAllShortcuts: () => Promise<string | null>;
}>();
const emit = defineEmits<{
  page: ['settings' | 'shortcut-manager'];
  save: [Partial<Pick<Settings, 'global_shortcuts'>>];
}>();

const opened = ref(false);
const recordingKind = ref<ShortcutKind | null>(null);
const draft = ref<ShortcutBinding | null>(null);
const heldModifiers = ref<ShortcutModifier[]>([]);
const recordingError = ref('');
const actionErrors = reactive<Record<ShortcutKind, string>>({ toggle: '', pause: '' });
const pageError = ref('');
const saving = ref(false);
let capturing = false;

const kinds = computed(() => [
  {
    id: 'toggle' as const,
    name: copy.value.shortcutToggleName,
    purpose: copy.value.shortcutTogglePurpose,
  },
  {
    id: 'pause' as const,
    name: copy.value.shortcutPauseName,
    purpose: copy.value.shortcutPausePurpose,
  },
]);
const platform = computed(() => props.shortcuts?.platform ?? 'other');
const bindings = computed(() => props.settings.shortcut_bindings ?? defaultShortcutBindings);
const allDefault = computed(() =>
  props.shortcuts
    ? props.shortcuts.toggle.is_default && props.shortcuts.pause.is_default
    : bindingEquals(bindings.value.toggle, defaultShortcutBindings.toggle) &&
      bindingEquals(bindings.value.pause, defaultShortcutBindings.pause),
);
const homeIssue = computed(() => {
  if (!props.shortcuts) return '';
  if (props.shortcuts.recording) return copy.value.shortcutRecording;
  const bindings = props.shortcuts;
  if (!props.settings.global_shortcuts) {
    return bindings.toggle.stale_bindings.length || bindings.pause.stale_bindings.length
      ? copy.value.shortcutUnregisterFailed
      : '';
  }
  if (bindings.toggle.error || bindings.pause.error) return copy.value.shortcutNeedsAttention;
  if (
    bindings.toggle.stale_bindings.length ||
    bindings.pause.stale_bindings.length ||
    mismatch('toggle') ||
    mismatch('pause')
  )
    return copy.value.shortcutNeedsAttention;
  return '';
});

function configured(kind: ShortcutKind): ShortcutBinding {
  return bindings.value[kind];
}
function statusFor(kind: ShortcutKind) {
  return props.shortcuts?.[kind] ?? null;
}
function isDefault(kind: ShortcutKind) {
  return (
    statusFor(kind)?.is_default ?? bindingEquals(configured(kind), defaultShortcutBindings[kind])
  );
}
function mismatch(kind: ShortcutKind) {
  const status = statusFor(kind);
  return !!status?.active_binding && !bindingEquals(status.active_binding, configured(kind));
}
function bindingEnabled(kind: ShortcutKind) {
  return (
    statusFor(kind)?.enabled ??
    bindings.value[kind === 'toggle' ? 'toggle_enabled' : 'pause_enabled']
  );
}
function switchChecked(kind: ShortcutKind) {
  return props.settings.global_shortcuts && bindingEnabled(kind);
}
function switchLabel(kind: ShortcutKind) {
  const name = kinds.value.find((item) => item.id === kind)?.name ?? '';
  return `${name}：${
    switchChecked(kind) ? copy.value.shortcutStatusActive : copy.value.shortcutStatusDisabled
  }`;
}
async function toggleBinding(kind: ShortcutKind) {
  if (props.busy || saving.value || recordingKind.value || !props.settings.global_shortcuts) return;
  actionErrors[kind] = '';
  pageError.value = '';
  const error = await props.setShortcutEnabled(kind, !bindingEnabled(kind));
  if (error) actionErrors[kind] = error;
}
function keycaps(binding: ShortcutBinding) {
  return bindingKeycaps(binding, platform.value, locale.value);
}
function text(binding: ShortcutBinding) {
  return bindingText(binding, platform.value, locale.value);
}
function activeText(kind: ShortcutKind) {
  const status = statusFor(kind);
  return status?.active_binding ? text(status.active_binding) : '';
}
function staleText(kind: ShortcutKind) {
  return (
    statusFor(kind)
      ?.stale_bindings.map((binding) => text(binding))
      .join(' / ') ?? ''
  );
}
function previewKeycaps() {
  if (draft.value) return keycaps(draft.value);
  return heldModifiers.value.map((modifier) => modifierLabel(modifier, platform.value));
}

function stopCapture() {
  if (!capturing) return;
  window.removeEventListener('keydown', onKeyDown, true);
  window.removeEventListener('keyup', onKeyUp, true);
  capturing = false;
}
function onKeyDown(event: KeyboardEvent) {
  if (!recordingKind.value) return;
  event.preventDefault();
  event.stopPropagation();
  if (event.key === 'Escape') {
    void cancelRecording();
    return;
  }
  if (event.repeat) return;
  if (isModifierCode(event.code)) {
    heldModifiers.value = modifiersFromEvent(event);
    return;
  }
  const result = bindingFromKeyboardEvent(event);
  if (result.error) {
    draft.value = null;
    heldModifiers.value = modifiersFromEvent(event);
    recordingError.value =
      result.error === 'modifier_required'
        ? copy.value.shortcutModifierRequired
        : copy.value.shortcutInvalid;
    return;
  }
  const other = configured(recordingKind.value === 'toggle' ? 'pause' : 'toggle');
  draft.value = result.binding;
  heldModifiers.value = result.binding.modifiers;
  recordingError.value = bindingEquals(result.binding, other) ? copy.value.shortcutDuplicate : '';
}
function onKeyUp(event: KeyboardEvent) {
  if (!recordingKind.value || draft.value) return;
  heldModifiers.value = modifiersFromEvent(event);
}

async function startRecording(kind: ShortcutKind) {
  if (props.busy || saving.value || recordingKind.value) return;
  actionErrors[kind] = '';
  pageError.value = '';
  const error = await props.beginShortcutRecording();
  if (error) {
    actionErrors[kind] = error;
    return;
  }
  recordingKind.value = kind;
  draft.value = null;
  heldModifiers.value = [];
  recordingError.value = '';
  if (!capturing) {
    window.addEventListener('keydown', onKeyDown, true);
    window.addEventListener('keyup', onKeyUp, true);
    capturing = true;
  }
}
async function cancelRecording() {
  const kind = recordingKind.value;
  stopCapture();
  recordingKind.value = null;
  draft.value = null;
  heldModifiers.value = [];
  recordingError.value = '';
  const error = await props.endShortcutRecording();
  if (error && kind) actionErrors[kind] = error;
}
async function saveRecording() {
  if (!recordingKind.value || !draft.value || recordingError.value || saving.value) return;
  saving.value = true;
  const kind = recordingKind.value;
  const binding = draft.value;
  const error = await props.updateShortcut(kind, binding);
  stopCapture();
  recordingKind.value = null;
  draft.value = null;
  heldModifiers.value = [];
  recordingError.value = '';
  if (error) actionErrors[kind] = error;
  saving.value = false;
}
async function restore(kind: ShortcutKind) {
  if (props.busy || saving.value || recordingKind.value) return;
  actionErrors[kind] = '';
  pageError.value = '';
  const error = await props.restoreShortcut(kind);
  if (error) actionErrors[kind] = error;
}
async function restoreAll() {
  if (props.busy || saving.value || recordingKind.value || allDefault.value) return;
  pageError.value = '';
  const error = await props.restoreAllShortcuts();
  if (error) pageError.value = error;
}
async function retryAll() {
  if (props.busy || saving.value) return;
  pageError.value = '';
  const error = await props.retryShortcuts();
  if (error) pageError.value = error;
}
function openManager() {
  pageError.value = '';
  opened.value = true;
  emit('page', 'shortcut-manager');
}
async function backToSettings() {
  if (recordingKind.value) await cancelRecording();
  opened.value = false;
  emit('page', 'settings');
}

watch(
  () => props.shortcuts?.recording,
  (recording) => {
    if (!recording && recordingKind.value && !saving.value) {
      stopCapture();
      recordingKind.value = null;
      draft.value = null;
      heldModifiers.value = [];
      recordingError.value = '';
    }
  },
);
onBeforeUnmount(() => {
  stopCapture();
  if (recordingKind.value) void props.endShortcutRecording();
});
</script>

<template>
  <section :class="opened ? '' : 'mt-4'" :aria-label="copy.shortcutSettings">
    <template v-if="!opened">
      <h2 class="mb-2.5 text-sm font-medium">{{ copy.shortcutSettings }}</h2>
      <div class="rounded-xl border border-line bg-surface">
        <div class="flex min-h-12 items-center justify-between gap-4 px-3.5 py-2">
          <label for="global-shortcuts" class="text-xs">{{ copy.globalShortcuts }}</label>
          <span class="relative flex h-5 w-9 shrink-0">
            <input
              id="global-shortcuts"
              :checked="settings.global_shortcuts"
              @change="
                emit('save', { global_shortcuts: ($event.target as HTMLInputElement).checked })
              "
              class="peer h-5 w-9 cursor-pointer appearance-none rounded-full bg-line-strong transition-colors checked:bg-dialog-confirm disabled:cursor-not-allowed disabled:opacity-50"
              type="checkbox"
              role="switch"
              :disabled="busy"
            />
            <span
              aria-hidden="true"
              class="pointer-events-none absolute left-0.5 top-0.5 size-4 rounded-full bg-surface transition-transform peer-checked:translate-x-4 peer-checked:bg-paper"
            ></span>
          </span>
        </div>
        <button
          type="button"
          class="mx-3.5 flex min-h-12 w-[calc(100%-1.75rem)] items-center justify-between gap-4 border-t border-line py-2 text-left"
          @click="openManager"
        >
          <span class="text-xs">{{ copy.shortcutManage }}</span>
          <ChevronRight :size="14" class="shrink-0 text-faint" />
        </button>
        <p
          v-if="homeIssue || pageError"
          class="mx-3.5 flex items-center gap-2 border-t border-line py-2 text-xs leading-normal text-danger"
          role="alert"
        >
          <span>{{ pageError || homeIssue }}</span>
          <button
            type="button"
            class="text-button shrink-0 !text-danger"
            :disabled="busy"
            @click="retryAll"
          >
            {{ copy.shortcutRetry }}
          </button>
        </p>
      </div>
    </template>

    <SettingsSubpage
      v-else
      :title="copy.shortcutSettings"
      :back-label="copy.settingsBack"
      :description="copy.shortcutSettingsDescription"
      :busy="busy || saving"
      @back="backToSettings"
    >
      <template #actions>
        <button
          v-if="!allDefault"
          type="button"
          class="text-button min-h-8 gap-2"
          :disabled="busy || saving || !!recordingKind"
          @click="restoreAll"
        >
          <RotateCcw :size="14" />{{ copy.shortcutRestoreAll }}
        </button>
      </template>
      <p v-if="pageError" class="banner error" role="alert">{{ pageError }}</p>
      <div class="overflow-hidden rounded-xl border border-line bg-surface">
        <div
          class="grid grid-cols-[minmax(140px,1fr)_140px_minmax(0,1fr)_44px_80px] items-center gap-1.5 border-b border-line bg-surface-muted px-3.5 py-2 text-xs text-faint"
        >
          <span>{{ copy.shortcutColumnFunction }}</span>
          <span>{{ copy.shortcutColumnShortcut }}</span>
          <span></span>
          <span>{{ copy.shortcutColumnStatus }}</span>
          <span class="text-right">{{ copy.shortcutColumnActions }}</span>
        </div>
        <section
          v-for="kind in kinds"
          :key="kind.id"
          class="[&+&]:border-t [&+&]:border-line"
          :aria-label="`${kind.name}。${kind.purpose}`"
        >
          <div
            class="grid grid-cols-[minmax(140px,1fr)_140px_minmax(0,1fr)_44px_80px] items-center gap-1.5 px-3.5 py-2.5"
          >
            <h2
              class="min-w-0 truncate text-xs font-medium"
              :title="`${kind.name}：${kind.purpose}`"
            >
              {{ kind.name }}
            </h2>
            <div
              class="flex items-center gap-0.5 whitespace-nowrap"
              role="group"
              :aria-label="text(configured(kind.id))"
            >
              <template
                v-for="(label, index) in recordingKind === kind.id
                  ? previewKeycaps()
                  : keycaps(configured(kind.id))"
                :key="`${label}-${index}`"
              >
                <span v-if="index > 0" class="text-xs text-faint" aria-hidden="true">+</span>
                <kbd
                  class="inline-flex min-h-6 items-center rounded-md border border-line bg-paper px-1 text-xs text-ink"
                  >{{ label }}</kbd
                >
              </template>
            </div>
            <div aria-hidden="true"></div>
            <div class="flex items-center">
              <button
                type="button"
                role="switch"
                :aria-checked="switchChecked(kind.id)"
                :aria-label="switchLabel(kind.id)"
                :title="switchLabel(kind.id)"
                :disabled="busy || saving || !!recordingKind || !settings.global_shortcuts"
                class="relative h-5 w-9 shrink-0 rounded-full transition-colors disabled:cursor-not-allowed disabled:opacity-50"
                :class="switchChecked(kind.id) ? 'bg-dialog-confirm' : 'bg-line-strong'"
                @click="toggleBinding(kind.id)"
              >
                <span
                  aria-hidden="true"
                  class="pointer-events-none absolute left-0.5 top-0.5 size-4 rounded-full bg-surface transition-transform"
                  :class="switchChecked(kind.id) ? 'translate-x-4 bg-paper' : ''"
                ></span>
              </button>
            </div>
            <div class="flex justify-end gap-1">
              <template v-if="recordingKind !== kind.id">
                <button
                  type="button"
                  class="icon-button"
                  :aria-label="copy.shortcutModify"
                  :title="copy.shortcutModify"
                  :disabled="busy || saving || !!recordingKind"
                  @click="startRecording(kind.id)"
                >
                  <Pencil :size="14" />
                </button>
                <button
                  v-if="!isDefault(kind.id)"
                  type="button"
                  class="icon-button"
                  :aria-label="copy.shortcutRestoreDefault"
                  :title="copy.shortcutRestoreDefault"
                  :disabled="busy || saving || !!recordingKind"
                  @click="restore(kind.id)"
                >
                  <RotateCcw :size="14" />
                </button>
              </template>
            </div>
          </div>

          <div
            v-if="recordingKind === kind.id"
            class="mx-3.5 mb-2.5 rounded-lg border border-dashed border-line bg-paper px-3 py-2.5"
          >
            <p v-if="!draft" class="text-xs leading-normal text-faint">
              {{ copy.shortcutRecordingHint }}
            </p>
            <p v-if="recordingError" class="mt-1 text-xs leading-normal text-danger" role="alert">
              {{ recordingError }}
            </p>
            <div class="mt-2.5 flex gap-2">
              <button
                type="button"
                class="button secondary compact"
                :disabled="saving"
                @click="cancelRecording"
              >
                {{ copy.cancel }}
              </button>
              <button
                type="button"
                class="button dialog-confirm compact"
                :disabled="saving || !draft || !!recordingError"
                @click="saveRecording"
              >
                {{ copy.save }}
              </button>
            </div>
          </div>

          <p
            v-if="mismatch(kind.id) && activeText(kind.id)"
            class="px-3.5 pb-2 text-xs leading-normal text-danger"
          >
            {{ copy.shortcutActiveMismatch.replace('{active}', activeText(kind.id)) }}
          </p>
          <p v-if="staleText(kind.id)" class="px-3.5 pb-2 text-xs leading-normal text-danger">
            {{ copy.shortcutPendingCleanup.replace('{binding}', staleText(kind.id)) }}
          </p>
          <p
            v-if="actionErrors[kind.id]"
            class="px-3.5 pb-2 text-xs leading-normal text-danger"
            role="alert"
          >
            {{ actionErrors[kind.id] }}
          </p>
        </section>
      </div>
    </SettingsSubpage>
  </section>
</template>
