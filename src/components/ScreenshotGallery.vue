<template>
  <div class="fixed inset-0 bg-black/80 flex items-center justify-center z-50" @click.self="$emit('close')">
    <div class="bg-dark rounded-2xl w-[90vw] h-[90vh] max-w-6xl overflow-hidden border border-gray-700 flex flex-col">
      <div class="px-6 py-4 border-b border-gray-700 flex items-center justify-between">
        <h2 class="text-lg font-semibold">📷 {{ t('screenshotGallery.title') }}</h2>
        <div class="flex items-center gap-2">
          <!-- View toggle buttons -->
          <button
            @click="viewMode = 'grid'"
            :class="viewMode === 'grid' ? 'bg-primary text-white' : 'bg-darker text-gray-400 hover:text-white'"
            class="px-3 py-1.5 rounded-lg text-sm transition-colors"
            :aria-label="t('screenshotGallery.gridView')"
          >
            {{ t('screenshotGallery.gridView') }}
          </button>
          <button
            @click="viewMode = 'list'"
            :class="viewMode === 'list' ? 'bg-primary text-white' : 'bg-darker text-gray-400 hover:text-white'"
            class="px-3 py-1.5 rounded-lg text-sm transition-colors"
            :aria-label="t('screenshotGallery.listView')"
          >
            {{ t('screenshotGallery.listView') }}
          </button>
          <button @click="$emit('close')" class="text-gray-400 hover:text-white ml-2">✕</button>
        </div>
      </div>

      <!-- Date Filter Section -->
      <div class="px-6 py-3 border-b border-gray-700 flex items-center gap-4 flex-wrap">
        <div class="flex items-center gap-2">
          <label class="text-sm text-gray-400">{{ t('screenshotGallery.startDate') }}</label>
          <input
            type="date"
            v-model="startDate"
            class="bg-darker border border-gray-600 rounded px-2 py-1 text-sm text-white focus:border-primary focus:outline-none"
          />
        </div>
        <div class="flex items-center gap-2">
          <label class="text-sm text-gray-400">{{ t('screenshotGallery.endDate') }}</label>
          <input
            type="date"
            v-model="endDate"
            class="bg-darker border border-gray-600 rounded px-2 py-1 text-sm text-white focus:border-primary focus:outline-none"
          />
        </div>
        <button
          @click="applyFilter"
          class="px-4 py-1 bg-primary text-white rounded text-sm hover:bg-primary/80 transition-colors"
        >
          {{ t('screenshotGallery.filter') }}
        </button>
        <button
          @click="resetFilter"
          class="px-4 py-1 bg-gray-600 text-white rounded text-sm hover:bg-gray-500 transition-colors"
        >
          {{ t('screenshotGallery.reset') }}
        </button>
        <span v-if="screenshots.length > 0" class="text-sm text-gray-400 ml-auto">
          {{ t('screenshotGallery.total', { count: screenshots.length }) }}
        </span>
      </div>

      <div class="flex-1 overflow-auto p-6" ref="scrollContainer" @scroll="handleScroll">
        <div v-if="screenshots.length === 0" class="text-center py-8 text-gray-500">
          {{ t('screenshotGallery.noScreenshots') }}
        </div>

        <template v-else>
          <!-- Grid View -->
          <div v-if="viewMode === 'grid'" class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            <div
              v-for="screenshot in paginatedScreenshots"
              :key="screenshot.id"
              @click="openScreenshot(screenshot)"
              class="bg-darker rounded-lg overflow-hidden border border-gray-700 cursor-pointer hover:border-primary transition-colors"
            >
              <div class="aspect-video relative bg-gray-800">
                <img
                  v-if="screenshot.thumbnail"
                  :src="screenshot.thumbnail"
                  :alt="String(screenshot.id)"
                  class="w-full h-full object-cover"
                />
                <div v-else class="w-full h-full flex items-center justify-center text-gray-500">
                  {{ t('screenshotGallery.loading') }}
                </div>
              </div>
              <div class="p-2">
                <p class="text-xs text-gray-500">{{ formatTimeShort(screenshot.timestamp) }}</p>
                <p class="text-xs text-gray-400 truncate">{{ parseContent(screenshot.content) }}</p>
                <!-- EXP-003: Reanalyze button for grid view -->
                <div class="mt-1 flex justify-end">
                  <button
                    @click.stop="reanalyzeRecord(screenshot)"
                    :disabled="reanalyzingIds.has(screenshot.id)"
                    class="px-2 py-0.5 text-xs rounded transition-colors"
                    :class="reanalyzingIds.has(screenshot.id)
                      ? 'bg-gray-600 text-gray-400 cursor-not-allowed'
                      : 'bg-primary/20 hover:bg-primary/30 text-primary'"
                  >
                    {{ reanalyzingIds.has(screenshot.id) ? t('screenshotModal.reanalyzing') : t('screenshotModal.reanalyze') }}
                  </button>
                </div>
              </div>
            </div>
          </div>

          <!-- List View -->
          <div v-else class="flex flex-col divide-y divide-gray-700">
            <div
              v-for="screenshot in paginatedScreenshots"
              :key="screenshot.id"
              @click="openScreenshot(screenshot)"
              class="flex items-center py-3 px-4 bg-darker rounded-lg mb-2 cursor-pointer hover:bg-gray-800 transition-colors"
            >
              <!-- Thumbnail -->
              <div class="w-24 h-16 flex-shrink-0 rounded overflow-hidden bg-gray-800 mr-4">
                <img
                  v-if="screenshot.thumbnail"
                  :src="screenshot.thumbnail"
                  :alt="String(screenshot.id)"
                  class="w-full h-full object-cover"
                />
              </div>
              <!-- Time -->
              <div class="w-20 flex-shrink-0">
                <span class="text-sm text-gray-400">{{ formatTimeShort(screenshot.timestamp) }}</span>
              </div>
              <!-- AI Summary -->
              <div class="flex-1 min-w-0">
                <p class="text-sm text-gray-300 truncate">{{ parseContent(screenshot.content) }}</p>
              </div>
              <!-- Actions -->
              <div class="flex-shrink-0 flex items-center gap-2">
                <!-- EXP-003: Reanalyze button for list view -->
                <button
                  @click.stop="reanalyzeRecord(screenshot)"
                  :disabled="reanalyzingIds.has(screenshot.id)"
                  class="px-2 py-1 text-xs rounded transition-colors"
                  :class="reanalyzingIds.has(screenshot.id)
                    ? 'bg-gray-600 text-gray-400 cursor-not-allowed'
                    : 'bg-primary/20 hover:bg-primary/30 text-primary'"
                >
                  {{ reanalyzingIds.has(screenshot.id) ? t('screenshotModal.reanalyzing') : t('screenshotModal.reanalyze') }}
                </button>
                <button class="text-xs text-gray-400 hover:text-primary transition-colors">
                  {{ t('screenshotGallery.view') }}
                </button>
              </div>
            </div>
          </div>

          <!-- Load More Section -->
          <div v-if="hasMorePages" class="text-center py-4 mt-4 border-t border-gray-700">
            <div v-if="isLoadingMore" class="text-gray-400 text-sm">
              <span class="animate-pulse">{{ t('screenshotGallery.loadingMore') }}</span>
            </div>
            <button
              v-else
              @click="loadMore"
              class="px-6 py-2 bg-gray-700 text-gray-300 rounded-lg hover:bg-gray-600 transition-colors"
            >
              {{ t('screenshotGallery.loadMore', { count: remainingCount }) }}
            </button>
          </div>
        </template>
      </div>
    </div>

    <!-- Screenshot detail modal -->
    <ScreenshotModal
      v-if="showDetail && selectedScreenshot"
      :record="selectedScreenshot"
      @close="showDetail = false"
      @updated="handleRecordUpdated"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import ScreenshotModal from './ScreenshotModal.vue'
