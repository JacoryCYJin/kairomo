<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
import { Plus, X } from 'lucide-vue-next';
import { copy } from '../i18n';
import type { Task } from '../ipc/types';
const props = defineProps<{
  tasks: Task[];
  selected: string | null;
  busy: boolean;
  selectTask: (id: string | null) => Promise<boolean>;
  createTask: (name: string) => Promise<string | null>;
}>();
const modal = ref<HTMLDialogElement>();
const field = ref<HTMLInputElement>();
const name = ref(''),
  error = ref('');
const options = computed(() => [
  { value: '', label: copy.value.uncategorized },
  ...props.tasks.filter((t) => !t.archived).map((t) => ({ value: t.id, label: t.name })),
]);
const ticks = computed(() => {
  const slots = options.value.length - 1;
  if (slots < 1) return [];
  return Array.from({ length: slots * 8 + 1 }, (_, index) => ({
    left: index * 6,
    size:
      index % 8 === 0
        ? 'h-1.5 opacity-80'
        : index % 4 === 0
          ? 'h-1 opacity-30'
          : 'h-0.5 opacity-20',
  }));
});
// Native horizontal scrolling owns momentum. Only the final alignment is animated here.
const step = 48;
const track = ref<HTMLDivElement>();
const createButton = ref<HTMLButtonElement>();
const position = ref(0),
  displayIndex = ref(0),
  interacting = ref(false),
  creating = ref(false),
  saving = ref(false);
const blocked = computed(() => props.busy && !saving.value);
const selectedIndex = computed(() =>
  Math.max(
    0,
    options.value.findIndex((o) => o.value === (props.selected ?? '')),
  ),
);
let settleTimer: ReturnType<typeof setTimeout> | undefined;
let clickTimer: ReturnType<typeof setTimeout> | undefined;
let animationFrame = 0,
  paintFrame = 0,
  writeFrame = 0;
let animating = false,
  disposed = false,
  generation = 0;
