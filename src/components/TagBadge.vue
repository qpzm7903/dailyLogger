<template>
  <span
    :class="[
      'inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs font-medium transition-colors',
      colorClasses
    ]"
  >
    {{ tag.name }}
    <button
      v-if="removable"
      @click.stop="$emit('remove')"
      class="ml-1 hover:text-white/80"
    >
      ✕
    </button>
  </span>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { Tag } from '../types/tauri'
import { TAG_COLOR_CLASSES_SOLID, DEFAULT_TAG_COLOR_SOLID } from '../utils/tagColors'

interface Props {
  tag: Tag
  removable?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  removable: false
})

defineEmits<{
  remove: []
}>()

const colorClasses = computed(() => {
  return TAG_COLOR_CLASSES_SOLID[props.tag.color] || TAG_COLOR_CLASSES_SOLID.blue
})
</script>