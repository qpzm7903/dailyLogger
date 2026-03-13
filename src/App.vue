<template>
  <div class="h-screen bg-darker text-white flex flex-col">
    <header class="bg-dark/80 backdrop-blur-sm border-b border-gray-700 px-6 py-4 flex items-center justify-between sticky top-0 z-10">
      <div class="flex items-center gap-3">
        <div class="w-9 h-9 bg-gradient-to-br from-primary to-blue-600 rounded-xl flex items-center justify-center shadow-lg shadow-primary/20">
          <span class="text-lg">📝</span>
        </div>
        <div>
          <h1 class="text-xl font-semibold tracking-tight">DailyLogger</h1>
          <p class="text-xs text-gray-500">AI 驱动的工作流记忆</p>
        </div>
      </div>
      <div class="flex items-center gap-3">
        <span class="text-xs text-gray-500 bg-darker/50 px-3 py-1.5 rounded-lg border border-gray-700/50">{{ currentTime }}</span>
        <button @click="showLogViewer = true" class="flex items-center gap-2 px-3 py-2 text-xs text-gray-400 hover:text-white hover:bg-gray-700/50 rounded-lg transition-all border border-transparent hover:border-gray-600">
          <span>🗒️</span> 日志
        </button>
        <button @click="showSettings = true" class="p-2 hover:bg-gray-700/50 rounded-lg transition-colors text-gray-400 hover:text-white" title="设置">
          ⚙️
        </button>
      </div>
    </header>

    <main class="flex-1 overflow-auto p-6">
      <div class="max-w-5xl mx-auto space-y-6">
        <!-- Top Cards -->
        <div class="grid grid-cols-1 md:grid-cols-2 gap-5">
          <!-- Auto Perception Card -->
          <div class="bg-dark/80 backdrop-blur rounded-xl p-5 border border-gray-700 hover:border-gray-600 transition-all shadow-lg">
            <div class="flex items-center gap-2.5 mb-3">
              <div class="w-8 h-8 bg-gradient-to-br from-purple-500/20 to-blue-500/20 rounded-lg flex items-center justify-center">
                <span class="text-xl">🖥️</span>
              </div>
              <h2 class="font-medium text-base">自动感知</h2>
            </div>
            <p class="text-sm text-gray-400 mb-4 h-10">定时截取屏幕并 AI 分析工作上下文</p>
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <span :class="autoCaptureEnabled ? 'bg-green-400 animate-pulse shadow-[0_0_8px_rgba(74,222,128,0.5)]' : 'bg-gray-500'" class="w-2.5 h-2.5 rounded-full inline-block"></span>
                <span class="text-xs" :class="autoCaptureEnabled ? 'text-green-400' : 'text-gray-400'">{{ autoCaptureEnabled ? '运行中' : '已停止' }}</span>
              </div>
              <div class="flex items-center gap-1.5">
                <button
                  @click="takeScreenshot"
                  :disabled="isCapturing"
                  class="px-2.5 py-1.5 text-xs bg-gray-700/80 hover:bg-gray-600 disabled:opacity-50 rounded-lg transition-all border border-gray-600 hover:border-gray-500"
                  title="截图查看，不做 AI 分析"
                >
                  <span class="hidden lg:inline">{{ isCapturing ? '截图中…' : '📸 截图' }}</span>
                  <span class="lg:hidden">📸</span>
                </button>
                <button
                  @click="triggerCapture"
                  :disabled="isCapturing"
                  class="px-2.5 py-1.5 text-xs bg-gradient-to-r from-primary/80 to-blue-600 hover:from-primary hover:to-blue-500 disabled:opacity-50 rounded-lg transition-all shadow-md hover:shadow-lg"
                  title="截图并进行 AI 分析，保存到记录"
                >
                  🤖 分析
                </button>
                <button
                  @click="toggleAutoCapture"
                  :class="autoCaptureEnabled ? 'bg-red-500/90 hover:bg-red-500' : 'bg-green-500/90 hover:bg-green-500'"
                  class="px-3 py-1.5 rounded-lg text-xs font-medium transition-all shadow-md hover:shadow-lg min-w-[60px]"
                >
                  {{ autoCaptureEnabled ? '停止' : '启动' }}
                </button>
              </div>
            </div>
            <div v-if="captureError" class="mt-2 bg-red-900/20 border border-red-700/50 rounded-lg px-3 py-2 flex items-start justify-between gap-2">
              <p class="text-xs text-red-400">{{ captureError }}</p>
              <button @click="captureError = ''" class="text-red-500 hover:text-red-300 text-xs flex-shrink-0 p-0.5">✕</button>
            </div>
          </div>

          <!-- Quick Note Card -->
          <div class="bg-dark/80 backdrop-blur rounded-xl p-5 border border-gray-700 hover:border-gray-600 transition-all shadow-lg">
            <div class="flex items-center gap-2.5 mb-3">
              <div class="w-8 h-8 bg-gradient-to-br from-amber-500/20 to-orange-500/20 rounded-lg flex items-center justify-center">
                <span class="text-xl">⚡</span>
              </div>
              <h2 class="font-medium text-base">闪念胶囊</h2>
            </div>
            <p class="text-sm text-gray-400 mb-4 h-10">快捷键 <kbd class="px-1.5 py-0.5 bg-darker rounded border border-gray-600 text-xs">Alt+Space</kbd> 随时记录</p>
            <div class="flex items-center justify-between">
              <span class="text-xs text-gray-500">
                <span class="inline-block w-2 h-2 rounded-full bg-primary mr-1.5"></span>
                今日记录：<span class="text-white font-medium">{{ quickNotesCount }}</span> 条
              </span>
              <button
                @click="openQuickNote"
                class="bg-gradient-to-r from-primary/90 to-blue-600 hover:from-primary hover:to-blue-500 px-4 py-1.5 rounded-lg text-sm font-medium transition-all shadow-md hover:shadow-lg hover:shadow-primary/20"
              >
                ✏️ 记录
              </button>
            </div>
          </div>
        </div>

        <!-- Today's Workflow -->
        <div class="bg-dark/80 backdrop-blur rounded-xl p-5 border border-gray-700 hover:border-gray-600 transition-all shadow-lg">
          <div class="flex items-center justify-between mb-4">
            <div class="flex items-center gap-2">
              <div class="w-8 h-8 bg-gradient-to-br from-green-500/20 to-emerald-500/20 rounded-lg flex items-center justify-center">
                <span class="text-xl">📊</span>
              </div>
              <h2 class="font-medium text-base">今日工作流</h2>
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
              class="bg-gradient-to-r from-primary/90 to-blue-600 hover:from-primary hover:to-blue-500 disabled:opacity-50 px-4 py-1.5 rounded-lg text-sm font-medium transition-all shadow-md hover:shadow-lg"
            >
              {{ isGenerating ? '生成中...' : '📄 生成日报' }}
            </button>
          </div>
          <div v-if="todayRecords.length === 0" class="text-center py-12">
            <div class="text-5xl mb-4">📭</div>
            <p class="text-gray-500 text-sm">暂无记录</p>
            <p class="text-gray-600 text-xs mt-1">启动自动感知或记录闪念开始使用</p>
          </div>
          <div v-else class="space-y-2 max-h-96 overflow-y-auto pr-1 custom-scrollbar">
            <div
              v-for="record in todayRecords"
              :key="record.id"
              @click="record.source_type === 'auto' && record.screenshot_path && openScreenshot(record)"
              :class="record.source_type === 'auto' && record.screenshot_path
                ? 'cursor-pointer hover:border-primary/50 hover:bg-gray-800/50 group'
                : 'cursor-default'"
              class="bg-darker/50 rounded-lg p-3 border border-gray-700/50 transition-all"
            >
              <div class="flex items-center justify-between mb-1.5">
                <span class="text-xs text-gray-500 font-mono">{{ formatTime(record.timestamp) }}</span>
                <div class="flex items-center gap-2">
                  <span
                    v-if="record.source_type === 'auto' && record.screenshot_path"
                    class="text-xs text-gray-500 group-hover:text-primary transition-colors"
                  >👁️ 点击查看</span>
                  <span :class="record.source_type === 'auto' ? 'text-blue-400' : 'text-green-400'" class="text-xs font-medium">
                    {{ record.source_type === 'auto' ? '🖥️ 自动' : '⚡ 手动' }}
                  </span>
                </div>
              </div>
              <p class="text-sm text-gray-300 line-clamp-3 leading-relaxed">{{ record.content }}</p>
            </div>
          </div>
        </div>

        <!-- Output Files -->
        <div class="bg-dark/80 backdrop-blur rounded-xl p-5 border border-gray-700 hover:border-gray-600 transition-all shadow-lg">
          <div class="flex items-center gap-2 mb-4">
            <div class="w-8 h-8 bg-gradient-to-br from-pink-500/20 to-rose-500/20 rounded-lg flex items-center justify-center">
              <span class="text-xl">📁</span>
            </div>
            <h2 class="font-medium text-base">输出文件</h2>
          </div>
          <div v-if="summaryError" class="bg-red-900/20 border border-red-700/50 rounded-lg p-3 mb-2">
            <p class="text-sm text-red-400">生成失败：{{ summaryError }}</p>
          </div>
          <div v-if="summaryPath" class="bg-darker/50 rounded-lg p-3 border border-gray-700/50 hover:border-primary/50 transition-colors">
            <p
              @click="showSummaryViewer = true"
              class="text-sm text-gray-300 cursor-pointer hover:text-primary hover:underline flex items-center gap-2"
            >
              <span>📄</span> {{ summaryPath }}
              <span class="text-xs text-gray-500 ml-auto">点击预览</span>
            </p>
          </div>
          <div v-else-if="!summaryError" class="text-center py-8">
            <div class="text-4xl mb-3">📝</div>
            <p class="text-gray-500 text-sm">尚未生成日报</p>
            <p class="text-gray-600 text-xs mt-1">点击"生成日报"按钮创建今日工作总结</p>
          </div>
        </div>
      </div>
    </main>

    <!-- Modals -->
    <SettingsModal v-if="showSettings" @close="showSettings = false" />
    <QuickNoteModal v-if="showQuickNote" @close="showQuickNote = false" @save="handleQuickNote" />
    <ScreenshotModal v-if="showScreenshot" :record="selectedScreenshot" @close="showScreenshot = false" />
    <ScreenshotGallery v-if="showScreenshotGallery" @close="showScreenshotGallery = false" />
    <DailySummaryViewer v-if="showSummaryViewer" :summaryPath="summaryPath" @close="showSummaryViewer = false" />
    <LogViewer v-if="showLogViewer" @close="showLogViewer = false" />
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

