<template>
  <div class="h-screen bg-darker text-white flex flex-col">
    <!-- UX-003: Offline status top banner -->
    <OfflineBanner :isOnline="isOnline" />

    <header
      :class="!isOnline ? 'mt-9' : ''"
      class="bg-dark border-b border-gray-700 px-6 py-4 flex items-center justify-between transition-[margin] duration-300"
    >
      <div class="flex items-center gap-3">
        <div class="w-8 h-8 bg-primary rounded-lg flex items-center justify-center">
          <span class="text-lg">📝</span>
        </div>
        <h1 class="text-xl font-semibold">DailyLogger</h1>
      </div>
      <div class="flex items-center gap-4">
        <div v-if="offlineQueueCount > 0" class="flex items-center gap-1.5 px-2.5 py-1 bg-yellow-500/20 text-yellow-400 rounded-full text-xs">
          <span class="w-2 h-2 rounded-full bg-yellow-400 inline-block"></span>
          {{ t('header.pendingSync', { count: offlineQueueCount }) }}
        </div>
        <span class="text-sm text-gray-400">{{ currentTime }}</span>
        <button @click="open('logViewer')" class="flex items-center gap-1.5 px-3 py-1.5 text-xs text-gray-400 hover:text-white hover:bg-gray-700 rounded-lg transition-colors">
          🗒️ {{ t('header.log') }}
        </button>
        <button @click="open('historyViewer')" class="flex items-center gap-1.5 px-3 py-1.5 text-xs text-gray-400 hover:text-white hover:bg-gray-700 rounded-lg transition-colors">
          📚 {{ t('header.history') }}
        </button>
        <button @click="open('search')" class="flex items-center gap-1.5 px-3 py-1.5 text-xs text-gray-400 hover:text-white hover:bg-gray-700 rounded-lg transition-colors">
          🔍 {{ t('header.search') }}
        </button>
        <button @click="open('tagCloud')" class="flex items-center gap-1.5 px-3 py-1.5 text-xs text-gray-400 hover:text-white hover:bg-gray-700 rounded-lg transition-colors">
          🏷️ {{ t('header.tags') }}
        </button>
        <button @click="open('export')" class="flex items-center gap-1.5 px-3 py-1.5 text-xs text-gray-400 hover:text-white hover:bg-gray-700 rounded-lg transition-colors">
          📤 {{ t('header.export') }}
        </button>
        <button @click="open('timeline')" class="flex items-center gap-1.5 px-3 py-1.5 text-xs text-gray-400 hover:text-white hover:bg-gray-700 rounded-lg transition-colors">
          📈 {{ t('header.timeline') }}
        </button>
        <button @click="open('backup')" class="flex items-center gap-1.5 px-3 py-1.5 text-xs text-gray-400 hover:text-white hover:bg-gray-700 rounded-lg transition-colors">
          💾 {{ t('header.backup') }}
        </button>
        <!-- Auth buttons -->
        <div v-if="currentUser" class="flex items-center gap-2">
          <span class="text-sm text-gray-400">{{ currentUser.username }}</span>
          <button @click="handleLogout" class="flex items-center gap-1.5 px-3 py-1.5 text-xs text-gray-400 hover:text-white hover:bg-gray-700 rounded-lg transition-colors">
            {{ t('auth.logout') }}
          </button>
        </div>
        <button v-else @click="open('loginModal')" class="flex items-center gap-1.5 px-3 py-1.5 text-xs text-gray-400 hover:text-white hover:bg-gray-700 rounded-lg transition-colors">
          {{ t('auth.login') }}
        </button>
        <button @click="open('settings')" class="p-2 hover:bg-gray-700 rounded-lg transition-colors">
          ⚙️
        </button>
      </div>
    </header>

    <main class="flex-1 overflow-auto p-6">
      <div class="max-w-4xl mx-auto space-y-6">
        <div :class="isDesktop ? 'grid grid-cols-2 gap-4' : ''">
          <div v-if="isDesktop" class="bg-dark rounded-xl p-5 border border-gray-700">
            <div class="flex items-center gap-2 mb-3">
              <span class="text-2xl">🖥️</span>
              <h2 class="font-medium">{{ t('autoCapture.title') }}</h2>
            </div>
            <p class="text-sm text-gray-400 mb-4">{{ t('autoCapture.description') }}</p>
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <span :class="autoCaptureEnabled ? 'bg-green-400 animate-pulse' : 'bg-gray-500'" class="w-2 h-2 rounded-full inline-block"></span>
                <span class="text-xs text-gray-400">{{ autoCaptureEnabled ? t('autoCapture.running') : t('autoCapture.stopped') }}</span>
              </div>
              <div class="flex items-center gap-2">
                <button
                  @click="takeScreenshot"
                  :disabled="isCapturing"
                  class="px-3 py-1.5 text-xs bg-gray-600 hover:bg-gray-500 disabled:opacity-50 rounded-lg transition-colors"
                  :title="t('autoCapture.screenshot')"
                >
                  {{ isCapturing ? t('autoCapture.screenshotting') : '📸 ' + t('autoCapture.screenshot') }}
                </button>
                <button
                  @click="triggerCapture"
                  :disabled="isCapturing"
                  class="px-3 py-1.5 text-xs bg-gray-600 hover:bg-gray-500 disabled:opacity-50 rounded-lg transition-colors"
                  :title="t('autoCapture.analyze')"
                >
                  🤖 {{ t('autoCapture.analyze') }}
                </button>
                <button
                  @click="toggleAutoCapture"
                  :class="autoCaptureEnabled ? 'bg-red-500 hover:bg-red-600' : 'bg-green-500 hover:bg-green-600'"
                  class="px-4 py-1.5 rounded-lg text-sm font-medium transition-colors"
                >
                  {{ autoCaptureEnabled ? t('autoCapture.stop') : t('autoCapture.start') }}
                </button>
              </div>
            </div>
          </div>

          <div class="bg-dark rounded-xl p-5 border border-gray-700">
            <div class="flex items-center gap-2 mb-3">
              <span class="text-2xl">⚡</span>
              <h2 class="font-medium">{{ t('quickNote.title') }}</h2>
            </div>
            <p class="text-sm text-gray-400 mb-4">{{ isDesktop ? t('quickNote.shortcut') : '' }}</p>
            <div class="flex items-center justify-between">
              <span class="text-xs text-gray-500">{{ t('quickNote.todayRecords', { count: quickNotesCount }) }}</span>
              <button
                @click="openQuickNote"
                class="bg-primary hover:bg-blue-600 px-4 py-1.5 rounded-lg text-sm font-medium transition-colors"
              >
                {{ t('quickNote.record') }}
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
                @click="open('screenshotGallery')"
                class="ml-2 text-xs text-primary hover:underline"
              >
                (📷 {{ screenshotCount }} 张截图)
              </button>
            </div>
            <div class="flex items-center gap-2">
              <ReportDropdown
                :isGeneratingDaily="isGenerating"
                :isGeneratingWeekly="isGeneratingWeekly"
                :isGeneratingMonthly="isGeneratingMonthly"
                @generate="handleReportGenerate"
              />
              <button
                @click="open('customReport')"
                class="bg-orange-600 hover:bg-orange-700 px-4 py-1.5 rounded-lg text-sm font-medium transition-colors"
              >
                自定义报告
              </button>
              <button
                @click="open('comparisonReport')"
                class="bg-teal-600 hover:bg-teal-700 px-4 py-1.5 rounded-lg text-sm font-medium transition-colors"
              >
                对比分析
              </button>
            </div>
          </div>
          <!-- AI-004: Tag filter | UX-005: Tag collapse -->
          <div v-if="tagEntries.length > 0" class="flex flex-wrap items-center gap-2 mb-4 pb-3 border-b border-gray-700">
            <button
              @click="selectedTagFilter = ''"
              :class="selectedTagFilter === '' ? 'bg-primary text-white' : 'bg-gray-700 text-gray-300 hover:bg-gray-600'"
              class="px-2.5 py-1 rounded-full text-xs transition-colors"
            >
              全部 ({{ todayRecords.length }})
            </button>
            <button
              v-for="([tag, count]) in visibleTagEntries"
              :key="tag"
              @click="selectedTagFilter = tag"
              :class="[
                getTagColor(tag),
                'px-2.5 py-1 rounded-full text-xs transition-colors',
                selectedTagFilter === tag ? 'ring-2 ring-primary ring-offset-1 ring-offset-dark' : ''
              ]"
            >
              {{ tag }} ({{ count }})
            </button>
            <!-- UX-005: Overflow expand button -->
            <button
              v-if="hiddenTagCount > 0 && !tagFilterExpanded"
              @click="tagFilterExpanded = true"
              :class="hasHiddenSelectedTag ? 'text-primary font-medium' : 'text-blue-400 hover:text-blue-300'"
              class="text-xs cursor-pointer whitespace-nowrap"
            >
              +{{ hiddenTagCount }} 个标签{{ hasHiddenSelectedTag ? ' (已选)' : '' }}
            </button>
            <!-- UX-005: Collapse button -->
            <button
              v-if="tagFilterExpanded && tagEntries.length > TAG_VISIBLE_THRESHOLD"
              @click="tagFilterExpanded = false"
              class="text-xs text-gray-400 hover:text-gray-300 cursor-pointer whitespace-nowrap"
            >
              收起
            </button>
          </div>
          <div v-if="filteredRecords.length === 0" class="text-center py-8 text-gray-500">
            {{ todayRecords.length === 0 ? '暂无记录' : '无匹配标签的记录' }}
          </div>
          <div v-else class="space-y-3 max-h-80 overflow-y-auto pr-1">
            <div
              v-for="record in filteredRecords"
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
                v-if="getWindowInfo(record)?.title || getWindowInfo(record)?.process_name"
                class="window-info flex items-center gap-1.5 mb-1.5 text-xs text-gray-400"
              >
                <span>{{ getWindowIcon(getWindowInfo(record)?.process_name) }}</span>
                <span class="truncate max-w-[200px]" :title="getWindowInfo(record)?.title">
                  {{ getWindowInfo(record)?.title || getWindowInfo(record)?.process_name }}
                </span>
              </div>
              <!-- UX-004: Show extracted summary for auto records, raw content for manual records -->
              <p v-if="record.source_type === 'auto'" class="text-sm text-gray-300 line-clamp-1 truncate">
                {{ extractSummary(record.content) || '分析完成' }}
              </p>
              <p v-else class="text-sm text-gray-300 line-clamp-3">{{ record.content }}</p>
              <!-- AI-004: Tag badges -->
              <div v-if="getRecordTags(record).length > 0" class="flex flex-wrap gap-1.5 mt-2">
                <span
                  v-for="tag in getRecordTags(record)"
                  :key="tag"
                  :class="getTagColor(tag)"
                  class="px-2 py-0.5 rounded-full text-xs"
                >
                  {{ tag }}
                </span>
              </div>
            </div>
          </div>
        </div>

        <div class="bg-dark rounded-xl p-5 border border-gray-700">
          <div class="flex items-center justify-between mb-4">
            <div class="flex items-center gap-2">
              <span class="text-2xl">📁</span>
              <h2 class="font-medium">输出文件</h2>
            </div>
            <button
              @click="open('reportHistory')"
              class="px-3 py-1.5 text-xs bg-gray-700 hover:bg-gray-600 rounded-lg text-gray-300 transition-colors"
            >
              {{ t('reportHistory.title') }}
            </button>
          </div>
          <div v-if="summaryPath" class="bg-darker rounded-lg p-3 border border-gray-700 mb-3">
            <p class="text-xs text-gray-500 mb-1">日报</p>
            <p
              @click="open('summaryViewer')"
              class="text-sm text-gray-300 cursor-pointer hover:text-primary hover:underline"
            >{{ summaryPath }}</p>
          </div>
          <div v-else class="text-center py-4 text-gray-500 text-sm">
            尚未生成日报
          </div>
          <div v-if="weeklyReportPath" class="bg-darker rounded-lg p-3 border border-gray-700">
            <p class="text-xs text-gray-500 mb-1">周报</p>
            <p
              @click="open('weeklyReportViewer')"
              class="text-sm text-gray-300 cursor-pointer hover:text-green-400 hover:underline"
            >{{ weeklyReportPath }}</p>
          </div>
          <div v-if="!weeklyReportPath && summaryPath" class="text-center py-2 text-gray-500 text-sm">
            尚未生成周报
          </div>
          <div v-if="monthlyReportPath" class="bg-darker rounded-lg p-3 border border-gray-700">
            <p class="text-xs text-gray-500 mb-1">月报</p>
            <p
              @click="open('monthlyReportViewer')"
              class="text-sm text-gray-300 cursor-pointer hover:text-purple-400 hover:underline"
            >{{ monthlyReportPath }}</p>
          </div>
          <div v-if="!monthlyReportPath && summaryPath" class="text-center py-2 text-gray-500 text-sm">
            尚未生成月报
          </div>
          <div v-if="customReportPath" class="bg-darker rounded-lg p-3 border border-gray-700">
            <p class="text-xs text-gray-500 mb-1">自定义报告</p>
            <p
              @click="open('customReportViewer')"
              class="text-sm text-gray-300 cursor-pointer hover:text-orange-400 hover:underline"
            >{{ customReportPath }}</p>
          </div>
          <div v-if="comparisonReportPath" class="bg-darker rounded-lg p-3 border border-gray-700">
            <p class="text-xs text-gray-500 mb-1">对比分析报告</p>
            <p
              @click="open('comparisonReportViewer')"
              class="text-sm text-gray-300 cursor-pointer hover:text-teal-400 hover:underline"
            >{{ comparisonReportPath }}</p>
          </div>
        </div>
      </div>
    </main>

    <SettingsModal v-if="isOpen('settings')" @close="close('settings')" />
    <BackupModal v-if="isOpen('backup')" @close="close('backup')" />
    <QuickNoteModal v-if="isOpen('quickNote')" @close="close('quickNote')" @save="handleQuickNote" />
    <ScreenshotModal v-if="isOpen('screenshot')" :record="selectedScreenshot!" @close="close('screenshot')" />
    <ScreenshotGallery v-if="isOpen('screenshotGallery')" @close="close('screenshotGallery')" />
    <DailySummaryViewer v-if="isOpen('summaryViewer')" :summaryPath="summaryPath!" @close="close('summaryViewer')" />
    <DailySummaryViewer v-if="isOpen('weeklyReportViewer')" :summaryPath="weeklyReportPath!" @close="close('weeklyReportViewer')" />
    <DailySummaryViewer v-if="isOpen('monthlyReportViewer')" :summaryPath="monthlyReportPath!" @close="close('monthlyReportViewer')" />
    <DailySummaryViewer v-if="isOpen('customReportViewer')" :summaryPath="customReportPath!" @close="close('customReportViewer')" />
    <CustomReportModal v-if="isOpen('customReport')" @close="close('customReport')" @generated="handleCustomReportGenerated" />
    <ReportComparisonModal v-if="isOpen('comparisonReport')" @close="close('comparisonReport')" @generated="handleComparisonReportGenerated" />
    <DailySummaryViewer v-if="isOpen('comparisonReportViewer')" :summaryPath="comparisonReportPath!" @close="close('comparisonReportViewer')" />
    <ReportHistoryViewer v-if="isOpen('reportHistory')" @close="close('reportHistory')" @viewFile="handleViewReportFile" />
    <LogViewer v-if="isOpen('logViewer')" @close="close('logViewer')" />
    <HistoryViewer v-if="isOpen('historyViewer')" :initialTag="initialFilterTag" :currentUser="currentUser" @close="close('historyViewer'); initialFilterTag = null" />
    <SearchPanel v-if="isOpen('search')" @close="close('search')" />
    <TagCloud v-if="isOpen('tagCloud')" @close="close('tagCloud')" @tagSelected="handleTagSelected" />
    <ExportModal v-if="isOpen('export')" @close="close('export')" />
    <TimelineVisualization
      v-if="isOpen('timeline')"
      @close="close('timeline')"
      @viewScreenshot="handleTimelineViewScreenshot"
    />
    <LoginModal v-if="isOpen('loginModal')" @close="close('loginModal')" @loggedIn="handleLogin" />
    <Toast />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, type Ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { usePlatform } from './composables/usePlatform'
