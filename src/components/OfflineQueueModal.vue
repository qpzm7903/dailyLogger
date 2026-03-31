<template>
  <BaseModal content-class="w-[90vw] max-w-xl overflow-hidden flex flex-col max-h-[80vh]" @close="$emit('close')">
    <!-- Header -->
    <div class="px-6 py-4 border-b border-[var(--color-border)] flex items-center justify-between">
      <h2 class="text-lg font-semibold text-[var(--color-text-primary)]">{{ t('offlineQueue.title') }}</h2>
      <button @click="$emit('close')" class="text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]">✕</button>
    </div>

      <!-- Content -->
      <div class="flex-1 overflow-auto p-4">
        <div v-if="isLoading" class="text-center py-8 text-[var(--color-text-muted)]">
          {{ t('common.loading') }}
        </div>
        <div v-else-if="tasks.length === 0" class="text-center py-8 text-[var(--color-text-muted)]">
          {{ t('offlineQueue.noTasks') }}
        </div>
        <div v-else class="flex flex-col divide-y divide-[var(--color-border)]">
          <div
            v-for="task in tasks"
            :key="task.id"
            class="py-3 px-2 hover:bg-[var(--color-surface-0)]/50 transition-colors"
          >
            <div class="flex items-start justify-between gap-2">
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2 mb-1">
                  <span
                    :class="getTaskTypeClass(task.task_type)"
                    class="px-2 py-0.5 rounded text-xs"
                  >
                    {{ getTaskTypeLabel(task.task_type) }}
                  </span>
                  <span class="text-xs text-[var(--color-text-secondary)]">{{ formatTime(task.created_at) }}</span>
                </div>
                <p class="text-sm text-[var(--color-text-secondary)]">{{ getTaskDescription(task) }}</p>
                <p v-if="task.error_message" class="text-xs text-red-400 mt-1">{{ task.error_message }}</p>
                <p class="text-xs text-[var(--color-text-muted)] mt-1">
                  {{ t('offlineQueue.retryCount', { current: task.retry_count, max: task.max_retries }) }}
                </p>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="px-6 py-4 border-t border-[var(--color-border)] flex items-center justify-between">
        <span class="text-sm text-[var(--color-text-secondary)]">
          {{ t('offlineQueue.totalPending', { count: tasks.length }) }}
        </span>
        <button
          v-if="tasks.length > 0"
          @click="processQueue"
          :disabled="isProcessing"
          class="px-4 py-2 bg-primary text-white rounded text-sm hover:bg-primary/80 transition-colors disabled:opacity-50"
        >
          {{ isProcessing ? t('offlineQueue.processing') : t('offlineQueue.processNow') }}
        </button>
      </div>
  </BaseModal>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import { formatDateTime } from '../utils/dateFormat'
import { showSuccess, showError } from '../stores/toast'
import BaseModal from './BaseModal.vue'

interface OfflineTask {
  id: number
  task_type: string
  payload: string
  record_id: number | null
  status: string
  error_message: string | null
  created_at: string
  completed_at: string | null
  retry_count: number
  max_retries: number
}

const { t } = useI18n()

const emit = defineEmits<{
  (e: 'close'): void
}>()

const tasks = ref<OfflineTask[]>([])
const isLoading = ref(true)
const isProcessing = ref(false)

onMounted(async () => {
  await loadTasks()
})

async function loadTasks() {
  isLoading.value = true
  try {
    const result = await invoke<OfflineTask[]>('get_pending_offline_tasks')
    tasks.value = result
  } catch (error) {
    showError(t('offlineQueue.loadFailed', { error }))
  } finally {
    isLoading.value = false
  }
}

async function processQueue() {
  isProcessing.value = true
  try {
    const result = await invoke<string>('process_offline_queue')
    showSuccess(result)
    await loadTasks()
  } catch (error) {
    showError(String(error))
  } finally {
    isProcessing.value = false
  }
}

function getTaskTypeClass(taskType: string): string {
  switch (taskType) {
    case 'screenshot_analysis':
      return 'bg-blue-500/20 text-blue-400'
    case 'daily_summary':
      return 'bg-green-500/20 text-green-400'
    case 'weekly_report':
      return 'bg-purple-500/20 text-purple-400'
    case 'monthly_report':
      return 'bg-orange-500/20 text-orange-400'
    default:
      return 'bg-[var(--color-action-neutral)]/20 text-[var(--color-text-muted)]'
  }
}

function getTaskTypeLabel(taskType: string): string {
  switch (taskType) {
    case 'screenshot_analysis':
      return t('offlineQueue.types.screenshot')
    case 'daily_summary':
      return t('offlineQueue.types.daily')
    case 'weekly_report':
      return t('offlineQueue.types.weekly')
    case 'monthly_report':
      return t('offlineQueue.types.monthly')
    default:
      return taskType
  }
}

function getTaskDescription(task: OfflineTask): string {
  try {
    const payload = JSON.parse(task.payload) as { screenshot_path?: string }
    if (task.task_type === 'screenshot_analysis' && payload.screenshot_path) {
      return t('offlineQueue.screenshotAnalysis', { path: payload.screenshot_path.split('/').pop() || payload.screenshot_path })
    }
  } catch {
    // Ignore JSON parse errors
  }
  return t('offlineQueue.taskType.' + task.task_type, task.task_type)
}

function formatTime(timestamp: string): string {
  return formatDateTime(timestamp)
}
</script>