const currentTime = ref('')
const autoCaptureEnabled = ref(false)
const quickNotesCount = ref(0)
const todayRecords = ref([])
const isGenerating = ref(false)
const isCapturing = ref(false)
const summaryPath = ref('')
const summaryError = ref('')
const captureError = ref('')
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

const formatTime = (timestamp) => {
  const date = new Date(timestamp)
  if (isNaN(date.getTime())) return '--:--'
  const h = date.getHours().toString().padStart(2, '0')
  const m = date.getMinutes().toString().padStart(2, '0')
  return `${h}:${m}`
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
  captureError.value = ''
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
    captureError.value = `截图失败：${err}`
  } finally {
    isCapturing.value = false
  }
}

const triggerCapture = async () => {
  if (isCapturing.value) return
  isCapturing.value = true
  captureError.value = ''
  try {
    await invoke('trigger_capture')
    await loadTodayRecords()
  } catch (err) {
    console.error('Failed to trigger capture:', err)
    captureError.value = `分析失败：${err}`
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
  summaryError.value = ''
  try {
    const result = await invoke('generate_daily_summary')
    summaryPath.value = result
  } catch (err) {
    console.error('Failed to generate summary:', err)
    summaryError.value = String(err)
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

  await loadSettings()
  await loadTodayRecords()
})

onUnmounted(() => {
  if (timeInterval) clearInterval(timeInterval)
})
</script>

<style>
.custom-scrollbar::-webkit-scrollbar {
  width: 6px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: rgba(30, 41, 59, 0.5);
  border-radius: 3px;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(71, 85, 105, 0.8);
  border-radius: 3px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(99, 102, 241, 0.6);
}
</style>
