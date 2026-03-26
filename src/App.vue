<template>
  <div class="h-screen bg-[var(--color-surface-0)] text-[var(--color-text-primary)] flex">
    <!-- UX-003: Offline status top banner -->
    <OfflineBanner :isOnline="isOnline" />

    <!-- Sidebar Navigation -->
    <Sidebar
      :offlineQueueCount="offlineQueueCount"
      @open="open"
    />

    <!-- Main Content Area -->
    <div class="flex-1 flex flex-col overflow-hidden">
      <!-- Header -->
      <Header
        :isOnline="isOnline"
        :offlineQueueCount="offlineQueueCount"
        :currentTime="currentTime"
        :autoCaptureEnabled="autoCaptureEnabled"
        :todayRecordsCount="todayRecords.length"
        @showOfflineQueue="open('offlineQueue')"
      />

      <!-- Dashboard -->
      <Dashboard
        :isDesktop="isDesktop"
        :autoCaptureEnabled="autoCaptureEnabled"
        :isCapturing="isCapturing"
        :isLoading="isLoadingTodayRecords"
        :quickNotesCount="quickNotesCount"
        :todayRecords="todayRecords"
        :isGenerating="isGenerating"
        :isGeneratingWeekly="isGeneratingWeekly"
        :isGeneratingMonthly="isGeneratingMonthly"
        :screenshotCount="screenshotCount"
        :summaryPath="summaryPath"
        :weeklyReportPath="weeklyReportPath"
        :monthlyReportPath="monthlyReportPath"
        :customReportPath="customReportPath"
        :comparisonReportPath="comparisonReportPath"
        @open="open"
        @takeScreenshot="takeScreenshot"
        @triggerCapture="triggerCapture"
        @toggleAutoCapture="toggleAutoCapture"
        @openQuickNote="openQuickNote"
        @generateReport="handleReportGenerate"
        @generateMultilingualReport="handleGenerateMultilingualReport"
        @languageChange="handleLanguageChange"
        @customAction="handleCustomAction"
        @viewScreenshot="openScreenshot"
      />
    </div>

    <!-- Modal Container with Teleport and Transitions -->
    <Teleport to="body">
      <Transition name="fade" mode="out-in">
        <SettingsModal v-if="isOpen('settings')" @close="close('settings')" />
      </Transition>
      <Transition name="scale" mode="out-in">
        <BackupModal v-if="isOpen('backup')" @close="close('backup')" />
      </Transition>
      <Transition name="slide-up" mode="out-in">
        <QuickNoteModal v-if="isOpen('quickNote')" @close="close('quickNote')" @save="handleQuickNote" />
      </Transition>
      <Transition name="scale" mode="out-in">
        <ScreenshotModal v-if="isOpen('screenshot')" :record="selectedScreenshot!" @close="close('screenshot')" />
      </Transition>
      <Transition name="slide-up" mode="out-in">
        <ScreenshotGallery v-if="isOpen('screenshotGallery')" @close="close('screenshotGallery')" />
      </Transition>
      <Transition name="slide-up" mode="out-in">
        <DailySummaryViewer v-if="isOpen('summaryViewer')" :summaryPath="summaryPath!" @close="close('summaryViewer')" />
      </Transition>
      <Transition name="slide-up" mode="out-in">
        <DailySummaryViewer v-if="isOpen('weeklyReportViewer')" :summaryPath="weeklyReportPath!" @close="close('weeklyReportViewer')" />
      </Transition>
      <Transition name="slide-up" mode="out-in">
        <DailySummaryViewer v-if="isOpen('monthlyReportViewer')" :summaryPath="monthlyReportPath!" @close="close('monthlyReportViewer')" />
      </Transition>
      <Transition name="slide-up" mode="out-in">
        <DailySummaryViewer v-if="isOpen('customReportViewer')" :summaryPath="customReportPath!" @close="close('customReportViewer')" />
      </Transition>
      <Transition name="scale" mode="out-in">
        <CustomReportModal v-if="isOpen('customReport')" @close="close('customReport')" @generated="handleCustomReportGenerated" />
      </Transition>
      <Transition name="scale" mode="out-in">
        <ReportComparisonModal v-if="isOpen('comparisonReport')" @close="close('comparisonReport')" @generated="handleComparisonReportGenerated" />
      </Transition>
      <Transition name="slide-up" mode="out-in">
        <DailySummaryViewer v-if="isOpen('comparisonReportViewer')" :summaryPath="comparisonReportPath!" @close="close('comparisonReportViewer')" />
      </Transition>
      <Transition name="slide-up" mode="out-in">
        <ReportHistoryViewer v-if="isOpen('reportHistory')" @close="close('reportHistory')" @viewFile="handleViewReportFile" />
      </Transition>
      <Transition name="slide-up" mode="out-in">
        <LogViewer v-if="isOpen('logViewer')" @close="close('logViewer')" />
      </Transition>
      <Transition name="slide-up" mode="out-in">
        <HistoryViewer v-if="isOpen('historyViewer')" :initialTag="initialFilterTag" @close="close('historyViewer'); initialFilterTag = null" />
      </Transition>
      <Transition name="fade" mode="out-in">
        <SearchPanel v-if="isOpen('search')" @close="close('search')" @viewScreenshot="handleSearchViewScreenshot" />
      </Transition>
      <Transition name="slide-up" mode="out-in">
        <TagCloud v-if="isOpen('tagCloud')" @close="close('tagCloud')" @tagSelected="handleTagSelected" />
      </Transition>
      <Transition name="scale" mode="out-in">
        <ExportModal v-if="isOpen('export')" @close="close('export')" />
      </Transition>
      <Transition name="slide-up" mode="out-in">
        <TimelineVisualization
          v-if="isOpen('timeline')"
          @close="close('timeline')"
          @viewScreenshot="handleTimelineViewScreenshot"
        />
      </Transition>
      <Transition name="slide-up" mode="out-in">
        <OfflineQueueModal v-if="isOpen('offlineQueue')" @close="close('offlineQueue')" />
      </Transition>
      <Transition name="scale" mode="out-in">
        <ReanalyzeByDateModal v-if="isOpen('reanalyzeByDate')" @close="close('reanalyzeByDate')" @reanalyzed="handleReanalyzedByDate" />
      </Transition>
      <!-- PERF-002: Onboarding Modal -->
      <OnboardingModal v-if="showOnboarding" @close="showOnboarding = false" @completed="showOnboarding = false" />
      <Transition name="slide-up" mode="out-in">
        <SessionListModal
          v-if="isOpen('sessionList')"
          @close="close('sessionList')"
          @viewSession="handleViewSession"
          @sessionAnalyzed="handleSessionAnalyzed"
        />
      </Transition>
      <Transition name="slide-up" mode="out-in">
        <SessionDetailView
          v-if="selectedSession !== null"
          :session="selectedSession!"
          @close="selectedSession = null"
          @updated="handleSessionUpdated"
        />
      </Transition>
      <Transition name="scale" mode="out-in">
        <StatisticsPanel v-if="isOpen('statistics')" @close="close('statistics')" />
      </Transition>
      <Toast />
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { register, unregister } from '@tauri-apps/plugin-global-shortcut'
import { usePlatform } from './composables/usePlatform'
import { useModal } from './composables/useModal'
import { loadLanguageFromBackend } from './i18n'
import { initTheme } from './theme'