import { useModal } from './composables/useModal'
import SettingsModal from './components/SettingsModal.vue'
import BackupModal from './components/BackupModal.vue'
import QuickNoteModal from './components/QuickNoteModal.vue'
import ScreenshotModal from './components/ScreenshotModal.vue'
import ScreenshotGallery from './components/ScreenshotGallery.vue'
import DailySummaryViewer from './components/DailySummaryViewer.vue'
import ReportHistoryViewer from './components/ReportHistoryViewer.vue'
import LogViewer from './components/LogViewer.vue'
import HistoryViewer from './components/HistoryViewer.vue'
import SearchPanel from './components/SearchPanel.vue'
import TagCloud from './components/TagCloud.vue'
import ExportModal from './components/ExportModal.vue'
import CustomReportModal from './components/CustomReportModal.vue'
import ReportComparisonModal from './components/ReportComparisonModal.vue'
import TimelineVisualization from './components/TimelineVisualization.vue'
import Toast from './components/Toast.vue'
import LoginModal from './components/LoginModal.vue'
import OfflineBanner from './components/OfflineBanner.vue'
import ReportDropdown from './components/ReportDropdown.vue'
import { showError, showSuccess, initToastI18n } from './stores/toast'
import { extractSummary } from './utils/contentUtils'
import type { LogRecord, Tag, User, Settings } from './types/tauri'

