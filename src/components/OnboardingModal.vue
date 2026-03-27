<template>
  <div class="fixed inset-0 bg-black/70 flex items-center justify-center z-50" @click.self="handleClose">
    <div class="bg-darker border border-gray-700 rounded-xl w-full max-w-lg mx-4 shadow-2xl">
      <!-- Header -->
      <div class="flex items-center justify-between px-6 py-4 border-b border-gray-700">
        <h2 class="text-lg font-medium text-white">{{ isCompleted ? '🎉 欢迎使用 DailyLogger' : '欢迎使用 DailyLogger' }}</h2>
        <button
          v-if="isCompleted"
          @click="handleClose"
          class="text-gray-400 hover:text-white transition-colors p-1 rounded hover:bg-gray-700"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Step Indicator -->
      <div v-if="!isCompleted" class="px-6 py-4 border-b border-gray-700">
        <div class="flex items-center justify-center gap-2">
          <div v-for="step in 3" :key="step" class="flex items-center">
            <div
              class="w-8 h-8 rounded-full flex items-center justify-center text-sm font-medium transition-colors"
              :class="step < currentStep ? 'bg-green-500 text-white' : step === currentStep ? 'bg-primary text-white' : 'bg-gray-700 text-gray-400'"
            >
              <span v-if="step < currentStep">✓</span>
              <span v-else>{{ step }}</span>
            </div>
            <div v-if="step < 3" class="w-12 h-0.5" :class="step < currentStep ? 'bg-green-500' : 'bg-gray-700'"></div>
          </div>
        </div>
        <div class="flex justify-center gap-8 mt-2 text-xs text-gray-400">
          <span>API 配置</span>
          <span>输出路径</span>
          <span>完成</span>
        </div>
      </div>

      <!-- Content -->
      <div class="px-6 py-6">
        <!-- Step 1: API Configuration -->
        <div v-if="currentStep === 1 && !isCompleted">
          <h3 class="text-sm font-medium text-gray-300 mb-4">Step 1: 配置 AI API</h3>
          <div class="space-y-4">
            <div>
              <label class="text-xs text-gray-300 block mb-1">API Base URL</label>
              <input
                v-model="apiBaseUrl"
                type="text"
                placeholder="https://api.openai.com/v1"
                class="w-full bg-gray-800 border border-gray-600 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
              />
            </div>
            <div>
              <label class="text-xs text-gray-300 block mb-1">API Key</label>
              <input
                v-model="apiKey"
                type="password"
                placeholder="sk-..."
                class="w-full bg-gray-800 border border-gray-600 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
              />
            </div>
            <div>
              <label class="text-xs text-gray-300 block mb-1">Model Name</label>
              <input
                v-model="modelName"
                type="text"
                placeholder="gpt-4o"
                class="w-full bg-gray-800 border border-gray-600 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
              />
            </div>
            <div class="pt-2">
              <button
                @click="testConnection"
                :disabled="isTestingConnection || !apiBaseUrl || !modelName"
                class="px-4 py-2 text-sm bg-gray-700 hover:bg-gray-600 disabled:opacity-50 disabled:cursor-not-allowed rounded-lg transition-colors"
              >
                {{ isTestingConnection ? '测试中...' : '测试连接' }}
              </button>
              <span v-if="connectionTestResult" :class="connectionTestResult.success ? 'text-green-400' : 'text-red-400'" class="ml-3 text-xs">
                {{ connectionTestResult.message }}
              </span>
            </div>
          </div>
        </div>

        <!-- Step 2: Obsidian Path -->
        <div v-if="currentStep === 2 && !isCompleted">
          <h3 class="text-sm font-medium text-gray-300 mb-4">Step 2: 配置输出路径</h3>
          <div class="space-y-4">
            <div>
              <label class="text-xs text-gray-300 block mb-1">Obsidian Vault 路径</label>
              <div class="flex gap-2">
                <input
                  v-model="obsidianPath"
                  type="text"
                  placeholder="选择文件夹路径..."
                  class="flex-1 bg-gray-800 border border-gray-600 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
                  readonly
                />
                <button
                  @click="selectFolder"
                  class="px-4 py-2 text-sm bg-gray-700 hover:bg-gray-600 rounded-lg transition-colors"
                >
                  选择文件夹
                </button>
              </div>
              <p class="text-xs text-gray-500 mt-1">选择用于保存日报的 Obsidian Vault 文件夹</p>
            </div>
            <div v-if="obsidianPathError" class="text-xs text-red-400">
              {{ obsidianPathError }}
            </div>
            <div v-if="obsidianPath && !obsidianPathError" class="text-xs text-green-400">
              ✓ 路径已选择
            </div>
          </div>
        </div>

        <!-- Step 3: Completion -->
        <div v-if="currentStep === 3 && !isCompleted">
          <h3 class="text-sm font-medium text-gray-300 mb-4">Step 3: 完成设置</h3>
          <div class="space-y-3">
            <div class="bg-gray-800 rounded-lg p-4">
              <div class="text-sm text-gray-300 space-y-2">
                <div v-if="apiBaseUrl" class="flex items-center gap-2">
                  <span class="text-green-400">✓</span>
                  <span>API 已配置 ({{ apiBaseUrl }})</span>
                </div>
                <div v-else class="flex items-center gap-2">
                  <span class="text-gray-500">○</span>
                  <span class="text-gray-500">API 未配置（可稍后在设置中补充）</span>
                </div>
                <div v-if="obsidianPath" class="flex items-center gap-2">
                  <span class="text-green-400">✓</span>
                  <span>输出路径已设置</span>
                </div>
                <div v-else class="flex items-center gap-2">
                  <span class="text-gray-500">○</span>
                  <span class="text-gray-500">输出路径未设置（可稍后在设置中补充）</span>
                </div>
              </div>
            </div>
            <p class="text-xs text-gray-500">
              提示：这些配置稍后都可以在设置中进行修改
            </p>
          </div>
        </div>

        <!-- Completion Screen -->
        <div v-if="isCompleted" class="text-center py-4">
          <div class="text-5xl mb-4">🎉</div>
          <h3 class="text-lg font-medium text-white mb-2">设置完成！</h3>
          <p class="text-sm text-gray-400">欢迎使用 DailyLogger，开始记录您的工作日志吧</p>
        </div>
      </div>

      <!-- Footer -->
      <div class="px-6 py-4 border-t border-gray-700 flex justify-between">
        <button
          v-if="currentStep > 1 && !isCompleted"
          @click="previousStep"
          class="px-4 py-2 text-sm text-gray-400 hover:text-white transition-colors"
        >
          ← 上一步
        </button>
        <div v-else></div>
        <div class="flex gap-2">
          <button
            v-if="!isCompleted"
            @click="skip"
            class="px-4 py-2 text-sm text-gray-400 hover:text-white transition-colors"
          >
            跳过
          </button>
          <button
            v-if="!isCompleted && currentStep < 3"
            @click="nextStep"
            :disabled="currentStep === 1 && !connectionTested"
            class="px-4 py-2 text-sm bg-primary hover:bg-primary/80 disabled:opacity-50 disabled:cursor-not-allowed rounded-lg transition-colors"
          >
            下一步 →
          </button>
          <button
            v-if="!isCompleted && currentStep === 3"
            @click="completeOnboarding"
            class="px-4 py-2 text-sm bg-green-600 hover:bg-green-500 rounded-lg transition-colors"
          >
            开始使用
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { settingsActions } from '../features/settings/actions'

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'completed'): void
}>()

