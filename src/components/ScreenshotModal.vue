<template>
  <div class="fixed inset-0 bg-black/80 flex items-center justify-center z-50" @click.self="$emit('close')">
    <div class="bg-dark rounded-2xl max-w-4xl max-h-[90vh] overflow-hidden border border-gray-700">
      <div class="px-6 py-4 border-b border-gray-700 flex items-center justify-between">
        <h2 class="text-lg font-semibold">截图详情</h2>
        <button @click="$emit('close')" class="text-gray-400 hover:text-white">✕</button>
      </div>

      <div class="p-6 overflow-auto max-h-[70vh]">
        <img
          v-if="screenshotData"
          :src="screenshotData"
          alt="Screenshot"
          class="w-full h-auto rounded-lg"
        />
        <div v-else class="text-center py-8 text-gray-500">
          加载中...
        </div>

        <!-- Window Info Section (SMART-001 Task 6) -->
        <div
          v-if="windowInfo && (windowInfo.title || windowInfo.process_name)"
          class="mt-4 p-3 bg-darker rounded-lg border border-gray-700 window-info-section"
        >
          <div class="flex items-center gap-2 mb-1">
            <span class="text-sm">{{ getWindowIcon(windowInfo.process_name) }}</span>
            <span class="text-xs text-gray-400">窗口</span>
          </div>
          <p v-if="windowInfo.title" class="text-sm text-gray-300 truncate" :title="windowInfo.title">
            {{ windowInfo.title }}
          </p>
          <p v-if="windowInfo.process_name" class="text-xs text-gray-500 mt-1">
            {{ windowInfo.process_name }}
          </p>
        </div>

        <div class="mt-4 p-4 bg-darker rounded-lg">
          <div class="flex items-center justify-between mb-2">
            <span class="text-xs text-gray-500">{{ formatTime(record.timestamp) }}</span>
            <span class="text-xs" :class="record.content ? 'text-blue-400' : 'text-gray-500'">
              {{ record.content ? '🖥️ 已分析' : '📸 仅截图预览' }}
            </span>
          </div>
          <p v-if="record.content" class="text-sm text-gray-300 whitespace-pre-wrap">{{ parseContent(record.content) }}</p>
          <p v-else class="text-sm text-gray-500 italic">未进行 AI 分析</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps({
  record: {
    type: Object,
    required: true
  }
})

const emit = defineEmits(['close'])

const screenshotData = ref('')

const formatTime = (timestamp) => {
  const date = new Date(timestamp)
  return date.toLocaleString('zh-CN')
}

const parseContent = (content) => {
  try {
    const parsed = JSON.parse(content)
    return `当前焦点: ${parsed.current_focus}\n使用软件: ${parsed.active_software}\n关键词: ${parsed.context_keywords?.join(', ') || '无'}`
  } catch {
    return content
  }
}

// SMART-001 Task 6: Extract window info from content JSON
const windowInfo = computed(() => {
  if (!props.record.content) return null
  try {
    const parsed = JSON.parse(props.record.content)
    return parsed.active_window || null
  } catch {
    return null
  }
})

// SMART-001 Task 6: Get icon based on process name
const getWindowIcon = (processName) => {
  if (!processName) return '🖥️'
  const name = processName.toLowerCase()

  // Common development tools
  if (name.includes('code') || name.includes('vscode')) return '💻'
  if (name.includes('idea') || name.includes('intellij')) return '☕'
  if (name.includes('atom') || name.includes('sublime')) return '📝'

  // Browsers
  if (name.includes('chrome')) return '🌐'
  if (name.includes('firefox')) return '🦊'
  if (name.includes('edge') || name.includes('msedge')) return '🌊'
  if (name.includes('safari')) return '🧭'

  // Communication
  if (name.includes('slack') || name.includes('discord') || name.includes('teams')) return '💬'
  if (name.includes('wechat') || name.includes('微信')) return '💬'

  // Terminal
  if (name.includes('terminal') || name.includes('cmd') || name.includes('bash') || name.includes('powershell')) return '⌨️'

  // Office
  if (name.includes('word') || name.includes('excel') || name.includes('powerpoint')) return '📊'

  // Default
  return '🖥️'
}

const loadScreenshot = async () => {
  if (props.record.screenshot_path) {
    try {
      screenshotData.value = await invoke('get_screenshot', { path: props.record.screenshot_path })
    } catch (err) {
      console.error('Failed to load screenshot:', err)
      screenshotData.value = ''
    }
  }
}

onMounted(() => {
  loadScreenshot()
})
</script>
