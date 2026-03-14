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
              <label class="text-xs text-gray-300 block mb-1">Base URL</label>
              <input
                v-model="settings.api_base_url"
                type="text"
                placeholder="https://api.openai.com/v1"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
              />
            </div>
            <div>
              <label class="text-xs text-gray-300 block mb-1">API Key</label>
              <div class="relative">
                <input
                  v-model="settings.api_key"
                  :type="showApiKey ? 'text' : 'password'"
                  placeholder="sk-..."
                  class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 pr-16 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
                />
                <button
                  @click="showApiKey = !showApiKey"
                  type="button"
                  class="absolute right-2 top-1/2 -translate-y-1/2 text-gray-500 hover:text-gray-300 transition-colors text-xs px-2 py-1 rounded hover:bg-gray-700"
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
              <label class="text-xs text-gray-300 block mb-1">分析模型</label>
              <input
                v-model="settings.model_name"
                type="text"
                placeholder="gpt-4o"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
              />
              <span class="text-xs text-gray-500 mt-1 block">需要支持 Vision 能力的模型</span>
            </div>
            <div>
              <label class="text-xs text-gray-300 block mb-1">分析 Prompt</label>
              <textarea
                v-model="settings.analysis_prompt"
                rows="4"
                placeholder="留空使用默认 Prompt"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none resize-y"
              />
              <div class="flex gap-3 mt-2">
                <button
                  type="button"
                  @click="showDefaultPrompt"
                  class="text-xs text-gray-400 hover:text-primary transition-colors"
                >
                  查看默认
                </button>
                <button
                  type="button"
                  @click="resetPrompt"
                  class="text-xs text-gray-400 hover:text-primary transition-colors"
                >
                  重置为默认
                </button>
              </div>
            </div>
          </div>
        </div>

        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">日报生成</h3>
          <div class="space-y-3">
            <div>
              <label class="text-xs text-gray-300 block mb-1">日报标题格式</label>
              <input
                v-model="settings.summary_title_format"
                type="text"
                placeholder="工作日报 - {date}"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
              />
              <span class="text-xs text-gray-500 mt-1 block">使用 {date} 作为日期占位符，留空使用默认格式</span>
            </div>
            <div>
              <label class="text-xs text-gray-300 block mb-1">日报模型</label>
              <input
                v-model="settings.summary_model_name"
                type="text"
                placeholder="留空则使用分析模型"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
              />
              <span class="text-xs text-gray-500 mt-1 block">纯文本模型即可，不需要 Vision</span>
            </div>
            <div>
              <label class="text-xs text-gray-300 block mb-1">日报 Prompt</label>
              <textarea
                v-model="settings.summary_prompt"
                rows="4"
                placeholder="留空使用默认 Prompt。用 {records} 表示今日记录的插入位置"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none resize-y"
              />
            </div>
            <div class="flex items-center gap-2 pt-1">
              <input
                v-model="settings.include_manual_records"
                type="checkbox"
                id="include_manual_records"
                class="w-4 h-4 rounded border-gray-600 bg-darker text-primary focus:ring-primary focus:ring-offset-0 cursor-pointer"
              />
              <label for="include_manual_records" class="text-xs text-gray-300 cursor-pointer">
                包含闪念胶囊记录
              </label>
              <span class="text-xs text-gray-500">（取消勾选则仅使用自动截图分析）</span>
            </div>
          </div>
        </div>

        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">时间策略</h3>
          <div class="space-y-3">
            <div>
              <label class="text-xs text-gray-300 block mb-1">截图间隔 (分钟)</label>
              <input
                v-model.number="settings.screenshot_interval"
                type="number"
                min="1"
                max="60"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
              />
            </div>
            <div>
              <label class="text-xs text-gray-300 block mb-1">每日总结时间</label>
              <input
                v-model="settings.summary_time"
                type="time"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
              />
            </div>
          </div>
        </div>

        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">智能去重</h3>
          <div class="space-y-3">
            <div>
              <label class="text-xs text-gray-300 block mb-1">变化阈值 (%)</label>
              <input
                v-model.number="settings.change_threshold"
                type="number"
                min="1"
                max="20"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
              />
              <span class="text-xs text-gray-500 mt-1 block">屏幕变化低于此比例时跳过截图，避免重复记录</span>
            </div>
            <div>
              <label class="text-xs text-gray-300 block mb-1">最大静默时间 (分钟)</label>
              <input
                v-model.number="settings.max_silent_minutes"
                type="number"
                min="5"
                max="120"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
              />
              <span class="text-xs text-gray-500 mt-1 block">即使屏幕无变化，超过此时间也会强制记录一次</span>
            </div>
          </div>
        </div>

        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">输出配置</h3>
          <div>
            <label class="text-xs text-gray-300 block mb-1">Obsidian Vault 路径</label>
            <input
              v-model="settings.obsidian_path"
              type="text"
              placeholder="/Users/你的名字/Documents/Obsidian Vault"
              class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
            />
          </div>
        </div>

        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">快捷键</h3>
          <div class="bg-darker rounded-lg px-3 py-2 text-sm text-gray-400 border border-gray-700">
            闪念胶囊: Alt + Space
          </div>
        </div>

        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">调试工具</h3>
          <div class="space-y-3">
            <button
              @click="exportLogs"
              :disabled="isExportingLogs"
              class="w-full px-4 py-2 bg-gray-700 hover:bg-gray-600 disabled:opacity-50 rounded-lg text-sm text-gray-200 transition-colors flex items-center justify-center gap-2"
            >
              {{ isExportingLogs ? '导出中…' : '📤 导出日志' }}
            </button>
            <span v-if="exportError" class="text-xs text-red-400 block">{{ exportError }}</span>
          </div>
        </div>
      </div>

      <!-- Default Prompt Modal -->
      <div v-if="showDefaultPromptModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-[60]" @click.self="showDefaultPromptModal = false">
        <div class="bg-dark rounded-2xl w-[500px] max-h-[80vh] overflow-hidden border border-gray-700">
          <div class="px-6 py-4 border-b border-gray-700 flex items-center justify-between">
            <h3 class="text-lg font-semibold">默认分析 Prompt</h3>
            <button @click="showDefaultPromptModal = false" class="text-gray-400 hover:text-white">✕</button>
          </div>
          <div class="p-6 overflow-y-auto max-h-[60vh]">
            <pre class="text-sm text-gray-300 whitespace-pre-wrap bg-darker p-4 rounded-lg">{{ defaultPromptContent }}</pre>
          </div>
          <div class="px-6 py-4 border-t border-gray-700 flex justify-end">
            <button
              @click="showDefaultPromptModal = false"
              class="px-4 py-2 bg-primary rounded-lg text-sm font-medium text-white hover:bg-blue-600 transition-colors"
            >
              关闭
            </button>
          </div>
        </div>
      </div>

      <div class="px-6 py-4 border-t border-gray-700 flex items-center justify-between gap-3">
        <div class="flex flex-col">
          <span v-if="saveStatus === 'ok'" class="text-green-400 text-xs flex items-center gap-1">
            <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20"><path d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"/></svg>
            已保存
          </span>
          <span v-else-if="saveStatus === 'err'" class="text-red-400 text-xs flex items-center gap-1">
            <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20"><path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd"/></svg>
            保存失败
          </span>
          <span v-if="saveError" class="text-xs text-red-400 mt-1">{{ saveError }}</span>
          <span v-else-if="!saveStatus" class="text-xs text-transparent select-none">placeholder</span>
        </div>
        <div class="flex gap-3">
          <button
            @click="$emit('close')"
            class="px-4 py-2 rounded-lg text-sm text-gray-300 hover:bg-gray-700 hover:text-white transition-colors"
          >
            取消
          </button>
          <button
            @click="saveSettings"
            :disabled="isSaving"
            class="px-4 py-2 bg-primary rounded-lg text-sm font-medium text-white hover:bg-blue-600 disabled:opacity-50 transition-colors"
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
import { save } from '@tauri-apps/plugin-dialog'
import { writeFile, writeTextFile } from '@tauri-apps/plugin-fs'
import { showError, showSuccess } from '../stores/toast.js'

