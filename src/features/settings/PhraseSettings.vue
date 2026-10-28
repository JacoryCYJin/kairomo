<script setup lang="ts">
import { computed, onBeforeUnmount, reactive, ref, watch } from 'vue';
import { ArrowLeft, Check, ChevronRight, Circle, Plus, Trash2 } from 'lucide-vue-next';
import { copy } from '../../shared/i18n';
import { emptyPhrases, type Phrase, type Phrases } from '../../shared/ipc/types';
import { libraryAttribution, selectedPhrasesFirst } from './phraseLibrary';
import SettingsSelect from '../../shared/ui/SettingsSelect.vue';
const props = defineProps<{ settings: Phrases; busy: boolean }>();
const emit = defineEmits<{ save: [Phrases]; page: [boolean] }>();
const draft = reactive<Phrases>(emptyPhrases());
const page = ref<'settings' | 'library' | 'editor'>('settings');
const editor = ref<Phrase | null>(null);
const adding = ref(false);
let dirty = false;
let timer: ReturnType<typeof setTimeout> | undefined;
watch(
  () => JSON.stringify(props.settings),
  (value) => {
    if (JSON.stringify(draft) === value) dirty = false;
    else if (!dirty) Object.assign(draft, JSON.parse(value));
  },
  { immediate: true },
);
const entries = computed(() => selectedPhrasesFirst(draft.entries, draft.selected_id));
function save() {
  clearTimeout(timer);
  const value: Phrases = JSON.parse(JSON.stringify(draft));
  if (JSON.stringify(value) !== JSON.stringify(props.settings)) {
    dirty = true;
    emit('save', value);
  }
}
function flushEditor() {
  clearTimeout(timer);
  const entry = editor.value;
  if (!entry) return;
  if (!entry.text.trim()) {
    // A newly created message that is cleared again is still an empty draft.
    if (adding.value && draft.entries.some((item) => item.id === entry.id)) {
      draft.entries = draft.entries.filter((item) => item.id !== entry.id);
      if (draft.selected_id === entry.id) draft.selected_id = draft.entries[0]?.id ?? null;
      save();
    }
    return;
  }
  const value = {
    ...entry,
    text: entry.text.trim(),
    author: entry.author?.trim() ?? '',
    source_title: entry.source_title?.trim() ?? '',
  };
  const index = draft.entries.findIndex((item) => item.id === entry.id);
  if (index < 0) draft.entries.push(value);
  else draft.entries[index] = value;
  if (!draft.selected_id) draft.selected_id = value.id;
  save();
}
function schedule() {
  clearTimeout(timer);
  timer = setTimeout(flushEditor, 700);
}
function navigate(next: 'settings' | 'library' | 'editor') {
  page.value = next;
  emit('page', next !== 'settings');
}
function open(entry?: Phrase) {
  adding.value = !entry;
  editor.value = entry
    ? { ...entry }
    : { id: crypto.randomUUID(), text: '', author: '', source_title: '', source_kind: null };
  navigate('editor');
}
function back() {
  flushEditor();
  editor.value = null;
  navigate('library');
}
function remove() {
  clearTimeout(timer);
  draft.entries = draft.entries.filter((item) => item.id !== editor.value?.id);
  if (!draft.entries.some((item) => item.id === draft.selected_id))
    draft.selected_id = draft.entries[0]?.id ?? null;
  editor.value = null;
  save();
  navigate('library');
}
function use(id: string) {
  draft.selected_id = id;
  save();
}
onBeforeUnmount(flushEditor);
</script>
<template>
  <section :class="page === 'settings' ? 'mt-5' : ''" :aria-label="copy.customPhrases">
    <template v-if="page === 'settings'">
      <h2 class="mb-2.5 text-sm font-medium">{{ copy.customPhrases }}</h2>
      <fieldset :disabled="busy" class="min-w-0 rounded-xl border border-line bg-surface px-3.5">
        <label class="flex min-h-12 cursor-pointer items-center justify-between gap-4 py-2 text-xs">
          <span>{{ copy.showPhrases }}</span>
          <span class="relative flex h-5 w-9 shrink-0">
            <input
              v-model="draft.enabled"
              type="checkbox"
              role="switch"
              class="peer h-5 w-9 cursor-pointer appearance-none rounded-full bg-line-strong checked:bg-ink"
              @change="save"
            />
            <span
              aria-hidden="true"
              class="pointer-events-none absolute left-0.5 top-0.5 size-4 rounded-full bg-surface peer-checked:translate-x-4 peer-checked:bg-paper"
            ></span>
          </span>
        </label>
        <div class="flex min-h-12 items-center justify-between gap-4 border-t border-line py-2">
          <span class="text-xs">{{ copy.phraseMode }}</span>
          <SettingsSelect
            id="phrase-mode"
            :model-value="draft.mode"
            :label="copy.phraseMode"
            :disabled="busy"
            :options="[
              { value: 'fixed', label: copy.phraseFixed },
              { value: 'random', label: copy.phraseRandom },
            ]"
            @update:model-value="
              draft.mode = $event as Phrases['mode'];
              save();
            "
          />
        </div>
        <div
          class="flex min-h-12 items-center justify-between gap-4 border-t border-line py-2 text-xs"
        >
          <span>{{ copy.phraseLibrary }}</span>
          <button
            type="button"
            class="inline-flex items-center gap-2 py-2"
            @click="navigate('library')"
          >
            {{ copy.editPhraseLibrary }}<ChevronRight :size="14" class="text-faint" />
          </button>
        </div>
      </fieldset>
    </template>
    <template v-else-if="page === 'library'">
      <button
        type="button"
        class="mb-6 inline-flex items-center gap-2 py-1 text-xs text-muted"
        @click="navigate('settings')"
      >
        <ArrowLeft :size="14" />{{ copy.settings }}
      </button>
      <div class="mb-5 flex items-center justify-between gap-3">
        <div class="flex items-baseline gap-2.5">
          <h1 class="text-lg font-semibold">{{ copy.phraseLibrary }}</h1>
          <span class="text-xs tabular-nums text-faint">{{
            copy.phraseCount.replace('{count}', String(draft.entries.length))
          }}</span>
        </div>
        <button
          type="button"
          class="inline-flex items-center gap-2 py-2 text-xs"
          :disabled="busy"
          @click="open()"
        >
          <Plus :size="14" />{{ copy.addPhrase }}
        </button>
      </div>
      <div :aria-label="copy.phraseLibrary">
        <p v-if="!entries.length" class="py-10 text-xs text-faint">{{ copy.noPhrases }}</p>
        <div
          v-for="entry in entries"
          :key="entry.id"
          class="flex min-h-16 items-center gap-3 border-b border-line last:border-0"
        >
          <button
            type="button"
            class="min-w-0 flex-1 py-3 text-left"
            :aria-label="`${copy.phraseEdit}: ${entry.text}`"
            @click="open(entry)"
          >
            <span class="block truncate text-sm">{{ entry.text }}</span>
            <span v-if="libraryAttribution(entry)" class="mt-1 block truncate text-xs text-faint">{{
              libraryAttribution(entry)
            }}</span>
          </button>
          <button
            v-if="draft.mode === 'fixed'"
            type="button"
            class="inline-flex size-8 shrink-0 items-center justify-center"
            :disabled="busy"
            :aria-label="`${draft.selected_id === entry.id ? copy.phraseCurrent : copy.phraseUse}: ${entry.text}`"
            :aria-pressed="draft.selected_id === entry.id"
            @click="use(entry.id)"
          >
            <Check v-if="draft.selected_id === entry.id" :size="16" />
            <Circle v-else :size="15" class="text-line-strong" :stroke-width="1.25" />
          </button>
        </div>
      </div>
    </template>
    <template v-else-if="editor">
      <div class="mb-6 flex items-center justify-between gap-3">
        <button
          type="button"
          class="inline-flex items-center gap-2 py-1 text-xs text-muted"
          @click="back"
        >
          <ArrowLeft :size="14" />{{ copy.phraseLibrary }}
        </button>
        <button
          v-if="!adding"
          type="button"
          class="inline-flex size-8 items-center justify-center text-muted"
          :aria-label="copy.deletePhrase"
          :disabled="busy"
          @click="remove"
        >
          <Trash2 :size="15" />
        </button>
      </div>
      <h1 class="mb-5 text-lg font-semibold">{{ adding ? copy.addPhrase : copy.phraseEdit }}</h1>
      <label class="block text-xs text-muted"
        >{{ copy.phraseBody }}
        <textarea
          v-model="editor.text"
          rows="5"
          :placeholder="copy.phrasePlaceholder"
          class="mt-2 block w-full resize-y rounded-lg border border-line bg-paper p-3 text-sm leading-relaxed text-ink outline-none focus:border-line-strong"
          @input="schedule"
          @blur="flushEditor"
        />
      </label>
      <h2 class="mb-3 mt-6 text-xs font-medium">
        {{ copy.phraseSource
        }}<span class="ml-2 font-normal text-faint">{{ copy.phraseOptional }}</span>
      </h2>
      <div class="grid grid-cols-1 gap-4 min-[560px]:grid-cols-2">
        <label class="block min-w-0 text-xs text-muted"
          >{{ copy.phraseAuthor
          }}<input
            v-model="editor.author"
            type="text"
            :placeholder="copy.phraseAuthorPlaceholder"
            class="mt-2 block h-10 w-full min-w-0 rounded-lg border border-line bg-paper px-3 text-left text-sm text-ink focus-visible:outline-none focus:border-line-strong"
            @input="schedule"
            @blur="flushEditor"
        /></label>
        <label class="block min-w-0 text-xs text-muted"
          >{{ copy.phraseWork
          }}<input
            v-model="editor.source_title"
            type="text"
            :placeholder="copy.phraseWorkPlaceholder"
            class="mt-2 block h-10 w-full min-w-0 rounded-lg border border-line bg-paper px-3 text-left text-sm text-ink focus-visible:outline-none focus:border-line-strong"
            @input="schedule"
            @blur="flushEditor"
        /></label>
      </div>
      <section class="mt-7 border-t border-line pt-4" :aria-label="copy.phrasePreview">
        <h2 class="mb-3 text-xs text-faint">{{ copy.phrasePreview }}</h2>
        <div class="px-3 py-3 text-center">
          <p
            class="whitespace-pre-wrap break-words text-sm leading-relaxed"
            :class="editor.text.trim() ? 'text-ink' : 'text-faint'"
          >
            {{ editor.text.trim() || copy.phrasePreviewEmpty }}
          </p>
          <p
            v-if="libraryAttribution(editor)"
            class="mt-2 break-words text-xs leading-relaxed text-faint"
          >
            {{ libraryAttribution(editor) }}
          </p>
        </div>
      </section>
    </template>
  </section>
</template>