// Layout Components
import Sidebar from './components/layout/Sidebar.vue'
import Header from './components/layout/Header.vue'
import Dashboard from './components/layout/Dashboard.vue'

// Modal Components
import SettingsModal from './components/SettingsModal.vue'
import BackupModal from './components/BackupModal.vue'
import QuickNoteModal from './components/QuickNoteModal.vue'
import OnboardingModal from './components/OnboardingModal.vue'
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
import OfflineBanner from './components/OfflineBanner.vue'
import OfflineQueueModal from './components/OfflineQueueModal.vue'
import ReanalyzeByDateModal from './components/ReanalyzeByDateModal.vue'
import SessionListModal from './components/SessionListModal.vue'
import SessionDetailView from './components/SessionDetailView.vue'
import StatisticsPanel from './components/StatisticsPanel.vue'

import { showError, showSuccess, initToastI18n } from './stores/toast'
import type { LogRecord, Tag, Settings } from './types/tauri'

interface Session {
  id: number
  date: string
  start_time: string
  end_time: string | null
  ai_summary: string | null
  user_summary: string | null
  context_for_next: string | null
  status: 'active' | 'ended' | 'analyzed'
  screenshot_count?: number
}

const { t } = useI18n()
const { isDesktop } = usePlatform()

// State
const currentTime = ref('')
const isOnline = ref(true)
const offlineQueueCount = ref(0)
const autoCaptureEnabled = ref(false)
const quickNotesCount = ref(0)
const todayRecords = ref<LogRecord[]>([])
const isLoadingTodayRecords = ref(true)
const isGenerating = ref(false)
const isGeneratingWeekly = ref(false)
const isGeneratingMonthly = ref(false)
const isReanalyzing = ref(false)
const isCapturing = ref(false)
const isDashboardLoading = ref(true)
const summaryPath = ref('')
const weeklyReportPath = ref('')
const monthlyReportPath = ref('')
const customReportPath = ref('')
const comparisonReportPath = ref('')
const selectedScreenshot = ref<LogRecord | null>(null)
const initialFilterTag = ref<Tag | null>(null)
const selectedSession = ref<Session | null>(null)

