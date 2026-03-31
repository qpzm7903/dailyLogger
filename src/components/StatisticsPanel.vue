<template>
  <BaseModal @close="handleClose" contentClass="w-[900px] max-h-[85vh] overflow-hidden flex flex-col">
    <!-- Header -->
      <div class="px-6 py-4 border-b border-[var(--color-border)] flex items-center justify-between">
        <h2 class="text-lg font-semibold">{{ t('statistics.title') }}</h2>
        <button @click="handleClose" class="text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]">✕</button>
      </div>

      <!-- Tab Selector -->
      <div class="px-6 py-3 border-b border-[var(--color-border)] flex items-center gap-4">
        <div class="flex gap-2">
          <button
            @click="activeTab = 'details'"
            :class="activeTab === 'details' ? 'bg-primary text-[var(--color-text-primary)]' : 'bg-[var(--color-surface-1)]/50 text-[var(--color-text-secondary)] hover:bg-[var(--color-action-neutral)]'"
            class="px-4 py-2 rounded-lg text-sm transition-colors"
          >
            {{ t('statistics.details') }}
          </button>
          <button
            @click="switchToTrends"
            :class="activeTab === 'trends' ? 'bg-primary text-[var(--color-text-primary)]' : 'bg-[var(--color-surface-1)]/50 text-[var(--color-text-secondary)] hover:bg-[var(--color-action-neutral)]'"
            class="px-4 py-2 rounded-lg text-sm transition-colors"
          >
            {{ t('statistics.trendsAndComparison') }}
          </button>
        </div>
      </div>

      <!-- Details Tab Content -->
      <template v-if="activeTab === 'details'">
        <!-- Time Range Selector -->
        <div class="px-6 py-4 border-b border-[var(--color-border)] flex items-center gap-3">
          <button
            v-for="range in timeRanges"
            :key="range.id"
            @click="selectRange(range.id)"
            :class="selectedRange === range.id ? 'bg-primary text-[var(--color-text-primary)]' : 'bg-[var(--color-surface-1)]/50 text-[var(--color-text-secondary)] hover:bg-[var(--color-action-neutral)]'"
            class="px-4 py-2 rounded-lg text-sm transition-colors"
          >
            {{ range.label }}
          </button>

          <!-- Custom Range -->
          <div class="flex items-center gap-2 ml-2">
            <input
              type="date"
              v-model="customStart"
              class="bg-[var(--color-surface-1)]/50 border border-[var(--color-border-subtle)] rounded-lg px-3 py-1.5 text-sm text-[var(--color-text-primary)] focus:border-primary focus:outline-none"
            />
            <span class="text-[var(--color-text-muted)]">-</span>
            <input
              type="date"
              v-model="customEnd"
              class="bg-[var(--color-surface-1)]/50 border border-[var(--color-border-subtle)] rounded-lg px-3 py-1.5 text-sm text-[var(--color-text-primary)] focus:border-primary focus:outline-none"
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
            <div class="text-sm text-[var(--color-text-secondary)]">
              {{ statistics.date_range.label }}
            </div>

            <!-- Summary Cards -->
            <div class="grid grid-cols-4 gap-4">
              <div class="bg-[var(--color-surface-0)] rounded-xl p-4 border border-[var(--color-border)]">
                <div class="text-3xl font-bold text-[var(--color-text-primary)]">{{ statistics.screenshot_count }}</div>
                <div class="text-sm text-[var(--color-text-secondary)] mt-1">{{ t('statistics.screenshots') }}</div>
              </div>
              <div class="bg-[var(--color-surface-0)] rounded-xl p-4 border border-[var(--color-border)]">
                <div class="text-3xl font-bold text-[var(--color-text-primary)]">{{ statistics.session_count }}</div>
                <div class="text-sm text-[var(--color-text-secondary)] mt-1">{{ t('statistics.sessions') }}</div>
              </div>
              <div class="bg-[var(--color-surface-0)] rounded-xl p-4 border border-[var(--color-border)]">
                <div class="text-3xl font-bold text-[var(--color-text-primary)]">{{ statistics.record_count }}</div>
                <div class="text-sm text-[var(--color-text-secondary)] mt-1">{{ t('statistics.records') }}</div>
              </div>
              <div class="bg-darker rounded-xl p-4 border border-[var(--color-border)]">
                <div class="text-3xl font-bold" :class="statistics.analysis_success_rate >= 80 ? 'text-green-400' : statistics.analysis_success_rate >= 50 ? 'text-yellow-400' : 'text-red-400'">
                  {{ statistics.analysis_success_rate.toFixed(1) }}%
                </div>
                <div class="text-sm text-[var(--color-text-secondary)] mt-1">{{ t('statistics.analysisRate') }}</div>
              </div>
            </div>

            <!-- Daily Breakdown Chart -->
            <div class="bg-[var(--color-surface-0)] rounded-xl p-4 border border-[var(--color-border)]">
              <h3 class="text-sm font-medium text-[var(--color-text-primary)] mb-4">{{ t('statistics.dailyBreakdown') }}</h3>
              <div v-if="statistics.daily_breakdown.length === 0" class="text-center py-8 text-[var(--color-text-muted)]">
                {{ t('statistics.noData') }}
              </div>
              <div v-else class="space-y-2">
                <div
                  v-for="day in statistics.daily_breakdown"
                  :key="day.date"
                  class="flex items-center gap-3"
                >
                  <div class="w-20 text-xs text-[var(--color-text-secondary)]">{{ day.date }}</div>
                  <div class="flex-1 flex items-center gap-2">
                    <!-- Screenshot Bar -->
                    <div class="flex-1 h-6 bg-[var(--color-surface-1)]/50 rounded overflow-hidden">
                      <div
                        :style="{ width: getBarWidth(day.screenshot_count, maxScreenshotCount) + '%' }"
                        class="h-full bg-blue-500/80 rounded"
                        :title="`${t('statistics.screenshots')}: ${day.screenshot_count}`"
                      ></div>
                    </div>
                    <div class="w-12 text-xs text-[var(--color-text-secondary)] text-right">{{ day.screenshot_count }}</div>
                  </div>
                  <div class="flex-1 flex items-center gap-2">
                    <!-- Session Bar -->
                    <div class="flex-1 h-6 bg-[var(--color-surface-1)]/50 rounded overflow-hidden">
                      <div
                        :style="{ width: getBarWidth(day.session_count, maxSessionCount) + '%' }"
                        class="h-full bg-green-500/80 rounded"
                        :title="`${t('statistics.sessions')}: ${day.session_count}`"
                      ></div>
                    </div>
                    <div class="w-12 text-xs text-[var(--color-text-muted)] text-right">{{ day.session_count }}</div>
                  </div>
                  <div class="flex-1 flex items-center gap-2">
                    <!-- Record Bar -->
                    <div class="flex-1 h-6 bg-[var(--color-surface-1)]/50 rounded overflow-hidden">
                      <div
                        :style="{ width: getBarWidth(day.record_count, maxRecordCount) + '%' }"
                        class="h-full bg-purple-500/80 rounded"
                        :title="`${t('statistics.records')}: ${day.record_count}`"
                      ></div>
                    </div>
                    <div class="w-12 text-xs text-[var(--color-text-muted)] text-right">{{ day.record_count }}</div>
                  </div>
                </div>
                <!-- Legend -->
                <div class="flex items-center gap-6 mt-4 pt-3 border-t border-[var(--color-border)]">
                  <div class="flex items-center gap-2">
                    <div class="w-3 h-3 bg-blue-500/80 rounded"></div>
                    <span class="text-xs text-[var(--color-text-secondary)]">{{ t('statistics.screenshots') }}</span>
                  </div>
                  <div class="flex items-center gap-2">
                    <div class="w-3 h-3 bg-green-500/80 rounded"></div>
                    <span class="text-xs text-[var(--color-text-secondary)]">{{ t('statistics.sessions') }}</span>
                  </div>
                  <div class="flex items-center gap-2">
                    <div class="w-3 h-3 bg-purple-500/80 rounded"></div>
                    <span class="text-xs text-[var(--color-text-secondary)]">{{ t('statistics.records') }}</span>
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
      </template>

      <!-- Trends Tab Content -->
      <template v-else-if="activeTab === 'trends'">
        <!-- Period Selector -->
        <div class="px-6 py-4 border-b border-[var(--color-border)] flex items-center gap-4">
          <button
            @click="selectTrendPeriod('week')"
            :class="trendPeriod === 'week' ? 'bg-primary text-[var(--color-text-primary)]' : 'bg-[var(--color-surface-1)]/50 text-[var(--color-text-secondary)] hover:bg-[var(--color-action-neutral)]'"
            class="px-4 py-2 rounded-lg text-sm transition-colors"
          >
            {{ t('statistics.thisWeekVsLastWeek') }}
          </button>
          <button
            @click="selectTrendPeriod('month')"
            :class="trendPeriod === 'month' ? 'bg-primary text-[var(--color-text-primary)]' : 'bg-[var(--color-surface-1)]/50 text-[var(--color-text-secondary)] hover:bg-[var(--color-action-neutral)]'"
            class="px-4 py-2 rounded-lg text-sm transition-colors"
          >
            {{ t('statistics.thisMonthVsLastMonth') }}
          </button>
        </div>

        <!-- Trends Content -->
        <div class="flex-1 overflow-y-auto p-6">
          <!-- Loading State -->
          <div v-if="isLoadingTrend" class="flex items-center justify-center py-12">
            <div class="w-8 h-8 border-4 border-primary border-t-transparent rounded-full animate-spin"></div>
          </div>

          <!-- Trends Data -->
          <div v-else-if="productivityTrend" class="space-y-6">
            <!-- Period Labels -->
            <div class="flex justify-between text-sm text-[var(--color-text-secondary)]">
              <span>{{ productivityTrend.current_period.label }}: {{ formatDateRange(productivityTrend.current_period) }}</span>
              <span>{{ productivityTrend.previous_period.label }}: {{ formatDateRange(productivityTrend.previous_period) }}</span>
            </div>

            <!-- Comparison Cards -->
            <div class="grid grid-cols-2 gap-4">
              <!-- Screenshot Comparison -->
              <div class="bg-[var(--color-surface-0)] rounded-xl p-4 border border-[var(--color-border)]">
                <div class="text-sm text-[var(--color-text-secondary)] mb-2">{{ t('statistics.screenshots') }}</div>
                <div class="flex items-end gap-4">
                  <div>
                    <div class="text-2xl font-bold text-[var(--color-text-primary)]">{{ productivityTrend.screenshot_comparison.current_total }}</div>
                    <div class="text-xs text-[var(--color-text-muted)]">{{ productivityTrend.current_period.label }}</div>
                  </div>
                  <div class="text-lg font-bold" :class="getTrendColor(productivityTrend.screenshot_comparison.trend)">
                    {{ formatChangePercent(productivityTrend.screenshot_comparison.change_percent) }}
                  </div>
                  <div class="text-sm text-[var(--color-text-muted)]">
                    {{ productivityTrend.screenshot_comparison.previous_total }} {{ t('statistics.vsLastPeriod') }}
                  </div>
                </div>
              </div>

              <!-- Record Comparison -->
              <div class="bg-[var(--color-surface-0)] rounded-xl p-4 border border-[var(--color-border)]">
                <div class="text-sm text-[var(--color-text-secondary)] mb-2">{{ t('statistics.records') }}</div>
                <div class="flex items-end gap-4">
                  <div>
                    <div class="text-2xl font-bold text-[var(--color-text-primary)]">{{ productivityTrend.record_comparison.current_total }}</div>
                    <div class="text-xs text-[var(--color-text-muted)]">{{ productivityTrend.current_period.label }}</div>
                  </div>
                  <div class="text-lg font-bold" :class="getTrendColor(productivityTrend.record_comparison.trend)">
                    {{ formatChangePercent(productivityTrend.record_comparison.change_percent) }}
                  </div>
                  <div class="text-sm text-[var(--color-text-muted)]">
                    {{ productivityTrend.record_comparison.previous_total }} {{ t('statistics.vsLastPeriod') }}
                  </div>
                </div>
              </div>
            </div>

            <!-- Daily Trend Line Chart -->
            <div class="bg-[var(--color-surface-0)] rounded-xl p-4 border border-[var(--color-border)]">
              <h3 class="text-sm font-medium text-[var(--color-text-primary)] mb-4">
                {{ t('statistics.dailyTrend') }} ({{ productivityTrend.average_daily_records.toFixed(1) }} {{ t('statistics.avgDaily') }})
              </h3>
              <div v-if="productivityTrend.daily_trend.length === 0" class="text-center py-8 text-[var(--color-text-muted)]">
                {{ t('statistics.noData') }}
              </div>
              <div v-else class="h-48">
                <svg class="w-full h-full" :viewBox="'0 0 ' + chartWidth + ' ' + chartHeight" preserveAspectRatio="none">
                  <!-- Grid lines -->
                  <line v-for="i in 4" :key="'grid-'+i"
                    :x1="padding" :y1="padding + (chartHeight - 2 * padding) * (i - 1) / 3"
                    :x2="chartWidth - padding" :y2="padding + (chartHeight - 2 * padding) * (i - 1) / 3"
                    stroke="var(--color-border)" stroke-width="1" stroke-dasharray="4"
                  />

                  <!-- Screenshot line (blue) -->
                  <polyline
                    :points="screenshotLinePoints"
                    fill="none"
                    stroke="#3b82f6"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  />

                  <!-- Record line (purple) -->
                  <polyline
                    :points="recordLinePoints"
                    fill="none"
                    stroke="#a855f7"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  />

                  <!-- X-axis labels -->
                  <text
                    v-for="(point, index) in xAxisLabels"
                    :key="'xlabel-'+index"
                    :x="padding + (chartWidth - 2 * padding) * index / (productivityTrend.daily_trend.length - 1)"
                    :y="chartHeight - padding + 20"
                    text-anchor="middle"
                    class="text-xs fill-[var(--color-text-muted)]"
                  >
                    {{ point }}
                  </text>
                </svg>
                <!-- Legend -->
                <div class="flex items-center justify-center gap-6 mt-4">
                  <div class="flex items-center gap-2">
                    <div class="w-4 h-0.5 bg-blue-500 rounded"></div>
                    <span class="text-xs text-[var(--color-text-secondary)]">{{ t('statistics.screenshots') }}</span>
                  </div>
                  <div class="flex items-center gap-2">
                    <div class="w-4 h-0.5 bg-purple-500 rounded"></div>
                    <span class="text-xs text-[var(--color-text-secondary)]">{{ t('statistics.records') }}</span>
                  </div>
                </div>
              </div>
            </div>

            <!-- Peak Hours -->
            <div class="bg-[var(--color-surface-0)] rounded-xl p-4 border border-[var(--color-border)]">
              <h3 class="text-sm font-medium text-[var(--color-text-primary)] mb-4">{{ t('statistics.peakHours') }}</h3>
              <div v-if="productivityTrend.peak_hours.length === 0" class="text-center py-4 text-[var(--color-text-muted)]">
                {{ t('statistics.noData') }}
              </div>
              <div v-else class="flex gap-2">
                <div
                  v-for="(hour, index) in productivityTrend.peak_hours"
                  :key="hour.hour"
                  class="flex-1 bg-[var(--color-surface-1)]/50 rounded-lg p-3 text-center"
                >
                  <div class="text-lg font-bold text-[var(--color-text-primary)]">{{ String(hour.hour).padStart(2, '0') }}:00</div>
                  <div class="text-sm text-[var(--color-text-secondary)]">{{ hour.count }} {{ t('statistics.records') }}</div>
                  <div class="text-xs text-[var(--color-text-muted)]">{{ hour.percentage.toFixed(1) }}%</div>
                </div>
              </div>
            </div>
          </div>

          <!-- Error State -->
          <div v-else-if="trendError" class="bg-red-900/20 border border-red-700 rounded-lg p-4">
            <p class="text-red-400 text-sm">{{ trendError }}</p>
          </div>
        </div>
      </template>

      <!-- Footer -->
      <div class="px-6 py-4 border-t border-[var(--color-border)] flex justify-between items-center">
        <button
          @click="exportData"
          :disabled="!canExport || isExporting"
          class="px-4 py-2 bg-[var(--color-action-neutral)] hover:bg-[var(--color-action-neutral)] disabled:opacity-50 rounded-lg text-sm transition-colors"
        >
          {{ isExporting ? t('statistics.exporting') : t('statistics.export') }}
        </button>
        <button
          @click="handleClose"
          class="px-4 py-2 bg-primary hover:bg-primary-hover rounded-lg text-sm font-medium transition-colors"
        >
          {{ t('statistics.close') }}
        </button>
      </div>
  </BaseModal>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import BaseModal from './BaseModal.vue'
