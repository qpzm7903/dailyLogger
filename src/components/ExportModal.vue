<template>
  <div class="fixed inset-0 bg-black/80 flex items-center justify-center z-50" @click.self="$emit('close')">
    <div class="bg-dark rounded-2xl w-[90vw] max-w-lg overflow-hidden border border-gray-700 flex flex-col">
      <!-- Header -->
      <div class="px-6 py-4 border-b border-gray-700 flex items-center justify-between">
        <h2 class="text-lg font-semibold">数据导出</h2>
        <button @click="$emit('close')" class="text-gray-400 hover:text-white">✕</button>
      </div>

      <!-- Body -->
      <div class="p-6 space-y-5">
        <!-- Date Range -->
        <div class="space-y-3">
          <label class="text-sm text-gray-400 block">日期范围</label>
          <div class="flex items-center gap-3">
            <input
              type="date"
              v-model="startDate"
              class="flex-1 bg-darker border border-gray-600 rounded-lg px-3 py-2 text-sm text-white focus:border-primary focus:outline-none"
            />
            <span class="text-gray-500">至</span>
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
          <label class="text-sm text-gray-400 block">导出格式</label>
          <div class="flex gap-3">
            <button
              @click="exportFormat = 'json'"
              :class="exportFormat === 'json' ? 'border-primary bg-primary/10 text-white' : 'border-gray-600 text-gray-400 hover:border-gray-500'"
              class="flex-1 border rounded-lg px-4 py-3 text-sm transition-colors text-center"
            >
              <div class="font-medium">JSON</div>
              <div class="text-xs mt-1 opacity-60">结构化数据，适合分析</div>
            </button>
            <button
              @click="exportFormat = 'markdown'"
              :class="exportFormat === 'markdown' ? 'border-primary bg-primary/10 text-white' : 'border-gray-600 text-gray-400 hover:border-gray-500'"
              class="flex-1 border rounded-lg px-4 py-3 text-sm transition-colors text-center"
            >
              <div class="font-medium">Markdown</div>
              <div class="text-xs mt-1 opacity-60">可读文档，适合归档</div>
            </button>
          </div>
        </div>

        <!-- Export Result -->
        <div v-if="exportResult" class="bg-darker rounded-lg p-4 space-y-2 border border-gray-600">
          <div class="flex items-center gap-2 text-green-400 text-sm">
            <span>导出成功</span>
          </div>
          <div class="text-xs text-gray-400 space-y-1">
            <p>记录数: {{ exportResult.record_count }} 条</p>
            <p>文件大小: {{ formatFileSize(exportResult.file_size) }}</p>
            <p class="break-all">路径: {{ exportResult.path }}</p>
          </div>
          <button
            @click="openExportDir"
            class="text-xs text-primary hover:underline mt-1"
          >
            打开所在目录
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
          关闭
        </button>
        <button
          @click="doExport"
          :disabled="isExporting || !!dateError"
          class="px-5 py-2 bg-primary hover:bg-blue-600 disabled:opacity-50 rounded-lg text-sm font-medium transition-colors"
        >
          {{ isExporting ? '导出中...' : '导出' }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

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
  if (!startDate.value || !endDate.value) return '请选择日期范围'
  if (startDate.value > endDate.value) return '开始日期不能晚于结束日期'
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
    exportError.value = typeof e === 'string' ? e : e.message || '导出失败'
  } finally {
    isExporting.value = false
  }
}

async function openExportDir() {
  if (!exportResult.value) return
  try {
    // Extract directory from the file path
    const path = exportResult.value.path
    const dir = path.substring(0, path.lastIndexOf('/')) || path.substring(0, path.lastIndexOf('\\'))
    await invoke('plugin:shell|open', { path: dir })
  } catch (e) {
    console.error('Failed to open directory:', e)
  }
}
</script>
