<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
import { Check, ChevronDown } from 'lucide-vue-next';
const props = defineProps<{
  id: string;
  modelValue: string;
  label: string;
  describedby?: string;
  options: { value: string; label: string }[];
  disabled?: boolean;
}>();
const emit = defineEmits<{ 'update:modelValue': [string] }>();
const trigger = ref<HTMLButtonElement>();
const panel = ref<HTMLElement>();
const opened = ref(false);
const positioned = ref(false);
const active = ref(0);
const position = ref({
  left: '0px',
  top: '0px',
  width: 'max-content',
  maxWidth: 'calc(100vw - 16px)',
  maxHeight: '224px',
});
const selectedLabel = computed(
  () => props.options.find((o) => o.value === props.modelValue)?.label ?? '',
);
const optionId = (index: number) => `${props.id}-option-${index}`;
let search = '',
  lastKey = 0;
let openedOnPointerDown: boolean | null = null;
function rememberPointerState(event: PointerEvent) {
  openedOnPointerDown = event.button === 0 ? opened.value : null;
}
function toggle(event: MouseEvent) {
  if (props.disabled) return;
  // Blur can close the popup between pointerdown and click. Preserve the user's toggle intent.
  const shouldClose = event.detail > 0 ? (openedOnPointerDown ?? opened.value) : opened.value;
  openedOnPointerDown = null;
  if (shouldClose) close(true);
  else show();
}
function reveal() {
  void nextTick(() => {
    const list = panel.value;
    const item = document.getElementById(optionId(active.value));
    if (!list || !item) return;
    const top = item.offsetTop;
    const bottom = top + item.offsetHeight;
    if (top < list.scrollTop) list.scrollTop = top;
    else if (bottom > list.scrollTop + list.clientHeight)
      list.scrollTop = bottom - list.clientHeight;
  });
}
function show() {
  if (props.disabled || !props.options.length) return;
  const rect = trigger.value?.getBoundingClientRect();
  if (!rect) return;
  const height = Math.min(224, props.options.length * 32 + 8);
  const below = window.innerHeight - rect.bottom - 14;
  const above = rect.top - 14;
  const up = below < height && above > below;
  const available = Math.max(0, up ? above : below);
  const shownHeight = Math.min(height, available);
  position.value = {
    ...position.value,
    left: '0px',
    top: `${up ? rect.top - 6 - shownHeight : rect.bottom + 6}px`,
    maxHeight: `${shownHeight}px`,
  };
  active.value = Math.max(
    0,
    props.options.findIndex((o) => o.value === props.modelValue),
  );
  search = '';
  trigger.value?.focus({ preventScroll: true });
  positioned.value = false;
  opened.value = true;
  void nextTick(() => {
    if (!opened.value || !panel.value) return;
    // Measure the rendered options, including their check column, padding and border.
    const width = panel.value.getBoundingClientRect().width;
    position.value.left = `${Math.max(8, Math.min(rect.right - width, window.innerWidth - width - 8))}px`;
    positioned.value = true;
    reveal();
  });
}
function close(restore = false) {
  opened.value = false;
  search = '';
  if (restore) trigger.value?.focus({ preventScroll: true });
}
function choose(index: number) {
  if (props.disabled) return;
  const option = props.options[index];
  if (option && option.value !== props.modelValue) emit('update:modelValue', option.value);
  close(true);
}
function keydown(event: KeyboardEvent) {
  openedOnPointerDown = null;
  if (props.disabled) return;
  if (event.key === 'Tab') {
    close();
    return;
  }
  if (event.key === 'Escape') {
    if (opened.value) {
      event.preventDefault();
      event.stopPropagation();
      close(true);
    }
    return;
  }
  if (['Enter', ' ', 'ArrowDown', 'ArrowUp', 'Home', 'End'].includes(event.key)) {
    event.preventDefault();
    if (!opened.value) {
      show();
      if (event.key === 'Home') active.value = 0;
      if (event.key === 'End') active.value = props.options.length - 1;
    } else if (event.key === 'Enter' || event.key === ' ') {
      choose(active.value);
      return;
    } else if (event.key === 'Home') active.value = 0;
    else if (event.key === 'End') active.value = props.options.length - 1;
    else
      active.value = Math.max(
        0,
        Math.min(props.options.length - 1, active.value + (event.key === 'ArrowDown' ? 1 : -1)),
      );
    reveal();
  } else if (event.key.length === 1 && !event.metaKey && !event.ctrlKey && !event.altKey) {
    event.preventDefault();
    if (!opened.value) show();
    const now = Date.now();
    search = (now - lastKey < 700 ? search : '') + event.key.toLocaleLowerCase();
    lastKey = now;
    const repeated = [...search].every((char) => char === search[0]);
    const prefix = repeated ? search[0]! : search;
    const start = repeated ? active.value + 1 : active.value;
    for (let offset = 0; offset < props.options.length; offset++) {
      const index = (start + offset) % props.options.length;
      if (props.options[index]?.label.toLocaleLowerCase().startsWith(prefix)) {
        active.value = index;
        reveal();
        break;
      }
    }
  }
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
  close();
}
function resize() {
  close();
}
watch(
  () => props.disabled,
  (value) => {
    if (value) close();
  },
);
watch(
  () => props.options.map((o) => `${o.value}:${o.label}`).join('|'),
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
    role="combobox"
    :aria-label="label"
    :aria-describedby="describedby"
    :aria-expanded="opened"
    :aria-controls="opened ? `${id}-listbox` : undefined"
    :aria-activedescendant="opened ? optionId(active) : undefined"
    aria-haspopup="listbox"
    :disabled="disabled"
    class="inline-flex h-8 min-w-24 shrink-0 items-center justify-end gap-2 rounded-md border-0 bg-transparent pl-2 pr-1 text-xs text-ink disabled:cursor-not-allowed disabled:opacity-50"
    @pointerdown="rememberPointerState"
    @pointercancel="openedOnPointerDown = null"
    @click="toggle"
    @keydown="keydown"
    @blur="close()"
  >
    <span class="truncate">{{ selectedLabel }}</span>
    <ChevronDown class="size-3 text-muted" aria-hidden="true" />
  </button>
  <Teleport to="body">
    <div
      v-if="opened"
      :id="`${id}-listbox`"
      ref="panel"
      role="listbox"
      :aria-label="label"
      class="fixed z-50 overflow-y-auto overscroll-contain rounded-lg border border-line-strong bg-surface p-1 text-xs text-ink shadow-sm"
      :class="{ invisible: !positioned }"
      :style="position"
      @mousedown.prevent
    >
      <div
        v-for="(option, index) in options"
        :id="optionId(index)"
        :key="option.value"
        role="option"
        :aria-selected="option.value === modelValue"
        class="flex h-8 cursor-pointer items-center gap-2 whitespace-nowrap rounded px-2"
        :class="index === active ? 'bg-hover' : ''"
        @pointermove="active = index"
        @click="choose(index)"
      >
        <Check
          class="size-3"
          :class="option.value === modelValue ? 'opacity-100' : 'opacity-0'"
          aria-hidden="true"
        />
        <span class="min-w-0 truncate">{{ option.label }}</span>
      </div>
    </div>
  </Teleport>
</template>
