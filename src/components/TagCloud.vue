<template>
  <BaseModal @close="$emit('close')" content-class="w-[90vw] max-w-lg overflow-hidden flex flex-col">
    <!-- Header -->
    <div class="px-6 py-4 border-b border-[var(--color-border)] flex items-center justify-between">
        <h3 class="text-lg font-semibold">{{ t('tagCloud.title') }}</h3>
        <button @click="$emit('close')" class="text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]">✕</button>
      </div>

      <!-- Content -->
      <div class="p-6 flex-1 overflow-auto">
        <div class="flex items-center justify-between mb-4">
          <span class="text-sm text-[var(--color-text-secondary)]">{{ t('tagCloud.clickToFilter') }}</span>
          <button
            v-if="selectedTag"
            @click="clearSelection"
            class="text-sm text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]"
          >
            {{ t('tagCloud.clearFilter') }}
          </button>
        </div>

        <!-- Tag cloud -->
        <div v-if="isLoading" class="text-center py-8 text-[var(--color-text-muted)]">
          {{ t('tagCloud.loading') }}
        </div>
        <div v-else-if="tags.length === 0" class="text-center py-8 text-[var(--color-text-muted)]">
          {{ t('tagCloud.noTagsHint') }}
        </div>
        <div v-else class="flex flex-wrap gap-2">
          <button
            v-for="tag in tags"
            :key="tag.id"
            @click="toggleSelect(tag)"
            :class="[
              'px-3 py-1.5 rounded-full transition-all flex items-center gap-2',
              getTagSize(tag),
              selectedTag?.id === tag.id ? 'ring-2 ring-white' : '',
              getTagColor(tag.name)
            ]"
          >
            <span>{{ tag.name }}</span>
            <span class="text-xs opacity-70">{{ tag.usage_count || 0 }}</span>
          </button>
        </div>
      </div>
  </BaseModal>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import { showError } from '../stores/toast'
import { getTagColorClass } from '../utils/tagColors'
import type { Tag } from '../types/tauri'

interface TagCloudTag {
  id: number
  name: string
  color: string
  usage_count?: number
}

const { t } = useI18n()
const emit = defineEmits<{(e: 'tagSelected', tag: Tag | null): void; (e: 'close'): void}>()

// State
const tags = ref<TagCloudTag[]>([])
const isLoading = ref(false)
const selectedTag = ref<TagCloudTag | null>(null)

// Get tag size based on usage count
function getTagSize(tag: TagCloudTag) {
  const count = tag.usage_count || 0
  if (count >= 10) return 'text-base'
  if (count >= 5) return 'text-sm'
  return 'text-xs'
}

function getTagColor(tagName: string) {
  return getTagColorClass(tagName)
}

async function loadTags() {
  isLoading.value = true
  try {
    tags.value = await invoke<TagCloudTag[]>('get_tag_cloud_tags')
  } catch (e) {
    showError(t('tagCloud.loadFailed', { error: e }))
  } finally {
    isLoading.value = false
  }
}

function toSelectedTag(tag: TagCloudTag): Tag {
  return {
    id: tag.id,
    name: tag.name,
    color: tag.color,
    category_id: null,
  }
}

function clearSelection() {
  selectedTag.value = null
  emit('tagSelected', null)
}

function toggleSelect(tag: TagCloudTag) {
  if (selectedTag.value?.id === tag.id) {
    clearSelection()
  } else {
    selectedTag.value = tag
    emit('tagSelected', toSelectedTag(tag))
  }
}

defineExpose({
  loadTags,
})

onMounted(loadTags)
</script>
