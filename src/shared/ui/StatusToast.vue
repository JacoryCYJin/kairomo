<script setup lang="ts">
import { onUnmounted, ref, watch } from 'vue';
import { Check } from 'lucide-vue-next';
const props = defineProps<{ trigger: number; message: string }>();
const visible = ref(false);
let timeout: ReturnType<typeof setTimeout> | undefined;
watch(
  () => props.trigger,
  () => {
    clearTimeout(timeout);
    visible.value = true;
    timeout = setTimeout(() => (visible.value = false), 2000);
  },
);
onUnmounted(() => clearTimeout(timeout));
</script>
<template>
  <div
    class="pointer-events-none fixed inset-x-5 bottom-5 z-50 flex justify-center"
    role="status"
    aria-atomic="true"
  >
    <Transition
      enter-active-class="transition duration-150 ease-out"
      enter-from-class="translate-y-1 opacity-0"
      leave-active-class="transition duration-150 ease-in"
      leave-to-class="translate-y-1 opacity-0"
    >
      <div
        v-if="visible"
        class="flex max-w-full items-center gap-2 rounded-lg border border-line bg-surface px-3.5 py-2.5 text-xs text-ink shadow-lg"
      >
        <Check :size="15" aria-hidden="true" />
        <span>{{ message }}</span>
      </div>
    </Transition>
  </div>
</template>
