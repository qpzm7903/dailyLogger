<template>
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
</template>

<script setup lang="ts">
import type { LogRecord, Tag } from '../types/tauri'
import type { ModalId } from '../composables/useModal'
import type { Session } from '../features/sessions/actions'

// Modal Components
import SettingsModal from '../components/SettingsModal.vue'
import BackupModal from '../components/BackupModal.vue'
import QuickNoteModal from '../components/QuickNoteModal.vue'
import OnboardingModal from '../components/OnboardingModal.vue'
import ScreenshotModal from '../components/ScreenshotModal.vue'
import ScreenshotGallery from '../components/ScreenshotGallery.vue'
import DailySummaryViewer from '../components/DailySummaryViewer.vue'
import ReportHistoryViewer from '../components/ReportHistoryViewer.vue'
import LogViewer from '../components/LogViewer.vue'
import HistoryViewer from '../components/HistoryViewer.vue'
import SearchPanel from '../components/SearchPanel.vue'
import TagCloud from '../components/TagCloud.vue'
import ExportModal from '../components/ExportModal.vue'
import CustomReportModal from '../components/CustomReportModal.vue'
import ReportComparisonModal from '../components/ReportComparisonModal.vue'
import TimelineVisualization from '../components/TimelineVisualization.vue'
import Toast from '../components/Toast.vue'
import OfflineQueueModal from '../components/OfflineQueueModal.vue'
import ReanalyzeByDateModal from '../components/ReanalyzeByDateModal.vue'
import SessionListModal from '../components/SessionListModal.vue'
import SessionDetailView from '../components/SessionDetailView.vue'
import StatisticsPanel from '../components/StatisticsPanel.vue'

import { showError, showSuccess } from '../stores/toast'
import { useI18n } from 'vue-i18n'

// Feature actions
import { addQuickNote } from '../features/capture/actions'

export interface AppModalsProps {
  isOpen: (id: ModalId) => boolean
  selectedScreenshot: LogRecord | null
  initialFilterTag: Tag | null
  selectedSession: Session | null
  showOnboarding: boolean
  summaryPath: string
  weeklyReportPath: string
  monthlyReportPath: string
  customReportPath: string
  comparisonReportPath: string
}

defineProps<AppModalsProps>()

const emit = defineEmits<{
  close: [id?: ModalId]
  quickNoteSave: [content: string]
  viewReportFile: [path: string]
  searchViewScreenshot: [record: LogRecord]
  tagSelected: [tag: Tag | null]
  timelineViewScreenshot: [record: LogRecord]
  reanalyzedByDate: []
  viewSession: [session: Session]
  sessionUpdated: [session: Session]
  sessionAnalyzed: [session: Session]
  customReportGenerated: [path: string]
  comparisonReportGenerated: [path: string]
}>()

const { t } = useI18n()

const close = (id?: ModalId) => {
  emit('close', id)
}

const handleQuickNote = async (content: string) => {
  try {
    await addQuickNote(content)
    emit('close', 'quickNote')
    // Note: loadTodayRecords should be called by parent after this event
    showSuccess(t('quickNote.savedSuccess'))
  } catch (err) {
    console.error('Failed to save quick note:', err)
    showError(String(err))
  }
}

const handleCustomReportGenerated = (path: string) => {
  emit('customReportGenerated', path)
}

const handleComparisonReportGenerated = (path: string) => {
  emit('comparisonReportGenerated', path)
}

const handleViewReportFile = (path: string) => {
  emit('viewReportFile', path)
}

const handleSearchViewScreenshot = (record: LogRecord) => {
  emit('searchViewScreenshot', record)
}

const handleTagSelected = (tag: Tag | null) => {
  emit('tagSelected', tag)
}

const handleTimelineViewScreenshot = (record: LogRecord) => {
  emit('timelineViewScreenshot', record)
}

const handleReanalyzedByDate = () => {
  emit('reanalyzedByDate')
}

const handleViewSession = (session: Session) => {
  emit('viewSession', session)
}

const handleSessionUpdated = (session: Session) => {
  emit('sessionUpdated', session)
}

const handleSessionAnalyzed = (session: Session) => {
  emit('sessionAnalyzed', session)
}
</script>
