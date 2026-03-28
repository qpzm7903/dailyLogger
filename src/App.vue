<template>
  <AppShell
    :isDesktop="isDesktop"
    :currentTime="currentTime"
    :isOnline="isOnline"
    :offlineQueueCount="offlineQueueCount"
    :autoCaptureEnabled="autoCaptureEnabled"
    :quickNotesCount="quickNotesCount"
    :todayRecords="todayRecords"
    :isLoadingTodayRecords="isLoadingTodayRecords"
    :isGenerating="isGenerating"
    :isGeneratingWeekly="isGeneratingWeekly"
    :isGeneratingMonthly="isGeneratingMonthly"
    :isCapturing="isCapturing"
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

  <AppModals
    :isOpen="isOpen"
    :selectedScreenshot="selectedScreenshot"
    :initialFilterTag="initialFilterTag"
    :selectedSession="selectedSession"
    :showOnboarding="showOnboarding"
    :summaryPath="summaryPath"
    :weeklyReportPath="weeklyReportPath"
    :monthlyReportPath="monthlyReportPath"
    :customReportPath="customReportPath"
    :comparisonReportPath="comparisonReportPath"
    @close="closeModal"
    @quickNoteSave="handleQuickNote"
    @viewReportFile="handleViewReportFile"
    @searchViewScreenshot="handleSearchViewScreenshot"
    @tagSelected="handleTagSelected"
    @timelineViewScreenshot="handleTimelineViewScreenshot"
    @reanalyzedByDate="handleReanalyzedByDate"
    @viewSession="handleViewSession"
    @sessionUpdated="handleSessionUpdated"
    @sessionAnalyzed="handleSessionAnalyzed"
    @customReportGenerated="handleCustomReportGenerated"
    @comparisonReportGenerated="handleComparisonReportGenerated"
  />
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { usePlatform } from './composables/usePlatform'
import { useModal, type ModalId } from './composables/useModal'
import { useAppBootstrap } from './app/useAppBootstrap'

// App shell and modals
import AppShell from './app/AppShell.vue'
import AppModals from './app/AppModals.vue'

// Toast and errors
import { showError, showSuccess } from './stores/toast'
import type { LogRecord, Tag, Settings } from './types/tauri'

// Feature actions
import { captureActions, addQuickNote } from './features/capture/actions'
import { reportActions } from './features/reports/actions'
import { settingsActions } from './features/settings/actions'
import { recordsActions } from './features/records/actions'
import { systemActions } from './features/system/actions'

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

// Modal management
const { isOpen, open: openModal, close } = useModal()

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
const showOnboarding = ref(false)

// Computed
const screenshotCount = computed<number>(() => {
  return todayRecords.value.filter(r => r.source_type === 'auto' && r.screenshot_path).length
})

// Bootstrap the app
const {
  init: bootstrapInit,
  cleanup: bootstrapCleanup
} = useAppBootstrap({
  isDesktop: isDesktop.value,
  openModal,
  updateAutoCaptureEnabled: (enabled) => { autoCaptureEnabled.value = enabled },
  updateQuickNotesCount: (count) => { quickNotesCount.value = count },
  updateTodayRecords: (records) => { todayRecords.value = records },
  updateIsLoadingTodayRecords: (loading) => { isLoadingTodayRecords.value = loading },
  updateShowOnboarding: (show) => { showOnboarding.value = show },
  updateReportPaths: (paths) => {
    summaryPath.value = paths.summaryPath
    weeklyReportPath.value = paths.weeklyReportPath
    monthlyReportPath.value = paths.monthlyReportPath
    customReportPath.value = paths.customReportPath
    comparisonReportPath.value = paths.comparisonReportPath
  },
  t
})

// Modal helpers
const open = (modal: ModalId) => openModal(modal)
const closeModal = (modal?: ModalId) => close(modal)

// Business actions
const toggleAutoCapture = async () => {
  try {
    if (autoCaptureEnabled.value) {
      await captureActions.stopAutoCapture()
    } else {
      await captureActions.startAutoCapture()
    }
    autoCaptureEnabled.value = !autoCaptureEnabled.value
    const currentSettings = await settingsActions.getSettings()
    await settingsActions.saveSettings({ ...currentSettings, auto_capture_enabled: autoCaptureEnabled.value })
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
    const path = await captureActions.takeScreenshot()
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
    if (String(err).includes('permission') || String(err).includes('Permission')) {
      showError(t('error.screenshotPermissionDenied'))
    } else {
      showError(String(err), takeScreenshot)
    }
  } finally {
    isCapturing.value = false
  }
}

