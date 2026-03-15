<script setup>
import { ref, computed } from 'vue'

const props = defineProps({
  show: Boolean
})

const emit = defineEmits(['close', 'generate'])

const preset = ref('custom')
const startDate = ref('')
const endDate = ref('')
const reportName = ref('')

const presets = [
  { id: 'custom', name: '自定义日期' },
  { id: 'biweekly', name: '双周报 (最近14天)' },
  { id: 'quarterly', name: '季度报 (当前季度)' }
]

// Calculate biweekly range (last 14 days)
const calculateBiweeklyRange = () => {
  const today = new Date()
  const end = today.toISOString().split('T')[0]
  const start = new Date(today)
  start.setDate(start.getDate() - 13)
  return {
    start: start.toISOString().split('T')[0],
    end
  }
}

// Calculate current quarter range
const calculateQuarterlyRange = () => {
  const now = new Date()
  const year = now.getFullYear()
  const month = now.getMonth()
  const quarter = Math.floor(month / 3)
  const startMonth = quarter * 3 + 1
  const start = `${year}-${String(startMonth).padStart(2, '0')}-01`
  let endMonth
  if (quarter === 3) {
    endMonth = 12
  } else {
    endMonth = startMonth + 2
  }
  const lastDay = new Date(year, endMonth, 0).getDate()
  const end = `${year}-${String(endMonth).padStart(2, '0')}-${String(lastDay).padStart(2, '0')}`
  return { start, end }
}

const applyPreset = () => {
  if (preset.value === 'biweekly') {
    const range = calculateBiweeklyRange()
    startDate.value = range.start
    endDate.value = range.end
    reportName.value = '双周报'
  } else if (preset.value === 'quarterly') {
    const range = calculateQuarterlyRange()
    startDate.value = range.start
    endDate.value = range.end
    reportName.value = '季度报'
  }
}

const selectedDays = computed(() => {
  if (!startDate.value || !endDate.value) return 0
  const start = new Date(startDate.value)
  const end = new Date(endDate.value)
  const diffTime = Math.abs(end - start)
  const diffDays = Math.ceil(diffTime / (1000 * 60 * 60 * 24)) + 1
  return diffDays
})

const isValid = computed(() => {
  if (!startDate.value || !endDate.value) return false
  return new Date(startDate.value) <= new Date(endDate.value)
})

const handleGenerate = () => {
  if (!isValid.value) return
  emit('generate', {
    startDate: startDate.value,
    endDate: endDate.value,
    reportName: reportName.value || '自定义报告'
  })
}
</script>

<template>
  <div v-if="show" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="emit('close')">
    <div class="bg-darker rounded-xl p-6 w-full max-w-md border border-gray-700">
      <div class="flex justify-between items-center mb-4">
        <h2 class="text-xl font-semibold text-white">自定义报告</h2>
        <button @click="emit('close')" class="text-gray-400 hover:text-white">
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
          </svg>
        </button>
      </div>

      <div class="space-y-4">
        <!-- Preset selection -->
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-2">报告类型</label>
          <select v-model="preset" @change="applyPreset" class="w-full bg-gray-800 border border-gray-600 rounded-lg px-3 py-2 text-white">
            <option v-for="p in presets" :key="p.id" :value="p.id">{{ p.name }}</option>
          </select>
        </div>

        <!-- Date range -->
        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="block text-sm font-medium text-gray-300 mb-2">开始日期</label>
            <input
              type="date"
              v-model="startDate"
              class="w-full bg-gray-800 border border-gray-600 rounded-lg px-3 py-2 text-white"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-300 mb-2">结束日期</label>
            <input
              type="date"
              v-model="endDate"
              :min="startDate"
              class="w-full bg-gray-800 border border-gray-600 rounded-lg px-3 py-2 text-white"
            />
          </div>
        </div>

        <!-- Report name -->
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-2">报告名称 (可选)</label>
          <input
            type="text"
            v-model="reportName"
            placeholder="默认: 自定义报告"
            class="w-full bg-gray-800 border border-gray-600 rounded-lg px-3 py-2 text-white placeholder-gray-500"
          />
        </div>

        <!-- Days selected info -->
        <div v-if="selectedDays > 0" class="text-sm text-gray-400">
          已选择 {{ selectedDays }} 天
        </div>
      </div>

      <!-- Actions -->
      <div class="mt-6 flex justify-end gap-3">
        <button
          @click="emit('close')"
          class="px-4 py-2 rounded-lg bg-gray-700 hover:bg-gray-600 text-white transition-colors"
        >
          取消
        </button>
        <button
          @click="handleGenerate"
          :disabled="!isValid"
          class="px-4 py-2 rounded-lg bg-orange-600 hover:bg-orange-700 disabled:opacity-50 text-white transition-colors"
        >
          生成报告
        </button>
      </div>
    </div>
  </div>
</template>