const { t } = useI18n()
const { isDesktop } = usePlatform()

const currentTime = ref('')
const isOnline = ref(true)
const offlineQueueCount = ref(0)
const autoCaptureEnabled = ref(false)
const quickNotesCount = ref(0)
const todayRecords = ref<LogRecord[]>([])
const isGenerating = ref(false)
const isGeneratingWeekly = ref(false)
const isGeneratingMonthly = ref(false)
const isCapturing = ref(false)
const summaryPath = ref('')
const weeklyReportPath = ref('')
const monthlyReportPath = ref('')

// UX-010: useModal for centralized modal management
const { isOpen, open, close } = useModal()

const customReportPath = ref('')
const comparisonReportPath = ref('')
const selectedScreenshot = ref<LogRecord | null>(null)
const initialFilterTag = ref<Tag | null>(null)
const currentUser = ref<User | null>(null)

// AI-004: Tag filtering state
const selectedTagFilter = ref('')
const allTags = ref<Tag[]>([])

// Computed
const screenshotCount = computed<number>(() => {
  return todayRecords.value.filter(r => r.source_type === 'auto' && r.screenshot_path).length
})

// UX-002: Global report generation lock - prevents concurrent AI API calls
const isAnyReportGenerating = computed<boolean>(() =>
  isGenerating.value || isGeneratingWeekly.value || isGeneratingMonthly.value
)

