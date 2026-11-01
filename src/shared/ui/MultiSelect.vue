<script setup lang="ts">
import { nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
import { ChevronDown } from 'lucide-vue-next';
import { copy } from '../i18n';
const props = defineProps<{
  id: string;
  label: string;
  selected: string[];
  options: { value: string; label: string }[];
  disabled?: boolean;
}>();
const emit = defineEmits<{ 'update:selected': [string[]] }>();
const trigger = ref<HTMLButtonElement>(),
  panel = ref<HTMLDivElement>();
const opened = ref(false),
  positioned = ref(false);
const position = ref({
  left: '0px',
  top: '0px',
  width: 'max-content',
  maxWidth: 'calc(100vw - 16px)',
  maxHeight: '224px',
});
function close(restore = false) {
  opened.value = false;
  if (restore) trigger.value?.focus({ preventScroll: true });
}
async function show() {
  if (!trigger.value || props.disabled) return;
  opened.value = true;
  positioned.value = false;
  await place();
}
async function place() {
  await nextTick();
  const rect = trigger.value?.getBoundingClientRect();
  if (!rect || !panel.value || !opened.value) return;
  const width = panel.value.getBoundingClientRect().width;
  const height = Math.min(224, panel.value.scrollHeight + 2);
  const above = rect.top - 14,
    below = window.innerHeight - rect.bottom - 14;
  const up = below < height && above > below;
  const shown = Math.min(height, Math.max(0, up ? above : below));
  position.value = {
    ...position.value,
    left: Math.max(8, Math.min(rect.right - width, window.innerWidth - width - 8)) + 'px',
    top: (up ? rect.top - 6 - shown : rect.bottom + 6) + 'px',
    maxHeight: shown + 'px',
  };
  positioned.value = true;
}
function toggle(value: string) {
  if (!props.disabled)
    emit(
      'update:selected',
      props.selected.includes(value)
        ? props.selected.filter((v) => v !== value)
        : [...props.selected, value],
    );
}
function outside(event: PointerEvent) {
  if (
    event.target instanceof Node &&
    !trigger.value?.contains(event.target) &&
    !panel.value?.contains(event.target)
  )
    close();
}
function scroll(event: Event) {
  if (event.target instanceof Node && panel.value?.contains(event.target)) return;
  if (opened.value) void place();
}
function resize() {
  close();
}
async function keyboard(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    event.preventDefault();
    close(true);
    return;
  }
  if (!['ArrowDown', 'ArrowUp'].includes(event.key)) return;
  event.preventDefault();
  if (!opened.value) await show();
  const fields = Array.from(panel.value?.querySelectorAll<HTMLInputElement>('input') ?? []);
  if (!fields.length) return;
  const current = fields.indexOf(document.activeElement as HTMLInputElement);
  const next =
    current < 0
      ? event.key === 'ArrowDown'
        ? 0
        : fields.length - 1
      : (current + fields.length + (event.key === 'ArrowDown' ? 1 : -1)) % fields.length;
  fields[next]?.focus();
}
function blur(event: FocusEvent) {
  if (
    event.relatedTarget instanceof Node &&
    !panel.value?.contains(event.relatedTarget) &&
    !trigger.value?.contains(event.relatedTarget)
  )
    close();
}
watch(
  () => props.disabled,
  (value) => {
    if (value) close();
  },
);
watch(
  () => props.options.map((o) => o.label).join('|'),
  () => close(),
);
onMounted(() => {
  document.addEventListener('pointerdown', outside, true);
  window.addEventListener('scroll', scroll, true);
  window.addEventListener('resize', resize);
});
onUnmounted(() => {
  document.removeEventListener('pointerdown', outside, true);
  window.removeEventListener('scroll', scroll, true);
  window.removeEventListener('resize', resize);
});
</script>
<template>
  <button
    :id="id"
    ref="trigger"
    type="button"
    class="inline-flex h-8 shrink-0 items-center gap-1 rounded-md px-1 text-xs text-muted hover:text-ink disabled:opacity-50"
    :disabled="disabled"
    :aria-expanded="opened"
    :aria-controls="opened ? id + '-options' : undefined"
    aria-haspopup="dialog"
    @click="opened ? close(true) : show()"
    @keydown="keyboard"
    @focusout="blur"
  >
    {{ label }}<span v-if="selected.length">（{{ selected.length }}）</span
    ><ChevronDown class="size-3" aria-hidden="true" />
  </button>
  <Teleport to="body"
    ><div
      v-if="opened"
      :id="id + '-options'"
      ref="panel"
      role="dialog"
      :aria-label="label"
      class="fixed z-50 overflow-y-auto overscroll-contain rounded-lg border border-line-strong bg-surface p-1 text-xs text-ink shadow-sm [scrollbar-width:none] [&::-webkit-scrollbar]:hidden"
      :class="{ invisible: !positioned }"
      :style="position"
      @keydown="keyboard"
      @focusout="blur"
    >
      <div role="group" :aria-label="label">
        <label
          v-for="option in options"
          :key="option.value"
          class="flex h-8 cursor-pointer items-center gap-2 whitespace-nowrap rounded px-2 hover:bg-hover"
          ><input
            type="checkbox"
            class="size-3 shrink-0"
            :checked="selected.includes(option.value)"
            :disabled="disabled"
            style="accent-color: var(--ink)"
            @change="toggle(option.value)"
          /><span class="min-w-0 truncate">{{ option.label }}</span></label
        >
      </div>
      <button
        type="button"
        class="text-button px-2 py-1"
        :disabled="disabled"
        @click="emit('update:selected', [])"
      >
        {{ copy.clearTaskFilters }}
      </button>
    </div></Teleport
  >
</template>