const emit = defineEmits(['close'])

const showApiKey = ref(false)
const isSaving = ref(false)
const saveStatus = ref('')
const saveError = ref('')
const isExportingLogs = ref(false)
const exportError = ref('')
const showDefaultPromptModal = ref(false)
const defaultPromptContent = ref('')

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
  max_silent_minutes: 30,
  summary_title_format: '',
  include_manual_records: true
})

const loadSettings = async () => {
  try {
    const loaded = await invoke('get_settings')
    settings.value = { ...settings.value, ...loaded }
  } catch (err) {
    console.error('Failed to load settings:', err)
  }
}

const validateSettings = () => {
  // Validate API URL format
  if (settings.value.api_base_url && settings.value.api_base_url.trim()) {
    try {
      new URL(settings.value.api_base_url.trim())
    } catch {
      return 'API Base URL 格式无效，请输入有效的 URL'
    }
  }

  // Validate screenshot interval
  if (settings.value.screenshot_interval < 1 || settings.value.screenshot_interval > 60) {
    return '截图间隔必须在 1-60 分钟之间'
  }

  // Validate change threshold
  if (settings.value.change_threshold < 1 || settings.value.change_threshold > 20) {
    return '变化阈值必须在 1-20% 之间'
  }

  // Validate max silent minutes
  if (settings.value.max_silent_minutes < 5 || settings.value.max_silent_minutes > 120) {
    return '最大静默时间必须在 5-120 分钟之间'
  }

  return null
}