import type { LogRecord } from '../types/tauri'
import { showToast } from '../stores/toast'

interface ScreenshotRecord extends LogRecord {
  thumbnail?: string
  thumbnailLoaded?: boolean
}

interface ScreenAnalysis {
  current_focus?: string
  active_software?: string
  context_keywords?: string[]
}

const { t } = useI18n()
const emit = defineEmits<{(e: 'close'): void}>()

const screenshots = ref<ScreenshotRecord[]>([])
const showDetail = ref(false)
const selectedScreenshot = ref<ScreenshotRecord | null>(null)
const viewMode = ref<'grid' | 'list'>('grid') // 'grid' or 'list'
const startDate = ref('')
const endDate = ref('')
const currentPage = ref(1)
const pageSize = 20
const isLoadingMore = ref(false)
const scrollContainer = ref<HTMLElement | null>(null)
const reanalyzingIds = ref(new Set<number>())

// Computed: paginated screenshots for AC4
const paginatedScreenshots = computed(() => {
  const end = currentPage.value * pageSize
  return screenshots.value.slice(0, end)
})

// Computed: has more pages to load
const hasMorePages = computed(() => {
  return currentPage.value * pageSize < screenshots.value.length
})

// Computed: remaining count for display
const remainingCount = computed(() => {
  return Math.max(0, screenshots.value.length - currentPage.value * pageSize)
})

