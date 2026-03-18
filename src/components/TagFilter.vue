<template>
  <div class="flex flex-col gap-2">
    <div class="flex items-center gap-2">
      <span class="text-sm text-gray-400">{{ t('tagFilter.title') }}</span>
      <button
        v-if="selectedTags.length > 0"
        @click="clearAll"
        class="text-xs text-gray-500 hover:text-white"
      >
        {{ t('tagFilter.clearAll') }}
      </button>
    </div>

    <!-- Selected tags -->
    <div v-if="selectedTags.length > 0" class="flex flex-wrap gap-1.5">
      <TagBadge
        v-for="tag in selectedTags"
        :key="tag.id"
        :tag="tag"
        removable
        @remove="removeTag(tag.id)"
      />
    </div>

    <!-- Tag selector -->
    <div class="relative">
      <button
        @click="showDropdown = !showDropdown"
        class="w-full bg-darker border border-gray-600 rounded px-3 py-1.5 text-sm text-left text-gray-400 hover:border-gray-500 transition-colors"
      >
        {{ selectedTags.length > 0 ? t('tagFilter.addMoreTags') : t('tagFilter.selectTagToFilter') }}
      </button>

      <div
        v-if="showDropdown && availableTags.length > 0"
        class="absolute top-full left-0 right-0 mt-1 bg-dark border border-gray-600 rounded-lg shadow-lg z-10 max-h-48 overflow-auto"
      >
        <button
          v-for="tag in availableTags"
          :key="tag.id"
          @click="addTag(tag)"
          class="w-full text-left px-3 py-2 hover:bg-darker flex items-center justify-between"
        >
          <div class="flex items-center gap-2">
            <span :class="getDotClass(tag.color)"></span>
            <span>{{ tag.name }}</span>
          </div>
          <span class="text-xs text-gray-500">{{ t('tagFilter.times', { count: tag.usage_count || 0 }) }}</span>
        </button>
      </div>
    </div>

    <!-- Logic hint -->
    <p v-if="selectedTags.length > 1" class="text-xs text-gray-500">
      {{ t('tagFilter.andLogic') }}
    </p>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import { showError } from '../stores/toast'
import TagBadge from './TagBadge.vue'

const { t } = useI18n()

const props = defineProps({
  modelValue: {
    type: Array,
    default: () => []
  }
})

const emit = defineEmits(['update:modelValue'])

// State
const showDropdown = ref(false)
const allTags = ref([])

// Selected tag IDs
const selectedTags = computed(() => props.modelValue)

// Available tags (not yet selected)
const availableTags = computed(() => {
  const selectedIds = new Set(selectedTags.value.map(t => t.id))
  return allTags.value.filter(tag => !selectedIds.has(tag.id))
})

// Color dots
const colorDots = {
  blue: 'w-2 h-2 rounded-full bg-blue-500',
  green: 'w-2 h-2 rounded-full bg-green-500',
  yellow: 'w-2 h-2 rounded-full bg-yellow-400',
  red: 'w-2 h-2 rounded-full bg-red-500',
  purple: 'w-2 h-2 rounded-full bg-purple-500',
  pink: 'w-2 h-2 rounded-full bg-pink-500',
  cyan: 'w-2 h-2 rounded-full bg-cyan-500',
  orange: 'w-2 h-2 rounded-full bg-orange-500'
}

function getDotClass(color) {
  return colorDots[color] || colorDots.blue
}

// Load all tags
async function loadTags() {
  try {
    allTags.value = await invoke('get_all_manual_tags')
  } catch (e) {
    showError(t('tagFilter.loadFailed'))
  }
}

// Add tag to selection
function addTag(tag) {
  emit('update:modelValue', [...selectedTags.value, tag])
  showDropdown.value = false
}

// Remove tag from selection
function removeTag(tagId) {
  emit('update:modelValue', selectedTags.value.filter(t => t.id !== tagId))
}

// Clear all selected tags
function clearAll() {
  emit('update:modelValue', [])
}

// Close dropdown on outside click
function handleClickOutside(e) {
  if (!e.target.closest('.relative')) {
    showDropdown.value = false
  }
}

onMounted(() => {
  loadTags()
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})

// Expose loadTags for parent components
defineExpose({ loadTags })
</script>