const saveSettings = async () => {
  // Validate settings first
  const validationError = validateSettings()
  if (validationError) {
    saveStatus.value = 'err'
    saveError.value = validationError
    return
  }

  isSaving.value = true
  saveStatus.value = ''
  saveError.value = ''
  try {
    await invoke('save_settings', { settings: settings.value })
    saveStatus.value = 'ok'
    showSuccess('设置已保存')
    setTimeout(() => emit('close'), 800)
  } catch (err) {
    console.error('Failed to save settings:', err)
    saveStatus.value = 'err'
    saveError.value = String(err)
    showError(err)
  } finally {
    isSaving.value = false
  }
}

const exportLogs = async () => {
  isExportingLogs.value = true
  exportError.value = ''

  try {
    // Get log content
    const logContent = await invoke('get_logs_for_export')

    // Open save dialog
    const filePath = await save({
      defaultPath: `daily-logger-${new Date().toISOString().slice(0, 10)}.log`,
      filters: [
        { name: 'Log Files', extensions: ['log'] },
        { name: 'Text Files', extensions: ['txt'] },
        { name: 'All Files', extensions: ['*'] }
      ]
    })

    if (filePath) {
      // Write the log content to the selected file
      await writeTextFile(filePath, logContent)
      showSuccess('日志导出成功')
    }
  } catch (err) {
    console.error('Failed to export logs:', err)
    if (err !== 'Log file does not exist') {
      exportError.value = `导出失败: ${err}`
      showError(err)
    } else {
      exportError.value = '日志文件不存在'
    }
  } finally {
    isExportingLogs.value = false
  }
}

const showDefaultPrompt = async () => {
  try {
    defaultPromptContent.value = await invoke('get_default_analysis_prompt')
    showDefaultPromptModal.value = true
  } catch (err) {
    console.error('Failed to get default prompt:', err)
    showError(err)
  }
}

const resetPrompt = () => {
  settings.value.analysis_prompt = ''
  showSuccess('已重置为默认 Prompt，保存后生效')
}

onMounted(() => {
  loadSettings()
})
</script>
