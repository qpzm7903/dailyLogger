<template>
  <div class="fixed inset-0 bg-black/80 flex items-center justify-center z-50" @click.self="$emit('close')">
    <div class="bg-[var(--color-surface-1)] rounded-2xl w-[480px] max-h-[80vh] overflow-hidden border border-[var(--color-border)]">
      <!-- Header -->
      <div class="px-6 py-4 border-b border-[var(--color-border)] flex items-center justify-between">
        <h2 class="text-lg font-semibold">{{ t('reanalyze.title') }}</h2>
        <button @click="$emit('close')" class="text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]">✕</button>
      </div>

      <!-- Content -->
      <div class="p-6">
        <!-- Date Selection -->
        <div class="mb-6">
          <label class="text-sm text-[var(--color-text-secondary)] block mb-2">{{ t('reanalyze.selectDate') }}</label>
          <input
            v-model="selectedDate"
            type="date"
            :max="todayDate"
            class="w-full bg-[var(--color-surface-0)] border border-[var(--color-border)] rounded-lg px-4 py-3 text-sm text-[var(--color-text-primary)] focus:border-primary focus:outline-none"
          />
          <p class="text-xs text-[var(--color-text-muted)] mt-2">{{ t('reanalyze.dateHint') }}</p>
        </div>

        <!-- Preview Info -->
        <div v-if="selectedDate" class="bg-[var(--color-surface-0)] rounded-lg p-4 mb-4">
          <h3 class="text-sm font-medium text-[var(--color-text-secondary)] mb-2">{{ t('reanalyze.preview') }}</h3>
          <div class="text-xs text-[var(--color-text-secondary)] space-y-1">
            <p>{{ t('reanalyze.selectedDate') }}: {{ selectedDate }}</p>
            <p v-if="isLoadingCount" class="text-[var(--color-text-muted)]">{{ t('reanalyze.loadingCount') }}</p>
            <p v-else-if="recordCount !== null">
              {{ t('reanalyze.recordCount', { count: recordCount }) }}
            </p>
          </div>
        </div>

        <!-- Result Display -->
        <div v-if="result" class="bg-[var(--color-surface-0)] rounded-lg p-4 mb-4">
          <h4 class="text-sm font-medium mb-2" :class="result.failed > 0 ? 'text-yellow-400' : 'text-green-400'">
            {{ t('reanalyze.complete') }}
          </h4>
          <div class="text-xs text-[var(--color-text-secondary)] space-y-1">
            <p>{{ t('reanalyze.totalRecords') }}: {{ result.total }}</p>
            <p class="text-green-400">{{ t('reanalyze.successCount') }}: {{ result.success }}</p>
            <p v-if="result.failed > 0" class="text-red-400">{{ t('reanalyze.failedCount') }}: {{ result.failed }}</p>
          </div>
          <div v-if="result.errors && result.errors.length > 0" class="mt-3">
            <p class="text-xs text-[var(--color-text-secondary)] mb-1">{{ t('reanalyze.errors') }}:</p>
            <ul class="text-xs text-red-400 space-y-1 max-h-24 overflow-y-auto">
              <li v-for="(error, idx) in result.errors.slice(0, 5)" :key="idx">{{ error }}</li>
              <li v-if="result.errors.length > 5" class="text-[var(--color-text-muted)]">
                {{ t('reanalyze.moreErrors', { count: result.errors.length - 5 }) }}
              </li>
            </ul>
          </div>
        </div>

        <!-- Error Display -->
        <div v-if="error" class="bg-red-900/30 border border-red-700 rounded-lg p-4 mb-4">
          <p class="text-sm text-red-400">{{ error }}</p>
        </div>

        <!-- Actions -->
        <div class="flex justify-end gap-3">
          <button
            @click="$emit('close')"
            :disabled="isReanalyzing"
            class="px-4 py-2 bg-[var(--color-action-secondary)] hover:bg-[var(--color-action-neutral)] disabled:opacity-50 rounded-lg text-sm transition-colors"
          >
            {{ t('common.close') }}
          </button>
          <button
            @click="handleReanalyze"
            :disabled="!selectedDate || isReanalyzing"
            class="px-4 py-2 bg-primary hover:bg-primary-hover disabled:opacity-50 disabled:cursor-not-allowed rounded-lg text-sm font-medium transition-colors"
          >
            {{ isReanalyzing ? t('reanalyze.processing') : t('reanalyze.start') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import { showToast } from '../stores/toast'

interface ReanalyzeResult {
  total: number
  success: number
  failed: number
  errors: string[]
}

const { t } = useI18n()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'reanalyzed'): void
}>()

const selectedDate = ref('')
const todayDate = ref('')
const isReanalyzing = ref(false)
const isLoadingCount = ref(false)
const recordCount = ref<number | null>(null)
const result = ref<ReanalyzeResult | null>(null)
const error = ref('')

onMounted(() => {
  // Set today's date as max and default
  const today = new Date()
  todayDate.value = today.toISOString().split('T')[0]
  selectedDate.value = todayDate.value
})

// Watch for date changes to preview record count
watch(selectedDate, async (newDate) => {
  if (!newDate) {
    recordCount.value = null
    return
  }

  isLoadingCount.value = true
  recordCount.value = null

  try {
    const records = await invoke<{ id: number; screenshot_path: string | null }[]>('get_records_by_date_range', {
      startDate: newDate,
      endDate: newDate
    })
    // Count records with screenshots
    recordCount.value = records.filter(r => r.screenshot_path).length
  } catch (e) {
    console.error('Failed to fetch record count:', e)
  } finally {
    isLoadingCount.value = false
  }
})

const handleReanalyze = async () => {
  if (!selectedDate.value || isReanalyzing.value) return

  isReanalyzing.value = true
  error.value = ''
  result.value = null

  try {
    const res = await invoke<ReanalyzeResult>('reanalyze_records_by_date', {
      date: selectedDate.value
    })

    result.value = res

    if (res.success > 0) {
      showToast(t('reanalyze.successMessage', { count: res.success }), { type: 'success' })
      emit('reanalyzed')
    }

    if (res.failed > 0 && res.success === 0) {
      showToast(t('reanalyze.allFailed'), { type: 'error' })
    }
  } catch (e) {
    error.value = String(e)
    showToast(t('reanalyze.errorMessage'), { type: 'error' })
  } finally {
    isReanalyzing.value = false
  }
}
</script>