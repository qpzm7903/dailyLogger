<template>
  <div class="fixed inset-0 bg-black/80 flex items-center justify-center z-50" @click.self="$emit('close')">
    <div class="bg-dark rounded-2xl max-w-2xl w-full max-h-[90vh] overflow-hidden border border-gray-700">
      <!-- Header -->
      <div class="px-6 py-4 border-b border-gray-700 flex items-center justify-between">
        <h2 class="text-lg font-semibold">{{ t('sessionList.title') }}</h2>
        <button @click="$emit('close')" class="text-gray-400 hover:text-white">✕</button>
      </div>

      <!-- Filter Bar -->
      <div class="px-6 py-3 border-b border-gray-700 flex items-center justify-between">
        <div class="flex items-center gap-2">
          <span class="text-sm text-gray-400">{{ t('sessionList.filter') }}:</span>
          <select
            v-model="statusFilter"
            class="bg-darker border border-gray-600 rounded-lg px-3 py-1.5 text-sm text-gray-300 focus:outline-none focus:border-primary"
          >
            <option value="pending">{{ t('sessionList.pending') }}</option>
            <option value="analyzed">{{ t('sessionList.analyzed') }}</option>
            <option value="all">{{ t('sessionList.all') }}</option>
          </select>
        </div>
        <div class="flex items-center gap-2">
          <span class="text-xs text-gray-500">
            {{ selectedSessionIds.size }} / {{ filteredSessions.length }} {{ t('sessionList.selected') }}
          </span>
          <button
            v-if="selectedSessionIds.size > 1"
            @click="analyzeSelected"
            :disabled="isAnalyzing"
            class="btn btn-primary btn-sm"
          >
            {{ isAnalyzing ? t('sessionList.analyzing') : t('sessionList.analyzeSelected', { count: selectedSessionIds.size }) }}
          </button>
        </div>
      </div>

      <!-- Content -->
      <div class="p-6 overflow-auto max-h-[60vh]">
        <div v-if="isLoading" class="text-center py-8 text-gray-500">
          {{ t('sessionList.loading') }}
        </div>
        <div v-else-if="filteredSessions.length === 0" class="text-center py-8 text-gray-500">
          {{ t('sessionList.noSessions') }}
        </div>
        <div v-else class="space-y-3">
          <div
            v-for="session in filteredSessions"
            :key="session.id"
            :class="[
              'bg-darker/80 rounded-xl p-4 border transition-all duration-200',
              selectedSessionIds.has(session.id)
                ? 'border-primary bg-gray-800/40'
                : 'border-gray-700/50 hover:border-gray-600'
            ]"
          >
            <div class="flex items-start justify-between">
              <!-- Session Info -->
              <div class="flex items-start gap-3">
                <!-- Checkbox for batch selection -->
                <input
                  type="checkbox"
                  :checked="selectedSessionIds.has(session.id)"
                  @change="toggleSessionSelection(session.id)"
                  class="mt-1 w-4 h-4 rounded border-gray-600 bg-darker text-primary focus:ring-primary focus:ring-offset-0"
                />
                <div>
                  <div class="flex items-center gap-2 mb-1">
                    <span class="text-sm font-medium text-white">
                      {{ formatTime(session.start_time) }}
                      <span class="text-gray-500 mx-1">—</span>
                      {{ session.end_time ? formatTime(session.end_time) : t('sessionList.ongoing') }}
                    </span>
                    <span
                      :class="getStatusBadgeClass(session.status)"
                      class="px-2 py-0.5 rounded-full text-xs"
                    >
                      {{ t(`sessionList.status.${session.status}`) }}
                    </span>
                  </div>
                  <div class="text-xs text-gray-500">
                    {{ t('sessionList.screenshotCount', { count: session.screenshot_count || 0 }) }}
                  </div>
                  <p v-if="session.ai_summary || session.user_summary" class="text-sm text-gray-400 mt-2 line-clamp-2">
                    {{ session.user_summary || session.ai_summary }}
                  </p>
                </div>
              </div>

              <!-- Actions -->
              <div class="flex items-center gap-2 ml-4">
                <button
                  v-if="session.status !== 'analyzed'"
                  @click="analyzeSession(session)"
                  :disabled="isAnalyzing"
                  class="px-3 py-1.5 text-xs rounded-md bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 disabled:cursor-not-allowed text-white transition-colors"
                >
                  {{ isAnalyzing && analyzingSessionId === session.id ? t('sessionList.analyzing') : t('sessionList.analyze') }}
                </button>
                <button
                  @click="viewSession(session)"
                  class="px-3 py-1.5 text-xs rounded-md bg-gray-600 hover:bg-gray-500 text-white transition-colors"
                >
                  {{ t('sessionList.view') }}
                </button>
              </div>
            </div>

            <!-- Batch analysis progress -->
            <div v-if="isAnalyzing && analyzingSessionId === session.id" class="mt-3">
              <div class="w-full bg-gray-700 rounded-full h-1.5">
                <div class="bg-blue-500 h-1.5 rounded-full animate-pulse" style="width: 60%"></div>
              </div>
              <p class="text-xs text-gray-500 mt-1">{{ t('sessionList.analyzingProgress') }}</p>
            </div>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="px-6 py-4 border-t border-gray-700 flex justify-end">
        <button
          @click="$emit('close')"
          class="px-4 py-2 text-sm rounded-lg bg-gray-600 hover:bg-gray-500 text-white transition-colors"
        >
          {{ t('sessionList.close') }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import { showToast } from '../stores/toast'

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

const { t, locale } = useI18n()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'viewSession', session: Session): void
  (e: 'sessionAnalyzed', session: Session): void
}>()

