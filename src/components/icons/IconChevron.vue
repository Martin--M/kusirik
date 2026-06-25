<script setup lang="ts">
withDefaults(defineProps<{
  direction?: 'up' | 'down' | 'left' | 'right'
}>(), {
  direction: 'down'
})
</script>

<template>
  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" :class="`direction-${direction}`">
    <polyline v-if="direction === 'down'" points="6 9 12 15 18 9" />
    <polyline v-else-if="direction === 'up'" points="18 15 12 9 6 15" />
    <polyline v-else-if="direction === 'left'" points="15 18 9 12 15 6" />
    <polyline v-else-if="direction === 'right'" points="9 18 15 12 9 6" />
  </svg>
</template>
