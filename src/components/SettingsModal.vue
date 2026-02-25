<template>
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="$emit('close')">
    <div class="bg-dark rounded-2xl w-[500px] max-h-[80vh] overflow-y-auto border border-gray-700">
      <div class="px-6 py-4 border-b border-gray-700 flex items-center justify-between">
        <h2 class="text-lg font-semibold">设置</h2>
        <button @click="$emit('close')" class="text-gray-400 hover:text-white">✕</button>
      </div>
      
      <div class="p-6 space-y-6">
        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">API 配置</h3>
          <div class="space-y-3">
            <div>
              <label class="text-xs text-gray-500 block mb-1">Base URL</label>
              <input
                v-model="settings.api_base_url"
                type="text"
                placeholder="https://api.openai.com/v1"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm focus:border-primary focus:outline-none"
              />
            </div>
            <div>
              <label class="text-xs text-gray-500 block mb-1">API Key</label>
              <div class="relative">
                <input
                  v-model="settings.api_key"
                  :type="showApiKey ? 'text' : 'password'"
                  placeholder="sk-..."
                  class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 pr-10 text-sm focus:border-primary focus:outline-none"
                />
                <button
                  @click="showApiKey = !showApiKey"
                  type="button"
                  class="absolute right-2 top-1/2 -translate-y-1/2 text-gray-500 hover:text-gray-300 transition-colors text-xs px-1"
                  :title="showApiKey ? '隐藏' : '显示'"
                >{{ showApiKey ? '隐藏' : '显示' }}</button>
              </div>
            </div>
          </div>
        </div>

        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">截图分析 (Vision)</h3>
          <div class="space-y-3">
            <div>
              <label class="text-xs text-gray-500 block mb-1">分析模型</label>
              <input
                v-model="settings.model_name"
                type="text"
                placeholder="gpt-4o"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm focus:border-primary focus:outline-none"
              />
              <span class="text-xs text-gray-600 mt-1 block">需要支持 Vision 能力的模型</span>
            </div>
            <div>
              <label class="text-xs text-gray-500 block mb-1">分析 Prompt</label>
              <textarea
                v-model="settings.analysis_prompt"
                rows="4"
                placeholder="留空使用默认 Prompt"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm focus:border-primary focus:outline-none resize-y"
              />
            </div>
          </div>
        </div>

        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">日报生成</h3>
          <div class="space-y-3">
            <div>
              <label class="text-xs text-gray-500 block mb-1">日报模型</label>
              <input
                v-model="settings.summary_model_name"
                type="text"
                placeholder="留空则使用分析模型"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm focus:border-primary focus:outline-none"
              />
              <span class="text-xs text-gray-600 mt-1 block">纯文本模型即可，不需要 Vision</span>
            </div>
            <div>
              <label class="text-xs text-gray-500 block mb-1">日报 Prompt</label>
              <textarea
                v-model="settings.summary_prompt"
                rows="4"
                placeholder="留空使用默认 Prompt。用 {records} 表示今日记录的插入位置"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm focus:border-primary focus:outline-none resize-y"
              />
            </div>
          </div>
        </div>

        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">时间策略</h3>
          <div class="space-y-3">
            <div>
              <label class="text-xs text-gray-500 block mb-1">截图间隔 (分钟)</label>
              <input 
                v-model.number="settings.screenshot_interval"
                type="number" 
                min="1"
                max="60"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm focus:border-primary focus:outline-none"
              />
            </div>
            <div>
              <label class="text-xs text-gray-500 block mb-1">每日总结时间</label>
              <input
                v-model="settings.summary_time"
                type="time"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm focus:border-primary focus:outline-none"
              />
            </div>
          </div>
        </div>

        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">智能去重</h3>
          <div class="space-y-3">
            <div>
              <label class="text-xs text-gray-500 block mb-1">变化阈值 (%)</label>
              <input
                v-model.number="settings.change_threshold"
                type="number"
                min="1"
                max="20"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm focus:border-primary focus:outline-none"
              />
              <span class="text-xs text-gray-600 mt-1 block">屏幕变化低于此比例时跳过截图，避免重复记录</span>
            </div>
            <div>
              <label class="text-xs text-gray-500 block mb-1">最大静默时间 (分钟)</label>
              <input
                v-model.number="settings.max_silent_minutes"
                type="number"
                min="5"
                max="120"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm focus:border-primary focus:outline-none"
              />
              <span class="text-xs text-gray-600 mt-1 block">即使屏幕无变化，超过此时间也会强制记录一次</span>
            </div>
          </div>
        </div>

        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">输出配置</h3>
          <div>
            <label class="text-xs text-gray-500 block mb-1">Obsidian Vault 路径</label>
            <input
              v-model="settings.obsidian_path"
              type="text"
              placeholder="/Users/你的名字/Documents/Obsidian Vault"
              class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm focus:border-primary focus:outline-none"
            />
          </div>
        </div>

        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">快捷键</h3>
          <div class="bg-darker rounded-lg px-3 py-2 text-sm text-gray-400 border border-gray-700">
            闪念胶囊: Alt + Space
          </div>
        </div>
      </div>

      <div class="px-6 py-4 border-t border-gray-700 flex items-center justify-between gap-3">
        <span v-if="saveStatus" :class="saveStatus === 'ok' ? 'text-green-400' : 'text-red-400'" class="text-xs">
          {{ saveStatus === 'ok' ? '✓ 已保存' : '✗ 保存失败' }}
        </span>
        <span v-else class="text-xs text-transparent">placeholder</span>
        <div class="flex gap-3">
          <button
            @click="$emit('close')"
            class="px-4 py-2 rounded-lg text-sm hover:bg-gray-700 transition-colors"
          >
            取消
          </button>
          <button
            @click="saveSettings"
            :disabled="isSaving"
            class="px-4 py-2 bg-primary rounded-lg text-sm font-medium hover:bg-blue-600 disabled:opacity-50 transition-colors"
          >
            {{ isSaving ? '保存中…' : '保存' }}
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
