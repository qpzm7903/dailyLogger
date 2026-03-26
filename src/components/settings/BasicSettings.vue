<template>
  <div class="space-y-6">
    <!-- API Configuration -->
    <div>
      <h3 class="text-sm font-medium text-gray-300 mb-3">{{ $t('settings.apiConfig') }}</h3>
      <div class="space-y-3">
        <div>
          <label class="text-xs text-gray-300 block mb-1">Base URL</label>
          <div class="flex items-center gap-2">
            <input
              v-model="localSettings.api_base_url"
              type="text"
              placeholder="https://api.openai.com/v1"
              class="flex-1 bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
            />
            <!-- Ollama status indicator -->
            <span v-if="isOllama" class="text-xs text-green-400 whitespace-nowrap">🦙 Ollama</span>
          </div>
          <span class="text-xs text-gray-500 mt-1 block">
            {{ $t('settings.baseUrlOllamaHint') }}
          </span>
        </div>
        <div>
          <label class="text-xs text-gray-300 block mb-1">
            {{ $t('settings.apiKey') }}
            <span v-if="isOllama" class="text-gray-500">{{ $t('settings.apiKeyOllamaHint') }}</span>
          </label>
          <div class="relative">
            <input
              v-model="localSettings.api_key"
              :type="showApiKey ? 'text' : 'password'"
              placeholder="sk-..."
              class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 pr-16 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
            />
            <button
              @click="showApiKey = !showApiKey"
              type="button"
              class="absolute right-2 top-1/2 -translate-y-1/2 text-gray-500 hover:text-gray-300 transition-colors text-xs px-2 py-1 rounded hover:bg-gray-700"
              :title="showApiKey ? $t('common.hide') : $t('common.show')"
            >{{ showApiKey ? $t('common.hide') : $t('common.show') }}</button>
          </div>
        </div>
        <!-- PERF-001: Test Model Name -->
        <div>
          <label class="text-xs text-gray-300 block mb-1">Test Model</label>
          <input
            v-model="localSettings.test_model_name"
            type="text"
            placeholder="gpt-4o"
            class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
          />
          <span class="text-xs text-gray-500 mt-1 block">
            {{ $t('settings.testModelHint') }}
          </span>
        </div>

        <!-- Test Connection Button -->
        <div class="pt-2">
          <div class="flex items-center gap-2">
            <button
              @click="testConnection"
              :disabled="isTestingConnection || !localSettings.api_base_url || !localSettings.model_name || (!isOllama && !localSettings.api_key)"
              class="px-3 py-1.5 text-sm bg-gray-700 hover:bg-gray-600 disabled:opacity-50 disabled:cursor-not-allowed rounded-lg transition-colors"
            >
              {{ isTestingConnection ? $t('settings.testing') : $t('settings.testConnection') }}
            </button>
            <!-- Ollama model fetch button -->
            <button
              v-if="isOllama"
              @click="fetchOllamaModels"
              :disabled="isLoadingOllamaModels || !localSettings.api_base_url"
              class="px-3 py-1.5 text-sm bg-purple-700 hover:bg-purple-600 disabled:opacity-50 disabled:cursor-not-allowed rounded-lg transition-colors"
            >
              {{ isLoadingOllamaModels ? $t('settings.fetching') : $t('settings.fetchModels') }}
            </button>
          </div>
          <span v-if="connectionTestResult" :class="connectionTestResult.success ? 'text-green-400' : 'text-red-400'" class="ml-2 text-xs">
            {{ connectionTestResult.message }}
            <span v-if="connectionTestResult.latency_ms">({{ connectionTestResult.latency_ms }}ms)</span>
          </span>
        </div>

        <!-- Ollama model list -->
        <div v-if="isOllama" class="mt-3">
          <div class="flex items-center justify-between mb-1">
            <label class="text-xs text-gray-300">{{ $t('settings.selectModel') }}</label>
            <button
              @click="fetchOllamaModels"
              :disabled="isLoadingOllamaModels"
              type="button"
              class="text-xs text-primary hover:text-primary/80 disabled:opacity-50 transition-colors"
            >
              {{ isLoadingOllamaModels ? '...' : $t('settings.refreshModels') }}
            </button>
          </div>

          <!-- Pull model input -->
          <div class="flex gap-2 mb-2">
            <input
              v-model="pullModelName"
              type="text"
              :placeholder="$t('settings.pullModelPlaceholder')"
              class="flex-1 bg-darker border border-gray-700 rounded px-2 py-1 text-xs text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
            />
            <select
              v-model="pullModelQuantization"
              class="bg-darker border border-gray-700 rounded px-2 py-1 text-xs text-gray-100 focus:border-primary focus:outline-none"
              :title="$t('settings.quantizationTooltip')"
            >
              <option value="">{{ $t('settings.defaultQuantization') }}</option>
              <option value="q4_0">q4_0 ({{ $t('settings.smallest') }})</option>
              <option value="q4_1">q4_1</option>
              <option value="q5_0">q5_0</option>
              <option value="q5_1">q5_1</option>
              <option value="q8_0">q8_0 ({{ $t('settings.largest') }})</option>
              <option value="f16">f16 ({{ $t('settings.noCompression') }})</option>
            </select>
            <button
              @click="pullModel"
              :disabled="isPullingModel || !pullModelName"
              type="button"
              class="px-2 py-1 text-xs rounded bg-primary hover:bg-primary/80 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              {{ isPullingModel ? $t('settings.pulling') : $t('settings.pullModel') }}
            </button>
          </div>

          <!-- Model list -->
          <div v-if="ollamaModels.length > 0" class="flex flex-wrap gap-2">
            <div
              v-for="model in ollamaModels"
              :key="model.name"
              class="flex items-center gap-1 px-2 py-1 text-xs rounded border transition-colors"
              :class="localSettings.model_name === model.name ? 'bg-primary border-primary text-white' : 'bg-darker border-gray-600 text-gray-300 hover:border-primary'"
            >
              <button
                @click="selectOllamaModel(model.name)"
                type="button"
                class="hover:text-white transition-colors"
              >
                {{ model.name }}<span v-if="model.size" class="text-gray-400 ml-1">({{ model.size }})</span>
              </button>
              <button
                @click="openCopyModelModal(model.name)"
                type="button"
                class="ml-1 text-gray-400 hover:text-blue-400 transition-colors"
                :title="$t('settings.copyModel')"
              >⧉</button>
              <button
                @click="deleteModel(model.name)"
                :disabled="isDeletingModel === model.name"
                type="button"
                class="ml-1 text-gray-400 hover:text-red-400 disabled:opacity-50 transition-colors"
                :title="$t('settings.deleteModel')"
              >×</button>
            </div>
          </div>
          <p v-else-if="!isLoadingOllamaModels" class="text-xs text-gray-500">{{ $t('settings.noModelsFound') }}</p>

          <!-- Running models status -->
          <div class="mt-3 pt-3 border-t border-gray-700">
            <div class="flex items-center justify-between mb-2">
              <span class="text-xs text-gray-400">{{ $t('settings.runningModels') }}</span>
              <button
                @click="fetchRunningModels"
                :disabled="isLoadingRunningModels"
                type="button"
                class="text-xs text-primary hover:text-primary/80 disabled:opacity-50 transition-colors"
              >
                {{ isLoadingRunningModels ? '...' : $t('settings.refreshModels') }}
              </button>
            </div>
            <div v-if="runningModels.length > 0" class="space-y-1">
              <div
                v-for="model in runningModels"
                :key="model.name"
                class="flex items-center justify-between px-2 py-1 text-xs bg-green-900/30 border border-green-800 rounded"
              >
                <span class="text-green-300">{{ model.name }}</span>
                <span v-if="model.size_vram" class="text-green-400 text-xs">
                  {{ $t('settings.vramUsage', { size: formatModelSize(model.size_vram) }) }}
                </span>
              </div>
            </div>
            <p v-else-if="!isLoadingRunningModels" class="text-xs text-gray-500">{{ $t('settings.noRunningModels') }}</p>
          </div>

          <!-- Create custom model button -->
          <div v-if="ollamaModels.length > 0" class="mt-3 pt-3 border-t border-gray-700">
            <button
              @click="showCreateModelModal = true"
              type="button"
              class="w-full px-3 py-2 text-xs bg-linear-to-r from-purple-700 to-indigo-700 hover:from-purple-600 hover:to-indigo-600 rounded-lg transition-colors"
            >
              {{ $t('settings.createCustomModel') }}
            </button>
          </div>



          <p v-if="ollamaModelError" class="text-xs text-red-400 mt-1">{{ ollamaModelError }}</p>
        </div>
      </div>
    </div>

    <!-- Custom API Headers (AI-006) -->
    <div>
      <h3 class="text-sm font-medium text-gray-300 mb-3">{{ $t('settings.customHeaders') }}</h3>
      <p class="text-xs text-gray-500 mb-3">{{ $t('settings.customHeadersHint') }}</p>

      <div class="space-y-3">
        <!-- Preset Templates -->
        <div class="flex gap-2">
          <select
            v-model="selectedPreset"
            @change="applyPreset"
            class="flex-1 bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 focus:border-primary focus:outline-none"
          >
            <option value="">{{ $t('settings.selectPreset') }}</option>
            <option value="openrouter">OpenRouter</option>
            <option value="azure">Azure OpenAI</option>
            <option value="claude">Claude API</option>
          </select>
          <button
            @click="clearAllHeaders"
            type="button"
            class="px-3 py-2 text-sm bg-red-700 hover:bg-red-600 rounded-lg transition-colors"
          >
            {{ $t('settings.clearAllHeaders') }}
          </button>
        </div>

        <!-- Headers List -->
        <div v-if="customHeaders.length > 0" class="space-y-2">
          <div
            v-for="(header, index) in customHeaders"
            :key="index"
            class="flex items-center gap-2 p-2 bg-darker border border-gray-700 rounded-lg"
          >
            <input
              v-model="header.key"
              type="text"
              :placeholder="$t('settings.headerKeyPlaceholder')"
              class="flex-1 bg-transparent border-none text-sm text-gray-100 placeholder:text-gray-500 focus:outline-none"
            />
            <input
              v-model="header.value"
              :type="header.sensitive ? 'password' : 'text'"
              :placeholder="$t('settings.headerValuePlaceholder')"
              class="flex-1 bg-transparent border-none text-sm text-gray-100 placeholder:text-gray-500 focus:outline-none"
            />
            <label class="flex items-center gap-1 text-xs text-gray-400 cursor-pointer">
              <input
                v-model="header.sensitive"
                type="checkbox"
                class="rounded border-gray-600 bg-darker text-primary focus:ring-primary"
              />
              <span>{{ $t('settings.sensitive') }}</span>
            </label>
            <button
              @click="removeHeader(index)"
              type="button"
              class="text-gray-400 hover:text-red-400 transition-colors px-2"
            >×</button>
          </div>
        </div>
        <p v-else class="text-xs text-gray-500">{{ $t('settings.noHeaders') }}</p>

        <!-- Add Header Button -->
        <button
          @click="addHeader"
          type="button"
          class="w-full px-3 py-2 text-sm bg-gray-700 hover:bg-gray-600 rounded-lg transition-colors"
        >
          + {{ $t('settings.addHeader') }}
        </button>

        <!-- Headers Preview -->
        <div v-if="customHeaders.length > 0" class="mt-3 p-3 bg-gray-800/50 rounded-lg">
          <h4 class="text-xs font-medium text-gray-400 mb-2">{{ $t('settings.headerPreview') }}</h4>
          <div class="text-xs text-gray-500 font-mono space-y-1">
            <div v-for="(header, index) in customHeaders" :key="index">
              {{ header.key }}: {{ header.sensitive ? '***' : header.value || '(empty)' }}
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- PERF-001: Proxy Configuration -->
    <div>
      <button
        @click="showProxyConfig = !showProxyConfig"
        type="button"
        class="w-full flex items-center justify-between text-sm font-medium text-gray-300 mb-3 hover:text-white transition-colors"
      >
        <span>{{ $t('settings.proxyConfig') }}</span>
        <span :class="showProxyConfig ? 'rotate-180' : ''" class="transition-transform">▼</span>
      </button>

      <div v-if="showProxyConfig" class="space-y-3 pl-2 border-l-2 border-gray-700">
        <!-- Enable Proxy Toggle -->
        <div class="flex items-center gap-2">
          <label class="flex items-center gap-2 cursor-pointer">
            <input
              v-model="localSettings.proxy_enabled"
              type="checkbox"
              class="rounded border-gray-600 bg-darker text-primary focus:ring-primary"
            />
            <span class="text-xs text-gray-300">{{ $t('settings.enableProxy') }}</span>
          </label>
        </div>

        <!-- Proxy Host and Port -->
        <div class="flex gap-2">
          <div class="flex-1">
            <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.proxyHost') }}</label>
            <input
              v-model="localSettings.proxy_host"
              type="text"
              :placeholder="$t('settings.proxyHostPlaceholder')"
              :disabled="!localSettings.proxy_enabled"
              class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none disabled:opacity-50 disabled:cursor-not-allowed"
            />
          </div>
          <div class="w-24">
            <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.proxyPort') }}</label>
            <input
              v-model.number="localSettings.proxy_port"
              type="number"
              min="1"
              max="65535"
              placeholder="8080"
              :disabled="!localSettings.proxy_enabled"
              class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none disabled:opacity-50 disabled:cursor-not-allowed"
            />
          </div>
        </div>

        <!-- Proxy Username (Optional) -->
        <div>
          <label class="text-xs text-gray-300 block mb-1">
            {{ $t('settings.proxyUsername') }}
            <span class="text-gray-500">({{ $t('settings.optional') }})</span>
          </label>
          <input
            v-model="localSettings.proxy_username"
            type="text"
            :placeholder="$t('settings.proxyUsernamePlaceholder')"
            :disabled="!localSettings.proxy_enabled"
            class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none disabled:opacity-50 disabled:cursor-not-allowed"
          />
        </div>

        <!-- Proxy Password (Optional) -->
        <div>
          <label class="text-xs text-gray-300 block mb-1">
            {{ $t('settings.proxyPassword') }}
            <span class="text-gray-500">({{ $t('settings.optional') }})</span>
          </label>
          <div class="relative">
            <input
              v-model="localSettings.proxy_password"
              :type="showProxyPassword ? 'text' : 'password'"
              :placeholder="$t('settings.proxyPasswordPlaceholder')"
              :disabled="!localSettings.proxy_enabled"
              class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 pr-16 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none disabled:opacity-50 disabled:cursor-not-allowed"
            />
            <button
              @click="showProxyPassword = !showProxyPassword"
              type="button"
              class="absolute right-2 top-1/2 -translate-y-1/2 text-gray-500 hover:text-gray-300 transition-colors text-xs px-2 py-1 rounded hover:bg-gray-700 disabled:opacity-50"
              :disabled="!localSettings.proxy_enabled"
              :title="showProxyPassword ? $t('common.hide') : $t('common.show')"
            >{{ showProxyPassword ? $t('common.hide') : $t('common.show') }}</button>
          </div>
        </div>
      </div>
    </div>

    <!-- Language Settings -->
    <div>
      <h3 class="text-sm font-medium text-gray-300 mb-3">{{ $t('settings.language') }}</h3>
      <div class="space-y-3">
        <div class="flex gap-2">
          <button
            @click="changeLanguage('en')"
            type="button"
            class="flex-1 px-3 py-2 text-sm rounded-lg border transition-colors"
            :class="locale === 'en' ? 'bg-primary border-primary text-white' : 'bg-darker border-gray-600 text-gray-300 hover:border-primary'"
          >
            {{ $t('settings.languageEn') }}
          </button>
          <button
            @click="changeLanguage('zh-CN')"
            type="button"
            class="flex-1 px-3 py-2 text-sm rounded-lg border transition-colors"
            :class="locale === 'zh-CN' ? 'bg-primary border-primary text-white' : 'bg-darker border-gray-600 text-gray-300 hover:border-primary'"
          >
            {{ $t('settings.languageZhCN') }}
          </button>
        </div>
        <p class="text-xs text-gray-500">{{ $t('settings.languageHint') }}</p>
      </div>
    </div>

    <!-- Shortcuts -->
    <div v-if="isDesktop">
      <h3 class="text-sm font-medium text-gray-300 mb-3">{{ $t('settings.shortcuts') }}</h3>
      <div class="bg-darker rounded-lg px-3 py-2 text-sm text-gray-400 border border-gray-700">
        {{ $t('settings.quickNoteShortcut') }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import { showError, showSuccess } from '../../stores/toast'
import { setLocale } from '../../i18n'
import type { Locale } from '@/i18n'
import { usePlatform } from '../../composables/usePlatform'
import {
  isOllamaEndpoint,
  formatModelSize,
  type ConnectionTestResult,
  type OllamaModel,
  type RunningModel
} from './shared/types'

// Props
interface Props {
  settings: {
    api_base_url: string
    api_key: string
    model_name: string
    custom_headers?: string
    // PERF-001: Proxy settings
    proxy_enabled?: boolean
    proxy_host?: string
    proxy_port?: number
    proxy_username?: string
    proxy_password?: string
    // PERF-001: Test model name
    test_model_name?: string
  }
}

const props = defineProps<Props>()

// Emits
const emit = defineEmits<{
  (e: 'update:settings', value: Props['settings']): void
  (e: 'show-create-model-modal'): void
  (e: 'show-copy-model-modal', source: string): void
}>()

// Composables
const { t, locale } = useI18n()
const { isDesktop } = usePlatform()

// Local state (synced with parent)
const localSettings = ref({ ...props.settings })

// AI-006: Custom Headers State
interface CustomHeader {
  key: string
  value: string
  sensitive: boolean
}

const customHeaders = ref<CustomHeader[]>([])
const selectedPreset = ref('')

// Preset templates
const headerPresets: Record<string, CustomHeader[]> = {
  openrouter: [
    { key: 'HTTP-Referer', value: 'https://dailylogger.app', sensitive: false },
    { key: 'X-Title', value: 'DailyLogger', sensitive: false }
  ],
  azure: [
    { key: 'api-key', value: '', sensitive: true }
  ],
  claude: [
    { key: 'anthropic-version', value: '2023-06-01', sensitive: false }
  ]
}

// Watch for external changes
watch(() => props.settings, (newVal) => {
  localSettings.value = { ...newVal }
  // AI-006: Parse custom headers from JSON
  if (newVal.custom_headers) {
    try {
      customHeaders.value = JSON.parse(newVal.custom_headers)
    } catch {
      customHeaders.value = []
    }
  } else {
    customHeaders.value = []
  }
}, { deep: true, immediate: true })

// Watch for local changes and emit
watch(localSettings, (newVal) => {
  emit('update:settings', newVal)
}, { deep: true })

// AI-006: Watch custom headers and sync to localSettings
watch(customHeaders, (newVal) => {
  localSettings.value.custom_headers = JSON.stringify(newVal)
}, { deep: true })

// UI State
const showApiKey = ref(false)
const isTestingConnection = ref(false)
const connectionTestResult = ref<ConnectionTestResult | null>(null)
// PERF-001: Proxy config UI state
const showProxyConfig = ref(false)
const showProxyPassword = ref(false)

// Ollama State
const isLoadingOllamaModels = ref(false)
const ollamaModels = ref<OllamaModel[]>([])
const ollamaModelError = ref('')
const pullModelName = ref('')
const pullModelQuantization = ref('')
const isPullingModel = ref(false)
const isDeletingModel = ref('')
const runningModels = ref<RunningModel[]>([])
const isLoadingRunningModels = ref(false)
const showCreateModelModal = ref(false)

// Computed
const isOllama = computed(() => isOllamaEndpoint(localSettings.value.api_base_url))

// Methods
async function testConnection() {
  if (!localSettings.value.api_base_url || !localSettings.value.model_name) {
    showError(t('settings.apiBaseUrlRequired'))
    return
  }

  if (!isOllama.value && !localSettings.value.api_key) {
    showError(t('settings.apiKeyRequired'))
    return
  }

  // PERF-001: Use test_model_name if configured, otherwise use model_name
  const testModel = localSettings.value.test_model_name || localSettings.value.model_name

  isTestingConnection.value = true
  connectionTestResult.value = null

  try {
    const result = await invoke<ConnectionTestResult>('test_api_connection_with_ollama', {
      apiBaseUrl: localSettings.value.api_base_url,
      apiKey: localSettings.value.api_key || null,
      modelName: localSettings.value.test_model_name || localSettings.value.model_name,
      // PERF-001: Proxy configuration
      proxyEnabled: localSettings.value.proxy_enabled || false,
      proxyHost: localSettings.value.proxy_host || null,
      proxyPort: localSettings.value.proxy_port || null,
      proxyUsername: localSettings.value.proxy_username || null,
      proxyPassword: localSettings.value.proxy_password || null
    })
    connectionTestResult.value = result
    if (result.success) {
      showSuccess(t('settings.connectionSuccess', { latency: result.latency_ms }))
    } else {
      showError(result.message)
    }
  } catch (err) {
    console.error('Failed to test connection:', err)
    connectionTestResult.value = { success: false, message: String(err) }
    showError(err)
  } finally {
    isTestingConnection.value = false
  }
}

async function fetchOllamaModels() {
  if (!localSettings.value.api_base_url) {
    showError(t('settings.apiBaseUrlRequired'))
    return
  }

  isLoadingOllamaModels.value = true
  ollamaModelError.value = ''

  try {
    const result = await invoke<{ success: boolean; models: OllamaModel[]; message?: string }>('get_ollama_models', {
      baseUrl: localSettings.value.api_base_url
    })

    if (result.success) {
      ollamaModels.value = result.models
      if (result.models.length === 0) {
        ollamaModelError.value = t('settings.ollamaModelsNotFound')
      } else {
        showSuccess(t('settings.ollamaModelsFound', { count: result.models.length }))
      }
    } else {
      ollamaModelError.value = result.message || ''
      showError(result.message || '')
    }
  } catch (err) {
    console.error('Failed to fetch Ollama models:', err)
    ollamaModelError.value = String(err)
    showError(err)
  } finally {
    isLoadingOllamaModels.value = false
  }
}

function selectOllamaModel(modelName: string) {
  localSettings.value.model_name = modelName
}

async function pullModel() {
  if (!pullModelName.value.trim()) {
    showError(t('settings.modelNameRequired'))
    return
  }

  if (!localSettings.value.api_base_url) {
    showError(t('settings.apiBaseUrlRequired'))
    return
  }

  isPullingModel.value = true
  ollamaModelError.value = ''

  try {
    const result = await invoke<{ success: boolean; message: string }>('pull_ollama_model', {
      baseUrl: localSettings.value.api_base_url,
      modelName: pullModelName.value,
      quantization: pullModelQuantization.value || null
    })

    if (result.success) {
      showSuccess(result.message)
      pullModelName.value = ''
      pullModelQuantization.value = ''
      await fetchOllamaModels()
    } else {
      showError(result.message)
    }
  } catch (err) {
    console.error('Failed to pull model:', err)
    showError(err)
  } finally {
    isPullingModel.value = false
  }
}

async function deleteModel(modelName: string) {
  if (!confirm(t('settings.confirmDeleteModel', { model: modelName }))) {
    return
  }

  isDeletingModel.value = modelName

  try {
    const result = await invoke<{ success: boolean; message: string }>('delete_ollama_model', {
      baseUrl: localSettings.value.api_base_url,
      modelName: modelName
    })

    if (result.success) {
      showSuccess(result.message)
      await fetchOllamaModels()
    } else {
      showError(result.message)
    }
  } catch (err) {
    console.error('Failed to delete model:', err)
    showError(err)
  } finally {
    isDeletingModel.value = ''
  }
}

async function fetchRunningModels() {
  if (!localSettings.value.api_base_url) return

  isLoadingRunningModels.value = true

  try {
    const result = await invoke<{ success: boolean; running_models?: RunningModel[] }>('get_running_ollama_models', {
      baseUrl: localSettings.value.api_base_url
    })

    if (result.success && result.running_models) {
      runningModels.value = result.running_models
    }
  } catch (err) {
    console.error('Failed to fetch running models:', err)
  } finally {
    isLoadingRunningModels.value = false
  }
}

function openCopyModelModal(source: string) {
  emit('show-copy-model-modal', source)
}

function changeLanguage(lang: Locale) {
  setLocale(lang)
  locale.value = lang
}

// AI-006: Custom Headers Methods
function addHeader() {
  customHeaders.value.push({ key: '', value: '', sensitive: false })
}

function removeHeader(index: number) {
  customHeaders.value.splice(index, 1)
}

function applyPreset() {
  if (!selectedPreset.value) return

  const preset = headerPresets[selectedPreset.value]
  if (preset) {
    // Add preset headers (don't overwrite existing ones with same key)
    for (const header of preset) {
      if (!customHeaders.value.some(h => h.key.toLowerCase() === header.key.toLowerCase())) {
        customHeaders.value.push({ ...header })
      }
    }
    showSuccess(t('settings.headersCount', { count: customHeaders.value.length }))
  }
  selectedPreset.value = ''
}

function clearAllHeaders() {
  if (!confirm(t('settings.confirmClearHeaders'))) return
  customHeaders.value = []
}
</script>