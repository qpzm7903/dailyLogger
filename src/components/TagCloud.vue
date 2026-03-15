<template>
  <div class="fixed inset-0 bg-black/80 flex items-center justify-center z-50" @click.self="$emit('close')">
    <div class="bg-dark rounded-2xl w-[90vw] max-w-lg overflow-hidden border border-gray-700 flex flex-col">
      <!-- Header -->
      <div class="px-6 py-4 border-b border-gray-700 flex items-center justify-between">
        <h3 class="text-lg font-semibold">标签云</h3>
        <button @click="$emit('close')" class="text-gray-400 hover:text-white">✕</button>
      </div>

      <!-- Content -->
      <div class="p-6 flex-1 overflow-auto">
        <div class="flex items-center justify-between mb-4">
          <span class="text-sm text-gray-400">点击标签筛选记录</span>
          <button
            v-if="selectedTag"
            @click="selectedTag = null"
            class="text-sm text-gray-400 hover:text-white"
          >
            清除筛选
          </button>
        </div>

        <!-- Tag cloud -->
        <div v-if="isLoading" class="text-center py-8 text-gray-500">
          加载中...
        </div>
        <div v-else-if="tags.length === 0" class="text-center py-8 text-gray-500">
          暂无标签，可在历史记录中为记录添加标签
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
              getTagColor(tag.color)
            ]"
          >
            <span>{{ tag.name }}</span>
            <span class="text-xs opacity-70">{{ tag.usage_count || 0 }}</span>
          </button>
        </div>
      </div>

      <!-- Delete confirmation -->
      <div
        v-if="tagToDelete"
        class="fixed inset-0 bg-black/50 flex items-center justify-center z-60"
      >
        <div class="bg-dark rounded-xl p-6 max-w-sm border border-gray-700">
          <h3 class="text-lg font-semibold mb-4">删除标签</h3>
          <p class="text-gray-400 mb-2">确定要删除标签 "{{ tagToDelete.name }}" 吗？</p>
          <p class="text-sm text-yellow-500 mb-6">该标签将从所有关联记录中移除。</p>
          <div class="flex justify-end gap-3">
            <button
              @click="tagToDelete = null"
              class="px-4 py-2 bg-gray-600 text-white rounded hover:bg-gray-500 transition-colors"
            >
              取消
            </button>
            <button
              @click="confirmDelete"
              :disabled="isDeleting"
              class="px-4 py-2 bg-red-500 text-white rounded hover:bg-red-400 transition-colors disabled:opacity-50"
            >
              {{ isDeleting ? '删除中...' : '删除' }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { showSuccess, showError } from '../stores/toast'

const emit = defineEmits(['tagSelected', 'close'])

// State
const tags = ref([])
const isLoading = ref(false)
const selectedTag = ref(null)
const tagToDelete = ref(null)
const isDeleting = ref(false)

// Color mapping
const colorClasses = {
  blue: 'bg-blue-500/30 text-blue-300 hover:bg-blue-500/50',
  green: 'bg-green-500/30 text-green-300 hover:bg-green-500/50',
  yellow: 'bg-yellow-400/30 text-yellow-200 hover:bg-yellow-400/50',
  red: 'bg-red-500/30 text-red-300 hover:bg-red-500/50',
  purple: 'bg-purple-500/30 text-purple-300 hover:bg-purple-500/50',
  pink: 'bg-pink-500/30 text-pink-300 hover:bg-pink-500/50',
  cyan: 'bg-cyan-500/30 text-cyan-300 hover:bg-cyan-500/50',
  orange: 'bg-orange-500/30 text-orange-300 hover:bg-orange-500/50'
}

// Get tag size based on usage count
function getTagSize(tag) {
  const count = tag.usage_count || 0
  if (count >= 10) return 'text-base'
  if (count >= 5) return 'text-sm'
  return 'text-xs'
}

// Get tag color classes
function getTagColor(color) {
  return colorClasses[color] || colorClasses.blue
}

// Load all tags
async function loadTags() {
  isLoading.value = true
  try {
    tags.value = await invoke('get_all_manual_tags')
  } catch (e) {
    showError('加载标签失败: ' + e)
  } finally {
    isLoading.value = false
  }
}

// Toggle select tag
function toggleSelect(tag) {
  if (selectedTag.value?.id === tag.id) {
    selectedTag.value = null
    emit('tagSelected', null)
  } else {
    selectedTag.value = tag
    emit('tagSelected', tag)
  }
}

// Request delete
function requestDelete(tag) {
  tagToDelete.value = tag
}

// Confirm delete
async function confirmDelete() {
  if (!tagToDelete.value) return

  isDeleting.value = true
  try {
    await invoke('delete_manual_tag', { id: tagToDelete.value.id })
    tags.value = tags.value.filter(t => t.id !== tagToDelete.value.id)
    if (selectedTag.value?.id === tagToDelete.value.id) {
      selectedTag.value = null
      emit('tagSelected', null)
    }
    showSuccess('标签已删除')
  } catch (e) {
    showError('删除失败: ' + e)
  } finally {
    isDeleting.value = false
    tagToDelete.value = null
  }
}

// Expose methods for parent components
defineExpose({
  loadTags,
  requestDelete
})

onMounted(loadTags)
</script>