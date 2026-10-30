<script setup lang="ts">
import { computed } from 'vue';
import type { CollectionProps } from '../contracts';
import { copy, duration } from '../../i18n';
import SnowScene from './SnowScene.vue';
import { collectionProgress } from './progress';
const props = defineProps<CollectionProps>();
const progress = computed(() => collectionProgress(props.total));
</script>
<template>
  <div class="flex h-full min-h-0 flex-col items-center justify-end" data-snow-collection>
    <SnowScene
      class="block w-full min-h-0 max-w-[260px] flex-1"
      :progress="progress"
      :class="fitHeight ? 'max-h-[250px]' : 'max-h-[320px]'"
    />
    <strong class="mt-2 translate-y-1 text-center text-sm font-medium text-ink">{{
      total === 0 ? copy.snowJourneyNotStarted : duration(total)
    }}</strong>
  </div>
</template>
