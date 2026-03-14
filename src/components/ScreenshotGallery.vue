<template>
  <div class="fixed inset-0 bg-black/80 flex items-center justify-center z-50" @click.self="$emit('close')">
    <div class="bg-dark rounded-2xl w-[90vw] h-[90vh] max-w-6xl overflow-hidden border border-gray-700 flex flex-col">
      <div class="px-6 py-4 border-b border-gray-700 flex items-center justify-between">
        <h2 class="text-lg font-semibold">📷 截图画廊</h2>
        <div class="flex items-center gap-2">
          <!-- View toggle buttons -->
          <button
            @click="viewMode = 'grid'"
            :class="viewMode === 'grid' ? 'bg-primary text-white' : 'bg-darker text-gray-400 hover:text-white'"
            class="px-3 py-1.5 rounded-lg text-sm transition-colors"
            aria-label="网格视图"
          >
            网格
          </button>
          <button
            @click="viewMode = 'list'"
            :class="viewMode === 'list' ? 'bg-primary text-white' : 'bg-darker text-gray-400 hover:text-white'"
            class="px-3 py-1.5 rounded-lg text-sm transition-colors"
            aria-label="列表视图"
          >
            列表
          </button>
          <button @click="$emit('close')" class="text-gray-400 hover:text-white ml-2">✕</button>
        </div>
      </div>

      <!-- Date Filter Section -->
      <div class="px-6 py-3 border-b border-gray-700 flex items-center gap-4 flex-wrap">
        <div class="flex items-center gap-2">
          <label class="text-sm text-gray-400">开始日期:</label>
          <input
            type="date"
            v-model="startDate"
            class="bg-darker border border-gray-600 rounded px-2 py-1 text-sm text-white focus:border-primary focus:outline-none"
          />
        </div>
        <div class="flex items-center gap-2">
          <label class="text-sm text-gray-400">结束日期:</label>
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
          筛选
        </button>
        <button
          @click="resetFilter"
          class="px-4 py-1 bg-gray-600 text-white rounded text-sm hover:bg-gray-500 transition-colors"
        >
          重置
        </button>
        <span v-if="screenshots.length > 0" class="text-sm text-gray-400 ml-auto">
          共 {{ screenshots.length }} 条
        </span>
      </div>

      <div class="flex-1 overflow-auto p-6">
        <div v-if="screenshots.length === 0" class="text-center py-8 text-gray-500">
          暂无截图记录
        </div>

        <!-- Grid View -->
        <div v-else-if="viewMode === 'grid'" class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          <div
            v-for="screenshot in screenshots"
            :key="screenshot.id"
            @click="openScreenshot(screenshot)"
            class="bg-darker rounded-lg overflow-hidden border border-gray-700 cursor-pointer hover:border-primary transition-colors"
          >
            <div class="aspect-video relative bg-gray-800">
              <img
                v-if="screenshot.thumbnail"
                :src="screenshot.thumbnail"
                :alt="screenshot.id"
                class="w-full h-full object-cover"
              />
              <div v-else class="w-full h-full flex items-center justify-center text-gray-500">
                加载中...
              </div>
            </div>
            <div class="p-2">
              <p class="text-xs text-gray-500">{{ formatTime(screenshot.timestamp) }}</p>
              <p class="text-xs text-gray-400 truncate">{{ parseContent(screenshot.content) }}</p>
            </div>
          </div>
        </div>

        <!-- List View -->
        <div v-else class="flex flex-col divide-y divide-gray-700">
          <div
            v-for="screenshot in screenshots"
            :key="screenshot.id"
            @click="openScreenshot(screenshot)"
            class="flex items-center py-3 px-4 bg-darker rounded-lg mb-2 cursor-pointer hover:bg-gray-800 transition-colors"
          >
            <!-- Thumbnail -->
            <div class="w-24 h-16 flex-shrink-0 rounded overflow-hidden bg-gray-800 mr-4">
              <img
                v-if="screenshot.thumbnail"
                :src="screenshot.thumbnail"
                :alt="screenshot.id"
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
            <!-- Action -->
            <div class="flex-shrink-0 ml-4">
              <span class="text-xs text-primary hover:underline">查看</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Screenshot detail modal -->
    <ScreenshotModal v-if="showDetail" :record="selectedScreenshot" @close="showDetail = false" />
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import ScreenshotModal from './ScreenshotModal.vue'

const emit = defineEmits(['close'])

const screenshots = ref([])
const showDetail = ref(false)
const selectedScreenshot = ref(null)
const viewMode = ref('grid') // 'grid' or 'list'
const startDate = ref('')
const endDate = ref('')

const formatTime = (timestamp) => {
  const date = new Date(timestamp)
  return date.toLocaleString('zh-CN')
}

const formatTimeShort = (timestamp) => {
  const date = new Date(timestamp)
  const hours = String(date.getHours()).padStart(2, '0')
  const minutes = String(date.getMinutes()).padStart(2, '0')
  const seconds = String(date.getSeconds()).padStart(2, '0')
  return `${hours}:${minutes}:${seconds}`
}

const parseContent = (content) => {
  try {
    const parsed = JSON.parse(content)
    return parsed.current_focus || parsed.active_software || '未知'
  } catch {
    return content.substring(0, 30)
  }
}

const loadThumbnails = async (records) => {
  for (const record of records) {
    try {
      const thumbnail = await invoke('get_screenshot', { path: record.screenshot_path })
      record.thumbnail = thumbnail
    } catch (err) {
      console.error('Failed to load thumbnail:', err)
    }
  }
}

const loadScreenshots = async () => {
  try {
    const records = await invoke('get_today_records')
    // Filter only auto records with screenshots
    const autoRecords = records.filter(r => r.source_type === 'auto' && r.screenshot_path)

    // Load thumbnails
    await loadThumbnails(autoRecords)

    screenshots.value = autoRecords
  } catch (err) {
    console.error('Failed to load screenshots:', err)
  }
}

const applyFilter = async () => {
  if (!startDate.value || !endDate.value) {
    return
  }

  try {
    const records = await invoke('get_records_by_date_range', {
      startDate: startDate.value,
      endDate: endDate.value
    })
    // Filter only auto records with screenshots
    const autoRecords = records.filter(r => r.source_type === 'auto' && r.screenshot_path)

    // Load thumbnails
    await loadThumbnails(autoRecords)

    screenshots.value = autoRecords
  } catch (err) {
    console.error('Failed to filter screenshots:', err)
  }
}

const resetFilter = async () => {
  startDate.value = ''
  endDate.value = ''
  await loadScreenshots()
}

const openScreenshot = (screenshot) => {
  selectedScreenshot.value = screenshot
  showDetail.value = true
}

onMounted(() => {
  loadScreenshots()
})
</script>