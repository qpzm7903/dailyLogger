<template>
  <div class="h-screen bg-darker text-white flex flex-col">
    <header class="bg-dark border-b border-gray-700 px-6 py-4 flex items-center justify-between">
      <div class="flex items-center gap-3">
        <div class="w-8 h-8 bg-primary rounded-lg flex items-center justify-center">
          <span class="text-lg">📝</span>
        </div>
        <h1 class="text-xl font-semibold">DailyLogger</h1>
      </div>
      <div class="flex items-center gap-4">
        <span class="text-sm text-gray-400">{{ currentTime }}</span>
        <button @click="showLogViewer = true" class="flex items-center gap-1.5 px-3 py-1.5 text-xs text-gray-400 hover:text-white hover:bg-gray-700 rounded-lg transition-colors">
          🗒️ 日志
        </button>
        <button @click="showSettings = true" class="p-2 hover:bg-gray-700 rounded-lg transition-colors">
          ⚙️
        </button>
      </div>
    </header>

    <main class="flex-1 overflow-auto p-6">
      <div class="max-w-4xl mx-auto space-y-6">
        <div class="grid grid-cols-2 gap-4">
          <div class="bg-dark rounded-xl p-5 border border-gray-700">
            <div class="flex items-center gap-2 mb-3">
              <span class="text-2xl">🖥️</span>
              <h2 class="font-medium">自动感知</h2>
            </div>
            <p class="text-sm text-gray-400 mb-4">定时截取屏幕并分析工作上下文</p>
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <span :class="autoCaptureEnabled ? 'bg-green-400 animate-pulse' : 'bg-gray-500'" class="w-2 h-2 rounded-full inline-block"></span>
                <span class="text-xs text-gray-400">{{ autoCaptureEnabled ? '运行中' : '已停止' }}</span>
              </div>
              <div class="flex items-center gap-2">
                <button
                  @click="takeScreenshot"
                  :disabled="isCapturing"
                  class="px-3 py-1.5 text-xs bg-gray-600 hover:bg-gray-500 disabled:opacity-50 rounded-lg transition-colors"
                  title="截图查看，不做 AI 分析"
                >
                  {{ isCapturing ? '截图中…' : '📸 截图' }}
                </button>
                <button
                  @click="triggerCapture"
                  :disabled="isCapturing"
                  class="px-3 py-1.5 text-xs bg-gray-600 hover:bg-gray-500 disabled:opacity-50 rounded-lg transition-colors"
                  title="截图并进行 AI 分析，保存到记录"
                >
                  🤖 分析
                </button>
                <button
                  @click="toggleAutoCapture"
                  :class="autoCaptureEnabled ? 'bg-red-500 hover:bg-red-600' : 'bg-green-500 hover:bg-green-600'"
                  class="px-4 py-1.5 rounded-lg text-sm font-medium transition-colors"
                >
                  {{ autoCaptureEnabled ? '停止' : '启动' }}
                </button>
              </div>
            </div>
          </div>

          <div class="bg-dark rounded-xl p-5 border border-gray-700">
            <div class="flex items-center gap-2 mb-3">
              <span class="text-2xl">⚡</span>
              <h2 class="font-medium">闪念胶囊</h2>
            </div>
            <p class="text-sm text-gray-400 mb-4">快捷键: Alt + Space</p>
            <div class="flex items-center justify-between">
              <span class="text-xs text-gray-500">今日记录: {{ quickNotesCount }} 条</span>
              <button 
                @click="openQuickNote"
                class="bg-primary hover:bg-blue-600 px-4 py-1.5 rounded-lg text-sm font-medium transition-colors"
              >
                记录
              </button>
            </div>
          </div>
        </div>

        <div class="bg-dark rounded-xl p-5 border border-gray-700">
          <div class="flex items-center justify-between mb-4">
            <div class="flex items-center gap-2">
              <span class="text-2xl">📊</span>
              <h2 class="font-medium">今日工作流</h2>
              <button 
                v-if="screenshotCount > 0"
                @click="showScreenshotGallery = true"
                class="ml-2 text-xs text-primary hover:underline"
              >
                (📷 {{ screenshotCount }} 张截图)
              </button>
            </div>
            <button 
              @click="generateSummary"
              :disabled="isGenerating"
              class="bg-primary hover:bg-blue-600 disabled:opacity-50 px-4 py-1.5 rounded-lg text-sm font-medium transition-colors"
            >
              {{ isGenerating ? '生成中...' : '生成日报' }}
            </button>
          </div>
          <div v-if="todayRecords.length === 0" class="text-center py-8 text-gray-500">
            暂无记录
          </div>
          <div v-else class="space-y-3 max-h-80 overflow-y-auto pr-1">
            <div
              v-for="record in todayRecords"
              :key="record.id"
              @click="record.source_type === 'auto' && record.screenshot_path && openScreenshot(record)"
              :class="record.source_type === 'auto' && record.screenshot_path
                ? 'cursor-pointer hover:border-primary hover:bg-gray-800/40 group'
                : 'cursor-default'"
              class="bg-darker rounded-lg p-3 border border-gray-700 transition-colors"
            >
              <div class="flex items-center justify-between mb-1">
                <span class="text-xs text-gray-500">{{ formatTime(record.timestamp) }}</span>
                <div class="flex items-center gap-2">
                  <span
                    v-if="record.source_type === 'auto' && record.screenshot_path"
                    class="text-xs text-gray-600 group-hover:text-primary transition-colors"
                  >点击查看截图</span>
                  <span :class="record.source_type === 'auto' ? 'text-blue-400' : 'text-green-400'" class="text-xs">
                    {{ record.source_type === 'auto' ? '🖥️ 自动' : '⚡ 手动' }}
                  </span>
                </div>
              </div>
              <!-- Window Info Display (SMART-001 Task 6) -->
              <div
                v-if="getWindowInfo(record) && (getWindowInfo(record).title || getWindowInfo(record).process_name)"
                class="window-info flex items-center gap-1.5 mb-1.5 text-xs text-gray-400"
              >
                <span>{{ getWindowIcon(getWindowInfo(record)?.process_name) }}</span>
                <span class="truncate max-w-[200px]" :title="getWindowInfo(record)?.title">
                  {{ getWindowInfo(record)?.title || getWindowInfo(record)?.process_name }}
                </span>
              </div>
              <p class="text-sm text-gray-300 line-clamp-3">{{ record.content }}</p>
            </div>
          </div>
        </div>

        <div class="bg-dark rounded-xl p-5 border border-gray-700">
          <div class="flex items-center gap-2 mb-4">
            <span class="text-2xl">📁</span>
            <h2 class="font-medium">输出文件</h2>
          </div>
          <div v-if="summaryPath" class="bg-darker rounded-lg p-3 border border-gray-700">
            <p
              @click="showSummaryViewer = true"
              class="text-sm text-gray-300 cursor-pointer hover:text-primary hover:underline"
            >{{ summaryPath }}</p>
          </div>
          <div v-else class="text-center py-4 text-gray-500 text-sm">
            尚未生成日报
          </div>
        </div>
      </div>
    </main>

    <SettingsModal v-if="showSettings" @close="showSettings = false" />
    <QuickNoteModal v-if="showQuickNote" @close="showQuickNote = false" @save="handleQuickNote" />
    <ScreenshotModal v-if="showScreenshot" :record="selectedScreenshot" @close="showScreenshot = false" />
    <ScreenshotGallery v-if="showScreenshotGallery" @close="showScreenshotGallery = false" />
    <DailySummaryViewer v-if="showSummaryViewer" :summaryPath="summaryPath" @close="showSummaryViewer = false" />
    <LogViewer v-if="showLogViewer" @close="showLogViewer = false" />
    <Toast />
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import SettingsModal from './components/SettingsModal.vue'
import QuickNoteModal from './components/QuickNoteModal.vue'
import ScreenshotModal from './components/ScreenshotModal.vue'
import ScreenshotGallery from './components/ScreenshotGallery.vue'
import DailySummaryViewer from './components/DailySummaryViewer.vue'
import LogViewer from './components/LogViewer.vue'
import Toast from './components/Toast.vue'
import { showError, showSuccess } from './stores/toast.js'
import { parseError, getErrorMessage, getSuggestedAction, ErrorType } from './utils/errors.js'