// AI-004: Computed filtered records based on selected tag
const filteredRecords = computed<LogRecord[]>(() => {
  if (!selectedTagFilter.value) {
    return todayRecords.value
  }
  return todayRecords.value.filter(record => {
    const tags = getRecordTags(record)
    return tags.includes(selectedTagFilter.value)
  })
})

// AI-004: Computed tag counts for filter display
const tagCounts = computed<Record<string, number>>(() => {
  const counts: Record<string, number> = {}
  todayRecords.value.forEach(record => {
    const tags = getRecordTags(record)
    tags.forEach(tag => {
      counts[tag] = (counts[tag] || 0) + 1
    })
  })
  return counts
})

let timeInterval: ReturnType<typeof setInterval> | null = null
let recordsRefreshInterval: ReturnType<typeof setInterval> | null = null
let unlistenTrayOpenSettings: UnlistenFn | null = null
let unlistenTrayOpenQuickNote: UnlistenFn | null = null
let unlistenNetworkStatus: UnlistenFn | null = null
let unlistenQueueUpdated: UnlistenFn | null = null
let networkCheckInterval: ReturnType<typeof setInterval> | null = null

const formatTime = (timestamp: string): string => {
  const date = new Date(timestamp)
  if (isNaN(date.getTime())) return '--:--'
  // Use getHours/getMinutes (always local time) instead of toLocaleTimeString,
  // which can display UTC on some Windows WebView2 environments.
  const h = date.getHours().toString().padStart(2, '0')
  const m = date.getMinutes().toString().padStart(2, '0')
  return `${h}:${m}`
}