let manualOffset: number | undefined;
let pendingSelection: string | null | undefined;
let pointer: { id: number; x: number; offset: number; dragging: boolean } | undefined;
let suppressClick = false;
function clamp(index: number) {
  return Math.max(0, Math.min(options.value.length - 1, index));
}
function paint() {
  if (!track.value) return;
  position.value = track.value.scrollLeft / step;
  displayIndex.value = clamp(Math.round(position.value));
}
function cancelAnimation() {
  cancelAnimationFrame(animationFrame);
  animationFrame = 0;
  animating = false;
  clearTimeout(settleTimer);
}
function beginMotion() {
  generation++;
  cancelAnimation();
  interacting.value = true;
}
async function persist(index: number) {
  const choice = options.value[clamp(index)];
  if (!choice || blocked.value || disposed) return;
  const id = choice.value || null;
  if (!saving.value && id === props.selected) return;
  // Coalesce newer choices while a write is in flight, including A -> B -> A.
  pendingSelection = id;
  if (saving.value) return;
  saving.value = true;
  try {
    while (pendingSelection !== undefined && !disposed) {
      const requested = pendingSelection;
      pendingSelection = undefined;
      const attempt = generation;
      let success = false;
      try {
        success = await props.selectTask(requested);
      } catch {
        /* Restore the authoritative selection below. */
      }
      if (
        !success &&
        pendingSelection === undefined &&
        attempt === generation &&
        !interacting.value &&
        !disposed
      )
        align(selectedIndex.value, false);
    }
  } finally {
    saving.value = false;
  }
}
function align(index: number, commit: boolean) {
  if (!track.value || disposed) return;
  cancelAnimation();
  cancelAnimationFrame(writeFrame);
  writeFrame = 0;
  manualOffset = undefined;
  const element = track.value;
  const target = clamp(index) * step;
  const from = element.scrollLeft;
  const reduced = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  const finish = () => {
    element.scrollLeft = target;
    paint();
    animating = false;
    animationFrame = 0;
    interacting.value = false;
    if (commit) void persist(index);
  };
  if (Math.abs(target - from) < 0.5 || reduced) return finish();
  animating = true;
  interacting.value = true;
  const duration = Math.min(240, 140 + Math.abs(target - from) * 0.4);
  let started: number | undefined;
  function frame(now: number) {
    if (disposed) return;
    started ??= now;
    const progress = Math.min(1, (now - started) / duration);
    element.scrollLeft = from + (target - from) * (1 - Math.pow(1 - progress, 3));
    paint();
    if (progress < 1) animationFrame = requestAnimationFrame(frame);
    else finish();
  }
  animationFrame = requestAnimationFrame(frame);
}
function settle() {
  if (pointer || animating || !interacting.value || blocked.value) return;
  align(clamp(Math.round((track.value?.scrollLeft ?? 0) / step)), true);
}
function scheduleSettle() {
  clearTimeout(settleTimer);
  if (!pointer && !animating && interacting.value) settleTimer = setTimeout(settle, 170);
}
function scrolled() {
  if (!paintFrame)
    paintFrame = requestAnimationFrame(() => {
      paintFrame = 0;
      paint();
    });
  // Animation-generated scroll events must not launch a second alignment.
  scheduleSettle();
}
function choose(index: number) {
  if (blocked.value || suppressClick) return;
  beginMotion();
  align(index, true);
}
function writeOffset() {
  if (!track.value || manualOffset === undefined) return;
  track.value.scrollLeft = manualOffset;
  paint();
}
function queueOffset(offset: number) {
  manualOffset = Math.max(0, Math.min((options.value.length - 1) * step, offset));
  if (!writeFrame)
    writeFrame = requestAnimationFrame(() => {
      writeFrame = 0;
      writeOffset();
    });
}
function wheel(event: WheelEvent) {
  if (blocked.value || event.ctrlKey || creating.value || !track.value) return;
  beginMotion();
  event.stopPropagation();
  if (Math.abs(event.deltaX) > Math.abs(event.deltaY)) {
    // Do not preventDefault or rewrite scrollLeft: retain the trackpad's native inertia.
    manualOffset = undefined;
  } else {
    event.preventDefault();
    const unit = event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? 144 : 1;
    queueOffset((manualOffset ?? track.value.scrollLeft) + event.deltaY * unit);
  }
  scheduleSettle();
}
function pointerDown(event: PointerEvent) {
  if (blocked.value || !track.value) return;
  if (event.pointerType === 'touch') {
    beginMotion();
    return;
  }
  if (event.pointerType !== 'mouse' || event.button !== 0) return;
  pointer = {
    id: event.pointerId,
    x: event.clientX,
    offset: track.value.scrollLeft,
    dragging: false,
  };
}
function pointerMove(event: PointerEvent) {
  if (!pointer || pointer.id !== event.pointerId || !track.value || blocked.value) return;
  const distance = event.clientX - pointer.x;
  if (!pointer.dragging && Math.abs(distance) < 4) return;
  if (!pointer.dragging) {
    pointer.dragging = true;
    beginMotion();
    track.value.setPointerCapture(event.pointerId);
  }
  event.preventDefault();
  queueOffset(pointer.offset - distance);
}
function pointerUp(event: PointerEvent) {
  if (event.pointerType === 'touch') {
    scheduleSettle();
    return;
  }
  if (!pointer || pointer.id !== event.pointerId) return;
  const dragged = pointer.dragging;
  pointer = undefined;
  if (track.value?.hasPointerCapture(event.pointerId))
    track.value.releasePointerCapture(event.pointerId);
  if (dragged) {
    cancelAnimationFrame(writeFrame);
    writeFrame = 0;
    writeOffset();
    suppressClick = true;
    clearTimeout(clickTimer);
    clickTimer = setTimeout(() => {
      suppressClick = false;
    }, 0);
    settle();
  }
}
function keyboard(event: KeyboardEvent, index: number) {
  let next = index;
  if (event.key === 'ArrowRight') next++;
  else if (event.key === 'ArrowLeft') next--;
  else if (event.key === 'Home') next = 0;
  else if (event.key === 'End') next = options.value.length - 1;
  else return;
  event.preventDefault();
  next = clamp(next);
  choose(next);
  track.value
    ?.querySelectorAll<HTMLButtonElement>('[role=radio]')
    [next]?.focus({ preventScroll: true });
}
function emphasis(index: number) {
  return { opacity: 0.4 + 0.6 * Math.max(0, 1 - Math.abs(position.value - index)) };
}
async function create() {
  if (props.busy) return;
  beginMotion();
  align(selectedIndex.value, false);
  name.value = '';
  error.value = '';
  creating.value = true;
  modal.value?.showModal();
  await nextTick();
  field.value?.focus();
}
function closed() {
  creating.value = false;
  createButton.value?.focus({ preventScroll: true });
}
watch(
  () => props.selected,
  async () => {
    await nextTick();
    // Successful acknowledgements never reposition a moving or already aligned wheel.
    if (!saving.value && !interacting.value && !disposed) align(selectedIndex.value, false);
  },
);
watch(
  () => options.value.map((o) => o.value).join('\0'),
  async () => {
    await nextTick();
    if (!disposed) align(selectedIndex.value, false);
  },
);
onMounted(() => {
  if (track.value) {
    track.value.scrollLeft = selectedIndex.value * step;
    paint();
  }
  window.addEventListener('pointerup', pointerUp);
  window.addEventListener('pointercancel', pointerUp);
});
onUnmounted(() => {
  disposed = true;
  pendingSelection = undefined;
  cancelAnimation();
  cancelAnimationFrame(writeFrame);
  cancelAnimationFrame(paintFrame);
  clearTimeout(clickTimer);
  window.removeEventListener('pointerup', pointerUp);
  window.removeEventListener('pointercancel', pointerUp);
});
async function save() {
  if (!name.value.trim() || [...name.value.trim()].length > 40) {
    error.value = copy.value.taskNameInvalid;
    return;
  }
  const failure = await props.createTask(name.value);
  if (failure) error.value = failure;
  else modal.value?.close();
}
</script>
<template>
  <div
    class="flex w-[166px] items-center gap-1 text-muted"
    :data-task-interacting="interacting"
    @mousedown.stop
  >
    <div class="relative min-w-0 flex-1">
      <div
        ref="track"
        role="radiogroup"
        :aria-label="copy.currentTask"
        :aria-disabled="blocked"
        class="overflow-x-auto overscroll-contain [scrollbar-width:none] [touch-action:pan-x] [-webkit-mask-image:linear-gradient(to_right,transparent,black_24%,black_76%,transparent)] [mask-image:linear-gradient(to_right,transparent,black_24%,black_76%,transparent)] [&::-webkit-scrollbar]:hidden"
        @wheel="wheel"
        @scroll="scrolled"
        @pointerdown="pointerDown"
        @pointermove="pointerMove"
        @pointerup="pointerUp"
        @pointercancel="pointerUp"
      >
        <div class="relative flex h-[27px] w-max items-start px-12">
          <button
            v-for="(option, index) in options"
            :key="option.value"
            type="button"
            role="radio"
            :aria-checked="displayIndex === index"
            :tabindex="displayIndex === index ? 0 : -1"
            :title="option.label"
            :disabled="blocked"
            class="h-5 w-12 shrink-0 truncate border-0 bg-transparent px-0.5 text-center text-xs font-medium leading-5 text-ink"
            :style="emphasis(index)"
            @click="choose(index)"
            @keydown="keyboard($event, index)"
          >
            {{ option.label }}
          </button>
          <div
            v-if="ticks.length"
            aria-hidden="true"
            class="pointer-events-none absolute top-5 h-[7px]"
            :style="{ left: '72px', width: `${(ticks.length - 1) * 6}px` }"
          >
            <i
              v-for="tick in ticks"
              :key="tick.left"
              class="absolute bottom-0 block w-px bg-current"
              :class="tick.size"
              :style="{ left: `${tick.left}px` }"
            ></i>
          </div>
        </div>
      </div>
      <span
        v-if="ticks.length"
        aria-hidden="true"
        class="pointer-events-none absolute left-1/2 top-5 h-[7px] w-px -translate-x-1/2 bg-ink"
      ></span>
    </div>
    <button
      ref="createButton"
      type="button"
      class="flex h-6 w-[18px] shrink-0 items-center justify-center rounded text-muted enabled:hover:bg-hover enabled:hover:text-ink"
      :aria-label="copy.newTask"
      :title="copy.newTask"
      :aria-expanded="creating"
      :disabled="busy"
      @click="create"
    >
      <Plus :size="14" />
    </button>
  </div>
  <Teleport to="body">
    <dialog ref="modal" class="modal" @close="closed" @cancel="busy && $event.preventDefault()">
      <form @submit.prevent="save">
        <div class="modal-heading">
          <h2>{{ copy.newTask }}</h2>
          <button
            type="button"
            class="icon-button"
            :aria-label="copy.cancel"
            :disabled="busy"
            @click="modal?.close()"
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
        <p v-if="error" class="text-danger" role="alert">{{ error }}</p>
        <div class="modal-actions">
          <button type="button" class="button secondary" :disabled="busy" @click="modal?.close()">
            {{ copy.cancel }}</button
          ><button type="submit" class="button dialog-confirm" :disabled="busy">
            {{ copy.save }}
          </button>
        </div>
      </form>
    </dialog>
  </Teleport>
</template>
