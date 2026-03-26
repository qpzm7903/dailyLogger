<template>
  <div class="relative inline-flex" ref="dropdownRef">
    <!-- Main button (triggers daily report) -->
    <button
      @click="handleMainClick"
      :disabled="isGenerating"
      class="bg-primary hover:bg-blue-600 disabled:opacity-75 disabled:cursor-not-allowed px-4 py-1.5 rounded-l-lg text-sm font-medium transition-colors flex items-center gap-1.5"
    >
      <span v-if="isGeneratingDaily" class="inline-block w-3 h-3 border-2 border-white border-t-transparent rounded-full animate-spin"></span>
      {{ isGeneratingDaily ? '生成中...' : '生成日报' }}
    </button>

    <!-- Dropdown toggle button -->
    <button
      @click="toggleDropdown"
      :disabled="isGenerating"
      class="bg-primary hover:bg-blue-600 disabled:opacity-75 disabled:cursor-not-allowed border-l border-blue-400 px-2 py-1.5 rounded-r-lg text-sm font-medium transition-colors flex items-center"
      :title="isOpen ? '收起菜单' : '展开菜单'"
    >
      <svg
        :class="isOpen ? 'rotate-180' : ''"
        class="w-4 h-4 transition-transform duration-200"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
      </svg>
    </button>

    <!-- Dropdown menu -->
    <div
      v-if="isOpen"
      class="absolute top-full left-0 mt-1 bg-darker border border-gray-600 rounded-md shadow-lg z-20 min-w-[180px]"
    >
      <button
        v-for="option in reportOptions"
        :key="option.id"
        @click="selectOption(option.id)"
        :disabled="isGenerating"
        class="w-full px-4 py-2 text-left text-sm hover:bg-dark transition-colors flex items-center justify-between disabled:opacity-50"
      >
        <div>
          <div class="text-white">{{ option.label }}</div>
          <div v-if="option.shortcut" class="text-xs text-gray-400">{{ option.shortcut }}</div>
        </div>
        <span
          v-if="isGeneratingType(option.id)"
          class="inline-block w-3 h-3 border-2 border-primary border-t-transparent rounded-full animate-spin"
        ></span>
      </button>

      <!-- Additional options with divider -->
      <template v-if="additionalOptions && additionalOptions.length > 0">
        <div class="border-t border-gray-600 my-1"></div>
        <button
          v-for="option in additionalOptions"
          :key="option.id"
          @click="selectAdditionalOption(option)"
          :disabled="isGenerating"
          class="w-full px-4 py-2 text-left text-sm hover:bg-dark transition-colors flex items-center justify-between disabled:opacity-50"
        >
          <div class="flex items-center gap-2">
            <span v-if="option.icon" class="text-base">{{ option.icon }}</span>
            <div>
              <div class="text-white">{{ option.label }}</div>
              <div v-if="option.shortcut" class="text-xs text-gray-400">{{ option.shortcut }}</div>
            </div>
          </div>
        </button>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { onClickOutside } from '@vueuse/core'

export interface AdditionalOption {
  id: string
  label: string
  shortcut?: string
  type: 'report' | 'action'
  icon?: string
}

interface Props {
  isGeneratingDaily?: boolean
  isGeneratingWeekly?: boolean
  isGeneratingMonthly?: boolean
  additionalOptions?: AdditionalOption[]
}

interface Emits {
  (e: 'generate', type: 'daily' | 'weekly' | 'monthly'): void
  (e: 'openModal', modalId: string): void
  (e: 'customAction', actionId: string): void
}

const props = withDefaults(defineProps<Props>(), {
  isGeneratingDaily: false,
  isGeneratingWeekly: false,
  isGeneratingMonthly: false,
  additionalOptions: () => [],
})

const emit = defineEmits<Emits>()

const isOpen = ref(false)
const dropdownRef = ref<HTMLElement | null>(null)

// Close dropdown when clicking outside
onClickOutside(dropdownRef, () => {
  isOpen.value = false
})

// Computed: any report is generating
const isGenerating = computed(() =>
  props.isGeneratingDaily || props.isGeneratingWeekly || props.isGeneratingMonthly
)

// Report options
const reportOptions = [
  { id: 'daily' as const, label: '生成日报', shortcut: '今日工作总结' },
  { id: 'weekly' as const, label: '生成周报', shortcut: '本周工作汇总' },
  { id: 'monthly' as const, label: '生成月报', shortcut: '本月工作汇总' },
]

// Check if specific type is generating
const isGeneratingType = (type: 'daily' | 'weekly' | 'monthly'): boolean => {
  if (type === 'daily') return props.isGeneratingDaily
  if (type === 'weekly') return props.isGeneratingWeekly
  if (type === 'monthly') return props.isGeneratingMonthly
  return false
}

// Toggle dropdown
const toggleDropdown = () => {
  if (!isGenerating.value) {
    isOpen.value = !isOpen.value
  }
}

// Handle main button click (daily report)
const handleMainClick = () => {
  if (!isGenerating.value) {
    emit('generate', 'daily')
    isOpen.value = false
  }
}

// Select option from dropdown
const selectOption = (type: 'daily' | 'weekly' | 'monthly') => {
  if (!isGenerating.value) {
    emit('generate', type)
    isOpen.value = false
  }
}

// Select additional option
const selectAdditionalOption = (option: AdditionalOption) => {
  if (!isGenerating.value) {
    if (option.type === 'report') {
      emit('generate', option.id as 'daily' | 'weekly' | 'monthly')
    } else if (option.id === 'reanalyzeToday') {
      emit('customAction', 'reanalyzeToday')
    } else {
      emit('openModal', option.id)
    }
    isOpen.value = false
  }
}
</script>
