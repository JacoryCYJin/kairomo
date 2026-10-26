<script setup lang="ts">
import { reactive, watch } from 'vue';
import { Bell, Languages, Check, Monitor } from 'lucide-vue-next';
import { copy } from '../../shared/i18n';
import type { Settings } from '../../shared/ipc/types';
const props = defineProps<{ settings: Settings; busy: boolean; dark: boolean }>();
const emit = defineEmits<{ save: [Settings] }>();
const form = reactive({ ...props.settings });
watch(
  () => JSON.stringify(props.settings),
  () => Object.assign(form, props.settings),
);
</script>
<template>
  <section class="pb-5">
    <div class="pt-5">
      <h1 class="mb-2 text-xl font-semibold tracking-tight">{{ copy.settingsTitle }}</h1>
    </div>
    <form @submit.prevent="emit('save', { ...form })">
      <section class="mt-7 pb-6">
        <h2 class="mb-4 text-sm font-medium">{{ copy.timerSettings }}</h2>
        <div class="grid grid-cols-2 gap-4">
          <label class="text-xs text-muted"
            >{{ copy.focusDuration }}
            <div
              class="mt-2 flex h-11 items-center gap-2.5 rounded-xl border border-line bg-surface-muted px-3"
            >
              <input
                class="min-w-0 w-full border-0 bg-transparent text-lg tabular-nums outline-offset-0"
                v-model.number="form.focus_minutes"
                type="number"
                min="1"
                max="240"
                required
              /><span class="whitespace-nowrap text-xs">{{ copy.minutes }}</span>
            </div></label
          ><label class="text-xs text-muted"
            >{{ copy.breakDuration }}
            <div
              class="mt-2 flex h-11 items-center gap-2.5 rounded-xl border border-line bg-surface-muted px-3"
            >
              <input
                class="min-w-0 w-full border-0 bg-transparent text-lg tabular-nums outline-offset-0"
                v-model.number="form.break_minutes"
                type="number"
                min="1"
                max="120"
                required
              /><span class="whitespace-nowrap text-xs">{{ copy.minutes }}</span>
            </div></label
          >
        </div>
        <p class="mt-3 text-xs leading-relaxed text-faint">{{ copy.nextSession }}</p>
      </section>
      <div class="flex min-h-[73px] items-center gap-3 border-t border-line py-3.5">
        <Bell :size="18" :stroke-width="1.6" class="text-muted" />
        <div>
          <label for="notifications">{{ copy.reminders }}</label>
          <p class="mt-1.5 text-xs leading-relaxed text-faint">{{ copy.reminderHint }}</p>
        </div>
        <span class="relative ml-auto flex h-5 w-9 shrink-0">
          <input
            id="notifications"
            v-model="form.notifications"
            class="peer h-5 w-9 cursor-pointer appearance-none rounded-full bg-line-strong transition-colors checked:bg-ink"
            type="checkbox"
            role="switch"
          /><span
            aria-hidden="true"
            class="pointer-events-none absolute left-0.5 top-0.5 size-4 rounded-full bg-surface transition-transform peer-checked:translate-x-4 peer-checked:bg-paper"
          ></span>
        </span>
      </div>
      <div class="flex min-h-[73px] items-center gap-3 border-t border-line py-3.5">
        <Languages :size="18" :stroke-width="1.6" class="text-muted" /><label for="locale">{{
          copy.language
        }}</label
        ><select
          class="ml-auto max-w-[150px] rounded-lg border border-line bg-surface px-2.5 py-1.5 text-xs"
          id="locale"
          v-model="form.locale"
        >
          <option value="zh-CN">简体中文</option>
          <option value="en-US">English</option>
        </select>
      </div>
      <div class="flex min-h-[73px] items-center gap-3 border-t border-line py-3.5">
        <Monitor :size="18" :stroke-width="1.6" class="text-muted" />
        <div>
          <span>{{ copy.appearance }}</span>
          <p class="mt-1.5 text-xs leading-relaxed text-faint">{{ copy.followSystemHint }}</p>
        </div>
        <span class="ml-auto text-right text-xs text-muted min-[621px]:whitespace-nowrap"
          >{{ copy.followSystem }} · {{ dark ? copy.dark : copy.light }}</span
        >
      </div>
      <div class="flex justify-end pt-4">
        <button class="button primary" type="submit" :disabled="busy">
          <Check :size="15" />{{ copy.saveSettings }}
        </button>
      </div>
    </form>
  </section>
</template>
