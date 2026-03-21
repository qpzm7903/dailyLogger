<template>
  <div class="fixed inset-0 bg-black/80 flex items-center justify-center z-50" @click.self="$emit('close')">
    <div class="bg-dark rounded-2xl w-[90vw] h-[90vh] max-w-4xl overflow-hidden border border-gray-700 flex flex-col">
      <!-- Header -->
      <div class="px-6 py-4 border-b border-gray-700 flex items-center justify-between">
        <h2 class="text-lg font-semibold">{{ t('historyViewer.title') }}</h2>
        <button @click="$emit('close')" class="text-gray-400 hover:text-white">✕</button>
      </div>

      <!-- Filters -->
      <div class="px-6 py-3 border-b border-gray-700 flex items-center gap-4 flex-wrap">
        <div class="flex items-center gap-2">
          <label class="text-sm text-gray-300">{{ t('historyViewer.startDate') }}</label>
          <input
            type="date"
            v-model="startDate"
            class="bg-darker border border-gray-600 rounded px-2 py-1 text-sm text-white focus:border-primary focus:outline-none"
          />
        </div>
        <div class="flex items-center gap-2">
          <label class="text-sm text-gray-300">{{ t('historyViewer.endDate') }}</label>
          <input
            type="date"
            v-model="endDate"
            class="bg-darker border border-gray-600 rounded px-2 py-1 text-sm text-white focus:border-primary focus:outline-none"
          />
        </div>
        <div class="flex items-center gap-2">
          <label class="text-sm text-gray-300">{{ t('historyViewer.source') }}</label>
          <select
            v-model="sourceType"
            class="bg-darker border border-gray-600 rounded px-2 py-1 text-sm text-white focus:border-primary focus:outline-none"
          >
            <option :value="null">{{ t('historyViewer.all') }}</option>
            <option value="auto">{{ t('historyViewer.autoCapture') }}</option>
            <option value="manual">{{ t('historyViewer.manualRecord') }}</option>
          </select>
        </div>
        <button
          @click="loadRecords"
          :disabled="isLoading"
          class="px-4 py-1 bg-primary text-white rounded text-sm hover:bg-primary/80 transition-colors disabled:opacity-50"
        >
          {{ isLoading ? t('historyViewer.loading') : t('historyViewer.query') }}
        </button>
        <span v-if="records.length > 0" class="text-sm text-gray-300 ml-auto">
          {{ t('historyViewer.totalRecords', { count: records.length }) }}
        </span>
      </div>

      <!-- Tag Filter -->
      <div class="px-6 py-3 border-b border-gray-700">
        <TagFilter
          ref="tagFilterRef"
          v-model="selectedTags"
        />
      </div>

      <!-- Record List -->
      <div class="flex-1 overflow-auto p-4" ref="scrollContainer" @scroll="handleScroll">
        <div v-if="isLoading && records.length === 0" class="text-center py-8 text-gray-500">
          {{ t('historyViewer.loading') }}
        </div>
        <div v-else-if="records.length === 0" class="text-center py-8 text-gray-500">
          {{ t('historyViewer.noRecords') }}
        </div>

        <!-- UX-012: Virtual scroll for large datasets -->
        <div
          v-else-if="shouldUseVirtualScroll"
          class="relative"
          :style="{ height: `${virtualizer.getTotalSize()}px` }"
        >
          <div
            v-for="virtualItem in virtualItems"
            :key="virtualItem.index"
            class="absolute top-0 left-0 w-full py-3 px-2 hover:bg-darker/50 transition-colors group border-b border-gray-700"
            :style="{
              height: `${virtualItem.size}px`,
              transform: `translateY(${virtualItem.start}px)`,
            }"
            :data-index="virtualItem.index"
          >
            <template v-if="records[virtualItem.index]">
              <div class="flex items-start justify-between gap-2">
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2 mb-1">
                    <span
                      :class="records[virtualItem.index].source_type === 'auto' ? 'bg-blue-500/20 text-blue-400' : 'bg-green-500/20 text-green-400'"
                      class="px-2 py-0.5 rounded text-xs"
                    >
                      {{ records[virtualItem.index].source_type === 'auto' ? t('historyViewer.auto') : t('historyViewer.manual') }}
                    </span>
                    <span class="text-xs text-gray-400">{{ formatTime(records[virtualItem.index].timestamp) }}</span>
                  </div>
                  <p class="text-sm text-gray-300 line-clamp-3 whitespace-pre-wrap break-words">{{ truncateContent(records[virtualItem.index].content) }}</p>
                  <!-- Manual tags -->
                  <div v-if="getRecordTags(records[virtualItem.index].id).length > 0" class="flex flex-wrap gap-1 mt-2">
                    <TagBadge
                      v-for="tag in getRecordTags(records[virtualItem.index].id)"
                      :key="tag.id"
                      :tag="tag"
                    />
                  </div>
                </div>
                <button
                  @click="confirmDelete(records[virtualItem.index])"
                  class="opacity-0 group-hover:opacity-100 text-red-400 hover:text-red-300 text-sm px-2 py-1 transition-opacity"
                >
                  {{ t('historyViewer.delete') }}
                </button>
              </div>
            </template>
          </div>
        </div>

        <!-- Normal rendering for small datasets -->
        <div v-else class="flex flex-col divide-y divide-gray-700">
          <div
            v-for="record in records"
            :key="record.id"
            class="py-3 px-2 hover:bg-darker/50 transition-colors group"
          >
            <div class="flex items-start justify-between gap-2">
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2 mb-1">
                  <span
                    :class="record.source_type === 'auto' ? 'bg-blue-500/20 text-blue-400' : 'bg-green-500/20 text-green-400'"
                    class="px-2 py-0.5 rounded text-xs"
                  >
                    {{ record.source_type === 'auto' ? t('historyViewer.auto') : t('historyViewer.manual') }}
                  </span>
                  <span class="text-xs text-gray-500">{{ formatTime(record.timestamp) }}</span>
                </div>
                <p class="text-sm text-gray-300 truncate">{{ truncateContent(record.content) }}</p>
                <!-- Manual tags -->
                <div v-if="getRecordTags(record.id).length > 0" class="flex flex-wrap gap-1 mt-2">
                  <TagBadge
                    v-for="tag in getRecordTags(record.id)"
                    :key="tag.id"
                    :tag="tag"
                  />
                </div>
              </div>
              <button
                @click="confirmDelete(record)"
                class="opacity-0 group-hover:opacity-100 text-red-400 hover:text-red-300 text-sm px-2 py-1 transition-opacity"
              >
                {{ t('historyViewer.delete') }}
              </button>
            </div>
          </div>
        </div>

        <!-- Loading indicator for pagination -->
        <div v-if="isLoadingMore" class="text-center py-4 text-gray-500">
          {{ t('historyViewer.loadingMore') }}
        </div>
      </div>
    </div>

    <!-- Delete Confirmation Modal -->
    <div
      v-if="recordToDelete"
      class="fixed inset-0 bg-black/50 flex items-center justify-center z-60"
    >
      <div class="bg-dark rounded-xl p-6 max-w-sm border border-gray-700">
        <h3 class="text-lg font-semibold mb-4">{{ t('historyViewer.confirmDelete') }}</h3>
        <p class="text-gray-400 mb-6">{{ t('historyViewer.confirmDeleteMessage') }}</p>
        <div class="flex justify-end gap-3">
          <button
            @click="recordToDelete = null"
            class="px-4 py-2 bg-gray-600 text-white rounded hover:bg-gray-500 transition-colors"
          >
            {{ t('historyViewer.cancel') }}
          </button>
          <button
            @click="deleteRecord"
            :disabled="isDeleting"
            class="px-4 py-2 bg-red-500 text-white rounded hover:bg-red-400 transition-colors disabled:opacity-50"
          >
            {{ isDeleting ? t('historyViewer.deleting') : t('historyViewer.delete') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, nextTick, watch, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import { useVirtualizer } from '@tanstack/vue-virtual'
import { showSuccess, showError } from '../stores/toast'
import TagFilter from './TagFilter.vue'
import TagBadge from './TagBadge.vue'
import type { LogRecord, Tag } from '../types/tauri'

const { t } = useI18n()

const emit = defineEmits<{(e: 'close'): void}>()

const props = defineProps<{
  initialTag?: Tag | null
}>()

// UX-012: Virtual scroll configuration
const VIRTUAL_SCROLL_CONFIG = {
  itemHeight: 80,          // Fixed height per record (px)
  overscan: 5,             // Render extra items outside viewport
  threshold: 100,          // Enable virtual scroll above this count
}

// State
const startDate = ref('')
const endDate = ref('')
const sourceType = ref<'auto' | 'manual' | null>(null)
const selectedTags = ref<Tag[]>([])
const records = ref<LogRecord[]>([])
const recordTags = ref<Record<number, Tag[]>>({}) // Map of record id to tags
const isLoading = ref(false)
const isLoadingMore = ref(false)
const page = ref(0)
const pageSize = 50
const hasMore = ref(true)
const scrollContainer = ref<HTMLElement | null>(null)
const recordToDelete = ref<LogRecord | null>(null)
const isDeleting = ref(false)
const tagFilterRef = ref<InstanceType<typeof TagFilter> | null>(null)

// UX-012: Virtual scroll - only enable for large datasets
const shouldUseVirtualScroll = computed(() => records.value.length > VIRTUAL_SCROLL_CONFIG.threshold)

// UX-012: Virtualizer instance
const virtualizer = useVirtualizer({
  count: records.value.length,
  getScrollElement: () => scrollContainer.value,
  estimateSize: () => VIRTUAL_SCROLL_CONFIG.itemHeight,
  overscan: VIRTUAL_SCROLL_CONFIG.overscan,
})

// UX-012: Virtual items to render
const virtualItems = computed(() => virtualizer.value.getVirtualItems())

// Initialize dates to last 7 days
onMounted(() => {
  const end = new Date()
  const start = new Date()
  start.setDate(start.getDate() - 7)

  endDate.value = formatDate(end)
  startDate.value = formatDate(start)

  // Apply initial tag filter from TagCloud selection (FIX-003)
  if (props.initialTag) {
    selectedTags.value = [props.initialTag]
  }

  loadRecords()
})

// Watch for tag filter changes
watch(selectedTags, () => {
  loadRecords()
}, { deep: true })

function formatDate(date: Date) {
  return date.toISOString().split('T')[0]
}

function formatTime(timestamp: string) {
  const date = new Date(timestamp)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  })
}