import { useFocusTrap } from '../composables/useFocusTrap'
import type { Statistics, ProductivityTrend } from '../types/tauri'

const emit = defineEmits<{(e: 'close'): void}>()

const { t } = useI18n()

// Tab state
const activeTab = ref<'details' | 'trends'>('details')

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

// Trend state
const trendPeriod = ref<'week' | 'month'>('week')
const productivityTrend = ref<ProductivityTrend | null>(null)
const isLoadingTrend = ref(false)
const trendError = ref('')

// Chart dimensions
const chartWidth = 800
const chartHeight = 180
const padding = 30

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

// Can export based on active tab
const canExport = computed(() => {
  if (activeTab.value === 'details') {
    return statistics.value !== null
  } else {
    return productivityTrend.value !== null
  }
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
    const args: { rangeType: string; customStart?: string; customEnd?: string } = {
      rangeType: selectedRange.value,
    }

    if (selectedRange.value === 'custom') {
      args.customStart = customStart.value
      args.customEnd = customEnd.value
    }

    statistics.value = await invoke<Statistics>('get_statistics', args)
  } catch (err) {
    console.error('Failed to load statistics:', err)
    error.value = String(err)
  } finally {
    isLoading.value = false
  }
}

// Switch to trends tab and load trend data
async function switchToTrends() {
  activeTab.value = 'trends'
  await loadProductivityTrend()
}

