<template>
  <div class="fixed inset-0 bg-black/80 flex items-center justify-center z-50" @click.self="$emit('close')">
    <div class="bg-dark rounded-2xl w-[90vw] h-[90vh] max-w-4xl overflow-hidden border border-gray-700 flex flex-col">
      <div class="px-6 py-4 border-b border-gray-700 flex items-center justify-between">
        <h2 class="text-lg font-semibold">📝 日报预览</h2>
        <div class="flex items-center gap-2">
          <button 
            @click="openInObsidian"
            class="px-3 py-1 text-sm bg-gray-700 hover:bg-gray-600 rounded transition-colors"
          >
            在 Finder 中显示
          </button>
          <button @click="$emit('close')" class="text-gray-400 hover:text-white">✕</button>
        </div>
      </div>
      
      <div class="flex-1 overflow-auto p-6">
        <div v-if="loading" class="text-center py-8 text-gray-500">
          加载中...
        </div>
        <div v-else-if="error" class="text-center py-8 text-red-500">
          {{ error }}
        </div>
        <div v-else class="prose prose-invert max-w-none">
          <div class="text-sm text-gray-400 mb-4">
            文件路径: {{ summaryPath }}
          </div>
          <div class="whitespace-pre-wrap text-gray-300 leading-relaxed">
            {{ content }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-shell'

const props = defineProps({
  summaryPath: {
    type: String,
    required: true
  }
})

const emit = defineEmits(['close'])

const content = ref('')
const loading = ref(true)
const error = ref('')

const loadSummary = async () => {
  if (!props.summaryPath) {
    error.value = '日报路径为空'
    loading.value = false
    return
  }
  
  try {
    content.value = await invoke('read_file', { path: props.summaryPath })
  } catch (err) {
    error.value = `加载失败: ${err}`
    console.error('Failed to load summary:', err)
  } finally {
    loading.value = false
  }
}

const openInObsidian = async () => {
  try {
    // Extract directory from path
    const pathParts = props.summaryPath.split('/')
    pathParts.pop() // Remove filename
    const dirPath = pathParts.join('/')
    await open(dirPath)
  } catch (err) {
    console.error('Failed to open in Finder:', err)
  }
}

onMounted(() => {
  loadSummary()
})
</script>
