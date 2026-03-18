<template>
  <div class="fixed inset-0 bg-black/80 flex items-center justify-center z-50" @click.self="$emit('close')">
    <div class="bg-dark rounded-2xl w-[90vw] max-w-lg overflow-hidden border border-gray-700 flex flex-col">
      <!-- Header -->
      <div class="px-6 py-4 border-b border-gray-700 flex items-center justify-between">
        <h2 class="text-lg font-semibold">{{ t('exportModal.title') }}</h2>
        <button @click="$emit('close')" class="text-gray-400 hover:text-white">✕</button>
      </div>

      <!-- Body -->
      <div class="p-6 space-y-5">
        <!-- Date Range -->
        <div class="space-y-3">
          <label class="text-sm text-gray-400 block">{{ t('exportModal.dateRange') }}</label>
          <div class="flex items-center gap-3">
            <input
              type="date"
              v-model="startDate"
              class="flex-1 bg-darker border border-gray-600 rounded-lg px-3 py-2 text-sm text-white focus:border-primary focus:outline-none"
            />
            <span class="text-gray-500">{{ t('exportModal.to') }}</span>
            <input
              type="date"
              v-model="endDate"
              class="flex-1 bg-darker border border-gray-600 rounded-lg px-3 py-2 text-sm text-white focus:border-primary focus:outline-none"
            />
          </div>
          <p v-if="dateError" class="text-red-400 text-xs">{{ dateError }}</p>
        </div>

        <!-- Format Selection -->
        <div class="space-y-3">
          <label class="text-sm text-gray-400 block">{{ t('exportModal.exportFormat') }}</label>
          <div class="flex gap-3">
            <button
              @click="exportFormat = 'json'"
              :class="exportFormat === 'json' ? 'border-primary bg-primary/10 text-white' : 'border-gray-600 text-gray-400 hover:border-gray-500'"
              class="flex-1 border rounded-lg px-4 py-3 text-sm transition-colors text-center"
            >
              <div class="font-medium">{{ t('exportModal.jsonFormat') }}</div>
              <div class="text-xs mt-1 opacity-60">{{ t('exportModal.jsonDescription') }}</div>
            </button>
            <button
              @click="exportFormat = 'markdown'"
              :class="exportFormat === 'markdown' ? 'border-primary bg-primary/10 text-white' : 'border-gray-600 text-gray-400 hover:border-gray-500'"
              class="flex-1 border rounded-lg px-4 py-3 text-sm transition-colors text-center"
            >
              <div class="font-medium">{{ t('exportModal.markdownFormat') }}</div>
              <div class="text-xs mt-1 opacity-60">{{ t('exportModal.markdownDescription') }}</div>
            </button>
          </div>
        </div>

        <!-- Export Result -->
        <div v-if="exportResult" class="bg-darker rounded-lg p-4 space-y-2 border border-gray-600">
          <div class="flex items-center gap-2 text-green-400 text-sm">
            <span>{{ t('exportModal.exportSuccess') }}</span>
          </div>
          <div class="text-xs text-gray-400 space-y-1">
            <p>{{ t('exportModal.recordCount', { count: exportResult.record_count }) }}</p>
            <p>{{ t('exportModal.fileSize', { size: formatFileSize(exportResult.file_size) }) }}</p>
            <p class="break-all">{{ t('exportModal.path', { path: exportResult.path }) }}</p>
          </div>
          <button
            @click="openExportDir"
            class="text-xs text-primary hover:underline mt-1"
          >
            {{ t('exportModal.openDirectory') }}
          </button>
        </div>

        <!-- Export Error -->
        <div v-if="exportError" class="bg-red-900/20 border border-red-700 rounded-lg p-3">
          <p class="text-red-400 text-sm">{{ exportError }}</p>
        </div>
      </div>

      <!-- Footer -->
      <div class="px-6 py-4 border-t border-gray-700 flex justify-end gap-3">
        <button
          @click="$emit('close')"
          class="px-4 py-2 text-sm text-gray-400 hover:text-white transition-colors"
        >
          {{ t('exportModal.close') }}
        </button>
        <button
          @click="doExport"
          :disabled="isExporting || !!dateError"
          class="px-5 py-2 bg-primary hover:bg-blue-600 disabled:opacity-50 rounded-lg text-sm font-medium transition-colors"
        >
          {{ isExporting ? t('exportModal.exporting') : t('exportModal.export') }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
const emit = defineEmits(['close'])

// Date range - default to last 7 days
const today = new Date()
const weekAgo = new Date(today)
weekAgo.setDate(weekAgo.getDate() - 7)

const startDate = ref(formatDate(weekAgo))
const endDate = ref(formatDate(today))
const exportFormat = ref('json')
const isExporting = ref(false)
const exportResult = ref(null)
const exportError = ref('')

const dateError = computed(() => {
  if (!startDate.value || !endDate.value) return t('exportModal.selectDateRange')
  if (startDate.value > endDate.value) return t('exportModal.startDateAfterEnd')
  return ''
})

function formatDate(date) {
  const y = date.getFullYear()
  const m = String(date.getMonth() + 1).padStart(2, '0')
  const d = String(date.getDate()).padStart(2, '0')
  return `${y}-${m}-${d}`
}

function formatFileSize(bytes) {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

async function doExport() {
  if (dateError.value) return

  isExporting.value = true
  exportResult.value = null
  exportError.value = ''

  try {
    const result = await invoke('export_records', {
      request: {
        start_date: startDate.value,
        end_date: endDate.value,
        format: exportFormat.value,
      }
    })
    exportResult.value = result
  } catch (e) {
    exportError.value = typeof e === 'string' ? e : e.message || t('exportModal.exportFailed')
  } finally {
    isExporting.value = false
  }
}

async function openExportDir() {
  if (!exportResult.value?.path) return
  try {
    await invoke('open_export_dir', { path: exportResult.value.path })
  } catch (err) {
    console.error('Failed to open export directory:', err)
  }
}
</script>
