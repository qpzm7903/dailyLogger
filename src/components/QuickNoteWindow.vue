<template>
  <div class="h-screen bg-darker text-white flex flex-col">
    <header class="bg-dark border-b border-gray-700 px-4 py-3 flex items-center justify-between">
      <div class="flex items-center gap-2">
        <span class="text-lg">⚡</span>
        <h1 class="text-sm font-medium">快速记录</h1>
      </div>
      <span class="text-xs text-gray-500">Enter 保存 · Esc 关闭</span>
    </header>

    <main class="flex-1 p-4">
      <textarea
        ref="inputRef"
        v-model="content"
        @keydown.enter.exact.prevent="save"
        @keydown.esc="closeWindow"
        placeholder="记录此刻的想法..."
        class="w-full h-full bg-darker border border-gray-700 rounded-lg px-4 py-3 text-sm focus:border-primary focus:outline-none resize-none"
        autofocus
      ></textarea>
    </main>

    <footer class="bg-dark border-t border-gray-700 px-4 py-3 flex justify-between items-center">
      <span class="text-xs text-gray-500">{{ currentTime }}</span>
      <div class="flex gap-2">
        <button
          @click="closeWindow"
          class="px-3 py-1.5 rounded-lg text-xs hover:bg-gray-700 transition-colors"
        >
          取消
        </button>
        <button
          @click="save"
          :disabled="!content.trim() || isSaving"
          class="px-3 py-1.5 bg-primary rounded-lg text-xs font-medium hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          {{ isSaving ? '保存中...' : '保存' }}
        </button>
      </div>
    </footer>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'

const content = ref('')
const inputRef = ref(null)
const currentTime = ref('')
const isSaving = ref(false)

const updateTime = () => {
  currentTime.value = new Date().toLocaleString('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  })
}

const closeWindow = async () => {
  const window = getCurrentWindow()
  await window.close()
}

const save = async () => {
  if (!content.value.trim() || isSaving.value) return

  isSaving.value = true
  try {
    await invoke('tray_quick_note', { content: content.value.trim() })
    await closeWindow()
  } catch (err) {
    console.error('Failed to save quick note:', err)
    isSaving.value = false
  }
}

let timeInterval = null

onMounted(() => {
  updateTime()
  timeInterval = setInterval(updateTime, 1000)
  inputRef.value?.focus()
})

onUnmounted(() => {
  if (timeInterval) clearInterval(timeInterval)
})
</script>