<script setup lang="ts">
import CoffeeReward from './CoffeeReward.vue';
import type { BeanPose } from './layouts';
defineProps<{ pose: BeanPose; duration: number; description: string }>();
</script>
<template>
  <g
    :transform="`translate(${pose.x} ${pose.y}) rotate(${(pose.angle * 180) / Math.PI}) scale(${pose.size} ${pose.squash}) rotate(56.4)`"
    :data-bean-id="pose.id"
    data-exterior="true"
  >
    <title>{{ description }}</title>
    <CoffeeReward :duration="duration" x="-21" y="-21" width="42" height="42" />
  </g>
</template>
