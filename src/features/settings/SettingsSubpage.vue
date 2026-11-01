<script setup lang="ts">
import { ArrowLeft } from 'lucide-vue-next';
defineProps<{ title: string; backLabel: string; busy?: boolean; description?: string }>();
const emit = defineEmits<{ back: [] }>();
</script>
<template>
  <section class="mx-auto w-full min-w-0 max-w-[580px]" :aria-label="title" data-settings-subpage>
    <header
      class="mb-5 flex min-h-8 items-start justify-between gap-3"
      data-settings-subpage-header
    >
      <div class="flex min-w-0 flex-1 items-start gap-2">
        <button
          type="button"
          class="icon-button shrink-0"
          :aria-label="backLabel"
          :title="backLabel"
          :disabled="busy"
          @click="emit('back')"
        >
          <ArrowLeft :size="16" />
        </button>
        <div class="flex min-w-0 flex-wrap items-baseline gap-x-2.5 gap-y-1 pt-0.5">
          <h1 class="min-w-0 break-words text-lg font-semibold leading-7">{{ title }}</h1>
          <span v-if="$slots.meta" class="text-xs font-normal tabular-nums text-faint"
            ><slot name="meta"
          /></span>
        </div>
      </div>
      <div
        v-if="$slots.actions"
        class="flex min-h-8 shrink-0 items-center gap-2"
        data-settings-subpage-actions
      >
        <slot name="actions" />
      </div>
    </header>
    <div class="min-w-0" data-settings-subpage-content>
      <p v-if="description" class="mb-3 text-xs leading-relaxed text-faint">{{ description }}</p>
      <slot />
    </div>
  </section>
</template>
