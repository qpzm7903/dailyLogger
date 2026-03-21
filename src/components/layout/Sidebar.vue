<template>
  <aside
    class="w-16 bg-dark/80 backdrop-blur-md border-r border-gray-700/50 flex flex-col items-center py-4 gap-2"
  >
    <!-- Logo -->
    <div class="w-10 h-10 bg-primary rounded-xl flex items-center justify-center mb-4 shadow-lg shadow-primary/20">
      <span class="text-lg">📝</span>
    </div>

    <!-- Navigation Items -->
    <nav class="flex-1 flex flex-col items-center gap-1">
      <button
        v-for="item in navItems"
        :key="item.id"
        @click="item.action"
        :class="[
          'w-10 h-10 rounded-xl flex items-center justify-center transition-all duration-200',
          'hover:bg-gray-700/50 hover:-translate-y-0.5 hover:shadow-lg',
          'text-gray-400 hover:text-white',
          'group relative'
        ]"
        :title="item.label"
      >
        <span class="text-lg">{{ item.icon }}</span>
        <!-- Tooltip -->
        <span
          class="absolute left-14 px-2 py-1 bg-gray-800 text-white text-xs rounded-md opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap pointer-events-none z-50"
        >
          {{ item.label }}
        </span>
      </button>
    </nav>

    <!-- Bottom Actions -->
    <div class="flex flex-col items-center gap-1">
      <button
        @click="$emit('open', 'settings')"
        :class="[
          'w-10 h-10 rounded-xl flex items-center justify-center transition-all duration-200',
          'hover:bg-gray-700/50 hover:-translate-y-0.5 hover:shadow-lg',
          'text-gray-400 hover:text-white'
        ]"
        title="设置"
      >
        <span class="text-lg">⚙️</span>
      </button>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { ModalId } from '../../composables/useModal'

interface NavItem {
  id: string
  icon: string
  label: string
  action: () => void
}

const props = defineProps<{
  offlineQueueCount: number
}>()

const emit = defineEmits<{
  open: [modal: ModalId]
}>()

const navItems = computed<NavItem[]>(() => [
  {
    id: 'log',
    icon: '🗒️',
    label: '日志',
    action: () => emit('open', 'logViewer')
  },
  {
    id: 'history',
    icon: '📚',
    label: '历史',
    action: () => emit('open', 'historyViewer')
  },
  {
    id: 'search',
    icon: '🔍',
    label: '搜索',
    action: () => emit('open', 'search')
  },
  {
    id: 'tags',
    icon: '🏷️',
    label: '标签',
    action: () => emit('open', 'tagCloud')
  },
  {
    id: 'export',
    icon: '📤',
    label: '导出',
    action: () => emit('open', 'export')
  },
  {
    id: 'timeline',
    icon: '📈',
    label: '时间线',
    action: () => emit('open', 'timeline')
  },
  {
    id: 'backup',
    icon: '💾',
    label: '备份',
    action: () => emit('open', 'backup')
  }
])
</script>