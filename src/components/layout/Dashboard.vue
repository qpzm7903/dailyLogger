<template>
  <main class="flex-1 overflow-auto p-6">
    <div class="max-w-4xl mx-auto space-y-6">
      <div :class="isDesktop ? 'grid grid-cols-2 gap-4' : ''">
        <!-- Auto Capture Card (UX-4: Task 4 - compact layout) -->
        <div
          v-if="isDesktop"
          class="bg-dark/60 backdrop-blur-md rounded-2xl p-4 border border-gray-700/50 shadow-xl transition-all duration-200 hover:shadow-2xl"
        >
          <div class="flex items-center gap-2 mb-2">
            <span class="text-xl">🖥️</span>
            <h2 class="font-medium text-white">{{ t('autoCapture.title') }}</h2>
          </div>
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <span
                :class="autoCaptureEnabled ? 'bg-status-success animate-pulse' : 'bg-gray-500'"
                class="w-2 h-2 rounded-full inline-block transition-colors duration-300"
              ></span>
              <span class="text-xs text-gray-400">
                {{ autoCaptureEnabled ? t('autoCapture.running') : t('autoCapture.stopped') }}
              </span>
            </div>
            <div class="flex items-center gap-1.5">
              <button
                @click="$emit('takeScreenshot')"
                :disabled="isCapturing"
                class="btn btn-ghost btn-sm"
                :title="t('autoCapture.screenshot')"
              >
                {{ isCapturing ? t('autoCapture.screenshotting') : '📸' }}
              </button>
              <button
                @click="$emit('triggerCapture')"
                :disabled="isCapturing"
                class="btn btn-ghost btn-sm"
                :title="t('autoCapture.analyze')"
              >
                🤖
              </button>
              <button
                @click="$emit('toggleAutoCapture')"
                :class="autoCaptureEnabled ? 'btn btn-danger btn-sm' : 'btn btn-success btn-sm'"
              >
                {{ autoCaptureEnabled ? t('autoCapture.stop') : t('autoCapture.start') }}
              </button>
            </div>
          </div>
        </div>

        <!-- Quick Note Card (UX-4: Task 4 - compact layout) -->
        <div
          class="bg-dark/60 backdrop-blur-md rounded-2xl p-4 border border-gray-700/50 shadow-xl transition-all duration-200 hover:shadow-2xl"
        >
          <div class="flex items-center gap-2 mb-2">
            <span class="text-xl">⚡</span>
            <h2 class="font-medium text-white">{{ t('quickNote.title') }}</h2>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-xs text-gray-500">{{ t('quickNote.todayRecords', { count: quickNotesCount }) }}</span>
            <button
              @click="$emit('openQuickNote')"
              :title="isDesktop ? t('quickNote.shortcut') : ''"
              class="btn btn-primary btn-sm hover:shadow-primary/20"
            >
              {{ t('quickNote.record') }}
            </button>
          </div>
        </div>
      </div>

      <!-- Today Summary Widget (EXP-005) -->
      <TodaySummaryWidget ref="todaySummaryWidgetRef" />

      <!-- Timeline Widget -->
      <TimelineWidget ref="timelineWidgetRef" @open-full-timeline="$emit('open', 'timeline')" />

      <!-- Today's Workflow Card -->
      <div
        class="bg-dark/60 backdrop-blur-md rounded-2xl p-5 border border-gray-700/50 shadow-xl transition-all duration-200 hover:shadow-2xl"
      >
        <div class="flex items-center justify-between mb-4">
          <div class="flex items-center gap-2">
            <span class="text-2xl">📊</span>
            <h2 class="font-medium text-white">今日工作流</h2>
            <button
              v-if="screenshotCount > 0"
              @click="$emit('open', 'screenshotGallery')"
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
              :additionalOptions="[
                { id: 'customReport', label: '自定义报告', type: 'action', icon: '📄' },
                { id: 'comparisonReport', label: '对比分析', type: 'action', icon: '📊' },
                { id: 'reanalyzeByDate', label: '按日期重新分析', type: 'action', icon: '📅' },
                { id: 'reanalyzeToday', label: '重新分析今天', type: 'action', icon: '🔄' }
              ]"
              @generate="handleReportGenerate"
              @generateMultilingual="handleGenerateMultilingualReport"
              @languageChange="handleLanguageChange"
              @openModal="(modalId) => $emit('open', modalId as ModalId)"
              @customAction="handleCustomAction"
            />
            <button
              @click="$emit('open', 'sessionList')"
              class="btn btn-secondary btn-sm"
            >
              时段管理
            </button>
            <button
              @click="$emit('open', 'statistics')"
              class="btn btn-secondary btn-sm"
            >
              数据统计
            </button>
          </div>
        </div>

        <!-- Tag Filter -->
        <div v-if="tagEntries.length > 0" class="flex flex-wrap items-center gap-2 mb-4 pb-3 border-b border-gray-700/50">
          <button
            @click="selectedTagFilter = ''"
            :class="selectedTagFilter === '' ? 'bg-primary text-white' : 'bg-gray-700/50 text-gray-300 hover:bg-gray-600'"
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
          <button
            v-if="hiddenTagCount > 0 && !tagFilterExpanded"
            @click="tagFilterExpanded = true"
            :class="hasHiddenSelectedTag ? 'text-primary font-medium' : 'text-blue-400 hover:text-blue-300'"
            class="text-xs cursor-pointer whitespace-nowrap"
          >
            +{{ hiddenTagCount }} 个标签{{ hasHiddenSelectedTag ? ' (已选)' : '' }}
          </button>
          <button
            v-if="tagFilterExpanded && tagEntries.length > TAG_VISIBLE_THRESHOLD"
            @click="tagFilterExpanded = false"
            class="text-xs text-gray-400 hover:text-gray-300 cursor-pointer whitespace-nowrap"
          >
            收起
          </button>
        </div>

        <!-- Records List -->
        <SkeletonLoader v-if="props.isLoading" :count="5" />
        <EmptyState v-else-if="filteredRecords.length === 0" type="generic" :description="todayRecords.length === 0 ? t('emptyState.screenshots') : '无匹配标签的记录'" />
        <div v-else class="space-y-3 pr-1 custom-scrollbar">
          <div
            v-for="record in paginatedRecords"
            :key="record.id"
            @click="record.source_type === 'auto' && record.screenshot_path && openScreenshot(record)"
            :class="record.source_type === 'auto' && record.screenshot_path
              ? 'cursor-pointer hover:border-primary hover:bg-gray-800/40 group'
              : 'cursor-default'"
            class="bg-darker/80 rounded-xl p-3 border border-gray-700/50 transition-all duration-200 hover:-translate-y-0.5 hover:shadow-lg"
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
            <!-- Window Info -->
            <div
              v-if="getWindowInfo(record)?.title || getWindowInfo(record)?.process_name"
              class="window-info flex items-center gap-1.5 mb-1.5 text-xs text-gray-400"
            >
              <span>{{ getWindowIcon(getWindowInfo(record)?.process_name) }}</span>
              <span class="truncate max-w-[200px]" :title="getWindowInfo(record)?.title">
                {{ getWindowInfo(record)?.title || getWindowInfo(record)?.process_name }}
              </span>
            </div>
            <!-- Content -->
            <p v-if="record.source_type === 'auto'" class="text-sm text-gray-300 line-clamp-1 truncate">
              {{ extractSummary(record.content) || '分析完成' }}
            </p>
            <p v-else class="text-sm text-gray-300 line-clamp-3">{{ record.content }}</p>
            <!-- Tags -->
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

        <!-- Load More Button (UX-4: Task 2 - pagination) -->
        <div v-if="hasMore" class="mt-4 text-center">
          <button
            @click="loadMore"
            class="btn btn-secondary btn-sm"
          >
            {{ t('dashboard.loadMore', { current: paginatedRecords.length, total: filteredRecords.length }) }}
          </button>
        </div>
      </div>

      <!-- Output Files Card (UX-4: Task 3 - Tab-based) -->
      <div
        class="bg-dark/60 backdrop-blur-md rounded-2xl p-5 border border-gray-700/50 shadow-xl transition-all duration-200 hover:shadow-2xl"
      >
        <div class="flex items-center justify-between mb-4">
          <div class="flex items-center gap-2">
            <span class="text-2xl">📁</span>
            <h2 class="font-medium text-white">输出文件</h2>
          </div>
          <button
            @click="$emit('open', 'reportHistory')"
            class="btn btn-ghost btn-sm"
          >
            {{ t('reportHistory.title') }}
          </button>
        </div>

        <!-- Tab Buttons -->
        <div class="flex gap-1 mb-4 p-1 bg-surface-0/50 rounded-xl">
          <button
            @click="activeOutputTab = 'daily'"
            :class="activeOutputTab === 'daily' ? 'bg-surface-2 text-white' : 'text-gray-400 hover:text-white'"
            class="flex-1 py-2 px-4 rounded-lg text-sm font-medium transition-all duration-200"
          >
            {{ t('outputTabs.daily') }}
          </button>
          <button
            @click="activeOutputTab = 'weekly'"
            :class="activeOutputTab === 'weekly' ? 'bg-surface-2 text-white' : 'text-gray-400 hover:text-white'"
            class="flex-1 py-2 px-4 rounded-lg text-sm font-medium transition-all duration-200"
          >
            {{ t('outputTabs.weekly') }}
          </button>
          <button
            @click="activeOutputTab = 'monthly'"
            :class="activeOutputTab === 'monthly' ? 'bg-surface-2 text-white' : 'text-gray-400 hover:text-white'"
            class="flex-1 py-2 px-4 rounded-lg text-sm font-medium transition-all duration-200"
          >
            {{ t('outputTabs.monthly') }}
          </button>
        </div>

        <!-- Tab Content -->
        <div class="min-h-[80px]">
          <!-- Daily Report Tab -->
          <div v-if="activeOutputTab === 'daily'" class="space-y-2">
            <div v-if="summaryPath" class="bg-darker/80 rounded-xl p-3 border border-gray-700/50">
              <p
                @click="$emit('open', 'summaryViewer')"
                class="text-sm text-gray-300 cursor-pointer hover:text-primary hover:underline"
              >{{ summaryPath }}</p>
            </div>
            <EmptyState v-else type="dailyReport">
              {{ t('outputTabs.notGenerated') }}
            </EmptyState>
          </div>

          <!-- Weekly Report Tab -->
          <div v-if="activeOutputTab === 'weekly'" class="space-y-2">
            <div v-if="weeklyReportPath" class="bg-darker/80 rounded-xl p-3 border border-gray-700/50">
              <p
                @click="$emit('open', 'weeklyReportViewer')"
                class="text-sm text-gray-300 cursor-pointer hover:text-green-400 hover:underline"
              >{{ weeklyReportPath }}</p>
            </div>
            <EmptyState v-else type="dailyReport">
              {{ t('outputTabs.notGenerated') }}
            </EmptyState>
          </div>

          <!-- Monthly Report Tab -->
          <div v-if="activeOutputTab === 'monthly'" class="space-y-2">
            <div v-if="monthlyReportPath" class="bg-darker/80 rounded-xl p-3 border border-gray-700/50">
              <p
                @click="$emit('open', 'monthlyReportViewer')"
                class="text-sm text-gray-300 cursor-pointer hover:text-purple-400 hover:underline"
              >{{ monthlyReportPath }}</p>
            </div>
            <EmptyState v-else type="dailyReport">
              {{ t('outputTabs.notGenerated') }}
            </EmptyState>
          </div>
        </div>

        <!-- Additional Reports (Custom & Comparison) -->
        <div v-if="customReportPath || comparisonReportPath" class="mt-4 pt-4 border-t border-gray-700/50 space-y-2">
          <div v-if="customReportPath" class="bg-darker/80 rounded-xl p-3 border border-gray-700/50">
            <p class="text-xs text-gray-500 mb-1">{{ t('outputTabs.custom') }}</p>
            <p
              @click="$emit('open', 'customReportViewer')"
              class="text-sm text-gray-300 cursor-pointer hover:text-orange-400 hover:underline"
            >{{ customReportPath }}</p>
          </div>
          <div v-if="comparisonReportPath" class="bg-darker/80 rounded-xl p-3 border border-gray-700/50">
            <p class="text-xs text-gray-500 mb-1">{{ t('outputTabs.comparison') }}</p>
            <p
              @click="$emit('open', 'comparisonReportViewer')"
              class="text-sm text-gray-300 cursor-pointer hover:text-teal-400 hover:underline"
            >{{ comparisonReportPath }}</p>
          </div>
        </div>
      </div>
    </div>
  </main>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import ReportDropdown from '../ReportDropdown.vue'
