<script setup lang="ts">
import { reactive, watch } from 'vue';
import { Check, ChevronDown } from 'lucide-vue-next';
import { availableSeries } from '../../shared/series/catalog';
import { copy, locale } from '../../shared/i18n';
import type { Preferences, Settings } from '../../shared/ipc/types';
import UpdateSettings from './UpdateSettings.vue';
const props = defineProps<{ settings: Settings; busy: boolean }>();
const emit = defineEmits<{ save: [Settings]; preference: [Partial<Preferences>] }>();
const form = reactive({
  focus_minutes: props.settings.focus_minutes,
  break_minutes: props.settings.break_minutes,
  notifications: props.settings.notifications,
  focus_mode: props.settings.focus_mode,
});
watch(
  () => props.settings,
  (next, previous) => {
    // Preference saves and timer snapshots must not erase unsaved duration edits.
    for (const key of ['focus_minutes', 'break_minutes'] as const) {
      if (form[key] === previous[key]) form[key] = next[key];
    }
    if (form.notifications === previous.notifications) form.notifications = next.notifications;
    if (form.focus_mode === previous.focus_mode) form.focus_mode = next.focus_mode;
  },
);
function choosePreference(key: keyof Preferences, event: Event) {
  const select = event.target as HTMLSelectElement;
  const value = select.value;
  // Display only the persisted value; a failed save leaves the original choice selected.
  select.value = props.settings[key];
  if (key === 'locale') emit('preference', { locale: value as Preferences['locale'] });
  else if (key === 'appearance')
    emit('preference', { appearance: value as Preferences['appearance'] });
  else emit('preference', { visual_series: value as Preferences['visual_series'] });
}
</script>
<template>
  <section class="mx-auto max-w-[580px] pb-2 pt-4">
    <h1 class="text-xl font-semibold tracking-tight">{{ copy.settingsTitle }}</h1>
    <form @submit.prevent="emit('save', { ...settings, ...form })">
      <section class="mt-4" :aria-label="copy.timerSettings">
        <h2 class="mb-2.5 text-sm font-medium">{{ copy.timerSettings }}</h2>
        <div class="grid auto-rows-fr rounded-xl border border-line bg-surface">
          <label class="flex min-h-[52px] items-center justify-between gap-4 px-3.5 py-1.5">
            <span class="text-xs"
              >{{ copy.focusDuration
              }}<span id="focus-duration-hint" class="mt-0.5 block leading-normal text-faint">{{
                copy.nextFocusDuration
              }}</span></span
            >
            <span class="flex shrink-0 items-center gap-2 text-xs text-muted">
              <input
                v-model.number="form.focus_minutes"
                :aria-label="copy.focusDuration"
                aria-describedby="focus-duration-hint"
                class="h-7 w-16 rounded-md border border-line bg-paper px-2 text-xs tabular-nums"
                type="number"
                min="1"
                max="240"
                required
              />
              {{ copy.minutes }}
            </span>
          </label>
          <label
            class="mx-3.5 flex min-h-[52px] items-center justify-between gap-4 border-t border-line py-1.5"
          >
            <span class="text-xs"
              >{{ copy.breakDuration
              }}<span id="break-duration-hint" class="mt-0.5 block leading-normal text-faint">{{
                copy.breakEndHint
              }}</span></span
            >
            <span class="flex shrink-0 items-center gap-2 text-xs text-muted">
              <input
                v-model.number="form.break_minutes"
                :aria-label="copy.breakDuration"
                aria-describedby="break-duration-hint"
                class="h-7 w-16 rounded-md border border-line bg-paper px-2 text-xs tabular-nums"
                type="number"
                min="1"
                max="120"
                required
              />
              {{ copy.minutes }}
            </span>
          </label>
          <div
            class="mx-3.5 flex min-h-[52px] items-center justify-between gap-4 border-t border-line py-1.5"
          >
            <label for="focus-mode" class="min-w-0 text-xs">
              {{ copy.focusMode }}
              <span id="focus-mode-hint" class="mt-0.5 block leading-normal text-faint">
                {{ form.focus_mode === 'manual' ? copy.manualFocusHint : copy.timedFocusHint }}
              </span>
            </label>
            <div class="relative shrink-0">
              <select
                id="focus-mode"
                v-model="form.focus_mode"
                :aria-label="copy.focusMode"
                aria-describedby="focus-mode-hint"
                class="compact-select"
                :disabled="busy"
              >
                <option value="manual">{{ copy.manualFocus }}</option>
                <option value="timed">{{ copy.timedFocus }}</option></select
              ><ChevronDown
                class="pointer-events-none absolute right-1 top-1/2 size-3 -translate-y-1/2 text-muted"
                aria-hidden="true"
              />
            </div>
          </div>
          <div
            class="mx-3.5 flex min-h-[52px] items-center justify-between gap-4 border-t border-line py-1.5"
          >
            <div>
              <label for="notifications" class="text-xs">{{ copy.reminders }}</label>
              <p id="notifications-hint" class="mt-0.5 text-xs leading-normal text-faint">
                {{ copy.targetReminderHint }}
              </p>
            </div>
            <span class="relative flex h-5 w-9 shrink-0">
              <input
                id="notifications"
                v-model="form.notifications"
                class="peer h-5 w-9 cursor-pointer appearance-none rounded-full bg-line-strong transition-colors checked:bg-ink"
                type="checkbox"
                role="switch"
                aria-describedby="notifications-hint"
              />
              <span
                aria-hidden="true"
                class="pointer-events-none absolute left-0.5 top-0.5 size-4 rounded-full bg-surface transition-transform peer-checked:translate-x-4 peer-checked:bg-paper"
              ></span>
            </span>
          </div>
        </div>
      </section>
      <section class="mt-4" :aria-label="copy.pageSettings">
        <h2 class="mb-2.5 text-sm font-medium">{{ copy.pageSettings }}</h2>
        <div class="rounded-xl border border-line bg-surface">
          <div class="flex min-h-12 items-center justify-between gap-4 px-3.5 py-2">
            <label for="locale" class="text-xs">{{ copy.language }}</label>
            <div class="relative shrink-0">
              <select
                id="locale"
                class="compact-select"
                :value="settings.locale"
                :disabled="busy"
                @change="choosePreference('locale', $event)"
              >
                <option value="zh-CN">简体中文</option>
                <option value="en-US">English</option></select
              ><ChevronDown
                class="pointer-events-none absolute right-1 top-1/2 size-3 -translate-y-1/2 text-muted"
                aria-hidden="true"
              />
            </div>
          </div>
          <div
            class="mx-3.5 flex min-h-12 items-center justify-between gap-4 border-t border-line py-2"
          >
            <label for="appearance" class="text-xs">{{ copy.appearance }}</label>
            <div class="relative shrink-0">
              <select
                id="appearance"
                class="compact-select"
                :value="settings.appearance"
                :disabled="busy"
                @change="choosePreference('appearance', $event)"
              >
                <option value="system">{{ copy.followSystem }}</option>
                <option value="light">{{ copy.light }}</option>
                <option value="dark">{{ copy.dark }}</option></select
              ><ChevronDown
                class="pointer-events-none absolute right-1 top-1/2 size-3 -translate-y-1/2 text-muted"
                aria-hidden="true"
              />
            </div>
          </div>
          <div
            class="mx-3.5 flex min-h-12 items-center justify-between gap-4 border-t border-line py-2"
          >
            <label for="visual-series" class="text-xs">{{ copy.visualSeries }}</label>
            <div class="relative shrink-0">
              <select
                id="visual-series"
                class="compact-select"
                :value="settings.visual_series"
                :disabled="busy"
                @change="choosePreference('visual_series', $event)"
              >
                <option v-for="series in availableSeries" :key="series.id" :value="series.id">
                  {{ series.id === 'coffee' ? copy.coffeeSeriesShort : series.name[locale] }}
                </option></select
              ><ChevronDown
                class="pointer-events-none absolute right-1 top-1/2 size-3 -translate-y-1/2 text-muted"
                aria-hidden="true"
              />
            </div>
          </div>
        </div>
      </section>
      <div class="flex justify-end pb-2 pt-6">
        <button class="button primary compact" type="submit" :disabled="busy">
          <Check :size="14" />{{ copy.saveSettings }}
        </button>
      </div>
    </form>
    <UpdateSettings />
  </section>
</template>