const sessions = ref<Session[]>([])
const isLoading = ref(true)
const statusFilter = ref<'pending' | 'analyzed' | 'all'>('pending')
const selectedSessionIds = ref<Set<number>>(new Set())
const isAnalyzing = ref(false)
const analyzingSessionId = ref<number | null>(null)

const filteredSessions = computed(() => {
  if (statusFilter.value === 'pending') {
    return sessions.value.filter(s => s.status !== 'analyzed')
  }
  if (statusFilter.value === 'analyzed') {
    return sessions.value.filter(s => s.status === 'analyzed')
  }
  return sessions.value
})

const loadSessions = async () => {
  isLoading.value = true
  try {
    sessions.value = await invoke<Session[]>('get_today_sessions')
  } catch (err) {
    const errorMsg = String(err)
    showToast(t('sessionList.loadFailed', { error: errorMsg }), { type: 'error' })
  } finally {
    isLoading.value = false
  }
}

const formatTime = (timestamp: string): string => {
  const date = new Date(timestamp)
  if (isNaN(date.getTime())) return '--:--'
  return date.toLocaleTimeString(locale.value === 'zh-CN' ? 'zh-CN' : 'en-US', {
    hour: '2-digit',
    minute: '2-digit',
    hour12: false
  })
}

const getStatusBadgeClass = (status: string): string => {
  switch (status) {
    case 'active':
      return 'bg-green-500/20 text-green-400'
    case 'ended':
      return 'bg-yellow-500/20 text-yellow-400'
    case 'analyzed':
      return 'bg-blue-500/20 text-blue-400'
    default:
      return 'bg-gray-500/20 text-gray-400'
  }
}

const toggleSessionSelection = (sessionId: number) => {
  if (selectedSessionIds.value.has(sessionId)) {
    selectedSessionIds.value.delete(sessionId)
  } else {
    selectedSessionIds.value.add(sessionId)
  }
  selectedSessionIds.value = new Set(selectedSessionIds.value)
}

const analyzeSession = async (session: Session) => {
  if (isAnalyzing.value) return

  isAnalyzing.value = true
  analyzingSessionId.value = session.id

  try {
    await invoke('analyze_session', { sessionId: session.id })
    showToast(t('sessionList.analyzeSuccess'), { type: 'success' })

    // Update session status locally
    const index = sessions.value.findIndex(s => s.id === session.id)
    if (index !== -1) {
      sessions.value[index] = { ...sessions.value[index], status: 'analyzed' }
    }
    selectedSessionIds.value.delete(session.id)
    selectedSessionIds.value = new Set(selectedSessionIds.value)

    emit('sessionAnalyzed', sessions.value[index])
  } catch (err) {
    const errorMsg = String(err)
    showToast(t('sessionList.analyzeFailed', { error: errorMsg }), { type: 'error', action: { label: t('toast.retry'), onClick: () => analyzeSession(session) } })
  } finally {
    isAnalyzing.value = false
    analyzingSessionId.value = null
  }
}

const analyzeSelected = async () => {
  if (selectedSessionIds.value.size === 0 || isAnalyzing.value) return

  isAnalyzing.value = true
  const sessionIds = Array.from(selectedSessionIds.value)
  let successCount = 0
  let failCount = 0

  for (const sessionId of sessionIds) {
    analyzingSessionId.value = sessionId
    try {
      await invoke('analyze_session', { sessionId })
      successCount++

      // Update session status locally
      const index = sessions.value.findIndex(s => s.id === sessionId)
      if (index !== -1) {
        sessions.value[index] = { ...sessions.value[index], status: 'analyzed' }
      }
      selectedSessionIds.value.delete(sessionId)
    } catch {
      failCount++
    }
  }

  selectedSessionIds.value = new Set()
  isAnalyzing.value = false
  analyzingSessionId.value = null

  if (failCount === 0) {
    showToast(t('sessionList.batchAnalyzeSuccess', { count: successCount }), { type: 'success' })
  } else {
    showToast(t('sessionList.batchAnalyzePartial', { success: successCount, fail: failCount }), { type: 'warning' })
  }
}

const viewSession = (session: Session) => {
  emit('viewSession', session)
}

// Load sessions on mount
loadSessions()
</script>