const formatTimeShort = (timestamp: string) => {
  const date = new Date(timestamp)
  const hours = String(date.getHours()).padStart(2, '0')
  const minutes = String(date.getMinutes()).padStart(2, '0')
  const seconds = String(date.getSeconds()).padStart(2, '0')
  return `${hours}:${minutes}:${seconds}`
}

const parseContent = (content: string) => {
  try {
    const parsed = JSON.parse(content) as { current_focus?: string; active_software?: string }
    const text = parsed.current_focus || parsed.active_software || t('screenshotGallery.unknown')
    return text.length > 50 ? text.substring(0, 50) + '...' : text
  } catch {
    return content.length > 50 ? content.substring(0, 50) + '...' : content
  }
}

// UX-023: Lazy load thumbnails only for visible items
const loadThumbnailsForPage = async (page: number) => {
  const start = (page - 1) * pageSize
  const end = page * pageSize
  const records = screenshots.value.slice(start, end)

  for (const record of records) {
    // Skip if already loaded or loading
    if (record.thumbnailLoaded || record.thumbnail) continue

    try {
      const thumbnail = await invoke<string>('get_screenshot', { path: record.screenshot_path })
      record.thumbnail = thumbnail
      record.thumbnailLoaded = true
    } catch (err) {
      console.error('Failed to load thumbnail:', err)
    }
  }
}

// Legacy function for compatibility - now just marks as needing load
const loadThumbnails = async (records: ScreenshotRecord[]) => {
  // UX-023: Don't load all thumbnails upfront, just mark them
  // Thumbnails will be loaded lazily via loadThumbnailsForPage
  records.forEach(r => {
    r.thumbnailLoaded = false
  })
}

const loadScreenshots = async () => {
  try {
    const records = await invoke<LogRecord[]>('get_today_records')
    // Filter only auto records with screenshots
    const autoRecords = records.filter(r => r.source_type === 'auto' && r.screenshot_path) as ScreenshotRecord[]

    // UX-023: Don't load thumbnails upfront - they will be lazy loaded
    autoRecords.forEach(r => {
      r.thumbnailLoaded = false
    })

    screenshots.value = autoRecords
    currentPage.value = 1 // Reset pagination

    // Load thumbnails for first page
    await loadThumbnailsForPage(1)
  } catch (err) {
    console.error('Failed to load screenshots:', err)
  }
}

