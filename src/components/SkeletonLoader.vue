<template>
  <div class="space-y-3">
    <div
      v-for="i in count"
      :key="i"
      class="h-16 rounded-lg shimmer"
      :class="{ 'h-24': i === 1 && variant === 'gallery' }"
    />
  </div>
</template>

<script setup lang="ts">
withDefaults(
  defineProps<{
    count?: number
    variant?: 'default' | 'gallery'
  }>(),
  {
    count: 3,
    variant: 'default',
  }
)
</script>
