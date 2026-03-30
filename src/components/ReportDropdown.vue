<template>
  <div class="relative inline-flex" ref="dropdownRef">
    <!-- Vault selector (shown when multiple vaults exist) -->
    <select
      v-if="vaults && vaults.length > 1"
      v-model="selectedVault"
      class="bg-primary hover:bg-primary-hover border-r border-blue-400 px-2 py-1.5 rounded-l-lg text-sm font-medium transition-colors cursor-pointer"
      :disabled="isGenerating"
      @click.stop
    >
      <option value="">{{ defaultVaultName }}</option>
      <option v-for="vault in vaults" :key="vault.name" :value="vault.name">
        {{ vault.name }}
      </option>
    </select>

    <!-- Main button (triggers daily report) -->
    <button
      @click="handleMainClick"
      :disabled="isGenerating"
      class="bg-primary hover:bg-primary-hover disabled:opacity-75 disabled:cursor-not-allowed px-4 py-1.5 text-sm font-medium transition-colors flex items-center gap-1.5"
      :class="vaults && vaults.length > 1 ? '' : 'rounded-l-lg'"
    >
      <span v-if="isGeneratingDaily" class="inline-block w-3 h-3 border-2 border-white border-t-transparent rounded-full animate-spin"></span>
      {{ isGeneratingDaily ? '生成中...' : '生成日报' }}
    </button>

    <!-- Dropdown toggle button -->
    <button
      @click="toggleDropdown"
      :disabled="isGenerating"
      class="bg-primary hover:bg-primary-hover disabled:opacity-75 disabled:cursor-not-allowed border-l border-blue-400 px-2 py-1.5 rounded-r-lg text-sm font-medium transition-colors flex items-center"
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
      class="absolute top-full left-0 mt-1 bg-[var(--color-surface-0)] border border-[var(--color-border-subtle)] rounded-md shadow-lg z-20 min-w-[180px]"
    >
      <button
        v-for="option in reportOptions"
        :key="option.id"
        @click="selectOption(option.id)"
        :disabled="isGenerating"
        class="w-full px-4 py-2 text-left text-sm hover:bg-[var(--color-surface-1)] transition-colors flex items-center justify-between disabled:opacity-50"
      >
        <div>
          <div class="text-[var(--color-text-primary)]">{{ option.label }}</div>
          <div v-if="option.shortcut" class="text-xs text-[var(--color-text-secondary)]">{{ option.shortcut }}</div>
        </div>
        <span
          v-if="isGeneratingType(option.id)"
          class="inline-block w-3 h-3 border-2 border-primary border-t-transparent rounded-full animate-spin"
        ></span>
      </button>

      <!-- Language selector submenu -->
      <div class="border-t border-[var(--color-border)] my-1"></div>
      <div class="relative">
        <button
          @click="toggleLanguageSubmenu"
          class="w-full px-4 py-2 text-left text-sm hover:bg-[var(--color-surface-1)] transition-colors flex items-center justify-between"
        >
          <div class="flex items-center gap-2">
            <span class="text-base">🌐</span>
            <div>
              <div class="text-[var(--color-text-primary)]">多语言日报</div>
              <div class="text-xs text-[var(--color-text-secondary)]">{{ selectedLanguageName }}</div>
            </div>
          </div>
          <svg
            :class="isLanguageSubmenuOpen ? 'rotate-90' : ''"
            class="w-3 h-3 transition-transform"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
          </svg>
        </button>

        <!-- Language submenu -->
        <div
          v-if="isLanguageSubmenuOpen"
          class="absolute left-full top-0 ml-1 bg-[var(--color-surface-0)] border border-[var(--color-border-subtle)] rounded-md shadow-lg z-30 min-w-[140px]"
        >
          <button
            v-for="lang in languageOptions"
            :key="lang.code"
            @click="selectLanguageAndGenerate(lang.code)"
            :disabled="isGeneratingDaily"
            class="w-full px-4 py-2 text-left text-sm hover:bg-[var(--color-surface-1)] transition-colors flex items-center justify-between disabled:opacity-50"
            :class="{ 'bg-[var(--color-surface-1)]': selectedLanguage === lang.code }"
          >
            <span class="text-[var(--color-text-primary)]">{{ lang.name }}</span>
            <span v-if="selectedLanguage === lang.code" class="text-primary">✓</span>
          </button>
        </div>
      </div>

      <!-- Additional options with divider -->
      <template v-if="additionalOptions && additionalOptions.length > 0">
        <div class="border-t border-[var(--color-border)] my-1"></div>
        <button
          v-for="option in additionalOptions"
          :key="option.id"
          @click="selectAdditionalOption(option)"
          :disabled="isGenerating"
          class="w-full px-4 py-2 text-left text-sm hover:bg-[var(--color-surface-1)] transition-colors flex items-center justify-between disabled:opacity-50"
        >
          <div class="flex items-center gap-2">
            <span v-if="option.icon" class="text-base">{{ option.icon }}</span>
            <div>
              <div class="text-[var(--color-text-primary)]">{{ option.label }}</div>
              <div v-if="option.shortcut" class="text-xs text-[var(--color-text-secondary)]">{{ option.shortcut }}</div>
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
import type { ObsidianVault } from '../types/tauri'

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
  preferredLanguage?: string
  vaults?: ObsidianVault[]
  selectedVault?: string
}