const currentTime = ref('')
const autoCaptureEnabled = ref(false)
const quickNotesCount = ref(0)
const todayRecords = ref([])
const isGenerating = ref(false)
const isCapturing = ref(false)
const summaryPath = ref('')
const showSettings = ref(false)
const showQuickNote = ref(false)
const showScreenshot = ref(false)
const showScreenshotGallery = ref(false)
const showSummaryViewer = ref(false)
const showLogViewer = ref(false)
const selectedScreenshot = ref(null)

// Computed
const screenshotCount = computed(() => {
  return todayRecords.value.filter(r => r.source_type === 'auto' && r.screenshot_path).length
})

let timeInterval = null
let unlistenTrayOpenSettings = null
let unlistenTrayOpenQuickNote = null

const formatTime = (timestamp) => {
  const date = new Date(timestamp)
  if (isNaN(date.getTime())) return '--:--'
  // Use getHours/getMinutes (always local time) instead of toLocaleTimeString,
  // which can display UTC on some Windows WebView2 environments.
  const h = date.getHours().toString().padStart(2, '0')
  const m = date.getMinutes().toString().padStart(2, '0')
  return `${h}:${m}`
}

// SMART-001 Task 6: Helper to parse window info from record content
const getWindowInfo = (record) => {
  if (!record.content) return null
  try {
    const parsed = JSON.parse(record.content)
    return parsed.active_window || null
  } catch {
    return null
  }
}

