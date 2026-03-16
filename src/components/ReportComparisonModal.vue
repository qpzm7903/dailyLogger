<template>
  <div class="fixed inset-0 bg-black/80 flex items-center justify-center z-50" @click.self="$emit('close')">
    <div class="bg-dark rounded-2xl w-[90vw] max-w-lg overflow-hidden border border-gray-700 flex flex-col">
      <!-- Header -->
      <div class="px-6 py-4 border-b border-gray-700 flex items-center justify-between">
        <h2 class="text-lg font-semibold">对比分析报告</h2>
        <button @click="$emit('close')" class="text-gray-400 hover:text-white">✕</button>
      </div>

      <!-- Body -->
      <div class="p-6 space-y-5">
        <!-- Period A -->
        <div class="space-y-2">
          <label class="text-sm text-gray-400 block">时段 A</label>
          <div class="flex items-center gap-3">
            <input
              type="date"
              v-model="startDateA"
              class="flex-1 bg-darker border border-gray-600 rounded-lg px-3 py-2 text-sm text-white focus:border-primary focus:outline-none"
            />
            <span class="text-gray-500">至</span>
            <input
              type="date"
              v-model="endDateA"
              class="flex-1 bg-darker border border-gray-600 rounded-lg px-3 py-2 text-sm text-white focus:border-primary focus:outline-none"
            />
          </div>
          <p v-if="dayCountA > 0" class="text-xs text-gray-500">{{ dayCountA }} 天</p>
        </div>

        <!-- Period B -->
        <div class="space-y-2">
          <label class="text-sm text-gray-400 block">时段 B</label>
          <div class="flex items-center gap-3">
            <input
              type="date"
              v-model="startDateB"
              class="flex-1 bg-darker border border-gray-600 rounded-lg px-3 py-2 text-sm text-white focus:border-primary focus:outline-none"
            />
            <span class="text-gray-500">至</span>
            <input
              type="date"
              v-model="endDateB"
              class="flex-1 bg-darker border border-gray-600 rounded-lg px-3 py-2 text-sm text-white focus:border-primary focus:outline-none"
            />
          </div>
          <p v-if="dayCountB > 0" class="text-xs text-gray-500">{{ dayCountB }} 天</p>
        </div>

        <!-- Preset buttons -->
        <div class="space-y-2">
          <label class="text-sm text-gray-400 block">快捷预设</label>
          <div class="flex gap-3">
            <button
              @click="applyPreset('week')"
              class="flex-1 border border-gray-600 text-gray-400 hover:border-gray-500 rounded-lg px-3 py-2 text-sm transition-colors text-center"
            >
              <div class="font-medium">本周 vs 上周</div>
            </button>
            <button
              @click="applyPreset('month')"
              class="flex-1 border border-gray-600 text-gray-400 hover:border-gray-500 rounded-lg px-3 py-2 text-sm transition-colors text-center"
            >
              <div class="font-medium">本月 vs 上月</div>
            </button>
          </div>
        </div>

        <p v-if="dateError" class="text-red-400 text-xs">{{ dateError }}</p>

        <!-- Result -->
        <div v-if="resultPath" class="bg-darker rounded-lg p-4 space-y-2 border border-green-700/50">
          <div class="flex items-center gap-2 text-green-400 text-sm">
            <span>对比报告生成成功</span>
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
          @click="generateComparison"
          :disabled="isGenerating || !!dateError || !startDateA || !endDateA || !startDateB || !endDateB"
          class="bg-primary hover:bg-blue-600 disabled:opacity-50 px-5 py-2 rounded-lg text-sm font-medium transition-colors"
        >
          {{ isGenerating ? '生成中...' : '生成对比报告' }}
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

const startDateA = ref('')
const endDateA = ref('')
const startDateB = ref('')
const endDateB = ref('')
const isGenerating = ref(false)
const resultPath = ref('')
const errorMsg = ref('')

const calcDays = (start, end) => {
  if (!start || !end) return 0
  const s = new Date(start)
  const e = new Date(end)
  if (isNaN(s.getTime()) || isNaN(e.getTime())) return 0
  const diff = Math.floor((e - s) / (1000 * 60 * 60 * 24)) + 1
  return diff > 0 ? diff : 0
}

const dayCountA = computed(() => calcDays(startDateA.value, endDateA.value))
const dayCountB = computed(() => calcDays(startDateB.value, endDateB.value))

const dateError = computed(() => {
  if (startDateA.value && endDateA.value && new Date(endDateA.value) < new Date(startDateA.value)) {
    return '时段A的结束日期不能早于起始日期'
  }
  if (startDateB.value && endDateB.value && new Date(endDateB.value) < new Date(startDateB.value)) {
    return '时段B的结束日期不能早于起始日期'
  }
  return ''
})

const formatDate = (d) => d.toISOString().split('T')[0]

const applyPreset = (preset) => {
  errorMsg.value = ''
  resultPath.value = ''
  const today = new Date()

  if (preset === 'week') {
    const day = today.getDay() || 7 // Mon=1, Sun=7
    const thisMonday = new Date(today)
    thisMonday.setDate(today.getDate() - day + 1)
    const thisSunday = new Date(thisMonday)
    thisSunday.setDate(thisMonday.getDate() + 6)
    const lastMonday = new Date(thisMonday)
    lastMonday.setDate(thisMonday.getDate() - 7)
    const lastSunday = new Date(thisMonday)
    lastSunday.setDate(thisMonday.getDate() - 1)

    startDateA.value = formatDate(lastMonday)
    endDateA.value = formatDate(lastSunday)
    startDateB.value = formatDate(thisMonday)
    endDateB.value = formatDate(thisSunday)
  } else if (preset === 'month') {
    const thisFirst = new Date(today.getFullYear(), today.getMonth(), 1)
    const thisLast = new Date(today.getFullYear(), today.getMonth() + 1, 0)
    const lastFirst = new Date(today.getFullYear(), today.getMonth() - 1, 1)
    const lastLast = new Date(today.getFullYear(), today.getMonth(), 0)

    startDateA.value = formatDate(lastFirst)
    endDateA.value = formatDate(lastLast)
    startDateB.value = formatDate(thisFirst)
    endDateB.value = formatDate(thisLast)
  }
}

const generateComparison = async () => {
  if (isGenerating.value || dateError.value) return
  isGenerating.value = true
  errorMsg.value = ''
  resultPath.value = ''

  try {
    const result = await invoke('compare_reports', {
      startDateA: startDateA.value,
      endDateA: endDateA.value,
      startDateB: startDateB.value,
      endDateB: endDateB.value,
    })
    resultPath.value = result
    showSuccess('对比报告生成成功')
    emit('generated', result)
  } catch (err) {
    console.error('Failed to generate comparison report:', err)
    errorMsg.value = typeof err === 'string' ? err : String(err)
    showError(err, generateComparison)
  } finally {
    isGenerating.value = false
  }
}
</script>