import TimelineWidget from '../TimelineWidget.vue'
import TodaySummaryWidget from '../TodaySummaryWidget.vue'
import EmptyState from '../EmptyState.vue'
import SkeletonLoader from '../SkeletonLoader.vue'
import { extractSummary } from '../../utils/contentUtils'
import { getTagColorClass } from '../../utils/tagColors'
import type { LogRecord, Tag } from '../../types/tauri'
import type { ModalId } from '../../composables/useModal'

const { t } = useI18n()

const props = defineProps<{
  isDesktop: boolean
  autoCaptureEnabled: boolean
  isCapturing: boolean
  quickNotesCount: number
  todayRecords: LogRecord[]
  isGenerating: boolean
  isGeneratingWeekly: boolean
  isGeneratingMonthly: boolean
  screenshotCount: number
  summaryPath: string
  weeklyReportPath: string
  monthlyReportPath: string
  customReportPath: string
  comparisonReportPath: string
  isLoading?: boolean
}>()

const emit = defineEmits<{
  open: [modal: ModalId]
  takeScreenshot: []
  triggerCapture: []
  toggleAutoCapture: []
  openQuickNote: []
  generateReport: [type: 'daily' | 'weekly' | 'monthly']
  generateMultilingualReport: [language: string]
  viewScreenshot: [record: LogRecord]
  customAction: [actionId: string]
  languageChange: [language: string]
}>()