// Select trend period
async function selectTrendPeriod(period: 'week' | 'month') {
  trendPeriod.value = period
  await loadProductivityTrend()
}

// Load productivity trend from backend
async function loadProductivityTrend() {
  isLoadingTrend.value = true
  trendError.value = ''

  try {
    productivityTrend.value = await invoke<ProductivityTrend>('get_productivity_trend', {
      comparisonType: trendPeriod.value,
    })
  } catch (err) {
    console.error('Failed to load productivity trend:', err)
    trendError.value = String(err)
  } finally {
    isLoadingTrend.value = false
  }
}

// Format change percent with sign
function formatChangePercent(percent: number): string {
  if (percent > 0) return `+${percent.toFixed(1)}%`
  return `${percent.toFixed(1)}%`
}

// Get trend color class
function getTrendColor(trend: string): string {
  if (trend === 'up') return 'text-green-400'
  if (trend === 'down') return 'text-red-400'
  return 'text-[var(--color-text-muted)]'
}

// Format date range for display
function formatDateRange(period: { start: string; end: string }): string {
  return `${period.start.slice(5, 10)} ~ ${period.end.slice(5, 10)}`
}

// Get line chart points (computed for performance)
const screenshotLinePoints = computed(() => {
  if (!productivityTrend.value || productivityTrend.value.daily_trend.length <= 1) {
    return ''
  }

  const points = productivityTrend.value.daily_trend
  const maxValue = Math.max(
    ...points.map(p => Math.max(p.screenshot_count, p.record_count)),
    1
  )

  return points
    .map((point, index) => {
      const x = padding + (chartWidth - 2 * padding) * index / (points.length - 1)
      const y = chartHeight - padding - ((point.screenshot_count / maxValue) * (chartHeight - 2 * padding))
      return `${x},${y}`
    })
    .join(' ')
})