// SMART-001 Task 6: Helper to parse window info from record content
interface WindowInfo {
  title?: string
  process_name?: string
}

interface ScreenAnalysis {
  current_focus?: string
  active_software?: string
  context_keywords?: string[]
  active_window?: WindowInfo
  tags?: string[]
}

const getWindowInfo = (record: LogRecord): WindowInfo | null => {
  if (!record.content) return null
  try {
    const parsed = JSON.parse(record.content) as ScreenAnalysis
    return parsed.active_window || null
  } catch {
    return null
  }
}

// SMART-001 Task 6: Get icon based on process name
const getWindowIcon = (processName?: string): string => {
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

// AI-004: Tag color mapping
const tagColors: Record<string, string> = {
  '开发': 'bg-blue-500/20 text-blue-400',
  '会议': 'bg-purple-500/20 text-purple-400',
  '写作': 'bg-green-500/20 text-green-400',
  '学习': 'bg-yellow-500/20 text-yellow-400',
  '研究': 'bg-cyan-500/20 text-cyan-400',
  '沟通': 'bg-orange-500/20 text-orange-400',
  '规划': 'bg-pink-500/20 text-pink-400',
  '文档': 'bg-indigo-500/20 text-indigo-400',
  '测试': 'bg-red-500/20 text-red-400',
  '设计': 'bg-teal-500/20 text-teal-400',
}
const defaultTagColor = 'bg-gray-500/20 text-gray-400'

// AI-004: Get tag color class
const getTagColor = (tag: string): string => {
  return tagColors[tag] || defaultTagColor
}

// AI-004: Parse tags from record
const getRecordTags = (record: LogRecord): string[] => {
  // First check if tags field exists (new records from AI-004)
  if ((record as LogRecord & { tags?: string }).tags) {
    try {
      const tags = JSON.parse((record as LogRecord & { tags?: string }).tags as string)
      if (Array.isArray(tags) && tags.length > 0) {
        return tags.slice(0, 3) // Limit to 3 tags
      }
    } catch {
      // Ignore parse errors
    }
  }
  // Fallback: try to parse tags from content (for auto records with ScreenAnalysis)
  if (record.source_type === 'auto' && record.content) {
    try {
      const parsed = JSON.parse(record.content) as ScreenAnalysis
      if (parsed.tags && Array.isArray(parsed.tags) && parsed.tags.length > 0) {
        return parsed.tags.slice(0, 3)
      }
    } catch {
      // Ignore parse errors
    }
  }
  return []
}

// UX-005: Tag filter collapse state
const TAG_VISIBLE_THRESHOLD = 6
const tagFilterExpanded = ref(false)

const tagEntries = computed<[string, number][]>(() => Object.entries(tagCounts.value) as [string, number][])

const visibleTagEntries = computed<[string, number][]>(() => {
  if (tagFilterExpanded.value || tagEntries.value.length <= TAG_VISIBLE_THRESHOLD) {
    return tagEntries.value
  }
  return tagEntries.value.slice(0, TAG_VISIBLE_THRESHOLD)
})

const hiddenTagCount = computed<number>(() => {
  if (tagEntries.value.length <= TAG_VISIBLE_THRESHOLD) return 0
  return tagEntries.value.length - TAG_VISIBLE_THRESHOLD
})

const hasHiddenSelectedTag = computed<boolean>(() => {
  if (!selectedTagFilter.value || tagFilterExpanded.value) return false
  const visibleTagNames = visibleTagEntries.value.map(([tag]) => tag)
  return !visibleTagNames.includes(selectedTagFilter.value)
})

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
    const path = await invoke<string>('take_screenshot')
    selectedScreenshot.value = {
      id: 0,
      screenshot_path: path,
      timestamp: new Date().toISOString(),
      content: '',
      source_type: 'auto'
    }
    open('screenshot')
  } catch (err) {
    console.error('Failed to take screenshot:', err)
    showError(String(err), takeScreenshot)
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
    showError(String(err), triggerCapture)
  } finally {
    isCapturing.value = false
  }
}

