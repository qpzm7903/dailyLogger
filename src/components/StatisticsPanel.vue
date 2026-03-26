<template>
  <div ref="containerRef" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="handleClose">
    <div class="bg-dark rounded-2xl w-[800px] max-h-[85vh] overflow-hidden border border-gray-700 flex flex-col">
      <!-- Header -->
      <div class="px-6 py-4 border-b border-gray-700 flex items-center justify-between">
        <h2 class="text-lg font-semibold">{{ t('statistics.title') }}</h2>
        <button @click="handleClose" class="text-gray-400 hover:text-white">✕</button>
      </div>

      <!-- Time Range Selector -->
      <div class="px-6 py-4 border-b border-gray-700 flex items-center gap-3">
        <button
          v-for="range in timeRanges"
          :key="range.id"
          @click="selectRange(range.id)"
          :class="selectedRange === range.id ? 'bg-primary text-white' : 'bg-gray-700/50 text-gray-300 hover:bg-gray-600'"
          class="px-4 py-2 rounded-lg text-sm transition-colors"
        >
          {{ range.label }}
        </button>

        <!-- Custom Range -->
        <div class="flex items-center gap-2 ml-2">
          <input
            type="date"
            v-model="customStart"
            class="bg-gray-700/50 border border-gray-600 rounded-lg px-3 py-1.5 text-sm text-white focus:border-primary focus:outline-none"
          />
          <span class="text-gray-500">-</span>
          <input
            type="date"
            v-model="customEnd"
            class="bg-gray-700/50 border border-gray-600 rounded-lg px-3 py-1.5 text-sm text-white focus:border-primary focus:outline-none"
          />
          <button
            @click="applyCustomRange"
            :disabled="!customStart || !customEnd"
            class="px-3 py-1.5 bg-primary/80 hover:bg-primary disabled:opacity-50 rounded-lg text-sm transition-colors"
          >
            {{ t('statistics.apply') }}
          </button>
        </div>
      </div>

      <!-- Statistics Content -->
      <div class="flex-1 overflow-y-auto p-6">
        <!-- Loading State -->
        <div v-if="isLoading" class="flex items-center justify-center py-12">
          <div class="w-8 h-8 border-4 border-primary border-t-transparent rounded-full animate-spin"></div>
        </div>

        <!-- Statistics Cards -->
        <div v-else-if="statistics" class="space-y-6">
          <!-- Date Range Label -->
          <div class="text-sm text-gray-400">
            {{ statistics.date_range.label }}
          </div>

          <!-- Summary Cards -->
          <div class="grid grid-cols-4 gap-4">
            <div class="bg-darker rounded-xl p-4 border border-gray-700">
              <div class="text-3xl font-bold text-white">{{ statistics.screenshot_count }}</div>
              <div class="text-sm text-gray-400 mt-1">{{ t('statistics.screenshots') }}</div>
            </div>
            <div class="bg-darker rounded-xl p-4 border border-gray-700">
              <div class="text-3xl font-bold text-white">{{ statistics.session_count }}</div>
              <div class="text-sm text-gray-400 mt-1">{{ t('statistics.sessions') }}</div>
            </div>
            <div class="bg-darker rounded-xl p-4 border border-gray-700">
              <div class="text-3xl font-bold text-white">{{ statistics.record_count }}</div>
              <div class="text-sm text-gray-400 mt-1">{{ t('statistics.records') }}</div>
            </div>
            <div class="bg-darker rounded-xl p-4 border border-gray-700">
              <div class="text-3xl font-bold" :class="statistics.analysis_success_rate >= 80 ? 'text-green-400' : statistics.analysis_success_rate >= 50 ? 'text-yellow-400' : 'text-red-400'">
                {{ statistics.analysis_success_rate.toFixed(1) }}%
              </div>
              <div class="text-sm text-gray-400 mt-1">{{ t('statistics.analysisRate') }}</div>
            </div>
          </div>

          <!-- Daily Breakdown Chart -->
          <div class="bg-darker rounded-xl p-4 border border-gray-700">
            <h3 class="text-sm font-medium text-white mb-4">{{ t('statistics.dailyBreakdown') }}</h3>
            <div v-if="statistics.daily_breakdown.length === 0" class="text-center py-8 text-gray-500">
              {{ t('statistics.noData') }}
            </div>
            <div v-else class="space-y-2">
              <div
                v-for="day in statistics.daily_breakdown"
                :key="day.date"
                class="flex items-center gap-3"
              >
                <div class="w-20 text-xs text-gray-400">{{ day.date }}</div>
                <div class="flex-1 flex items-center gap-2">
                  <!-- Screenshot Bar -->
                  <div class="flex-1 h-6 bg-gray-700/50 rounded overflow-hidden">
                    <div
                      :style="{ width: getBarWidth(day.screenshot_count, maxScreenshotCount) + '%' }"
                      class="h-full bg-blue-500/80 rounded"
                      :title="`${t('statistics.screenshots')}: ${day.screenshot_count}`"
                    ></div>
                  </div>
                  <div class="w-12 text-xs text-gray-400 text-right">{{ day.screenshot_count }}</div>
                </div>
                <div class="flex-1 flex items-center gap-2">
                  <!-- Session Bar -->
                  <div class="flex-1 h-6 bg-gray-700/50 rounded overflow-hidden">
                    <div
                      :style="{ width: getBarWidth(day.session_count, maxSessionCount) + '%' }"
                      class="h-full bg-green-500/80 rounded"
                      :title="`${t('statistics.sessions')}: ${day.session_count}`"
                    ></div>
                  </div>
                  <div class="w-12 text-xs text-gray-400 text-right">{{ day.session_count }}</div>
                </div>
                <div class="flex-1 flex items-center gap-2">
                  <!-- Record Bar -->
                  <div class="flex-1 h-6 bg-gray-700/50 rounded overflow-hidden">
                    <div
                      :style="{ width: getBarWidth(day.record_count, maxRecordCount) + '%' }"
                      class="h-full bg-purple-500/80 rounded"
                      :title="`${t('statistics.records')}: ${day.record_count}`"
                    ></div>
                  </div>
                  <div class="w-12 text-xs text-gray-400 text-right">{{ day.record_count }}</div>
                </div>
              </div>
              <!-- Legend -->
              <div class="flex items-center gap-6 mt-4 pt-3 border-t border-gray-700">
                <div class="flex items-center gap-2">
                  <div class="w-3 h-3 bg-blue-500/80 rounded"></div>
                  <span class="text-xs text-gray-400">{{ t('statistics.screenshots') }}</span>
                </div>
                <div class="flex items-center gap-2">
                  <div class="w-3 h-3 bg-green-500/80 rounded"></div>
                  <span class="text-xs text-gray-400">{{ t('statistics.sessions') }}</span>
                </div>
                <div class="flex items-center gap-2">
                  <div class="w-3 h-3 bg-purple-500/80 rounded"></div>
                  <span class="text-xs text-gray-400">{{ t('statistics.records') }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Error State -->
        <div v-else-if="error" class="bg-red-900/20 border border-red-700 rounded-lg p-4">
          <p class="text-red-400 text-sm">{{ error }}</p>
        </div>
      </div>

      <!-- Footer -->
      <div class="px-6 py-4 border-t border-gray-700 flex justify-between items-center">
        <button
          @click="exportData"
          :disabled="!statistics || isExporting"
          class="px-4 py-2 bg-gray-600 hover:bg-gray-500 disabled:opacity-50 rounded-lg text-sm transition-colors"
        >
          {{ isExporting ? t('statistics.exporting') : t('statistics.export') }}
        </button>
        <button
          @click="handleClose"
          class="px-4 py-2 bg-primary hover:bg-blue-600 rounded-lg text-sm font-medium transition-colors"
        >
          {{ t('statistics.close') }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import { useFocusTrap } from '../composables/useFocusTrap'
import type { Statistics } from '../types/tauri'

const emit = defineEmits<{(e: 'close'): void}>()

const { t } = useI18n()

// Focus trap for accessibility
const containerRef = ref<HTMLElement | null>(null)
const { activate: activateFocusTrap, deactivate: deactivateFocusTrap } = useFocusTrap(containerRef)

// Time range options
const timeRanges = [
  { id: 'today', label: computed(() => t('statistics.today')) },
  { id: 'week', label: computed(() => t('statistics.thisWeek')) },
  { id: 'month', label: computed(() => t('statistics.thisMonth')) },
]

// State
const selectedRange = ref('today')
const customStart = ref('')
const customEnd = ref('')
const statistics = ref<Statistics | null>(null)
const isLoading = ref(false)
const error = ref('')
const isExporting = ref(false)

// Computed values for chart scaling
const maxScreenshotCount = computed(() => {
  if (!statistics.value?.daily_breakdown.length) return 1
  return Math.max(...statistics.value.daily_breakdown.map(d => d.screenshot_count), 1)
})

const maxSessionCount = computed(() => {
  if (!statistics.value?.daily_breakdown.length) return 1
  return Math.max(...statistics.value.daily_breakdown.map(d => d.session_count), 1)
})

const maxRecordCount = computed(() => {
  if (!statistics.value?.daily_breakdown.length) return 1
  return Math.max(...statistics.value.daily_breakdown.map(d => d.record_count), 1)
})

// Helper function for bar width percentage
function getBarWidth(value: number, max: number): number {
  if (max === 0) return 0
  return Math.max((value / max) * 100, value > 0 ? 2 : 0)
}

// Select predefined time range
async function selectRange(rangeId: string) {
  selectedRange.value = rangeId
  await loadStatistics()
}

// Apply custom date range
async function applyCustomRange() {
  if (!customStart.value || !customEnd.value) return
  selectedRange.value = 'custom'
  await loadStatistics()
}

// Load statistics from backend
async function loadStatistics() {
  isLoading.value = true
  error.value = ''

  try {
    const args: { range_type: string; custom_start?: string; custom_end?: string } = {
      range_type: selectedRange.value,
    }

    if (selectedRange.value === 'custom') {
      args.custom_start = customStart.value
      args.custom_end = customEnd.value
    }

    statistics.value = await invoke<Statistics>('get_statistics', args)
  } catch (err) {
    console.error('Failed to load statistics:', err)
    error.value = String(err)
  } finally {
    isLoading.value = false
  }
}

// Export statistics data
async function exportData() {
  if (!statistics.value) return

  isExporting.value = true
  try {
    const data = statistics.value
    const csvContent = generateCsv(data)
    const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' })
    const url = URL.createObjectURL(blob)

    const link = document.createElement('a')
    link.href = url
    link.download = `statistics_${data.date_range.start.slice(0, 10)}_${data.date_range.end.slice(0, 10)}.csv`
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
    URL.revokeObjectURL(url)
  } catch (err) {
    console.error('Failed to export:', err)
    error.value = String(err)
  } finally {
    isExporting.value = false
  }
}

// Generate CSV content from statistics
function generateCsv(data: Statistics): string {
  const lines: string[] = []

  // Header info
  lines.push(`# Statistics Report`)
  lines.push(`# Date Range: ${data.date_range.label}`)
  lines.push(`# Generated: ${new Date().toISOString()}`)
  lines.push('')

  // Summary
  lines.push('Summary')
  lines.push(`Screenshot Count,${data.screenshot_count}`)
  lines.push(`Session Count,${data.session_count}`)
  lines.push(`Record Count,${data.record_count}`)
  lines.push(`Analysis Success Rate,${data.analysis_success_rate.toFixed(2)}%`)
  lines.push('')

  // Daily breakdown
  lines.push('Daily Breakdown')
  lines.push('Date,Screenshots,Sessions,Records')
  for (const day of data.daily_breakdown) {
    lines.push(`${day.date},${day.screenshot_count},${day.session_count},${day.record_count}`)
  }

  return lines.join('\n')
}

function handleClose() {
  emit('close')
}

onMounted(() => {
  activateFocusTrap()
  loadStatistics()
})

onBeforeUnmount(() => {
  deactivateFocusTrap()
})
</script>