// Tag filtering state
const selectedTagFilter = ref('')
const TAG_VISIBLE_THRESHOLD = 6
const tagFilterExpanded = ref(false)
const timelineWidgetRef = ref<InstanceType<typeof TimelineWidget> | null>(null)
const todaySummaryWidgetRef = ref<InstanceType<typeof TodaySummaryWidget> | null>(null)

// Pagination state (UX-4: Task 2)
const currentPage = ref(1)
const PAGE_SIZE = 20

// Output files tab state (UX-4: Task 3)
type OutputTab = 'daily' | 'weekly' | 'monthly'
const activeOutputTab = ref<OutputTab>('daily')

// Refresh TimelineWidget and TodaySummaryWidget when todayRecords changes (AC 3.3: real-time update)
watch(() => props.todayRecords, (newRecords, oldRecords) => {
  // Only refresh if the record count actually changed
  if (newRecords.length !== oldRecords?.length) {
    if (timelineWidgetRef.value) {
      timelineWidgetRef.value.refresh()
    }
    if (todaySummaryWidgetRef.value) {
      todaySummaryWidgetRef.value.refresh()
    }
  }
}, { deep: false })

// Computed
const tagCounts = computed<Record<string, number>>(() => {
  const counts: Record<string, number> = {}
  props.todayRecords.forEach(record => {
    const tags = getRecordTags(record)
    tags.forEach(tag => {
      counts[tag] = (counts[tag] || 0) + 1
    })
  })
  return counts
})

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