const openQuickNote = () => {
  open('quickNote')
}

const openScreenshot = (record: LogRecord) => {
  selectedScreenshot.value = record
  open('screenshot')
}

const handleTimelineViewScreenshot = (record: LogRecord) => {
  close('timeline')
  selectedScreenshot.value = record
  open('screenshot')
}

const handleQuickNote = async (content: string) => {
  try {
    await invoke('add_quick_note', { content })
    close('quickNote')
    await loadTodayRecords()
    showSuccess(t('quickNote.savedSuccess'))
  } catch (err) {
    console.error('Failed to save quick note:', err)
    showError(String(err))
  }
}

// Handle tag selection from TagCloud
const handleTagSelected = (tag: Tag | null) => {
  close('tagCloud')
  initialFilterTag.value = tag
  open('historyViewer')
}

// UX-011: Handle report generation from dropdown
const handleReportGenerate = (type: 'daily' | 'weekly' | 'monthly') => {
  if (type === 'daily') {
    generateSummary()
  } else if (type === 'weekly') {
    generateWeeklyReport()
  } else if (type === 'monthly') {
    generateMonthlyReport()
  }
}

const generateSummary = async () => {
  if (isGenerating.value) return
  isGenerating.value = true
  try {
    const result = await invoke<string>('generate_daily_summary')
    summaryPath.value = result
    showSuccess('日报生成成功')
  } catch (err) {
    console.error('Failed to generate summary:', err)
    showError(String(err), generateSummary)
  } finally {
    isGenerating.value = false
  }
}

