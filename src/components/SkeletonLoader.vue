<template>
  <div class="space-y-3">
    <div
      v-for="i in count"
      :key="i"
      class="h-16 rounded-lg shimmer"
      :class="{ 'w-full': true }"
    />
  </div>
</template>

<script setup lang="ts">
withDefaults(
  defineProps<{
    /** Number of skeleton items to show */
    count?: number
  }>(),
  {
    count: 3,
  }
)
</script>
