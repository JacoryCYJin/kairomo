<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
import { ChevronLeft, ChevronRight } from 'lucide-vue-next';
import { copy, locale } from '../../shared/i18n';
import { dateInput, monthGrid, parseDateInput } from './history';
import TimeWheel from './TimeWheel.vue';
const props = defineProps<{ id: string; label: string; modelValue: string; disabled?: boolean }>();
const emit = defineEmits<{ 'update:modelValue': [string] }>();
const root = ref<HTMLElement>(),
  panel = ref<HTMLElement>(),
  dateField = ref<HTMLInputElement>(),
  timeField = ref<HTMLInputElement>();
const hourWheel = ref<InstanceType<typeof TimeWheel>>(),
  minuteWheel = ref<InstanceType<typeof TimeWheel>>();
const portal = ref<string | HTMLElement>('body');
const active = ref<'date' | 'time' | null>(null);
const placement = ref({ left: '0px', top: '0px', maxHeight: '280px' });
const dateText = ref(''),
  timeText = ref('');
const month = ref(new Date());
const today = () => dateInput(new Date());
const days = computed(() => monthGrid(month.value));
const heading = computed(() =>
  month.value.toLocaleDateString(locale.value, { year: 'numeric', month: 'long' }),
);
const weekdays = computed(() =>
  Array.from({ length: 7 }, (_, i) =>
    new Date(2026, 9, 26 + i).toLocaleDateString(locale.value, { weekday: 'narrow' }),
  ),
);
const selectedDate = computed(() => props.modelValue.split('T')[0]);
const selectedHour = computed(() => props.modelValue.split('T')[1]?.slice(0, 2) ?? '');
const selectedMinute = computed(() => props.modelValue.split('T')[1]?.slice(3, 5) ?? '');
function sync() {
  dateText.value = props.modelValue.slice(0, 10).replaceAll('-', '/');
  timeText.value = props.modelValue.split('T')[1] ?? '';
}
// Keep partial keyboard input intact while the parent reports invalid dates or times.
watch(
  () => props.modelValue,
  () => {
    sync();
    if (active.value)
      void nextTick(() => {
        if (active.value) positionPanel(active.value);
      });
  },
  { immediate: true },
);
watch(
  () => props.disabled,
  (disabled) => {
    if (disabled) close(false);
  },
);
function commitDate() {
  const key = dateText.value.replaceAll('/', '-');
  emit('update:modelValue', `${key}T${timeText.value}`);
}
function commitTime() {
  emit('update:modelValue', `${dateText.value.replaceAll('/', '-')}T${timeText.value}`);
}
function inputTime() {
  // Explicit typing replaces any pending wheel gesture.
  if (active.value === 'time') active.value = null;
  commitTime();
}
function beforeSubmit() {
  if (active.value === 'time') close(false);
}
function close(restore = true) {
  if (active.value === 'time') {
    const hour = hourWheel.value?.flush(),
      minute = minuteWheel.value?.flush();
    if (hour?.changed || minute?.changed) {
      const [a = '00', b = '00'] = timeText.value.split(':');
      timeText.value =
        (hour ? String(hour.value).padStart(2, '0') : a) +
        ':' +
        (minute ? String(minute.value).padStart(2, '0') : b);
      commitTime();
    }
  }
  const field = active.value === 'date' ? dateField.value : timeField.value;
  active.value = null;
  if (restore) field?.focus({ preventScroll: true });
}
function positionPanel(kind: 'date' | 'time') {
  const field = kind === 'date' ? dateField.value : timeField.value;
  const rect = field?.getBoundingClientRect();
  if (!rect) return;
  const height = kind === 'date' ? 242 : 198;
  const width = kind === 'date' ? 260 : 160;
  const below = window.innerHeight - rect.bottom - 12,
    above = rect.top - 12;
  const up = below < height && above > below;
  const visible = Math.min(height, Math.max(0, up ? above : below));
  placement.value = {
    left: `${Math.max(8, Math.min(rect.right - width, window.innerWidth - width - 8))}px`,
    top: `${up ? rect.top - visible - 6 : rect.bottom + 6}px`,
    maxHeight: `${visible}px`,
  };
}
async function open(kind: 'date' | 'time') {
  if (props.disabled) return;
  active.value = kind;
  portal.value = root.value?.closest<HTMLDialogElement>('dialog[open]') ?? 'body';
  positionPanel(kind);
  const value = parseDateInput(selectedDate.value) ?? new Date();
  month.value = new Date(value.getFullYear(), value.getMonth(), 1);
  await nextTick();
}
function chooseDate(day: Date) {
  if (dateInput(day) > today()) return;
  emit('update:modelValue', `${dateInput(day)}T${timeText.value}`);
  close();
}
function chooseTime(kind: 'hour' | 'minute', value: string) {
  const [hour = '00', minute = '00'] = timeText.value.split(':');
  const next = kind === 'hour' ? `${value}:${minute}` : `${hour}:${value}`;
  timeText.value = next;
  commitTime();
}
function browse(delta: number) {
  month.value = new Date(month.value.getFullYear(), month.value.getMonth() + delta, 1);
}
function dayKey(event: KeyboardEvent, day: Date) {
  const next = new Date(day);
  const offsets: Record<string, number> = {
    ArrowLeft: -1,
    ArrowRight: 1,
    ArrowUp: -7,
    ArrowDown: 7,
  };
  if (event.key in offsets) next.setDate(next.getDate() + offsets[event.key]);
  else if (event.key === 'Home') next.setDate(next.getDate() - ((next.getDay() + 6) % 7));
  else if (event.key === 'End') next.setDate(next.getDate() + 6 - ((next.getDay() + 6) % 7));
  else if (event.key === 'PageUp' || event.key === 'PageDown') {
    next.setDate(1);
    next.setMonth(next.getMonth() + (event.key === 'PageUp' ? -1 : 1) * (event.shiftKey ? 12 : 1));
  } else return;
  event.preventDefault();
  if (dateInput(next) > today()) return;
  month.value = new Date(next.getFullYear(), next.getMonth(), 1);
  void nextTick(() =>
    panel.value?.querySelector<HTMLButtonElement>(`[data-date="${dateInput(next)}"]`)?.focus(),
  );
}
function intoPanel(event: KeyboardEvent, kind: 'date' | 'time') {
  if (event.key === 'Escape' && active.value) {
    event.preventDefault();
    event.stopPropagation();
    close();
  }
  if (event.key === 'ArrowDown') {
    event.preventDefault();
    void open(kind).then(() =>
      panel.value
        ?.querySelector<HTMLButtonElement>(
          kind === 'date' ? '[aria-selected=true]' : '[role=spinbutton]',
        )
        ?.focus(),
    );
  }
}
function outside(event: Event) {
  if (
    active.value &&
    event.target instanceof Node &&
    !root.value?.contains(event.target) &&
    !panel.value?.contains(event.target)
  )
    close(false);
}
function fieldFocused(kind: 'date' | 'time') {
  if (active.value && active.value !== kind) close(false);
}
function reposition() {
  close(false);
}
onMounted(() => {
  document.addEventListener('submit', beforeSubmit, true);
  document.addEventListener('pointerdown', outside);
  document.addEventListener('focusin', outside);
  window.addEventListener('resize', reposition);
});
onUnmounted(() => {
  document.removeEventListener('submit', beforeSubmit, true);
  document.removeEventListener('pointerdown', outside);
  document.removeEventListener('focusin', outside);
  window.removeEventListener('resize', reposition);
});
</script>
<template>
  <div
    ref="root"
    class="flex min-h-9 min-w-0 items-center justify-between gap-3"
    data-record-time-field
  >
    <span :id="id + '-label'" class="shrink-0 text-xs text-muted">{{ label }}</span>
    <div class="flex min-w-0 items-center gap-1.5">
      <span class="relative shrink-0" data-record-date-frame>
        <span
          aria-hidden="true"
          class="invisible block h-8 whitespace-pre rounded-md border border-transparent px-1.5 text-xs tabular-nums"
          >2000/00/00</span
        >
        <input
          ref="dateField"
          v-model="dateText"
          type="text"
          inputmode="numeric"
          :aria-label="label + ' · ' + copy.recordDate"
          :disabled="disabled"
          maxlength="10"
          placeholder="YYYY/MM/DD"
          autocomplete="off"
          aria-haspopup="dialog"
          :aria-expanded="active === 'date'"
          class="absolute inset-0 !m-0 !h-8 !w-full min-w-0 !rounded-md !border-transparent !bg-transparent !px-1.5 !py-0 !text-center !text-xs tabular-nums hover:!bg-hover focus:!border-line focus:!bg-surface-muted focus:outline-none"
          @focus="fieldFocused('date')"
          @click="open('date')"
          @input="commitDate"
          @keydown="intoPanel($event, 'date')"
        />
      </span>
      <span class="relative shrink-0" data-record-time-frame>
        <span
          aria-hidden="true"
          class="invisible block h-8 whitespace-pre rounded-md border border-transparent px-1.5 text-xs tabular-nums"
          >00:00</span
        >
        <input
          ref="timeField"
          v-model="timeText"
          type="text"
          inputmode="numeric"
          :aria-label="label + ' · ' + copy.recordTime"
          :disabled="disabled"
          maxlength="5"
          placeholder="HH:mm"
          autocomplete="off"
          aria-haspopup="dialog"
          :aria-expanded="active === 'time'"
          class="absolute inset-0 !m-0 !h-8 !w-full min-w-0 !rounded-md !border-transparent !bg-transparent !px-1.5 !py-0 !text-center !text-xs tabular-nums hover:!bg-hover focus:!border-line focus:!bg-surface-muted focus:outline-none"
          @focus="fieldFocused('time')"
          @click="open('time')"
          @input="inputTime"
          @keydown="intoPanel($event, 'time')"
        />
      </span>
    </div>
  </div>
  <Teleport :to="portal">
    <div
      v-if="active"
      ref="panel"
      role="dialog"
      :aria-label="label + ' · ' + (active === 'date' ? copy.recordDate : copy.recordTime)"
      class="fixed z-50 overflow-auto rounded-xl border border-line bg-surface p-2 text-ink shadow-sm"
      :class="active === 'date' ? 'w-[260px]' : 'w-40'"
      :style="placement"
      @keydown.esc.stop.prevent="close()"
    >
      <template v-if="active === 'date'">
        <div class="mb-2 flex h-7 items-center justify-between">
          <button
            type="button"
            class="icon-button"
            :aria-label="copy.previousMonth"
            @click="browse(-1)"
          >
            <ChevronLeft :size="14" />
          </button>
          <span class="text-xs font-medium" aria-live="polite">{{ heading }}</span>
          <button
            type="button"
            class="icon-button"
            :aria-label="copy.nextMonth"
            :disabled="dateInput(month).slice(0, 7) >= today().slice(0, 7)"
            @click="browse(1)"
          >
            <ChevronRight :size="14" />
          </button>
        </div>
        <div role="grid" :aria-label="heading">
          <div class="grid grid-cols-7" role="row">
            <span
              v-for="(name, i) in weekdays"
              :key="i"
              role="columnheader"
              class="pb-1 text-center text-xs text-faint"
              >{{ name }}</span
            >
          </div>
          <div v-for="week in 6" :key="week" role="row" class="grid grid-cols-7">
            <button
              v-for="day in days.slice((week - 1) * 7, week * 7)"
              :key="dateInput(day)"
              type="button"
              role="gridcell"
              :data-date="dateInput(day)"
              :aria-label="
                day.toLocaleDateString(locale, { year: 'numeric', month: 'long', day: 'numeric' })
              "
              :aria-selected="dateInput(day) === selectedDate"
              :disabled="dateInput(day) > today()"
              :tabindex="dateInput(day) === selectedDate ? 0 : -1"
              class="flex h-7 items-center justify-center rounded-md text-xs tabular-nums enabled:hover:bg-hover disabled:opacity-25"
              :class="
                dateInput(day) === selectedDate
                  ? 'bg-selected font-semibold text-ink'
                  : day.getMonth() !== month.getMonth()
                    ? 'text-faint'
                    : 'text-muted'
              "
              @click="chooseDate(day)"
              @keydown="dayKey($event, day)"
            >
              {{ day.getDate() }}
            </button>
          </div>
        </div>
      </template>
      <div v-else>
        <div class="mb-1 grid grid-cols-2 gap-2 text-center text-xs text-faint">
          <span>{{ copy.recordHour }}</span
          ><span>{{ copy.recordMinute }}</span>
        </div>
        <div class="relative">
          <div
            aria-hidden="true"
            class="pointer-events-none absolute inset-x-0 top-16 h-8 rounded-md bg-selected"
          />
          <div class="grid grid-cols-2 gap-2">
            <TimeWheel
              ref="hourWheel"
              :value="Number(selectedHour)"
              :count="24"
              :label="label + ' · ' + copy.recordHour"
              @change="chooseTime('hour', String($event).padStart(2, '0'))"
            />
            <TimeWheel
              ref="minuteWheel"
              :value="Number(selectedMinute)"
              :count="60"
              :label="label + ' · ' + copy.recordMinute"
              @change="chooseTime('minute', String($event).padStart(2, '0'))"
            />
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>