interface Emits {
  (e: 'generate', type: 'daily' | 'weekly' | 'monthly', vaultName?: string): void
  (e: 'generateMultilingual', language: string): void
  (e: 'openModal', modalId: string): void
  (e: 'customAction', actionId: string): void
  (e: 'languageChange', language: string): void
  (e: 'vaultChange', vaultName: string): void
}

const props = withDefaults(defineProps<Props>(), {
  isGeneratingDaily: false,
  isGeneratingWeekly: false,
  isGeneratingMonthly: false,
  additionalOptions: () => [],
  preferredLanguage: 'zh-CN',
  vaults: () => [],
  selectedVault: '',
})

const emit = defineEmits<Emits>()

const isOpen = ref(false)
const isLanguageSubmenuOpen = ref(false)
const dropdownRef = ref<HTMLElement | null>(null)
const selectedLanguage = ref(props.preferredLanguage || 'zh-CN')
const selectedVault = ref(props.selectedVault || '')

// Default vault name for the selector
const defaultVaultName = computed(() => {
  const defaultVault = props.vaults?.find(v => v.is_default)
  return defaultVault?.name || '默认 Vault'
})

// Language options
const languageOptions = [
  { code: 'zh-CN', name: '中文' },
  { code: 'en', name: 'English' },
  { code: 'ja', name: '日本語' },
  { code: 'ko', name: '한국어' },
  { code: 'es', name: 'Español' },
  { code: 'fr', name: 'Français' },
  { code: 'de', name: 'Deutsch' },
]

// Selected language display name
const selectedLanguageName = computed(() => {
  const lang = languageOptions.find(l => l.code === selectedLanguage.value)
  return lang ? lang.name : '中文'
})

// Close dropdown when clicking outside
onClickOutside(dropdownRef, () => {
  isOpen.value = false
  isLanguageSubmenuOpen.value = false
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
    isLanguageSubmenuOpen.value = false
  }
}

// Toggle language submenu
const toggleLanguageSubmenu = () => {
  isLanguageSubmenuOpen.value = !isLanguageSubmenuOpen.value
}

// Select language and generate multilingual report
const selectLanguageAndGenerate = (langCode: string) => {
  selectedLanguage.value = langCode
  emit('languageChange', langCode)
  emit('generateMultilingual', langCode)
  isOpen.value = false
  isLanguageSubmenuOpen.value = false
}

// Handle main button click (daily report)
const handleMainClick = () => {
  if (!isGenerating.value) {
    emit('generate', 'daily', selectedVault.value || undefined)
    isOpen.value = false
  }
}

// Select option from dropdown
const selectOption = (type: 'daily' | 'weekly' | 'monthly') => {
  if (!isGenerating.value) {
    emit('generate', type, selectedVault.value || undefined)
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