const filteredRecords = computed<LogRecord[]>(() => {
  if (!selectedTagFilter.value) {
    return props.todayRecords
  }
  return props.todayRecords.filter(record => {
    const tags = getRecordTags(record)
    return tags.includes(selectedTagFilter.value)
  })
})

// Pagination computed (UX-4: Task 2 - remove max-h-80, add pagination)
const totalPages = computed(() => Math.ceil(filteredRecords.value.length / PAGE_SIZE))
const hasMore = computed(() => currentPage.value < totalPages.value)
const paginatedRecords = computed(() => {
  const start = 0
  const end = currentPage.value * PAGE_SIZE
  return filteredRecords.value.slice(start, end)
})

// Reset pagination when filter changes
watch(selectedTagFilter, () => {
  currentPage.value = 1
})

const loadMore = () => {
  if (hasMore.value) {
    currentPage.value++
  }
}

// Methods
const formatTime = (timestamp: string): string => {
  const date = new Date(timestamp)
  if (isNaN(date.getTime())) return '--:--'
  const h = date.getHours().toString().padStart(2, '0')
  const m = date.getMinutes().toString().padStart(2, '0')
  return `${h}:${m}`
}

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

const getWindowIcon = (processName?: string): string => {
  if (!processName) return '🖥️'
  const name = processName.toLowerCase()

  if (name.includes('code') || name.includes('vscode')) return '💻'
  if (name.includes('idea') || name.includes('intellij')) return '☕'
  if (name.includes('atom') || name.includes('sublime')) return '📝'
  if (name.includes('chrome')) return '🌐'
  if (name.includes('firefox')) return '🦊'
  if (name.includes('edge') || name.includes('msedge')) return '🌊'
  if (name.includes('safari')) return '🧭'
  if (name.includes('slack') || name.includes('discord') || name.includes('teams')) return '💬'
  if (name.includes('wechat') || name.includes('微信')) return '💬'
  if (name.includes('terminal') || name.includes('cmd') || name.includes('bash') || name.includes('powershell')) return '⌨️'
  if (name.includes('word') || name.includes('excel') || name.includes('powerpoint')) return '📊'
  return '🖥️'
}

const getTagColor = (tag: string): string => {
  return getTagColorClass(tag)
}

const getRecordTags = (record: LogRecord): string[] => {
  if ((record as LogRecord & { tags?: string }).tags) {
    try {
      const tags = JSON.parse((record as LogRecord & { tags?: string }).tags as string)
      if (Array.isArray(tags) && tags.length > 0) {
        return tags.slice(0, 3)
      }
    } catch {
      // Ignore parse errors
    }
  }
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

const openScreenshot = (record: LogRecord) => {
  emit('viewScreenshot', record)
}

const handleReportGenerate = (type: 'daily' | 'weekly' | 'monthly') => {
  emit('generateReport', type)
}

const handleGenerateMultilingualReport = (language: string) => {
  emit('generateMultilingualReport', language)
}

const handleLanguageChange = (language: string) => {
  emit('languageChange', language)
}

const handleCustomAction = (actionId: string) => {
  if (actionId === 'reanalyzeToday') {
    emit('customAction', 'reanalyzeToday')
  }
}
</script>