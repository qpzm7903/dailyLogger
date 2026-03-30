<template>
  <div class="fixed inset-0 bg-black/80 flex items-center justify-center z-50" @click.self="$emit('close')">
    <div class="bg-[var(--color-surface-1)] rounded-2xl w-[90vw] max-w-lg overflow-hidden border border-[var(--color-border)] flex flex-col">
      <!-- Header -->
      <div class="px-6 py-4 border-b border-[var(--color-border)] flex items-center justify-between">
        <h2 class="text-lg font-semibold">{{ t('customReport.title') }}</h2>
        <button @click="$emit('close')" class="text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]">✕</button>
      </div>

      <!-- Body -->
      <div class="p-6 space-y-5">
        <!-- Preset Buttons -->
        <div class="space-y-3">
          <label class="text-sm text-[var(--color-text-secondary)] block">{{ t('customReport.presets') }}</label>
          <div class="flex gap-3">
            <button
              @click="applyPreset('biweekly')"
              :class="activePreset === 'biweekly' ? 'border-primary bg-primary/10 text-[var(--color-text-primary)]' : 'border-[var(--color-border-subtle)] text-[var(--color-text-secondary)] hover:border-[var(--color-border)]'"
              class="flex-1 border rounded-lg px-4 py-3 text-sm transition-colors text-center"
            >
              <div class="font-medium">{{ t('customReport.biweekly') }}</div>
              <div class="text-xs mt-1 opacity-60">{{ t('customReport.biweeklyDescription') }}</div>
            </button>
            <button
              @click="applyPreset('quarterly')"
              :class="activePreset === 'quarterly' ? 'border-primary bg-primary/10 text-[var(--color-text-primary)]' : 'border-[var(--color-border-subtle)] text-[var(--color-text-secondary)] hover:border-[var(--color-border)]'"
              class="flex-1 border rounded-lg px-4 py-3 text-sm transition-colors text-center"
            >
              <div class="font-medium">{{ t('customReport.quarterly') }}</div>
              <div class="text-xs mt-1 opacity-60">{{ t('customReport.quarterlyDescription') }}</div>
            </button>
            <button
              @click="applyPreset('custom')"
              :class="activePreset === 'custom' ? 'border-primary bg-primary/10 text-[var(--color-text-primary)]' : 'border-[var(--color-border-subtle)] text-[var(--color-text-secondary)] hover:border-[var(--color-border)]'"
              class="flex-1 border rounded-lg px-4 py-3 text-sm transition-colors text-center"
            >
              <div class="font-medium">{{ t('customReport.custom') }}</div>
              <div class="text-xs mt-1 opacity-60">{{ t('customReport.customDescription') }}</div>
            </button>
          </div>
        </div>

        <!-- Date Range -->
        <div class="space-y-3">
          <label class="text-sm text-[var(--color-text-secondary)] block">{{ t('customReport.dateRange') }}</label>
          <div class="flex items-center gap-3">
            <input
              type="date"
              v-model="startDate"
              @change="activePreset = 'custom'"
              class="flex-1 bg-[var(--color-surface-0)] border border-[var(--color-border-subtle)] rounded-lg px-3 py-2 text-sm text-[var(--color-text-primary)] focus:border-primary focus:outline-none"
            />
            <span class="text-[var(--color-text-muted)]">{{ t('customReport.to') }}</span>
            <input
              type="date"
              v-model="endDate"
              @change="activePreset = 'custom'"
              class="flex-1 bg-[var(--color-surface-0)] border border-[var(--color-border-subtle)] rounded-lg px-3 py-2 text-sm text-[var(--color-text-primary)] focus:border-primary focus:outline-none"
            />
          </div>
          <p v-if="dayCount > 0" class="text-xs text-[var(--color-text-muted)]">{{ t('customReport.daysSelected', { count: dayCount }) }}</p>
          <p v-if="dateError" class="text-red-400 text-xs">{{ dateError }}</p>
        </div>

        <!-- Report Name -->
        <div class="space-y-3">
          <label class="text-sm text-[var(--color-text-secondary)] block">{{ t('customReport.reportName') }}</label>
          <input
            v-model="reportName"
            :placeholder="t('customReport.reportNamePlaceholder')"
            class="w-full bg-[var(--color-surface-0)] border border-[var(--color-border-subtle)] rounded-lg px-3 py-2 text-sm text-[var(--color-text-primary)] focus:border-primary focus:outline-none"
          />
        </div>

        <!-- Result -->
        <div v-if="resultPath" class="bg-[var(--color-surface-0)] rounded-lg p-4 space-y-2 border border-green-700/50">
          <div class="flex items-center gap-2 text-green-400 text-sm">
            <span>{{ t('customReport.reportSuccess') }}</span>
          </div>
          <p class="text-xs text-[var(--color-text-secondary)] break-all">{{ resultPath }}</p>
        </div>

        <!-- Error -->
        <div v-if="errorMsg" class="bg-red-900/20 border border-red-700 rounded-lg p-3">
          <p class="text-red-400 text-sm">{{ errorMsg }}</p>
        </div>
      </div>

      <!-- Footer -->
      <div class="px-6 py-4 border-t border-[var(--color-border)] flex justify-end gap-3">
        <button
          @click="$emit('close')"
          class="px-4 py-2 text-sm text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)] transition-colors"
        >
          {{ t('customReport.close') }}
        </button>
        <button
          @click="generateReport"
          :disabled="isGenerating || !!dateError || !startDate || !endDate"
          class="bg-primary hover:bg-primary-hover disabled:opacity-50 px-5 py-2 rounded-lg text-sm font-medium transition-colors"
        >
          {{ isGenerating ? t('customReport.generating') : t('customReport.generate') }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { showError, showSuccess } from '../stores/toast'

const { t } = useI18n()
const emit = defineEmits<{(e: 'close'): void; (e: 'generated', path: string): void}>()

const startDate = ref('')
const endDate = ref('')
const reportName = ref('')
const activePreset = ref('')
const isGenerating = ref(false)
const resultPath = ref('')
const errorMsg = ref('')

const dayCount = computed<number>(() => {
  if (!startDate.value || !endDate.value) return 0
  const start = new Date(startDate.value)
  const end = new Date(endDate.value)
  if (isNaN(start.getTime()) || isNaN(end.getTime())) return 0
  const diff = Math.floor((end.getTime() - start.getTime()) / (1000 * 60 * 60 * 24)) + 1
  return diff > 0 ? diff : 0
})

const dateError = computed<string>(() => {
  if (!startDate.value || !endDate.value) return ''
  if (new Date(endDate.value) < new Date(startDate.value)) {
    return t('customReport.endDateBeforeStart')
  }
  return ''
})

const applyPreset = (preset: 'biweekly' | 'quarterly' | 'custom') => {
  activePreset.value = preset
  errorMsg.value = ''
  resultPath.value = ''

  const today = new Date()
  const formatDate = (d: Date) => d.toISOString().split('T')[0]

  if (preset === 'biweekly') {
    const start = new Date(today)
    start.setDate(today.getDate() - 13)
    startDate.value = formatDate(start)
    endDate.value = formatDate(today)
    reportName.value = '双周报'
  } else if (preset === 'quarterly') {
    const month = today.getMonth() // 0-indexed
    const quarterStartMonth = Math.floor(month / 3) * 3
    const start = new Date(today.getFullYear(), quarterStartMonth, 1)
    const end = new Date(today.getFullYear(), quarterStartMonth + 3, 0) // last day of quarter
    startDate.value = formatDate(start)
    endDate.value = formatDate(end)
    reportName.value = '季度报'
  } else {
    reportName.value = ''
  }
}

const generateReport = async () => {
  if (isGenerating.value || dateError.value) return
  isGenerating.value = true
  errorMsg.value = ''
  resultPath.value = ''

  try {
    const result = await invoke<string>('generate_custom_report', {
      startDate: startDate.value,
      endDate: endDate.value,
      reportName: reportName.value || null,
    })
    resultPath.value = result
    showSuccess(t('customReport.reportSuccess'))
    emit('generated', result)
  } catch (err) {
    console.error('Failed to generate custom report:', err)
    errorMsg.value = typeof err === 'string' ? err : String(err)
    showError(String(err), generateReport)
  } finally {
    isGenerating.value = false
  }
}
</script>
