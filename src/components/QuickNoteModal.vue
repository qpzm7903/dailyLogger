<template>
  <div class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50" @click.self="$emit('close')">
    <div class="bg-dark rounded-2xl w-[600px] border border-gray-700 shadow-2xl shadow-black/50 flex flex-col">
      <!-- Header -->
      <div class="px-6 py-4 border-b border-gray-700 flex items-center justify-between bg-darker/50">
        <div class="flex items-center gap-2.5">
          <div class="w-8 h-8 bg-gradient-to-br from-amber-500/20 to-orange-500/20 rounded-lg flex items-center justify-center">
            <span class="text-xl">⚡</span>
          </div>
          <h2 class="text-lg font-semibold">闪念胶囊</h2>
        </div>
        <div class="flex items-center gap-3">
          <span class="text-xs text-gray-500">
            <kbd class="px-1.5 py-0.5 bg-dark rounded border border-gray-600 text-[10px]">Enter</kbd> 保存
            <span class="mx-1">·</span>
            <kbd class="px-1.5 py-0.5 bg-dark rounded border border-gray-600 text-[10px]">Shift+Enter</kbd> 换行
          </span>
          <button @click="$emit('close')" class="text-gray-400 hover:text-white hover:bg-gray-700/50 p-1.5 rounded-lg transition-all">✕</button>
        </div>
      </div>

      <!-- Content -->
      <div class="p-6 flex-1">
        <div class="mb-2 flex items-center justify-between">
          <label class="text-xs text-gray-500">记录此刻的想法</label>
          <span class="text-xs text-gray-600">{{ content.length }} 字</span>
        </div>
        <textarea
          ref="inputRef"
          v-model="content"
          @keydown.enter.exact.prevent="save"
          placeholder="💭 记录此刻的想法..."
          class="w-full h-40 bg-darker border border-gray-700 rounded-xl px-4 py-3 text-sm focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary/50 resize-none transition-all placeholder:text-gray-600"
          autofocus
        ></textarea>
      </div>

      <!-- Footer -->
      <div class="px-6 py-4 border-t border-gray-700 flex justify-between items-center bg-darker/50">
        <span class="text-xs text-gray-500 font-mono">{{ currentTime }}</span>
        <div class="flex gap-2.5">
          <button
            @click="$emit('close')"
            class="px-4 py-2 rounded-lg text-sm hover:bg-gray-700/50 transition-all border border-gray-600"
          >
            取消
          </button>
          <button
            @click="save"
            :disabled="!content.trim()"
            class="px-5 py-2 bg-gradient-to-r from-primary/90 to-blue-600 hover:from-primary hover:to-blue-500 disabled:opacity-50 disabled:cursor-not-allowed rounded-lg text-sm font-medium transition-all shadow-md hover:shadow-lg"
          >
            💾 保存
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, nextTick } from 'vue'

const emit = defineEmits(['close', 'save'])

const content = ref('')
const inputRef = ref(null)
const currentTime = ref('')

const updateTime = () => {
  currentTime.value = new Date().toLocaleString('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  })
}

const save = () => {
  if (!content.value.trim()) return
  emit('save', content.value.trim())
}

onMounted(() => {
  updateTime()
  setInterval(updateTime, 1000)
  nextTick(() => {
    inputRef.value?.focus()
  })
})
</script>
