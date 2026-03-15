<template>
  <div class="fixed inset-0 bg-black/80 flex items-center justify-center z-50" @click.self="$emit('close')">
    <div class="bg-dark rounded-2xl w-[90vw] max-w-lg overflow-hidden border border-gray-700 flex flex-col">
      <!-- Header -->
      <div class="px-6 py-4 border-b border-gray-700 flex items-center justify-between">
        <h2 class="text-lg font-semibold">自定义报告</h2>
        <button @click="$emit('close')" class="text-gray-400 hover:text-white">✕</button>
      </div>

      <!-- Body -->
      <div class="p-6 space-y-5">
        <!-- Preset Buttons -->
        <div class="space-y-3">
          <label class="text-sm text-gray-400 block">快捷预设</label>
          <div class="flex gap-3">
            <button
              @click="applyPreset('biweekly')"
              :class="activePreset === 'biweekly' ? 'border-primary bg-primary/10 text-white' : 'border-gray-600 text-gray-400 hover:border-gray-500'"
              class="flex-1 border rounded-lg px-4 py-3 text-sm transition-colors text-center"
            >
              <div class="font-medium">双周报</div>
              <div class="text-xs mt-1 opacity-60">最近 14 天</div>
            </button>
            <button
              @click="applyPreset('quarterly')"
              :class="activePreset === 'quarterly' ? 'border-primary bg-primary/10 text-white' : 'border-gray-600 text-gray-400 hover:border-gray-500'"
              class="flex-1 border rounded-lg px-4 py-3 text-sm transition-colors text-center"
            >
              <div class="font-medium">季度报</div>
              <div class="text-xs mt-1 opacity-60">当前季度</div>
            </button>
            <button
              @click="applyPreset('custom')"
              :class="activePreset === 'custom' ? 'border-primary bg-primary/10 text-white' : 'border-gray-600 text-gray-400 hover:border-gray-500'"
              class="flex-1 border rounded-lg px-4 py-3 text-sm transition-colors text-center"
            >
              <div class="font-medium">自定义</div>
              <div class="text-xs mt-1 opacity-60">任意日期</div>
            </button>
          </div>
        </div>

        <!-- Date Range -->
        <div class="space-y-3">
          <label class="text-sm text-gray-400 block">日期范围</label>
          <div class="flex items-center gap-3">
            <input
              type="date"
              v-model="startDate"
              @change="activePreset = 'custom'"
              class="flex-1 bg-darker border border-gray-600 rounded-lg px-3 py-2 text-sm text-white focus:border-primary focus:outline-none"
            />
            <span class="text-gray-500">至</span>
            <input
              type="date"
              v-model="endDate"
              @change="activePreset = 'custom'"
              class="flex-1 bg-darker border border-gray-600 rounded-lg px-3 py-2 text-sm text-white focus:border-primary focus:outline-none"
            />
          </div>
          <p v-if="dayCount > 0" class="text-xs text-gray-500">已选择 {{ dayCount }} 天</p>
          <p v-if="dateError" class="text-red-400 text-xs">{{ dateError }}</p>
        </div>

        <!-- Report Name -->
        <div class="space-y-3">
          <label class="text-sm text-gray-400 block">报告名称（可选）</label>
          <input
            v-model="reportName"
            placeholder="默认：自定义报告"
            class="w-full bg-darker border border-gray-600 rounded-lg px-3 py-2 text-sm text-white focus:border-primary focus:outline-none"
          />
        </div>

        <!-- Result -->
        <div v-if="resultPath" class="bg-darker rounded-lg p-4 space-y-2 border border-green-700/50">
          <div class="flex items-center gap-2 text-green-400 text-sm">
            <span>报告生成成功</span>
          </div>
          <p class="text-xs text-gray-400 break-all">{{ resultPath }}</p>
        </div>

        <!-- Error -->
        <div v-if="errorMsg" class="bg-red-900/20 border border-red-700 rounded-lg p-3">
          <p class="text-red-400 text-sm">{{ errorMsg }}</p>
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
          @click="generateReport"
          :disabled="isGenerating || !!dateError || !startDate || !endDate"
          class="bg-primary hover:bg-blue-600 disabled:opacity-50 px-5 py-2 rounded-lg text-sm font-medium transition-colors"
        >
          {{ isGenerating ? '生成中...' : '生成报告' }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { showError, showSuccess } from '../stores/toast.js'

const emit = defineEmits(['close', 'generated'])

const startDate = ref('')
const endDate = ref('')
const reportName = ref('')
const activePreset = ref('')
const isGenerating = ref(false)
const resultPath = ref('')
const errorMsg = ref('')

const dayCount = computed(() => {
  if (!startDate.value || !endDate.value) return 0
  const start = new Date(startDate.value)
  const end = new Date(endDate.value)
  if (isNaN(start.getTime()) || isNaN(end.getTime())) return 0
  const diff = Math.floor((end - start) / (1000 * 60 * 60 * 24)) + 1
  return diff > 0 ? diff : 0
})

const dateError = computed(() => {
  if (!startDate.value || !endDate.value) return ''
  if (new Date(endDate.value) < new Date(startDate.value)) {
    return '结束日期不能早于起始日期'
  }
  return ''
})

const applyPreset = (preset) => {
  activePreset.value = preset
  errorMsg.value = ''
  resultPath.value = ''

  const today = new Date()
  const formatDate = (d) => d.toISOString().split('T')[0]

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
    const result = await invoke('generate_custom_report', {
      startDate: startDate.value,
      endDate: endDate.value,
      reportName: reportName.value || null,
    })
    resultPath.value = result
    showSuccess('自定义报告生成成功')
    emit('generated', result)
  } catch (err) {
    console.error('Failed to generate custom report:', err)
    errorMsg.value = typeof err === 'string' ? err : String(err)
    showError(err, generateReport)
  } finally {
    isGenerating.value = false
  }
}
</script>
