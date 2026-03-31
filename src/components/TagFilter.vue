<template>
  <div class="flex flex-col gap-2">
    <div class="flex items-center gap-2">
      <span class="text-sm text-[var(--color-text-secondary)]">{{ t('tagFilter.title') }}</span>
      <button
        v-if="selectedTags.length > 0"
        @click="clearAll"
        class="text-xs text-[var(--color-text-muted)] hover:text-[var(--color-text-primary)]"
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
        class="w-full bg-[var(--color-surface-0)] border border-[var(--color-border-subtle)] rounded px-3 py-1.5 text-sm text-left text-[var(--color-text-secondary)] hover:border-[var(--color-border)] transition-colors"
      >
        {{ selectedTags.length > 0 ? t('tagFilter.addMoreTags') : t('tagFilter.selectTagToFilter') }}
      </button>

      <div
        v-if="showDropdown && availableTags.length > 0"
        class="absolute top-full left-0 right-0 mt-1 bg-[var(--color-surface-1)] border border-[var(--color-border-subtle)] rounded-lg shadow-lg z-10 max-h-48 overflow-auto"
      >
        <button
          v-for="tag in availableTags"
          :key="tag.id"
          @click="addTag(tag)"
          class="w-full text-left px-3 py-2 hover:bg-[var(--color-surface-0)] flex items-center justify-between"
        >
          <div class="flex items-center gap-2">
            <span :class="getDotClass(tag.color)"></span>
            <span>{{ tag.name }}</span>
          </div>
          <span class="text-xs text-[var(--color-text-muted)]">{{ t('tagFilter.times', { count: tag.usage_count || 0 }) }}</span>
        </button>
      </div>
    </div>

    <!-- Logic hint -->
    <p v-if="selectedTags.length > 1" class="text-xs text-[var(--color-text-muted)]">
      {{ t('tagFilter.andLogic') }}
    </p>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import { showError } from '../stores/toast'
import { TAG_COLOR_BG, DEFAULT_TAG_COLOR_BG } from '../utils/tagColors'
import TagBadge from './TagBadge.vue'
import type { Tag } from '../types/tauri'

interface TagWithUsage extends Tag {
  usage_count?: number
}

const { t } = useI18n()

const props = withDefaults(defineProps<{
  modelValue?: Tag[]
}>(), {
  modelValue: () => []
})

const emit = defineEmits<{(e: 'update:modelValue', value: Tag[]): void}>()

// State
const showDropdown = ref(false)
const allTags = ref<TagWithUsage[]>([])

// Selected tag IDs
const selectedTags = computed(() => props.modelValue)

// Available tags (not yet selected)
const availableTags = computed(() => {
  const selectedIds = new Set(selectedTags.value.map(t => t.id))
  return allTags.value.filter(tag => !selectedIds.has(tag.id))
})

function getDotClass(color: string) {
  return `w-2 h-2 rounded-full ${TAG_COLOR_BG[color] || DEFAULT_TAG_COLOR_BG}`
}

// Load all tags
async function loadTags() {
  try {
    allTags.value = await invoke<TagWithUsage[]>('get_all_manual_tags')
  } catch (e) {
    showError(t('tagFilter.loadFailed'))
  }
}

// Add tag to selection
function addTag(tag: Tag) {
  emit('update:modelValue', [...selectedTags.value, tag])
  showDropdown.value = false
}

// Remove tag from selection
function removeTag(tagId: number) {
  emit('update:modelValue', selectedTags.value.filter(t => t.id !== tagId))
}

// Clear all selected tags
function clearAll() {
  emit('update:modelValue', [])
}

// Close dropdown on outside click
function handleClickOutside(e: MouseEvent) {
  if (!(e.target as HTMLElement).closest('.relative')) {
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