const recordLinePoints = computed(() => {
  if (!productivityTrend.value || productivityTrend.value.daily_trend.length <= 1) {
    return ''
  }

  const points = productivityTrend.value.daily_trend
  const maxValue = Math.max(
    ...points.map(p => Math.max(p.screenshot_count, p.record_count)),
    1
  )

  return points
    .map((point, index) => {
      const x = padding + (chartWidth - 2 * padding) * index / (points.length - 1)
      const y = chartHeight - padding - ((point.record_count / maxValue) * (chartHeight - 2 * padding))
      return `${x},${y}`
    })
    .join(' ')
})

// Get X-axis labels (computed, show every few days to avoid crowding)
const xAxisLabels = computed(() => {
  if (!productivityTrend.value || productivityTrend.value.daily_trend.length === 0) {
    return []
  }

  const points = productivityTrend.value.daily_trend
  const labels: string[] = []
  const step = Math.max(1, Math.floor(points.length / 7))

  for (let i = 0; i < points.length; i += step) {
    labels.push(points[i].date.slice(5, 10)) // MM-DD format
  }

  return labels
})

// Export statistics or trend data
async function exportData() {
  isExporting.value = true
  try {
    let csvContent: string
    let filename: string

    if (activeTab.value === 'details' && statistics.value) {
      csvContent = generateCsv(statistics.value)
      filename = `statistics_${statistics.value.date_range.start.slice(0, 10)}_${statistics.value.date_range.end.slice(0, 10)}.csv`
    } else if (productivityTrend.value) {
      csvContent = generateTrendCsv(productivityTrend.value)
      filename = `productivity_trend_${productivityTrend.value.comparison_type}_${new Date().toISOString().slice(0, 10)}.csv`
    } else {
      return
    }

    const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' })
    const url = URL.createObjectURL(blob)

    const link = document.createElement('a')
    link.href = url
    link.download = filename
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
    URL.revokeObjectURL(url)
  } catch (err) {
    console.error('Failed to export:', err)
    if (activeTab.value === 'details') {
      error.value = String(err)
    } else {
      trendError.value = String(err)
    }
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

// Generate CSV content from productivity trend
function generateTrendCsv(data: ProductivityTrend): string {
  const lines: string[] = []

  // Header info
  lines.push(`# Productivity Trend Report`)
  lines.push(`# Comparison Type: ${data.comparison_type === 'week' ? 'Week over Week' : 'Month over Month'}`)
  lines.push(`# Current Period: ${data.current_period.label} (${data.current_period.start.slice(0, 10)} ~ ${data.current_period.end.slice(0, 10)})`)
  lines.push(`# Previous Period: ${data.previous_period.label} (${data.previous_period.start.slice(0, 10)} ~ ${data.previous_period.end.slice(0, 10)})`)
  lines.push(`# Generated: ${new Date().toISOString()}`)
  lines.push('')

  // Comparison Summary
  lines.push('Comparison Summary')
  lines.push(`Current Screenshot Count,${data.screenshot_comparison.current_total}`)
  lines.push(`Previous Screenshot Count,${data.screenshot_comparison.previous_total}`)
  lines.push(`Screenshot Change,${formatChangePercent(data.screenshot_comparison.change_percent)}`)
  lines.push(`Current Record Count,${data.record_comparison.current_total}`)
  lines.push(`Previous Record Count,${data.record_comparison.previous_total}`)
  lines.push(`Record Change,${formatChangePercent(data.record_comparison.change_percent)}`)
  lines.push(`Average Daily Records,${data.average_daily_records.toFixed(2)}`)
  lines.push('')

  // Daily Trend
  lines.push('Daily Trend')
  lines.push('Date,Screenshots,Records')
  for (const day of data.daily_trend) {
    lines.push(`${day.date},${day.screenshot_count},${day.record_count}`)
  }
  lines.push('')

  // Peak Hours
  lines.push('Peak Hours')
  lines.push('Hour,Count,Percentage')
  for (const hour of data.peak_hours) {
    lines.push(`${String(hour.hour).padStart(2, '0')}:00,${hour.count},${hour.percentage.toFixed(1)}%`)
  }

  return lines.join('\n')
}

function handleClose() {
  emit('close')
}

onMounted(() => {
  loadStatistics()
})
</script>