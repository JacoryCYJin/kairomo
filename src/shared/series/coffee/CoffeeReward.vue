<script setup lang="ts">
import { computed } from 'vue';
import { coffeeTier } from './definition';
import CoffeeBeanIcon from '../../ui/CoffeeBeanIcon.vue';
const props = defineProps<{ duration: number }>();
const tier = computed(() => coffeeTier(props.duration));
</script>
<template>
  <CoffeeBeanIcon
    class="coffee-reward block"
    :style="{ '--bean-light': tier.light, '--bean-dark': tier.dark }"
  />
</template>
<style scoped>
.coffee-reward {
  color: var(--bean-light);
}
:global(:root[data-theme='dark'] .coffee-reward) {
  color: var(--bean-dark);
}
</style>
