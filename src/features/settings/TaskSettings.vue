<script setup lang="ts">
import { computed, nextTick, ref } from 'vue';
import {
  Archive,
  ArrowDown,
  ArrowUp,
  ChevronRight,
  GripVertical,
  Pencil,
  Plus,
  RotateCcw,
  Trash2,
  X,
} from 'lucide-vue-next';
import { copy } from '../../shared/i18n';
import { taskColors } from '../../shared/taskColor';
import ColorPicker from './ColorPicker.vue';
import SettingsSubpage from './SettingsSubpage.vue';
import SettingsSelect from '../../shared/ui/SettingsSelect.vue';
import type { Session, Settings, Task, TaskChange } from '../../shared/ipc/types';
const props = defineProps<{
  tasks: Task[];
  records: Session[];
  current: Session | null;
  settings: Settings;
  busy: boolean;
  manage: (change: TaskChange) => Promise<string | null>;
}>();
const emit = defineEmits<{
  page: ['settings' | 'task-manager'];
  retain: [boolean];
  colors: [string[]];
  picker: ['wheel' | 'select'];
}>();
const opened = ref(false),
  error = ref(''),
  editorOpen = ref(false),
  pickerOpen = ref(false),
  dragging = ref<string | null>(null);
const editor = ref<HTMLDialogElement>(),
  deleteDialog = ref<HTMLDialogElement>(),
  field = ref<HTMLInputElement>();
const editing = ref<string | null>(null),
  deleting = ref<string | null>(null),
  name = ref(''),
  color = ref<string | null>(null);
