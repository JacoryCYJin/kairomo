<script setup lang="ts">
import CoffeeReward from './CoffeeReward.vue';
import type { BeanPose } from './layouts';
defineProps<{ pose: BeanPose; duration: number; description: string }>();
</script>
<template>
  <g
    class="collection-bean"
    :transform="`translate(${pose.x} ${pose.y}) rotate(${(pose.angle * 180) / Math.PI}) scale(${pose.size} ${pose.size * pose.squash})`"
    :data-bean-id="pose.id"
    :data-bean-layer="pose.layer"
    :data-bean-depth="pose.depth"
  >
    <title>{{ description }}</title>
    <CoffeeReward :duration="duration" x="-21" y="-21" width="42" height="42" />
  </g>
</template>
<style scoped>
.collection-bean {
  filter: drop-shadow(0 0.65px 0.5px rgb(0 0 0 / 14%));
}
</style>
