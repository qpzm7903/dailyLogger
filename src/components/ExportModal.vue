<template>
  <div class="fixed inset-0 bg-black/80 flex items-center justify-center z-50" @click.self="$emit('close')">
    <div class="bg-dark rounded-2xl w-[500px] max-h-[80vh] overflow-hidden border border-gray-700 flex flex-col">
      <!-- Header -->
      <div class="px-6 py-4 border-b border-gray-700 flex items-center justify-between">
        <h2 class="text-lg font-semibold">数据导出</h2>
        <button @click="$emit('close')" class="text-gray-400 hover:text-white">✕</button>
      </div>

      <!-- Form -->
      <div class="px-6 py-5 space-y-5">
        <!-- Date Range -->
        <div class="space-y-3">
          <label class="text-sm font-medium text-gray-300">日期范围</label>
          <div class="flex items-center gap-3">
            <div class="flex-1">
              <label class="text-xs text-gray-500 mb-1 block">开始日期</label>
              <input
                type="date"
                v-model="startDate"
                class="w-full bg-darker border border-gray-600 rounded-lg px-3 py-2 text-sm text-white focus:border-primary focus:outline-none"
              />
            </div>
            <span class="text-gray-500 mt-5">至</span>
            <div class="flex-1">
              <label class="text-xs text-gray-500 mb-1 block">结束日期</label>
              <input
                type="date"
                v-model="endDate"
                class="w-full bg-darker border border-gray-600 rounded-lg px-3 py-2 text-sm text-white focus:border-primary focus:outline-none"
              />
            </div>
          </div>
          <p v-if="dateError" class="text-xs text-red-400">{{ dateError }}</p>
        </div>

        <!-- Format Selection -->
        <div class="space-y-3">
          <label class="text-sm font-medium text-gray-300">导出格式</label>
          <div class="flex gap-3">
            <button
              @click="format = 'json'"
              :class="format === 'json' ? 'border-primary bg-primary/10 text-primary' : 'border-gray-600 text-gray-400 hover:border-gray-500'"
              class="flex-1 border rounded-lg p-3 transition-colors text-left"
            >
              <div class="text-sm font-medium">JSON</div>
              <div class="text-xs mt-1 opacity-70">结构化数据，适合程序处理</div>
            </button>
            <button
              @click="format = 'markdown'"
              :class="format === 'markdown' ? 'border-primary bg-primary/10 text-primary' : 'border-gray-600 text-gray-400 hover:border-gray-500'"
              class="flex-1 border rounded-lg p-3 transition-colors text-left"
            >
              <div class="text-sm font-medium">Markdown</div>
              <div class="text-xs mt-1 opacity-70">可读性强，适合文档归档</div>
            </button>
          </div>
        </div>
      </div>

      <!-- Actions -->
      <div class="px-6 py-4 border-t border-gray-700 flex items-center justify-between">
        <button
          @click="$emit('close')"
          class="px-4 py-2 text-sm text-gray-400 hover:text-white transition-colors"
        >
          取消
        </button>
        <button
          @click="doExport"
          :disabled="!canExport || isExporting"
          class="px-5 py-2 bg-primary hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed rounded-lg text-sm font-medium transition-colors"
        >
          {{ isExporting ? '导出中...' : '导出' }}
        </button>
      </div>

      <!-- Result -->
      <div v-if="exportResult" class="px-6 py-4 border-t border-gray-700 bg-green-500/5">
        <div class="flex items-start gap-3">
          <span class="text-green-400 text-lg">✓</span>
          <div class="flex-1 min-w-0">
            <p class="text-sm text-green-400 font-medium">导出成功</p>
            <p class="text-xs text-gray-400 mt-1 truncate" :title="exportResult.path">{{ exportResult.path }}</p>
            <p class="text-xs text-gray-500 mt-0.5">{{ exportResult.record_count }} 条记录 · {{ formatFileSize(exportResult.file_size) }}</p>
          </div>
        </div>
      </div>

      <!-- Error -->
      <div v-if="exportError" class="px-6 py-4 border-t border-gray-700 bg-red-500/5">
        <div class="flex items-start gap-3">
          <span class="text-red-400 text-lg">✕</span>
          <div class="flex-1">
            <p class="text-sm text-red-400 font-medium">导出失败</p>
            <p class="text-xs text-gray-400 mt-1">{{ exportError }}</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

defineEmits(['close'])

// Default: last 7 days
const today = new Date()
const weekAgo = new Date(today)
weekAgo.setDate(weekAgo.getDate() - 6)

const startDate = ref(formatDate(weekAgo))
const endDate = ref(formatDate(today))
const format = ref('json')
const isExporting = ref(false)
const exportResult = ref(null)
const exportError = ref(null)

function formatDate(date) {
  const y = date.getFullYear()
  const m = String(date.getMonth() + 1).padStart(2, '0')
  const d = String(date.getDate()).padStart(2, '0')
  return `${y}-${m}-${d}`
}

const dateError = computed(() => {
  if (!startDate.value || !endDate.value) return null
  if (startDate.value > endDate.value) return '开始日期不能晚于结束日期'
  return null
})

const canExport = computed(() => {
  return startDate.value && endDate.value && !dateError.value
})

function formatFileSize(bytes) {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

async function doExport() {
  if (!canExport.value || isExporting.value) return

  isExporting.value = true
  exportResult.value = null
  exportError.value = null

  try {
    const result = await invoke('export_records', {
      request: {
        start_date: startDate.value,
        end_date: endDate.value,
        format: format.value,
      }
    })
    exportResult.value = result
  } catch (err) {
    exportError.value = typeof err === 'string' ? err : err.message || '导出失败'
  } finally {
    isExporting.value = false
  }
}
</script>
