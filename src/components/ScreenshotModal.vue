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
        
        <div class="mt-4 p-4 bg-darker rounded-lg">
          <div class="flex items-center justify-between mb-2">
            <span class="text-xs text-gray-500">{{ formatTime(record.timestamp) }}</span>
            <span class="text-xs text-blue-400">🖥️ 自动截图</span>
          </div>
          <p class="text-sm text-gray-300 whitespace-pre-wrap">{{ parseContent(record.content) }}</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
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
