<script setup lang="ts">
import type { RewardProps } from '../contracts';
defineProps<RewardProps>();
</script>
<template>
  <svg
    viewBox="0 0 24 24"
    xmlns="http://www.w3.org/2000/svg"
    shape-rendering="crispEdges"
    aria-hidden="true"
  >
    <path d="M6 21h2v-4h2v-4h2V9h2V5h-3v4H9v4H7v4H5v4z" fill="#51799C" />
    <path d="M8 3h8v2h4v2h2v3h-2V8h-4V6h-5V5H8z" fill="currentColor" />
    <path d="M4 19h4v3H4z" fill="#E96D2E" />
  </svg>
</template>
