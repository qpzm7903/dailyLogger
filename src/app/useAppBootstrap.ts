/**
 * useAppBootstrap composable
 *
 * Handles application bootstrap and initialization logic that was previously
 * in App.vue's onMounted. This separates the lifecycle concerns from
 * the main App.vue component.
 *
 * Responsibilities:
 * - Theme and i18n initialization
 * - Time and records refresh intervals
 * - Network status polling
 * - Tauri event listeners
 * - Global shortcuts registration
 * - Settings and records loading
 * - Onboarding check
 */

import { ref, onUnmounted, type Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { register, unregister } from '@tauri-apps/plugin-global-shortcut'
import { useI18n } from 'vue-i18n'
import { initTheme } from '../theme'
import { initToastI18n } from '../stores/toast'
import { loadLanguageFromBackend } from '../i18n'
import { showError, showSuccess } from '../stores/toast'
import { fetchTagColors } from '../composables/useTagColors'
import type { LogRecord, Settings, Tag } from '../types/tauri'
import type { ModalId } from '../composables/useModal'

export interface UseAppBootstrapReturn {
  // State that App.vue needs to render
  currentTime: Ref<string>
  isOnline: Ref<boolean>
  offlineQueueCount: Ref<number>
  autoCaptureEnabled: Ref<boolean>
  quickNotesCount: Ref<number>
  todayRecords: Ref<LogRecord[]>
  isLoadingTodayRecords: Ref<boolean>
  showOnboarding: Ref<boolean>

  // Actions that App.vue passes to child components
  open: (modal: ModalId) => void

  // Refresh function for manual refresh
  refreshTodayRecords: () => Promise<void>
}

export interface ReportPaths {
  summaryPath: string
  weeklyReportPath: string
  monthlyReportPath: string
  customReportPath: string
  comparisonReportPath: string
}

interface BootstrapOptions {
  isDesktop: boolean
  openModal: (modal: ModalId) => void
  updateAutoCaptureEnabled: (enabled: boolean) => void
  updateQuickNotesCount: (count: number) => void
  updateTodayRecords: (records: LogRecord[]) => void
  updateIsLoadingTodayRecords: (loading: boolean) => void
  updateShowOnboarding: (show: boolean) => void
  updateReportPaths: (paths: ReportPaths) => void
  t: (key: string) => string
}

/**
 * Create the app bootstrap composable
 *
 * @param options - Bootstrap options including platform check and state setters
 * @returns Bootstrap state and actions
 */
export function useAppBootstrap(options: BootstrapOptions): UseAppBootstrapReturn & {
  init: () => Promise<void>
  cleanup: () => Promise<void>
} {
  const {
    isDesktop,
    openModal,
    updateAutoCaptureEnabled,
    updateQuickNotesCount,
    updateTodayRecords,
    updateIsLoadingTodayRecords,
    updateShowOnboarding,
    updateReportPaths,
    t
  } = options

  // Reactive state
  const currentTime = ref('')
  const isOnline = ref(true)
  const offlineQueueCount = ref(0)
  const autoCaptureEnabled = ref(false)
  const quickNotesCount = ref(0)
  const todayRecords = ref<LogRecord[]>([])
  const isLoadingTodayRecords = ref(true)
  const showOnboarding = ref(false)

  // Internal state for cleanup
  let timeInterval: ReturnType<typeof setInterval> | null = null
  let recordsRefreshInterval: ReturnType<typeof setInterval> | null = null
  let networkCheckInterval: ReturnType<typeof setInterval> | null = null
  let unlistenTrayOpenSettings: UnlistenFn | null = null
  let unlistenTrayOpenQuickNote: UnlistenFn | null = null
  let unlistenNetworkStatus: UnlistenFn | null = null
  let unlistenQueueUpdated: UnlistenFn | null = null

  // Update time display
  const updateTime = () => {
    currentTime.value = new Date().toLocaleString('zh-CN', {
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit'
    })
  }

  // Load today's records
  const loadTodayRecords = async () => {
    updateIsLoadingTodayRecords(true)
    try {
      const records = await invoke<LogRecord[]>('get_today_records')
      updateTodayRecords(records)
      quickNotesCount.value = records.filter(r => r.source_type === 'manual').length
      updateQuickNotesCount(quickNotesCount.value)

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
      updateIsLoadingTodayRecords(false)
    }
  }

  // Load settings
  const loadSettings = async () => {
    try {
      const settings = await invoke<Settings>('get_settings')
      autoCaptureEnabled.value = settings.auto_capture_enabled || false
      updateAutoCaptureEnabled(autoCaptureEnabled.value)
      updateReportPaths({
        summaryPath: settings.last_summary_path || '',
        weeklyReportPath: (settings as Settings & { last_weekly_report_path?: string }).last_weekly_report_path || '',
        monthlyReportPath: (settings as Settings & { last_monthly_report_path?: string }).last_monthly_report_path || '',
        customReportPath: (settings as Settings & { last_custom_report_path?: string }).last_custom_report_path || '',
        comparisonReportPath: ''
      })
    } catch (err) {
      console.error('Failed to load settings:', err)
    }
  }

  // Check if onboarding is needed
  const checkOnboarding = async () => {
    try {
      const settings = await invoke<Settings>('get_settings')
      if (!settings.api_base_url || !settings.onboarding_completed) {
        showOnboarding.value = true
        updateShowOnboarding(true)
      }
    } catch {
      // Ignore - onboarding will show on error
    }
  }

  // Open modal helper
  const open = (modal: ModalId) => {
    openModal(modal)
  }

  // Refresh records manually
  const refreshTodayRecords = async () => {
    await loadTodayRecords()
  }

  // Initialize the app
  const init = async () => {
    // Initialize theme
    initTheme()
    initToastI18n({ t })

    // Start time updates
    updateTime()
    timeInterval = setInterval(updateTime, 1000)

    // Start records refresh interval (every 30 seconds)
    recordsRefreshInterval = setInterval(loadTodayRecords, 30000)

    // Check initial network status
    try {
      isOnline.value = await invoke<boolean>('get_network_status')
    } catch {
      // ignore
    }

    // Check initial queue status
    try {
      const queueStatus = await invoke<{ pending_count: number }>('get_offline_queue_status')
      offlineQueueCount.value = queueStatus.pending_count || 0
    } catch {
      // ignore
    }

    // Set up network status event listener
    unlistenNetworkStatus = await listen<boolean>('network-status-changed', (event) => {
      isOnline.value = event.payload
    })

    // Set up offline queue updated event listener
    unlistenQueueUpdated = await listen<{ pending_count: number }>('offline-queue-updated', (event) => {
      offlineQueueCount.value = event.payload?.pending_count || 0
    })

    // Start network check interval (every 60 seconds)
    networkCheckInterval = setInterval(async () => {
      try {
        isOnline.value = await invoke<boolean>('check_network_status')
        const queueStatus = await invoke<{ pending_count: number }>('get_offline_queue_status')
        offlineQueueCount.value = queueStatus.pending_count || 0
      } catch {
        // ignore
      }
    }, 60000)

    // Set up tray event listeners
    unlistenTrayOpenSettings = await listen('tray-open-settings', () => {
      open('settings')
    })

    unlistenTrayOpenQuickNote = await listen('tray-open-quick-note', () => {
      open('quickNote')
    })

    // Register global shortcut (Alt+Space for quick note)
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

    // Load settings, language, records, and tag colors
    await loadSettings()
    await loadLanguageFromBackend()
    await loadTodayRecords()
    await fetchTagColors()
    await checkOnboarding()
  }

  // Cleanup on unmount
  const cleanup = async () => {
    // Clear intervals
    if (timeInterval) clearInterval(timeInterval)
    if (recordsRefreshInterval) clearInterval(recordsRefreshInterval)
    if (networkCheckInterval) clearInterval(networkCheckInterval)

    // Remove event listeners
    if (unlistenTrayOpenSettings) unlistenTrayOpenSettings()
    if (unlistenTrayOpenQuickNote) unlistenTrayOpenQuickNote()
    if (unlistenNetworkStatus) unlistenNetworkStatus()
    if (unlistenQueueUpdated) unlistenQueueUpdated()

    // Unregister global shortcut
    if (isDesktop) {
      try {
        await unregister('Alt+Space')
      } catch {
        // ignore
      }
    }
  }

  return {
    // State
    currentTime,
    isOnline,
    offlineQueueCount,
    autoCaptureEnabled,
    quickNotesCount,
    todayRecords,
    isLoadingTodayRecords,
    showOnboarding,

    // Actions
    open,
    refreshTodayRecords,

    // Lifecycle
    init,
    cleanup
  }
}
