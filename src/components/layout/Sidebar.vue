<template>
  <aside
    :class="[
      'bg-[var(--color-surface-1)]/80 backdrop-blur-md border-r border-[var(--color-border)]/50 flex flex-col items-center py-4 gap-2 transition-all duration-300',
      isCollapsed ? 'w-16' : 'w-48'
    ]"
  >
    <!-- Logo -->
    <div class="w-10 h-10 bg-primary rounded-xl flex items-center justify-center mb-4 shadow-lg shadow-primary/20">
      <span class="text-lg">📝</span>
    </div>

    <!-- Version number (hidden when collapsed) -->
    <div
      v-if="!isCollapsed"
      class="text-[var(--color-text-muted)] text-xs mb-2"
    >
      v{{ appVersion }}
    </div>

    <!-- Navigation Items -->
    <nav class="flex-1 flex flex-col items-center gap-1">
      <button
        v-for="item in navItems"
        :key="item.id"
        @click="item.action"
        :class="[
          'rounded-xl flex items-center justify-center transition-all duration-200',
          'hover:bg-[var(--color-action-neutral)]/50 hover:-translate-y-0.5 hover:shadow-lg',
          isActive(item.modalId) ? 'bg-primary/20 text-[var(--color-text-primary)]' : 'text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]',
          isCollapsed ? 'w-10 h-10' : 'w-full px-3 h-10 gap-3'
        ]"
        :title="isCollapsed ? item.label : undefined"
      >
        <component :is="item.icon" class="w-5 h-5 flex-shrink-0" />
        <span v-if="!isCollapsed" class="text-sm whitespace-nowrap">{{ item.label }}</span>
      </button>
    </nav>

    <!-- Bottom Actions -->
    <div class="flex flex-col items-center gap-1">
      <button
        @click="emit('open', 'settings')"
        :class="[
          'rounded-xl flex items-center justify-center transition-all duration-200',
          'hover:bg-[var(--color-action-neutral)]/50 hover:-translate-y-0.5 hover:shadow-lg',
          isActive('settings') ? 'bg-primary/20 text-[var(--color-text-primary)]' : 'text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]',
          isCollapsed ? 'w-10 h-10' : 'w-full px-3 h-10 gap-3'
        ]"
        :title="isCollapsed ? t('header.settings') : undefined"
      >
        <Settings class="w-5 h-5 flex-shrink-0" />
        <span v-if="!isCollapsed" class="text-sm whitespace-nowrap">{{ t('header.settings') }}</span>
      </button>

      <!-- Collapse/Expand Toggle -->
      <button
        @click="isCollapsed = !isCollapsed"
        class="w-10 h-10 rounded-xl flex items-center justify-center transition-all duration-200 hover:bg-[var(--color-action-neutral)]/50 hover:-translate-y-0.5 hover:shadow-lg text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]"
        :title="isCollapsed ? '展开侧边栏' : '折叠侧边栏'"
      >
        <component :is="isCollapsed ? ChevronRight : ChevronLeft" class="w-5 h-5" />
      </button>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { getVersion } from '@tauri-apps/api/app'
import { useI18n } from 'vue-i18n'
import {
  FileText,
  History,
  Search,
  Tags,
  Upload,
  TrendingUp,
  Database,
  Settings,
  ChevronLeft,
  ChevronRight
} from 'lucide-vue-next'
import type { ModalId } from '../../composables/useModal'
import { useModal } from '../../composables/useModal'

const props = defineProps<{
  offlineQueueCount: number
}>()

const emit = defineEmits<{
  open: [modal: ModalId]
}>()

const { activeModal } = useModal()
const { t } = useI18n()

// Collapsed state
const isCollapsed = ref(false)

// App version from Tauri
const appVersion = ref('')
onMounted(async () => {
  try {
    appVersion.value = await getVersion()
  } catch {
    appVersion.value = ''
  }
})

interface NavItem {
  id: string
  icon: typeof FileText
  label: string
  action: () => void
  modalId: ModalId
}

const isActive = (modalId: ModalId): boolean => {
  return activeModal.value === modalId
}

const navItems = computed<NavItem[]>(() => [
  {
    id: 'log',
    icon: FileText,
    label: '日志',
    action: () => emit('open', 'logViewer'),
    modalId: 'logViewer'
  },
  {
    id: 'history',
    icon: History,
    label: '历史',
    action: () => emit('open', 'historyViewer'),
    modalId: 'historyViewer'
  },
  {
    id: 'search',
    icon: Search,
    label: '搜索',
    action: () => emit('open', 'search'),
    modalId: 'search'
  },
  {
    id: 'tags',
    icon: Tags,
    label: '标签',
    action: () => emit('open', 'tagCloud'),
    modalId: 'tagCloud'
  },
  {
    id: 'export',
    icon: Upload,
    label: '导出',
    action: () => emit('open', 'export'),
    modalId: 'export'
  },
  {
    id: 'timeline',
    icon: TrendingUp,
    label: '时间线',
    action: () => emit('open', 'timeline'),
    modalId: 'timeline'
  },
  {
    id: 'backup',
    icon: Database,
    label: '备份',
    action: () => emit('open', 'backup'),
    modalId: 'backup'
  }
])
</script>