// UX-010: useModal for centralized modal management
const { isOpen, open, close } = useModal()

// PERF-002: Onboarding state
const showOnboarding = ref(false)

// Computed
const screenshotCount = computed<number>(() => {
  return todayRecords.value.filter(r => r.source_type === 'auto' && r.screenshot_path).length
})

// Event listeners cleanup
let timeInterval: ReturnType<typeof setInterval> | null = null
let recordsRefreshInterval: ReturnType<typeof setInterval> | null = null
let unlistenTrayOpenSettings: UnlistenFn | null = null
let unlistenTrayOpenQuickNote: UnlistenFn | null = null
let unlistenNetworkStatus: UnlistenFn | null = null
let unlistenQueueUpdated: UnlistenFn | null = null
let networkCheckInterval: ReturnType<typeof setInterval> | null = null

// Methods
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
    const currentSettings = await invoke<Settings>('get_settings')
    await invoke('save_settings', { settings: { ...currentSettings, auto_capture_enabled: autoCaptureEnabled.value } })
    if (autoCaptureEnabled.value) {
      await loadTodayRecords()
    }
  } catch (err) {
    console.error('Failed to toggle auto capture:', err)
    showError(String(err))
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
    showSuccess(t('autoCapture.screenshotAnalysisComplete'))
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

// EXP-004: Handle search result screenshot viewing
const handleSearchViewScreenshot = (record: LogRecord) => {
  close('search')
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

const handleTagSelected = (tag: Tag | null) => {
  close('tagCloud')
  initialFilterTag.value = tag
  open('historyViewer')
}

const handleReportGenerate = (type: 'daily' | 'weekly' | 'monthly') => {
  if (type === 'daily') {
    generateSummary()
  } else if (type === 'weekly') {
    generateWeeklyReport()
  } else if (type === 'monthly') {
    generateMonthlyReport()
  }
}

const handleGenerateMultilingualReport = async (language: string) => {
  if (isGenerating.value) return
  isGenerating.value = true
  try {
    const result = await invoke<string>('generate_multilingual_daily_summary', { targetLang: language })
    summaryPath.value = result
    showSuccess(t('report.multilingualSuccess'))
  } catch (err) {
    console.error('Failed to generate multilingual summary:', err)
    showError(String(err), () => handleGenerateMultilingualReport(language))
  } finally {
    isGenerating.value = false
  }
}

const handleLanguageChange = async (language: string) => {
  try {
    const currentSettings = await invoke<Settings>('get_settings')
    await invoke('save_settings', {
      settings: {
        ...currentSettings,
        preferred_language: language,
      }
    })
  } catch (err) {
    console.error('Failed to save language preference:', err)
  }
}

const generateSummary = async () => {
  if (isGenerating.value) return
  isGenerating.value = true
  try {
    const result = await invoke<string>('generate_daily_summary')
    summaryPath.value = result
    showSuccess(t('report.dailySuccess'))
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
    showSuccess(t('report.weeklySuccess'))
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
    showSuccess(t('report.monthlySuccess'))
  } catch (err) {
    console.error('Failed to generate monthly report:', err)
    showError(String(err), generateMonthlyReport)
  } finally {
    isGeneratingMonthly.value = false
  }
}

// FEAT-003 (#63): Batch reanalyze all today's screenshot records
const reanalyzeTodayRecords = async () => {
  if (isReanalyzing.value) return
  isReanalyzing.value = true
  try {
    const result = await invoke<{ total: number; success: number; failed: number; errors: string[] }>('reanalyze_today_records')
    if (result.failed > 0) {
      showError(t('reanalyze.partialSuccess', { success: result.success, total: result.total, failed: result.failed }))
    } else {
      showSuccess(t('reanalyze.fullSuccess', { count: result.success }))
    }
    // Refresh records after reanalysis
    await loadTodayRecords()
  } catch (err) {
    console.error('Failed to reanalyze records:', err)
    showError(String(err))
  } finally {
    isReanalyzing.value = false
  }
}

const handleCustomAction = (actionId: string) => {
  if (actionId === 'reanalyzeToday') {
    reanalyzeTodayRecords()
  }
}

const handleCustomReportGenerated = (path: string) => {
  customReportPath.value = path
}

const handleComparisonReportGenerated = (path: string) => {
  comparisonReportPath.value = path
}

// FEAT-004 (#64): Handle reanalysis by date completion
const handleReanalyzedByDate = async () => {
  // Refresh records after reanalysis
  await loadTodayRecords()
}

const handleViewSession = (session: Session) => {
  selectedSession.value = session
}

const handleSessionUpdated = (session: Session) => {
  selectedSession.value = null
}

const handleSessionAnalyzed = (session: Session) => {
  // Optionally refresh records or do other updates
  loadTodayRecords()
}

const handleViewReportFile = (path: string) => {
  summaryPath.value = path
  open('summaryViewer')
}

const loadTodayRecords = async () => {
  isLoadingTodayRecords.value = true
  try {
    const records = await invoke<LogRecord[]>('get_today_records')
    todayRecords.value = records
    quickNotesCount.value = records.filter(r => r.source_type === 'manual').length

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
  } finally {
    isDashboardLoading.value = false
    isLoadingTodayRecords.value = false
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

onMounted(async () => {
  initTheme()
  initToastI18n(useI18n())

  updateTime()
  timeInterval = setInterval(updateTime, 1000)
  recordsRefreshInterval = setInterval(loadTodayRecords, 30000)

  try {
    isOnline.value = await invoke<boolean>('get_network_status')
  } catch { /* ignore */ }

  try {
    const queueStatus = await invoke<{ pending_count: number }>('get_offline_queue_status')
    offlineQueueCount.value = queueStatus.pending_count || 0
  } catch { /* ignore */ }

  unlistenNetworkStatus = await listen<boolean>('network-status-changed', (event) => {
    isOnline.value = event.payload
  })

  unlistenQueueUpdated = await listen<{ pending_count: number }>('offline-queue-updated', (event) => {
    offlineQueueCount.value = event.payload?.pending_count || 0
  })

  networkCheckInterval = setInterval(async () => {
    try {
      isOnline.value = await invoke<boolean>('check_network_status')
      const queueStatus = await invoke<{ pending_count: number }>('get_offline_queue_status')
      offlineQueueCount.value = queueStatus.pending_count || 0
    } catch { /* ignore */ }
  }, 60000)

  unlistenTrayOpenSettings = await listen('tray-open-settings', () => {
    open('settings')
  })

  unlistenTrayOpenQuickNote = await listen('tray-open-quick-note', () => {
    open('quickNote')
  })

  if (isDesktop) {
    try {
      await register('Alt+Space', (event) => {
        if (event.state === 'Pressed') {
          open('quickNote')
        }
      })
    } catch (err) {
      console.error('Failed to register global shortcut:', err)
    }
  }

  await loadSettings()
  // PERF-005: Load language from backend after settings are loaded
  await loadLanguageFromBackend()
  await loadTodayRecords()

  // PERF-002: Check if onboarding is needed
  try {
    const settings = await invoke<Settings>('get_settings')
    if (!settings.api_base_url || !settings.onboarding_completed) {
      showOnboarding.value = true
    }
  } catch { /* ignore */ }
})

onUnmounted(async () => {
  if (timeInterval) clearInterval(timeInterval)
  if (recordsRefreshInterval) clearInterval(recordsRefreshInterval)
  if (networkCheckInterval) clearInterval(networkCheckInterval)
  if (unlistenTrayOpenSettings) unlistenTrayOpenSettings()
  if (unlistenTrayOpenQuickNote) unlistenTrayOpenQuickNote()
  if (unlistenNetworkStatus) unlistenNetworkStatus()
  if (unlistenQueueUpdated) unlistenQueueUpdated()

  if (isDesktop) {
    try {
      await unregister('Alt+Space')
    } catch { /* ignore */ }
  }
})
</script>