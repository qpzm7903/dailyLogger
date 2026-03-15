<template>
  <div class="flex flex-col gap-2">
    <!-- Existing tags -->
    <div v-if="modelValue.length > 0" class="flex flex-wrap gap-1.5">
      <TagBadge
        v-for="tag in modelValue"
        :key="tag.id"
        :tag="tag"
        removable
        @remove="removeTag(tag.id)"
      />
    </div>

    <!-- Add tag input -->
    <div class="flex items-center gap-2">
      <div class="relative flex-1">
        <input
          v-model="searchQuery"
          @focus="showDropdown = true"
          @keydown.enter.prevent="createOrSelectTag"
          @keydown.escape="showDropdown = false"
          :placeholder="placeholder"
          class="w-full bg-darker border border-gray-600 rounded px-3 py-1.5 text-sm text-white focus:border-primary focus:outline-none"
        />

        <!-- Dropdown for existing tags -->
        <div
          v-if="showDropdown && filteredTags.length > 0"
          class="absolute top-full left-0 right-0 mt-1 bg-dark border border-gray-600 rounded-lg shadow-lg z-10 max-h-48 overflow-auto"
        >
          <button
            v-for="tag in filteredTags"
            :key="tag.id"
            @click="selectTag(tag)"
            class="w-full text-left px-3 py-2 hover:bg-darker flex items-center justify-between"
          >
            <div class="flex items-center gap-2">
              <span :class="getDotClass(tag.color)"></span>
              <span>{{ tag.name }}</span>
            </div>
            <span class="text-xs text-gray-500">{{ tag.usage_count || 0 }}次</span>
          </button>
        </div>
      </div>

      <!-- Color selector -->
      <div class="flex items-center gap-1">
        <button
          v-for="color in colors"
          :key="color.name"
          @click="selectedColor = color.name"
          :class="[
            'w-5 h-5 rounded transition-transform',
            color.bgClass,
            selectedColor === color.name ? 'ring-2 ring-white scale-110' : ''
          ]"
          :title="color.name"
        ></button>
      </div>

      <button
        @click="createOrSelectTag"
        :disabled="!searchQuery.trim() || isCreating"
        class="px-3 py-1.5 bg-primary text-white rounded text-sm hover:bg-blue-600 transition-colors disabled:opacity-50"
      >
        {{ isCreating ? '...' : '添加' }}
      </button>
    </div>

    <!-- Tag limit hint -->
    <p v-if="modelValue.length >= 10" class="text-xs text-yellow-500">
      已达标签上限 (10个)
    </p>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { showSuccess, showError } from '../stores/toast'
import TagBadge from './TagBadge.vue'

const props = defineProps({
  modelValue: {
    type: Array,
    default: () => []
  },
  recordId: {
    type: Number,
    required: true
  },
  placeholder: {
    type: String,
    default: '输入标签名...'
  }
})

const emit = defineEmits(['update:modelValue'])

// State
const searchQuery = ref('')
const showDropdown = ref(false)
const selectedColor = ref('blue')
const allTags = ref([])
const isCreating = ref(false)

// Preset colors
const colors = [
  { name: 'blue', bgClass: 'bg-blue-500' },
  { name: 'green', bgClass: 'bg-green-500' },
  { name: 'yellow', bgClass: 'bg-yellow-400' },
  { name: 'red', bgClass: 'bg-red-500' },
  { name: 'purple', bgClass: 'bg-purple-500' },
  { name: 'pink', bgClass: 'bg-pink-500' },
  { name: 'cyan', bgClass: 'bg-cyan-500' },
  { name: 'orange', bgClass: 'bg-orange-500' }
]

// Filter tags not already added
const filteredTags = computed(() => {
  const query = searchQuery.value.toLowerCase().trim()
  const addedIds = new Set(props.modelValue.map(t => t.id))

  return allTags.value.filter(tag => {
    if (addedIds.has(tag.id)) return false
    if (query && !tag.name.toLowerCase().includes(query)) return false
    return true
  })
})

// Get dot class for color
function getDotClass(color) {
  const colorObj = colors.find(c => c.name === color)
  return colorObj ? `w-2 h-2 rounded-full ${colorObj.bgClass}` : 'w-2 h-2 rounded-full bg-blue-500'
}

// Load all tags
async function loadAllTags() {
  try {
    allTags.value = await invoke('get_all_manual_tags')
  } catch (e) {
    console.error('Failed to load tags:', e)
  }
}

// Select existing tag
async function selectTag(tag) {
  if (props.modelValue.length >= 10) {
    showError('每条记录最多只能添加 10 个标签')
    return
  }

  try {
    await invoke('add_tag_to_record', { recordId: props.recordId, tagId: tag.id })
    emit('update:modelValue', [...props.modelValue, tag])
    searchQuery.value = ''
    showDropdown.value = false
  } catch (e) {
    showError(e)
  }
}

// Create new tag or select existing
async function createOrSelectTag() {
  const name = searchQuery.value.trim()
  if (!name) return

  if (props.modelValue.length >= 10) {
    showError('每条记录最多只能添加 10 个标签')
    return
  }

  // Check if tag already exists
  const existingTag = allTags.value.find(t => t.name.toLowerCase() === name.toLowerCase())
  if (existingTag) {
    // Check if already added to this record
    if (props.modelValue.some(t => t.id === existingTag.id)) {
      showDropdown.value = false
      searchQuery.value = ''
      return
    }
    await selectTag(existingTag)
    return
  }

  // Create new tag
  isCreating.value = true
  try {
    const newTag = await invoke('create_manual_tag', {
      name: name,
      color: selectedColor.value
    })

    // Add to record
    await invoke('add_tag_to_record', { recordId: props.recordId, tagId: newTag.id })

    // Update local state
    allTags.value.push(newTag)
    emit('update:modelValue', [...props.modelValue, newTag])

    searchQuery.value = ''
    showDropdown.value = false
    showSuccess('标签已创建')
  } catch (e) {
    showError(e)
  } finally {
    isCreating.value = false
  }
}

// Remove tag from record
async function removeTag(tagId) {
  try {
    await invoke('remove_tag_from_record', { recordId: props.recordId, tagId })
    emit('update:modelValue', props.modelValue.filter(t => t.id !== tagId))
  } catch (e) {
    showError(e)
  }
}

// Close dropdown when clicking outside
function handleClickOutside(e) {
  if (!e.target.closest('.relative')) {
    showDropdown.value = false
  }
}

onMounted(() => {
  loadAllTags()
  document.addEventListener('click', handleClickOutside)
})

// Cleanup
import { onUnmounted } from 'vue'
onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>