const triggerCapture = async () => {
  if (isCapturing.value) return
  isCapturing.value = true
  try {
    await captureActions.triggerCapture()
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

const handleSearchViewScreenshot = (record: LogRecord) => {
  close('search')
  selectedScreenshot.value = record
  open('screenshot')
}

const handleQuickNote = async (content: string) => {
  try {
    await addQuickNote(content)
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

const handleReportGenerate = (type: 'daily' | 'weekly' | 'monthly', vaultName?: string) => {
  if (type === 'daily') {
    generateSummary(vaultName)
  } else if (type === 'weekly') {
    generateWeeklyReport()
  } else if (type === 'monthly') {
    generateMonthlyReport()
  }
}

const handleGenerateMultilingualReport = async (language: string) => {
  if (isGenerating.value) return
  if (!isOnline.value) {
    showError(t('offlineBanner.offline'))
    return
  }
  isGenerating.value = true
  try {
    const result = await reportActions.generateMultilingualDailySummary(language)
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
    const currentSettings = await settingsActions.getSettings()
    await settingsActions.saveSettings({
      ...currentSettings,
      preferred_language: language,
    })
  } catch (err) {
    console.error('Failed to save language preference:', err)
  }
}

const generateSummary = async (vaultName?: string) => {
  if (isGenerating.value) return
  if (!isOnline.value) {
    showError(t('offlineBanner.offline'))
    return
  }
  isGenerating.value = true
  try {
    const result = await reportActions.generateDailySummary(vaultName)
    summaryPath.value = result
    showSuccess(t('report.dailySuccess'))
  } catch (err) {
    console.error('Failed to generate summary:', err)
    showError(String(err), () => generateSummary(vaultName))
  } finally {
    isGenerating.value = false
  }
}

const generateWeeklyReport = async () => {
  if (isGeneratingWeekly.value) return
  if (!isOnline.value) {
    showError(t('offlineBanner.offline'))
    return
  }
  isGeneratingWeekly.value = true
  try {
    const result = await reportActions.generateWeeklyReport()
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
  if (!isOnline.value) {
    showError(t('offlineBanner.offline'))
    return
  }
  isGeneratingMonthly.value = true
  try {
    const result = await reportActions.generateMonthlyReport()
    monthlyReportPath.value = result
    showSuccess(t('report.monthlySuccess'))
  } catch (err) {
    console.error('Failed to generate monthly report:', err)
    showError(String(err), generateMonthlyReport)
  } finally {
    isGeneratingMonthly.value = false
  }
}

const reanalyzeTodayRecords = async () => {
  if (isReanalyzing.value) return
  if (!isOnline.value) {
    showError(t('offlineBanner.offline'))
    return
  }
  isReanalyzing.value = true
  try {
    const result = await reportActions.reanalyzeTodayRecords()
    if (result.failed > 0) {
      showError(t('reanalyze.partialSuccess', { success: result.success, total: result.total, failed: result.failed }))
    } else {
      showSuccess(t('reanalyze.fullSuccess', { count: result.success }))
    }
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

const handleReanalyzedByDate = async () => {
  await loadTodayRecords()
}

const handleViewSession = (session: Session) => {
  selectedSession.value = session
}

const handleSessionUpdated = (session: Session) => {
  selectedSession.value = null
}

const handleSessionAnalyzed = (session: Session) => {
  loadTodayRecords()
}

const handleViewReportFile = (path: string) => {
  summaryPath.value = path
  open('summaryViewer')
}

// Load today's records (used by several handlers)
const loadTodayRecords = async () => {
  isLoadingTodayRecords.value = true
  try {
    const records = await recordsActions.getTodayRecords()
    todayRecords.value = records
    quickNotesCount.value = records.filter(r => r.source_type === 'manual').length

    try {
      const status = await systemActions.checkNetworkStatus()
      isOnline.value = status

      const queueStatus = await systemActions.getOfflineQueueStatus()
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

// App lifecycle
onMounted(async () => {
  await bootstrapInit()
})

onUnmounted(async () => {
  await bootstrapCleanup()
})
</script>