// SMART-001 Task 6: Get icon based on process name
const getWindowIcon = (processName) => {
  if (!processName) return '🖥️'
  const name = processName.toLowerCase()

  // Common development tools
  if (name.includes('code') || name.includes('vscode')) return '💻'
  if (name.includes('idea') || name.includes('intellij')) return '☕'
  if (name.includes('atom') || name.includes('sublime')) return '📝'

  // Browsers
  if (name.includes('chrome')) return '🌐'
  if (name.includes('firefox')) return '🦊'
  if (name.includes('edge') || name.includes('msedge')) return '🌊'
  if (name.includes('safari')) return '🧭'

  // Communication
  if (name.includes('slack') || name.includes('discord') || name.includes('teams')) return '💬'
  if (name.includes('wechat') || name.includes('微信')) return '💬'

  // Terminal
  if (name.includes('terminal') || name.includes('cmd') || name.includes('bash') || name.includes('powershell')) return '⌨️'

  // Office
  if (name.includes('word') || name.includes('excel') || name.includes('powerpoint')) return '📊'

  // Default
  return '🖥️'
}

const updateTime = () => {
  currentTime.value = new Date().toLocaleString('zh-CN', { 
    month: '2-digit', 
    day: '2-digit', 
    hour: '2-digit', 
    minute: '2-digit' 
  })
}

const toggleAutoCapture = async () => {
  try {
    if (autoCaptureEnabled.value) {
      await invoke('stop_auto_capture')
    } else {
      await invoke('start_auto_capture')
    }
    autoCaptureEnabled.value = !autoCaptureEnabled.value
  } catch (err) {
    console.error('Failed to toggle auto capture:', err)
  }
}

const takeScreenshot = async () => {
  if (isCapturing.value) return
  isCapturing.value = true
  try {
    const path = await invoke('take_screenshot')
    selectedScreenshot.value = {
      screenshot_path: path,
      timestamp: new Date().toISOString(),
      content: null,
    }
    showScreenshot.value = true
  } catch (err) {
    console.error('Failed to take screenshot:', err)
    showError(err, takeScreenshot)
  } finally {
    isCapturing.value = false
  }
}

const triggerCapture = async () => {
  if (isCapturing.value) return
  isCapturing.value = true
  try {
    await invoke('trigger_capture')
    await loadTodayRecords()
    showSuccess('截图分析完成')
  } catch (err) {
    console.error('Failed to trigger capture:', err)
    showError(err, triggerCapture)
  } finally {
    isCapturing.value = false
  }
}

const openQuickNote = () => {
  showQuickNote.value = true
}

const openScreenshot = (record) => {
  selectedScreenshot.value = record
  showScreenshot.value = true
}

const handleQuickNote = async (content) => {
  try {
    await invoke('add_quick_note', { content })
    showQuickNote.value = false
    await loadTodayRecords()
  } catch (err) {
    console.error('Failed to save quick note:', err)
  }
}

const generateSummary = async () => {
  if (isGenerating.value) return
  isGenerating.value = true
  try {
    const result = await invoke('generate_daily_summary')
    summaryPath.value = result
    showSuccess('日报生成成功')
  } catch (err) {
    console.error('Failed to generate summary:', err)
    showError(err, generateSummary)
  } finally {
    isGenerating.value = false
  }
}

const loadTodayRecords = async () => {
  try {
    const records = await invoke('get_today_records')
    todayRecords.value = records
    quickNotesCount.value = records.filter(r => r.source_type === 'manual').length
  } catch (err) {
    console.error('Failed to load records:', err)
  }
}

const loadSettings = async () => {
  try {
    const settings = await invoke('get_settings')
    autoCaptureEnabled.value = settings.auto_capture_enabled || false
    summaryPath.value = settings.last_summary_path || ''
  } catch (err) {
    console.error('Failed to load settings:', err)
  }
}

onMounted(async () => {
  updateTime()
  timeInterval = setInterval(updateTime, 1000)

  // Auto-refresh records every 30 seconds
  setInterval(loadTodayRecords, 30000)

  // Listen for tray-open-settings event
  unlistenTrayOpenSettings = await listen('tray-open-settings', () => {
    showSettings.value = true
  })

  // Listen for tray-open-quick-note event
  unlistenTrayOpenQuickNote = await listen('tray-open-quick-note', () => {
    showQuickNote.value = true
  })

  await loadSettings()
  await loadTodayRecords()
})

onUnmounted(() => {
  if (timeInterval) clearInterval(timeInterval)
  if (unlistenTrayOpenSettings) unlistenTrayOpenSettings()
  if (unlistenTrayOpenQuickNote) unlistenTrayOpenQuickNote()
})
</script>