function truncateContent(content: string) {
  if (!content) return ''
  // Try to parse JSON content
  try {
    const parsed = JSON.parse(content) as { summary?: string; note?: string }
    if (parsed.summary) return parsed.summary
    if (parsed.note) return parsed.note
    return content
  } catch {
    return content.length > 100 ? content.slice(0, 100) + '...' : content
  }
}

async function loadRecords() {
  isLoading.value = true
  page.value = 0
  records.value = []
  recordTags.value = {}
  hasMore.value = true

  try {
    // If tags are selected, use tag-based filtering
    if (selectedTags.value.length > 0) {
      const tagIds = selectedTags.value.map(t => t.id)
      const result = await invoke<LogRecord[]>('get_records_by_manual_tags', {
        tagIds: tagIds,
        startDate: startDate.value,
        endDate: endDate.value,
        sourceType: sourceType.value
      })

      records.value = result
      hasMore.value = false // Tag-based query doesn't support pagination
    } else {
      // Regular date/source filtering
      const result = await invoke<LogRecord[]>('get_history_records', {
        startDate: startDate.value,
        endDate: endDate.value,
        sourceType: sourceType.value,
        page: 0,
        pageSize: pageSize
      })

      records.value = result
      hasMore.value = result.length === pageSize
    }

    // Load tags for all records
    await loadRecordTags()
  } catch (error) {
    showError(t('historyViewer.loadFailed', { error }))
  } finally {
    isLoading.value = false
  }
}