const applyFilter = async () => {
  if (!startDate.value || !endDate.value) {
    return
  }

  try {
    const records = await invoke<LogRecord[]>('get_records_by_date_range', {
      startDate: startDate.value,
      endDate: endDate.value
    })
    // Filter only auto records with screenshots
    const autoRecords = records.filter(r => r.source_type === 'auto' && r.screenshot_path) as ScreenshotRecord[]

    // UX-023: Don't load thumbnails upfront
    autoRecords.forEach(r => {
      r.thumbnailLoaded = false
    })

    screenshots.value = autoRecords
    currentPage.value = 1 // Reset pagination

    // Load thumbnails for first page
    await loadThumbnailsForPage(1)
  } catch (err) {
    console.error('Failed to filter screenshots:', err)
  }
}

const resetFilter = async () => {
  startDate.value = ''
  endDate.value = ''
  await loadScreenshots()
}

const loadMore = async () => {
  if (hasMorePages.value && !isLoadingMore.value) {
    isLoadingMore.value = true
    try {
      const nextPage = currentPage.value + 1
      // UX-023: Load thumbnails for new page before incrementing
      await loadThumbnailsForPage(nextPage)
      currentPage.value = nextPage
    } finally {
      isLoadingMore.value = false
    }
  }
}

const handleScroll = (event: Event) => {
  const target = event.target as HTMLElement
  const scrollBottom = target.scrollHeight - target.scrollTop - target.clientHeight

  // Load more when user scrolls to bottom (within 100px threshold)
  if (scrollBottom < 100 && hasMorePages.value && !isLoadingMore.value) {
    loadMore()
  }
}

const openScreenshot = async (screenshot: ScreenshotRecord) => {
  // UX-023: Ensure thumbnail is loaded when opening
  if (!screenshot.thumbnail && !screenshot.thumbnailLoaded) {
    try {
      const thumbnail = await invoke<string>('get_screenshot', { path: screenshot.screenshot_path })
      screenshot.thumbnail = thumbnail
      screenshot.thumbnailLoaded = true
    } catch (err) {
      console.error('Failed to load thumbnail:', err)
    }
  }
  selectedScreenshot.value = screenshot
  showDetail.value = true
}

// FEAT-001: Handle record updated from reanalysis
const handleRecordUpdated = (updatedRecord: LogRecord) => {
  // Update the record in the screenshots array
  const index = screenshots.value.findIndex(s => s.id === updatedRecord.id)
  if (index !== -1) {
    // Preserve the thumbnail
    const thumbnail = screenshots.value[index].thumbnail
    const thumbnailLoaded = screenshots.value[index].thumbnailLoaded
    screenshots.value[index] = { ...updatedRecord, thumbnail, thumbnailLoaded } as ScreenshotRecord
  }
  // Also update the selected screenshot
  if (selectedScreenshot.value && selectedScreenshot.value.id === updatedRecord.id) {
    const thumbnail = selectedScreenshot.value.thumbnail
    const thumbnailLoaded = selectedScreenshot.value.thumbnailLoaded
    selectedScreenshot.value = { ...updatedRecord, thumbnail, thumbnailLoaded } as ScreenshotRecord
  }
}

// EXP-003: Reanalyze a single record
const reanalyzeRecord = async (screenshot: ScreenshotRecord) => {
  if (reanalyzingIds.value.has(screenshot.id)) return

  reanalyzingIds.value.add(screenshot.id)
  try {
    const analysis = await invoke<ScreenAnalysis>('reanalyze_record', { recordId: screenshot.id })

    // Update the record content
    const updatedRecord: ScreenshotRecord = {
      ...screenshot,
      content: JSON.stringify(analysis)
    }

    // Update in the screenshots array
    const index = screenshots.value.findIndex(s => s.id === screenshot.id)
    if (index !== -1) {
      screenshots.value[index] = updatedRecord
    }

    showToast(t('screenshotModal.reanalyzeSuccess'), { type: 'success' })
  } catch (err) {
    const errorMsg = String(err)
    showToast(t('screenshotModal.reanalyzeFailed', { error: errorMsg }), { type: 'error' })
  } finally {
    reanalyzingIds.value.delete(screenshot.id)
  }
}

onMounted(() => {
  loadScreenshots()
})
</script>