const active = computed(() => props.tasks.filter((t) => !t.archived));
const archived = computed(() => props.tasks.filter((t) => t.archived));
const customColors = computed(() => props.settings.task_colors ?? []);
const palette = computed(() => [
  { value: null as string | null, label: copy.value.categoryColorNone, custom: false },
  ...taskColors.map((value) => ({
    value: value as string | null,
    label: `${copy.value.categoryColor} ${value}`,
    custom: false,
  })),
  ...customColors.value.map((value) => ({
    value: value as string | null,
    label: `${copy.value.categoryColor} ${value}`,
    custom: true,
  })),
]);
const paletteColors = computed(() => [...taskColors, ...customColors.value]);
const customSelected = computed(() => !!color.value && customColors.value.includes(color.value));
function addColor(value: string) {
  if (paletteColors.value.includes(value)) return;
  emit('colors', [...customColors.value, value]);
  color.value = value;
  pickerOpen.value = false;
}
function removeColor(value: string) {
  emit(
    'colors',
    customColors.value.filter((item) => item !== value),
  );
}
const counts = computed(() => {
  const result = new Map<string, number>();
  for (const record of props.records) {
    if (record.task_id) result.set(record.task_id, (result.get(record.task_id) ?? 0) + 1);
  }
  return result;
});
const deletedRecords = computed(() =>
  deleting.value ? (counts.value.get(deleting.value) ?? 0) : 0,
);
const deleteHint = computed(() =>
  deletedRecords.value
    ? copy.value.categoryDeleteRecordsHint.replace('{count}', String(deletedRecords.value))
    : copy.value.categoryDeleteHint,
);
function inUse(id: string) {
  return props.current?.kind === 'focus' && props.current.task_id === id;
}
function navigate(open: boolean) {
  opened.value = open;
  error.value = '';
  emit('page', open ? 'task-manager' : 'settings');
}
async function perform(change: TaskChange) {
  if (props.busy) return false;
  error.value = '';
  try {
    error.value = (await props.manage(change)) ?? '';
  } catch {
    error.value = copy.value.taskSaveFailed;
  }
  return !error.value;
}
async function edit(task?: Task) {
  if (props.busy) return;
  error.value = '';
  editing.value = task?.id ?? null;
  name.value = task?.name ?? '';
  color.value = task?.color ?? null;
  pickerOpen.value = false;
  editorOpen.value = true;
  editor.value?.showModal();
  await nextTick();
  field.value?.focus();
}
async function save() {
  if (!name.value.trim() || [...name.value.trim()].length > 40) {
    error.value = copy.value.taskNameInvalid;
    return;
  }
  if (
    await perform(
      editing.value
        ? { action: 'update', id: editing.value, name: name.value.trim(), color: color.value }
        : { action: 'create', name: name.value.trim(), color: color.value },
    )
  )
    editor.value?.close();
}
async function archive(task: Task) {
  if (await perform({ action: 'archive', id: task.id, archived: !task.archived })) error.value = '';
}
function askDelete(task: Task) {
  if (props.busy || inUse(task.id)) return;
  error.value = '';
  deleting.value = task.id;
  deleteDialog.value?.showModal();
}
async function remove() {
  if (deleting.value && (await perform({ action: 'delete', id: deleting.value })))
    deleteDialog.value?.close();
}
async function reorder(ids: string[]) {
  await perform({ action: 'reorder', ids: [...ids, ...archived.value.map((t) => t.id)] });
}
function move(id: string, direction: number) {
  const ids = active.value.map((t) => t.id),
    from = ids.indexOf(id),
    to = from + direction;
  if (from < 0 || to < 0 || to >= ids.length || props.busy) return;
  [ids[from], ids[to]] = [ids[to]!, ids[from]!];
  void reorder(ids);
}
function dragStart(event: DragEvent, id: string) {
  if (props.busy) {
    event.preventDefault();
    return;
  }
  dragging.value = id;
  event.dataTransfer?.setData('text/plain', id);
  if (event.dataTransfer) event.dataTransfer.effectAllowed = 'move';
}
function drop(id: string) {
  const ids = active.value.map((t) => t.id),
    from = ids.indexOf(dragging.value ?? ''),
    to = ids.indexOf(id);
  dragging.value = null;
  if (from < 0 || to < 0 || from === to || props.busy) return;
  const [item] = ids.splice(from, 1);
  ids.splice(to, 0, item!);
  void reorder(ids);
}
</script>
<template>
  <section :class="opened ? '' : 'mt-4'" :aria-label="copy.categorySettings">
    <template v-if="!opened">
      <h2 class="mb-2.5 text-sm font-medium">{{ copy.categorySettings }}</h2>
      <div class="rounded-xl border border-line bg-surface">
        <div class="flex min-h-[52px] items-center justify-between gap-4 px-3.5 py-1.5">
          <div class="min-w-0">
            <label for="retain-task" class="text-xs">{{ copy.retainTask }}</label>
            <p id="retain-task-hint" class="mt-0.5 text-xs leading-normal text-faint">
              {{ copy.retainTaskHint }}
            </p>
          </div>
          <span class="relative flex h-5 w-9 shrink-0"
            ><input
              id="retain-task"
              :checked="settings.retain_task !== false"
              :disabled="busy"
              @change="emit('retain', ($event.target as HTMLInputElement).checked)"
              class="peer h-5 w-9 cursor-pointer appearance-none rounded-full bg-line-strong transition-colors checked:bg-dialog-confirm"
              type="checkbox"
              role="switch"
              aria-describedby="retain-task-hint" /><span
              aria-hidden="true"
              class="pointer-events-none absolute left-0.5 top-0.5 size-4 rounded-full bg-surface transition-transform peer-checked:translate-x-4 peer-checked:bg-paper"
            ></span
          ></span>
        </div>
        <div
          class="mx-3.5 flex min-h-12 items-center justify-between gap-4 border-t border-line py-2"
        >
          <span class="text-xs">{{ copy.taskPicker }}</span>
          <SettingsSelect
            id="task-picker"
            :model-value="settings.task_picker ?? 'wheel'"
            :label="copy.taskPicker"
            :options="[
              { value: 'wheel', label: copy.taskPickerWheel },
              { value: 'select', label: copy.taskPickerSelect },
            ]"
            :disabled="busy"
            @update:model-value="emit('picker', $event as 'wheel' | 'select')"
          />
        </div>
        <div
          class="mx-3.5 flex min-h-12 items-center justify-between gap-4 border-t border-line py-2"
        >
          <span class="text-xs">{{ copy.manageCategories }}</span
          ><button
            type="button"
            class="text-button gap-2"
            @click="navigate(true)"
            :aria-label="copy.manageCategories"
          >
            <span class="tabular-nums">{{ tasks.length }}</span
            ><ChevronRight :size="14" />
          </button>
        </div>
      </div>
    </template>
    <SettingsSubpage
      v-else
      :title="copy.manageCategories"
      :back-label="copy.settingsBack"
      :description="copy.categoryRecordHint"
      :busy="busy"
      @back="navigate(false)"
    >
      <template #actions
        ><button type="button" class="text-button min-h-8 gap-2" :disabled="busy" @click="edit()">
          <Plus :size="16" />{{ copy.newTask }}
        </button></template
      >
      <p v-if="error && !editorOpen && !deleting" class="banner error" role="alert">{{ error }}</p>
      <div v-if="active.length" class="overflow-hidden rounded-xl border border-line bg-surface">
        <div
          v-for="(task, index) in active"
          :key="task.id"
          class="group flex min-h-12 items-center gap-2 px-2.5 py-2 [&+&]:border-t [&+&]:border-line"
          @dragover.prevent
          @drop.prevent="drop(task.id)"
        >
          <span
            role="button"
            tabindex="0"
            class="flex size-6 shrink-0 cursor-grab items-center justify-center text-faint active:cursor-grabbing"
            :class="busy ? 'opacity-50' : ''"
            :title="copy.categoryDrag"
            :aria-label="copy.categoryDrag + '：' + task.name"
            :aria-disabled="busy"
            :draggable="!busy"
            @dragstart="dragStart($event, task.id)"
            @dragend="dragging = null"
          >
            <GripVertical :size="14" />
          </span>
          <div class="flex min-w-0 flex-1 items-center gap-1.5">
            <span
              aria-hidden="true"
              class="size-2 shrink-0 rounded-full"
              :class="task.color ? '' : 'border border-line-strong'"
              :style="task.color ? { backgroundColor: task.color } : undefined"
            />
            <span class="min-w-0 truncate text-xs text-ink" :title="task.name">{{
              task.name
            }}</span>
          </div>
          <div
            class="flex shrink-0 items-center gap-0.5 opacity-0 group-focus-within:opacity-100 group-hover:opacity-100"
          >
            <button
              type="button"
              class="icon-button !size-6"
              :disabled="busy || index === 0"
              :title="copy.categoryMoveUp"
              :aria-label="copy.categoryMoveUp + '：' + task.name"
              @click="move(task.id, -1)"
            >
              <ArrowUp :size="13" />
            </button>
            <button
              type="button"
              class="icon-button !size-6"
              :disabled="busy || index === active.length - 1"
              :title="copy.categoryMoveDown"
              :aria-label="copy.categoryMoveDown + '：' + task.name"
              @click="move(task.id, 1)"
            >
              <ArrowDown :size="13" />
            </button>
            <button
              type="button"
              class="icon-button !size-6"
              :disabled="busy"
              :title="copy.renameCategory"
              :aria-label="copy.renameCategory + '：' + task.name"
              @click="edit(task)"
            >
              <Pencil :size="13" />
            </button>
            <button
              v-if="counts.get(task.id)"
              type="button"
              class="icon-button !size-6"
              :disabled="busy || inUse(task.id)"
              :title="inUse(task.id) ? copy.taskInUse : copy.archiveCategory"
              :aria-label="copy.archiveCategory + '：' + task.name"
              @click="archive(task)"
            >
              <Archive :size="13" />
            </button>
            <button
              type="button"
              class="icon-button !size-6"
              :disabled="busy || inUse(task.id)"
              :title="inUse(task.id) ? copy.taskInUse : copy.deleteCategory"
              :aria-label="copy.deleteCategory + '：' + task.name"
              @click="askDelete(task)"
            >
              <Trash2 :size="13" />
            </button>
          </div>
        </div>
      </div>
      <p v-else class="py-8 text-center text-xs text-faint">{{ copy.noCategories }}</p>
      <details v-if="archived.length" class="mt-5">
        <summary class="cursor-pointer text-xs text-muted">
          {{ copy.archivedCategories }} <span class="tabular-nums">{{ archived.length }}</span>
        </summary>
        <div class="mt-2 overflow-hidden rounded-xl border border-line bg-surface">
          <div
            v-for="task in archived"
            :key="task.id"
            class="group flex min-h-12 items-center justify-between gap-3 px-3.5 py-2 [&+&]:border-t [&+&]:border-line"
          >
            <span class="min-w-0 truncate text-xs text-muted" :title="task.name">{{
              task.name
            }}</span>
            <div
              class="flex shrink-0 gap-0.5 opacity-0 group-focus-within:opacity-100 group-hover:opacity-100"
            >
              <button
                type="button"
                class="icon-button !size-6"
                :disabled="busy"
                :title="copy.renameCategory"
                :aria-label="copy.renameCategory + '：' + task.name"
                @click="edit(task)"
              >
                <Pencil :size="13" /></button
              ><button
                type="button"
                class="icon-button !size-6"
                :disabled="busy"
                :title="copy.restoreCategory"
                :aria-label="copy.restoreCategory + '：' + task.name"
                @click="archive(task)"
              >
                <RotateCcw :size="14" /></button
              ><button
                type="button"
                class="icon-button !size-6"
                :disabled="busy"
                :title="copy.deleteCategory"
                :aria-label="copy.deleteCategory + '：' + task.name"
                @click="askDelete(task)"
              >
                <Trash2 :size="13" />
              </button>
            </div>
          </div>
        </div>
      </details>
    </SettingsSubpage>
    <Teleport to="body">
      <dialog
        ref="editor"
        class="modal"
        @close="
          editorOpen = false;
          pickerOpen = false;
          error = '';
        "
        @cancel="busy && $event.preventDefault()"
      >
        <form @submit.prevent="save">
          <div class="modal-heading">
            <h2>{{ editing ? copy.renameCategory : copy.newTask }}</h2>
            <button
              type="button"
              class="icon-button"
              :aria-label="copy.cancel"
              :disabled="busy"
              @click="editor?.close()"
            >
              <X :size="18" />
            </button>
          </div>
          <label
            >{{ copy.taskName
            }}<input
              ref="field"
              v-model="name"
              :disabled="busy"
              :placeholder="copy.categoryPlaceholder"
              maxlength="40"
          /></label>
          <div class="mt-4">
            <span class="text-xs text-muted">{{ copy.categoryColor }}</span>
            <div class="mt-2 flex items-center gap-1.5">
              <div
                class="record-list-scroll flex min-w-0 flex-1 items-center gap-1.5 overflow-x-auto py-1"
                role="group"
                :aria-label="copy.categoryColor"
              >
                <button
                  v-for="option in palette"
                  :key="option.value ?? 'none'"
                  type="button"
                  class="flex size-7 shrink-0 items-center justify-center rounded-full border transition-colors"
                  :class="
                    color === option.value
                      ? 'border-ink bg-hover'
                      : 'border-transparent enabled:hover:bg-hover'
                  "
                  :aria-label="option.label"
                  :aria-pressed="color === option.value"
                  :title="option.label"
                  :disabled="busy"
                  @click="color = option.value"
                >
                  <span
                    aria-hidden="true"
                    class="size-4 rounded-full"
                    :class="option.value ? '' : 'border border-line-strong'"
                    :style="option.value ? { backgroundColor: option.value } : undefined"
                  />
                </button>
              </div>
              <button
                v-if="customSelected && color"
                type="button"
                class="icon-button !size-7 shrink-0"
                :title="copy.categoryColorRemove"
                :aria-label="copy.categoryColorRemove"
                :disabled="busy"
                @click="removeColor(color)"
              >
                <Trash2 :size="13" />
              </button>
              <button
                type="button"
                class="icon-button !size-7 shrink-0"
                :title="copy.categoryColorAdd"
                :aria-label="copy.categoryColorAdd"
                :aria-expanded="pickerOpen"
                :disabled="busy"
                @click="pickerOpen = !pickerOpen"
              >
                <Plus :size="14" />
              </button>
            </div>
            <ColorPicker
              v-if="pickerOpen"
              :taken="paletteColors"
              :initial="color"
              :disabled="busy"
              @add="addColor"
              @cancel="pickerOpen = false"
            />
          </div>
          <p v-if="error" class="text-xs text-danger" role="alert">{{ error }}</p>
          <div class="modal-actions">
            <button
              type="button"
              class="button secondary"
              :disabled="busy"
              @click="editor?.close()"
            >
              {{ copy.cancel }}</button
            ><button type="submit" class="button dialog-confirm" :disabled="busy">
              {{ copy.save }}
            </button>
          </div>
        </form>
      </dialog>
      <dialog
        ref="deleteDialog"
        class="modal"
        @close="
          deleting = null;
          error = '';
        "
        @cancel="busy && $event.preventDefault()"
      >
        <h2>{{ copy.categoryDeleteTitle }}</h2>
        <p>{{ deleteHint }}</p>
        <p v-if="error" class="text-xs text-danger" role="alert">{{ error }}</p>
        <div class="modal-actions">
          <button
            type="button"
            class="button secondary"
            :disabled="busy"
            @click="deleteDialog?.close()"
          >
            {{ copy.cancel }}</button
          ><button type="button" class="button dialog-confirm" :disabled="busy" @click="remove">
            {{ copy.deleteCategory }}
          </button>
        </div>
      </dialog>
    </Teleport>
  </section>
</template>