// Load tags for all displayed records (batch query - PERF-001)
async function loadRecordTags() {
  const ids = records.value.map(r => r.id)
  if (ids.length === 0) return

  try {
    const tagsMap = await invoke<Record<number, Tag[]>>('get_tags_for_records', { recordIds: ids })
    recordTags.value = tagsMap
  } catch (e) {
    console.error('Failed to load tags for records:', e)
    recordTags.value = {}
  }
}

// Get tags for a specific record
function getRecordTags(recordId: number) {
  return recordTags.value[recordId] || []
}

async function loadMoreRecords() {
  if (isLoadingMore.value || !hasMore.value) return
  if (selectedTags.value.length > 0) return // Tag-based query doesn't support pagination

  isLoadingMore.value = true
  page.value += 1

  try {
    const result = await invoke<LogRecord[]>('get_history_records', {
      startDate: startDate.value,
      endDate: endDate.value,
      sourceType: sourceType.value,
      page: page.value,
      pageSize: pageSize
    })

    records.value.push(...result)
    hasMore.value = result.length === pageSize

    // Load tags for new records (batch)
    const newIds = result.map(r => r.id)
    if (newIds.length > 0) {
      try {
        const tagsMap = await invoke<Record<number, Tag[]>>('get_tags_for_records', { recordIds: newIds })
        Object.assign(recordTags.value, tagsMap)
      } catch (e) {
        console.error('Failed to load tags for new records:', e)
      }
    }
  } catch (error) {
    showError(t('historyViewer.loadMoreFailed', { error }))
    page.value -= 1 // Revert page increment on error
  } finally {
    isLoadingMore.value = false
  }
}

function handleScroll() {
  if (!scrollContainer.value) return

  const { scrollTop, scrollHeight, clientHeight } = scrollContainer.value
  const isNearBottom = scrollHeight - scrollTop - clientHeight < 100

  if (isNearBottom && hasMore.value && !isLoadingMore.value) {
    loadMoreRecords()
  }
}

function confirmDelete(record: LogRecord) {
  recordToDelete.value = record
}

async function deleteRecord() {
  if (!recordToDelete.value) return

  isDeleting.value = true

  try {
    await invoke('delete_record', { id: recordToDelete.value.id })

    // Remove from local list
    records.value = records.value.filter(r => r.id !== recordToDelete.value!.id)

    showSuccess(t('historyViewer.recordDeleted'))
    recordToDelete.value = null
  } catch (error) {
    showError(t('historyViewer.deleteFailed', { error }))
  } finally {
    isDeleting.value = false
  }
}
</script>