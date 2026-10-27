<script setup lang="ts">
import { reactive, watch } from 'vue';
import { Check } from 'lucide-vue-next';
import { availableSeries } from '../../shared/series/catalog';
import { copy, locale } from '../../shared/i18n';
import type { Preferences, Settings } from '../../shared/ipc/types';
import SettingsSelect from '../../shared/ui/SettingsSelect.vue';
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
function choosePreference(key: keyof Preferences, value: string) {
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
            <SettingsSelect
              id="focus-mode"
              :model-value="form.focus_mode"
              :label="copy.focusMode"
              describedby="focus-mode-hint"
              :options="[
                { value: 'manual', label: copy.manualFocus },
                { value: 'timed', label: copy.timedFocus },
              ]"
              @update:model-value="form.focus_mode = $event as Settings['focus_mode']"
              :disabled="busy"
            />
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
            <SettingsSelect
              id="locale"
              :model-value="settings.locale"
              :label="copy.language"
              :options="[
                { value: 'zh-CN', label: '简体中文' },
                { value: 'en-US', label: 'English' },
              ]"
              @update:model-value="choosePreference('locale', $event)"
              :disabled="busy"
            />
          </div>
          <div
            class="mx-3.5 flex min-h-12 items-center justify-between gap-4 border-t border-line py-2"
          >
            <label for="appearance" class="text-xs">{{ copy.appearance }}</label>
            <SettingsSelect
              id="appearance"
              :model-value="settings.appearance"
              :label="copy.appearance"
              :options="[
                { value: 'system', label: copy.followSystem },
                { value: 'light', label: copy.light },
                { value: 'dark', label: copy.dark },
              ]"
              @update:model-value="choosePreference('appearance', $event)"
              :disabled="busy"
            />
          </div>
          <div
            class="mx-3.5 flex min-h-12 items-center justify-between gap-4 border-t border-line py-2"
          >
            <label for="visual-series" class="text-xs">{{ copy.visualSeries }}</label>
            <SettingsSelect
              id="visual-series"
              :model-value="settings.visual_series"
              :label="copy.visualSeries"
              :options="
                availableSeries.map((series) => ({
                  value: series.id,
                  label: series.id === 'coffee' ? copy.coffeeSeriesShort : series.name[locale],
                }))
              "
              @update:model-value="choosePreference('visual_series', $event)"
              :disabled="busy"
            />
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