const generateWeeklyReport = async () => {
  if (isGeneratingWeekly.value) return
  isGeneratingWeekly.value = true
  try {
    const result = await invoke<string>('generate_weekly_report')
    weeklyReportPath.value = result
    showSuccess('周报生成成功')
  } catch (err) {
    console.error('Failed to generate weekly report:', err)
    showError(String(err), generateWeeklyReport)
  } finally {
    isGeneratingWeekly.value = false
  }
}

const generateMonthlyReport = async () => {
  if (isGeneratingMonthly.value) return
  isGeneratingMonthly.value = true
  try {
    const result = await invoke<string>('generate_monthly_report')
    monthlyReportPath.value = result
    showSuccess('月报生成成功')
  } catch (err) {
    console.error('Failed to generate monthly report:', err)
    showError(String(err), generateMonthlyReport)
  } finally {
    isGeneratingMonthly.value = false
  }
}

const handleCustomReportGenerated = (path: string) => {
  customReportPath.value = path
}

const handleComparisonReportGenerated = (path: string) => {
  comparisonReportPath.value = path
}

// FIX-007: Handle viewing historical report file
const handleViewReportFile = (path: string) => {
  summaryPath.value = path
  open('summaryViewer')
}

const loadTodayRecords = async () => {
  try {
    const records = await invoke<LogRecord[]>('get_today_records')
    todayRecords.value = records
    quickNotesCount.value = records.filter(r => r.source_type === 'manual').length

    // CORE-007: Check network status and queue
    try {
      const status = await invoke<boolean>('check_network_status')
      isOnline.value = status

      const queueStatus = await invoke<{ pending_count: number }>('get_offline_queue_status')
      offlineQueueCount.value = queueStatus.pending_count
    } catch (e) {
      console.error('Failed to check network status:', e)
      isOnline.value = false
    }
  } catch (err) {
    console.error('Failed to load records:', err)
  }
}