interface ConnectionTestResult {
  success: boolean
  message: string
  latency_ms?: number
}

interface Settings {
  api_base_url: string
  api_key: string
  model_name: string
  obsidian_path: string
  onboarding_completed: boolean
  // PERF-001: Proxy settings
  proxy_enabled?: boolean
  proxy_host?: string
  proxy_port?: number
  proxy_username?: string
  proxy_password?: string
  test_model_name?: string
  is_ollama?: boolean
}

const currentStep = ref(1)
const isCompleted = ref(false)

// Form data
const apiBaseUrl = ref('')
const apiKey = ref('')
const modelName = ref('')
const obsidianPath = ref('')
const obsidianPathError = ref('')

// Connection testing
const isTestingConnection = ref(false)
const connectionTestResult = ref<ConnectionTestResult | null>(null)
const connectionTested = ref(false)

onMounted(async () => {
  try {
    const settings = await settingsActions.getSettings()
    apiBaseUrl.value = settings.api_base_url || ''
    apiKey.value = settings.api_key || ''
    modelName.value = settings.model_name || ''
    obsidianPath.value = settings.obsidian_path || ''

    // If already completed, skip onboarding
    if (settings.onboarding_completed) {
      emit('close')
      return
    }
  } catch (err) {
    console.error('Failed to load settings:', err)
  }
})

async function testConnection() {
  if (!apiBaseUrl.value || !modelName.value) return

  isTestingConnection.value = true
  connectionTestResult.value = null
  connectionTested.value = false

  try {
    const result = await invoke<ConnectionTestResult>('test_api_connection_with_ollama', {
      apiBaseUrl: apiBaseUrl.value,
      apiKey: apiKey.value || null,
      modelName: modelName.value,
      proxyEnabled: false,
      proxyHost: null,
      proxyPort: null,
      proxyUsername: null,
      proxyPassword: null,
    })
    connectionTestResult.value = result
    if (result.success) {
      connectionTested.value = true
    }
  } catch (err) {
    connectionTestResult.value = {
      success: false,
      message: String(err),
    }
  } finally {
    isTestingConnection.value = false
  }
}

async function selectFolder() {
  obsidianPathError.value = ''
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择 Obsidian Vault 路径',
    })
    if (selected) {
      obsidianPath.value = selected as string
    }
  } catch (err) {
    obsidianPathError.value = `选择文件夹失败: ${err}`
  }
}

function nextStep() {
  if (currentStep.value < 3) {
    currentStep.value++
  }
}

function previousStep() {
  if (currentStep.value > 1) {
    currentStep.value--
  }
}

function skip() {
  completeOnboarding()
}

async function completeOnboarding() {
  try {
    const currentSettings = await settingsActions.getSettings()

    const updatedSettings: Record<string, string | boolean | null> = {
      onboarding_completed: true,
    }

    // Save API settings if provided
    if (apiBaseUrl.value) {
      updatedSettings.api_base_url = apiBaseUrl.value
    }
    if (apiKey.value) {
      updatedSettings.api_key = apiKey.value
    }
    if (modelName.value) {
      updatedSettings.model_name = modelName.value
    }
    if (obsidianPath.value) {
      updatedSettings.obsidian_path = obsidianPath.value
    }

    await settingsActions.saveSettings({ ...currentSettings, ...updatedSettings } as Partial<Settings>)

    isCompleted.value = true

    setTimeout(() => {
      emit('completed')
      emit('close')
    }, 1500)
  } catch (err) {
    console.error('Failed to save settings:', err)
  }
}

function handleClose() {
  if (isCompleted.value) {
    emit('close')
  }
}
</script>
