<template>
  <ErrorBoundary>
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
    </div>
  </ErrorBoundary>
</template>

<script setup lang="ts">
import type { LogRecord } from '../types/tauri'
import type { ModalId } from '../composables/useModal'

import ErrorBoundary from '../components/ErrorBoundary.vue'
import Sidebar from '../components/layout/Sidebar.vue'
import Header from '../components/layout/Header.vue'
import Dashboard from '../components/layout/Dashboard.vue'
import OfflineBanner from '../components/OfflineBanner.vue'

export interface AppShellProps {
  isDesktop: boolean
  currentTime: string
  isOnline: boolean
  offlineQueueCount: number
  autoCaptureEnabled: boolean
  quickNotesCount: number
  todayRecords: LogRecord[]
  isLoadingTodayRecords: boolean
  isGenerating: boolean
  isGeneratingWeekly: boolean
  isGeneratingMonthly: boolean
  isCapturing: boolean
  screenshotCount: number
  summaryPath: string
  weeklyReportPath: string
  monthlyReportPath: string
  customReportPath: string
  comparisonReportPath: string
}

defineProps<AppShellProps>()

const emit = defineEmits<{
  open: [modal: ModalId]
  takeScreenshot: []
  triggerCapture: []
  toggleAutoCapture: []
  openQuickNote: []
  generateReport: [type: 'daily' | 'weekly' | 'monthly']
  generateMultilingualReport: [language: string]
  languageChange: [language: string]
  customAction: [actionId: string]
  viewScreenshot: [record: LogRecord]
}>()

const open = (modal: ModalId) => emit('open', modal)
const takeScreenshot = () => emit('takeScreenshot')
const triggerCapture = () => emit('triggerCapture')
const toggleAutoCapture = () => emit('toggleAutoCapture')
const openQuickNote = () => emit('openQuickNote')
const handleReportGenerate = (type: 'daily' | 'weekly' | 'monthly') => emit('generateReport', type)
const handleGenerateMultilingualReport = (language: string) => emit('generateMultilingualReport', language)
const handleLanguageChange = (language: string) => emit('languageChange', language)
const handleCustomAction = (actionId: string) => emit('customAction', actionId)
const openScreenshot = (record: LogRecord) => emit('viewScreenshot', record)
</script>