const loadSettings = async () => {
  try {
    const settings = await invoke<Settings>('get_settings')
    autoCaptureEnabled.value = settings.auto_capture_enabled || false
    summaryPath.value = settings.last_summary_path || ''
    weeklyReportPath.value = (settings as Settings & { last_weekly_report_path?: string }).last_weekly_report_path || ''
    monthlyReportPath.value = (settings as Settings & { last_monthly_report_path?: string }).last_monthly_report_path || ''
    customReportPath.value = (settings as Settings & { last_custom_report_path?: string }).last_custom_report_path || ''
  } catch (err) {
    console.error('Failed to load settings:', err)
  }
}

// Auth functions
interface Session {
  user_id: number
  username: string
}

const loadSession = async () => {
  try {
    const session = await invoke<Session | null>('get_current_session')
    if (session) {
      currentUser.value = { id: session.user_id, username: session.username, email: '', created_at: '' } as User
    }
  } catch (err) {
    console.error('Failed to load session:', err)
  }
}

const handleLogin = (user: { id: string | number; username: string }) => {
  currentUser.value = {
    id: typeof user.id === 'string' ? parseInt(user.id) : user.id,
    username: user.username,
    email: '',
    created_at: ''
  } as User
  showSuccess(t('auth.welcomeBack', { username: user.username }))
}

const handleLogout = async () => {
  try {
    await invoke('logout')
    currentUser.value = null
    showSuccess(t('auth.loggedOut'))
  } catch (err) {
    console.error('Failed to logout:', err)
    showError(String(err))
  }
}

onMounted(async () => {
  // Initialize toast i18n for error messages
  initToastI18n(useI18n())

  updateTime()
  timeInterval = setInterval(updateTime, 1000)

  // Auto-refresh records every 30 seconds
  recordsRefreshInterval = setInterval(loadTodayRecords, 30000)

  // CORE-007: Network status monitoring
  try {
    isOnline.value = await invoke<boolean>('get_network_status')
  } catch { /* ignore */ }

  // CORE-007: Load initial queue status
  try {
    const queueStatus = await invoke<{ pending_count: number }>('get_offline_queue_status')
    offlineQueueCount.value = queueStatus.pending_count || 0
  } catch { /* ignore */ }

  unlistenNetworkStatus = await listen<boolean>('network-status-changed', (event) => {
    isOnline.value = event.payload
  })

  // Listen for offline queue updates
  unlistenQueueUpdated = await listen<{ pending_count: number }>('offline-queue-updated', (event) => {
    offlineQueueCount.value = event.payload?.pending_count || 0
  })

  // Also poll network status every 60s as a fallback
  networkCheckInterval = setInterval(async () => {
    try {
      isOnline.value = await invoke<boolean>('check_network_status')
      const queueStatus = await invoke<{ pending_count: number }>('get_offline_queue_status')
      offlineQueueCount.value = queueStatus.pending_count || 0
    } catch { /* ignore */ }
  }, 60000)

  // Listen for tray-open-settings event
  unlistenTrayOpenSettings = await listen('tray-open-settings', () => {
    open('settings')
  })

  // Listen for tray-open-quick-note event
  unlistenTrayOpenQuickNote = await listen('tray-open-quick-note', () => {
    open('quickNote')
  })

  await loadSettings()
  await loadTodayRecords()
  await loadSession()
})

onUnmounted(() => {
  if (timeInterval) clearInterval(timeInterval)
  if (recordsRefreshInterval) clearInterval(recordsRefreshInterval)
  if (networkCheckInterval) clearInterval(networkCheckInterval)
  if (unlistenTrayOpenSettings) unlistenTrayOpenSettings()
  if (unlistenTrayOpenQuickNote) unlistenTrayOpenQuickNote()
  if (unlistenNetworkStatus) unlistenNetworkStatus()
  if (unlistenQueueUpdated) unlistenQueueUpdated()
})
</script>
