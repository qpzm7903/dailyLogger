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

<script setup>
import { computed } from 'vue'

const props = defineProps({
  tag: {
    type: Object,
    required: true
  },
  removable: {
    type: Boolean,
    default: false
  }
})

defineEmits(['remove'])

// Color mapping
const colorMap = {
  blue: 'bg-blue-500 text-white',
  green: 'bg-green-500 text-white',
  yellow: 'bg-yellow-400 text-slate-800',
  red: 'bg-red-500 text-white',
  purple: 'bg-purple-500 text-white',
  pink: 'bg-pink-500 text-white',
  cyan: 'bg-cyan-500 text-white',
  orange: 'bg-orange-500 text-white'
}

const colorClasses = computed(() => {
  return colorMap[props.tag.color] || colorMap.blue
})
</script>