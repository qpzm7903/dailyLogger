<template>
  <div class="fixed inset-0 bg-black/60 flex items-center justify-center z-50 p-4">
    <div class="bg-dark border border-gray-700 rounded-xl w-full max-w-4xl h-3/4 flex flex-col">
      <!-- Header -->
      <div class="flex items-center justify-between px-5 py-4 border-b border-gray-700 shrink-0">
        <div class="flex items-center gap-3">
          <span class="text-xl">📋</span>
          <h2 class="font-medium">运行日志</h2>
          <span class="text-xs text-gray-500 bg-darker px-2 py-0.5 rounded">{{ logPath }}</span>
        </div>
        <div class="flex items-center gap-2">
          <!-- Level filter -->
          <div class="flex gap-1 text-xs">
            <button
              v-for="level in levels"
              :key="level.key"
              @click="toggleLevel(level.key)"
              :class="[
                'px-2 py-1 rounded transition-colors',
                activelevels.has(level.key) ? level.activeClass : 'bg-gray-800 text-gray-500'
              ]"
            >{{ level.label }}</button>
          </div>
          <button
            @click="loadLogs"
            :disabled="loading"
            class="px-3 py-1.5 text-xs bg-gray-700 hover:bg-gray-600 rounded-lg transition-colors disabled:opacity-50"
          >
            {{ loading ? '加载中...' : '刷新' }}
          </button>
          <label class="flex items-center gap-1.5 text-xs text-gray-400 cursor-pointer select-none">
            <input type="checkbox" v-model="autoRefresh" class="accent-primary" />
            自动刷新
          </label>
          <button @click="$emit('close')" class="p-1.5 hover:bg-gray-700 rounded-lg transition-colors text-gray-400 hover:text-white">
            ✕
          </button>
        </div>
      </div>

      <!-- Log content -->
      <div ref="logContainer" class="flex-1 overflow-y-auto font-mono text-xs p-4 space-y-0.5 bg-darker rounded-b-xl">
        <div v-if="loading && filteredLines.length === 0" class="text-center py-8 text-gray-500">
          加载中...
        </div>
        <div v-else-if="filteredLines.length === 0" class="text-center py-8 text-gray-500">
          暂无日志
        </div>
        <div
          v-for="(line, i) in filteredLines"
          :key="i"
          :class="lineClass(line)"
          class="leading-5 whitespace-pre-wrap break-all hover:bg-white/5 px-1 rounded"
        >{{ line }}</div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'

defineEmits(['close'])

const logContainer = ref(null)
const rawLines = ref([])
const loading = ref(false)
const autoRefresh = ref(false)
const activelevels = ref(new Set(['INFO', 'WARN', 'ERROR']))

const logPath = 'DailyLogger/logs/daily-logger.log'

const levels = [
  { key: 'INFO',  label: 'INFO',  activeClass: 'bg-blue-900/60 text-blue-300' },
  { key: 'WARN',  label: 'WARN',  activeClass: 'bg-yellow-900/60 text-yellow-300' },
  { key: 'ERROR', label: 'ERROR', activeClass: 'bg-red-900/60 text-red-300' },
]

const filteredLines = computed(() => {
  if (activelevels.value.size === 3) return rawLines.value
  return rawLines.value.filter(line => {
    for (const level of activelevels.value) {
      if (line.includes(` ${level} `) || line.includes(` ${level}\t`)) return true
    }
    // Keep lines that don't have a recognized level marker (continuation lines)
    const hasAnyLevel = ['INFO', 'WARN', 'ERROR'].some(l =>
      line.includes(` ${l} `) || line.includes(` ${l}\t`)
    )
    return !hasAnyLevel
  })
})

const lineClass = (line) => {
  if (line.includes(' ERROR ') || line.includes(' ERROR\t')) return 'text-red-400'
  if (line.includes(' WARN ')  || line.includes(' WARN\t'))  return 'text-yellow-400'
  if (line.includes(' INFO ')  || line.includes(' INFO\t'))  return 'text-gray-300'
  return 'text-gray-500'
}

const toggleLevel = (key) => {
  const s = new Set(activelevels.value)
  s.has(key) ? s.delete(key) : s.add(key)
  // Keep at least one level active
  if (s.size > 0) activelevels.value = s
}

const scrollToBottom = async () => {
  await nextTick()
  if (logContainer.value) {
    logContainer.value.scrollTop = logContainer.value.scrollHeight
  }
}

const loadLogs = async () => {
  loading.value = true
  try {
    const content = await invoke('get_recent_logs', { lines: 500 })
    rawLines.value = content ? content.split('\n') : []
    await scrollToBottom()
  } catch (err) {
    rawLines.value = [`[日志加载失败] ${err}`]
  } finally {
    loading.value = false
  }
}

let refreshTimer = null

watch(autoRefresh, (val) => {
  clearInterval(refreshTimer)
  if (val) refreshTimer = setInterval(loadLogs, 3000)
})

onMounted(() => loadLogs())
onUnmounted(() => clearInterval(refreshTimer))
</script>
