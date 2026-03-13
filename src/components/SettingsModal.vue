<template>
  <div class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50" @click.self="$emit('close')">
    <div class="bg-dark rounded-2xl w-[650px] max-h-[85vh] overflow-hidden border border-gray-700 shadow-2xl shadow-black/50 flex flex-col">
      <!-- Header -->
      <div class="px-6 py-4 border-b border-gray-700 flex items-center justify-between bg-darker/50">
        <div class="flex items-center gap-2.5">
          <div class="w-8 h-8 bg-gradient-to-br from-primary/20 to-blue-500/20 rounded-lg flex items-center justify-center">
            <span class="text-lg">⚡</span>
          </div>
          <h2 class="text-lg font-semibold">设置</h2>
        </div>
        <button @click="$emit('close')" class="text-gray-400 hover:text-white hover:bg-gray-700/50 p-1.5 rounded-lg transition-all">✕</button>
      </div>

      <!-- Content -->
      <div class="p-6 space-y-6 overflow-y-auto flex-1 custom-scrollbar">
        <!-- API 配置 -->
        <section class="space-y-3">
          <div class="flex items-center gap-2 mb-1">
            <div class="w-1 h-4 bg-primary rounded-full"></div>
            <h3 class="text-sm font-medium text-gray-200">API 配置</h3>
          </div>
          <div class="bg-darker/50 rounded-xl p-4 border border-gray-700/50 space-y-3">
            <div>
              <label class="text-xs text-gray-500 block mb-1.5 flex items-center gap-1">
                <span>Base URL</span>
              </label>
              <input
                v-model="settings.api_base_url"
                type="text"
                placeholder="https://api.openai.com/v1"
                class="w-full bg-dark border border-gray-700 rounded-lg px-3 py-2.5 text-sm focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary/50 transition-all"
              />
            </div>
            <div>
              <label class="text-xs text-gray-500 block mb-1.5">API Key</label>
              <div class="relative">
                <input
                  v-model="settings.api_key"
                  :type="showApiKey ? 'text' : 'password'"
                  placeholder="sk-..."
                  class="w-full bg-dark border border-gray-700 rounded-lg px-3 py-2.5 pr-20 text-sm focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary/50 transition-all font-mono"
                />
                <button
                  @click="showApiKey = !showApiKey"
                  type="button"
                  class="absolute right-2 top-1/2 -translate-y-1/2 text-gray-500 hover:text-gray-300 transition-colors text-xs px-2 py-1 rounded hover:bg-gray-700"
                >{{ showApiKey ? '🙈 隐藏' : '👁️ 显示' }}</button>
              </div>
            </div>
          </div>
        </section>

        <!-- 截图分析 (Vision) -->
        <section class="space-y-3">
          <div class="flex items-center gap-2 mb-1">
            <div class="w-1 h-4 bg-purple-500 rounded-full"></div>
            <h3 class="text-sm font-medium text-gray-200">截图分析 (Vision)</h3>
          </div>
          <div class="bg-darker/50 rounded-xl p-4 border border-gray-700/50 space-y-3">
            <div>
              <label class="text-xs text-gray-500 block mb-1.5">分析模型</label>
              <input
                v-model="settings.model_name"
                type="text"
                placeholder="gpt-4o"
                class="w-full bg-dark border border-gray-700 rounded-lg px-3 py-2.5 text-sm focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary/50 transition-all"
              />
              <span class="text-xs text-gray-600 mt-1.5 block flex items-center gap-1">
                <span>💡</span> 需要支持 Vision 能力的模型
              </span>
            </div>
            <div>
              <label class="text-xs text-gray-500 block mb-1.5">分析 Prompt</label>
              <textarea
                v-model="settings.analysis_prompt"
                rows="3"
                placeholder="留空使用默认 Prompt"
                class="w-full bg-dark border border-gray-700 rounded-lg px-3 py-2.5 text-sm focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary/50 transition-all resize-y"
              />
            </div>
          </div>
        </section>

        <!-- 日报生成 -->
        <section class="space-y-3">
          <div class="flex items-center gap-2 mb-1">
            <div class="w-1 h-4 bg-green-500 rounded-full"></div>
            <h3 class="text-sm font-medium text-gray-200">日报生成</h3>
          </div>
          <div class="bg-darker/50 rounded-xl p-4 border border-gray-700/50 space-y-3">
            <div>
              <label class="text-xs text-gray-500 block mb-1.5">日报模型</label>
              <input
                v-model="settings.summary_model_name"
                type="text"
                placeholder="留空则使用分析模型"
                class="w-full bg-dark border border-gray-700 rounded-lg px-3 py-2.5 text-sm focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary/50 transition-all"
              />
              <span class="text-xs text-gray-600 mt-1.5 block flex items-center gap-1">
                <span>💡</span> 纯文本模型即可，不需要 Vision
              </span>
            </div>
            <div>
              <label class="text-xs text-gray-500 block mb-1.5">日报 Prompt</label>
              <textarea
                v-model="settings.summary_prompt"
                rows="3"
                placeholder="留空使用默认 Prompt。用 {records} 表示今日记录的插入位置"
                class="w-full bg-dark border border-gray-700 rounded-lg px-3 py-2.5 text-sm focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary/50 transition-all resize-y"
              />
            </div>
          </div>
        </section>

        <!-- 时间策略 -->
        <section class="space-y-3">
          <div class="flex items-center gap-2 mb-1">
            <div class="w-1 h-4 bg-amber-500 rounded-full"></div>
            <h3 class="text-sm font-medium text-gray-200">时间策略</h3>
          </div>
          <div class="bg-darker/50 rounded-xl p-4 border border-gray-700/50 grid grid-cols-2 gap-3">
            <div>
              <label class="text-xs text-gray-500 block mb-1.5">截图间隔 (分钟)</label>
              <input
                v-model.number="settings.screenshot_interval"
                type="number"
                min="1"
                max="60"
                class="w-full bg-dark border border-gray-700 rounded-lg px-3 py-2.5 text-sm focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary/50 transition-all"
              />
            </div>
            <div>
              <label class="text-xs text-gray-500 block mb-1.5">每日总结时间</label>
              <input
                v-model="settings.summary_time"
                type="time"
                class="w-full bg-dark border border-gray-700 rounded-lg px-3 py-2.5 text-sm focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary/50 transition-all"
              />
            </div>
          </div>
        </section>

        <!-- 智能去重 -->
        <section class="space-y-3">
          <div class="flex items-center gap-2 mb-1">
            <div class="w-1 h-4 bg-rose-500 rounded-full"></div>
            <h3 class="text-sm font-medium text-gray-200">智能去重</h3>
          </div>
          <div class="bg-darker/50 rounded-xl p-4 border border-gray-700/50 grid grid-cols-2 gap-3">
            <div>
              <label class="text-xs text-gray-500 block mb-1.5">变化阈值 (%)</label>
              <input
                v-model.number="settings.change_threshold"
                type="number"
                min="1"
                max="20"
                class="w-full bg-dark border border-gray-700 rounded-lg px-3 py-2.5 text-sm focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary/50 transition-all"
              />
              <span class="text-xs text-gray-600 mt-1.5 block">屏幕变化低于此比例时跳过</span>
            </div>
            <div>
              <label class="text-xs text-gray-500 block mb-1.5">最大静默时间 (分钟)</label>
              <input
                v-model.number="settings.max_silent_minutes"
                type="number"
                min="5"
                max="120"
                class="w-full bg-dark border border-gray-700 rounded-lg px-3 py-2.5 text-sm focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary/50 transition-all"
              />
              <span class="text-xs text-gray-600 mt-1.5 block">超时强制记录一次</span>
            </div>
          </div>
        </section>

        <!-- 输出配置 -->
        <section class="space-y-3">
          <div class="flex items-center gap-2 mb-1">
            <div class="w-1 h-4 bg-pink-500 rounded-full"></div>
            <h3 class="text-sm font-medium text-gray-200">输出配置</h3>
          </div>
          <div class="bg-darker/50 rounded-xl p-4 border border-gray-700/50">
            <label class="text-xs text-gray-500 block mb-1.5">Obsidian Vault 路径</label>
            <input
              v-model="settings.obsidian_path"
              type="text"
              placeholder="/Users/你的名字/Documents/Obsidian Vault"
              class="w-full bg-dark border border-gray-700 rounded-lg px-3 py-2.5 text-sm focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary/50 transition-all"
            />
          </div>
        </section>

        <!-- 快捷键 -->
        <section class="space-y-3">
          <div class="flex items-center gap-2 mb-1">
            <div class="w-1 h-4 bg-cyan-500 rounded-full"></div>
            <h3 class="text-sm font-medium text-gray-200">快捷键</h3>
          </div>
          <div class="bg-darker/50 rounded-xl p-4 border border-gray-700/50">
            <div class="flex items-center justify-between">
              <span class="text-sm text-gray-400">闪念胶囊</span>
              <kbd class="px-3 py-1.5 bg-dark rounded-lg border border-gray-600 text-xs font-mono text-gray-300 shadow-sm">Alt + Space</kbd>
            </div>
          </div>
        </section>
      </div>

      <!-- Footer -->
      <div class="px-6 py-4 border-t border-gray-700 flex items-center justify-between gap-3 bg-darker/50 shrink-0">
        <span v-if="saveStatus" :class="saveStatus === 'ok' ? 'text-green-400' : 'text-red-400'" class="text-xs font-medium">
          {{ saveStatus === 'ok' ? '✓ 已保存' : '✗ 保存失败' }}
        </span>
        <span v-else class="text-xs text-transparent">placeholder</span>
        <div class="flex gap-2.5">
          <button
            @click="$emit('close')"
            class="px-4 py-2 rounded-lg text-sm hover:bg-gray-700/50 transition-all border border-gray-600"
          >
            取消
          </button>
          <button
            @click="saveSettings"
            :disabled="isSaving"
            class="px-5 py-2 bg-gradient-to-r from-primary/90 to-blue-600 hover:from-primary hover:to-blue-500 disabled:opacity-50 disabled:cursor-not-allowed rounded-lg text-sm font-medium transition-all shadow-md hover:shadow-lg"
          >
            {{ isSaving ? '保存中…' : '💾 保存' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const emit = defineEmits(['close'])

const showApiKey = ref(false)
const isSaving = ref(false)
const saveStatus = ref('')

const settings = ref({
  api_base_url: '',
  api_key: '',
  model_name: 'gpt-4o',
  screenshot_interval: 5,
  summary_time: '18:00',
  obsidian_path: '',
  summary_model_name: '',
  analysis_prompt: '',
  summary_prompt: '',
  change_threshold: 3,
  max_silent_minutes: 30
})

const loadSettings = async () => {
  try {
    const loaded = await invoke('get_settings')
    settings.value = { ...settings.value, ...loaded }
  } catch (err) {
    console.error('Failed to load settings:', err)
  }
}

const saveSettings = async () => {
  isSaving.value = true
  saveStatus.value = ''
  try {
    await invoke('save_settings', { settings: settings.value })
    saveStatus.value = 'ok'
    setTimeout(() => emit('close'), 800)
  } catch (err) {
    console.error('Failed to save settings:', err)
    saveStatus.value = 'err'
  } finally {
    isSaving.value = false
  }
}

onMounted(() => {
  loadSettings()
})
</script>

<style>
.custom-scrollbar::-webkit-scrollbar {
  width: 6px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: rgba(30, 41, 59, 0.5);
  border-radius: 3px;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(71, 85, 105, 0.8);
  border-radius: 3px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(99, 102, 241, 0.6);
}
</style>
