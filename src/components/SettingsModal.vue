<template>
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="handleClose">
    <div class="bg-dark rounded-2xl w-[700px] max-h-[85vh] overflow-hidden border border-gray-700 flex flex-col">
      <div class="px-6 py-4 border-b border-gray-700 flex items-center justify-between">
        <h2 class="text-lg font-semibold">{{ $t('settings.title') }}</h2>
        <button @click="handleClose" class="text-gray-400 hover:text-white">✕</button>
      </div>

      <!-- Tab Navigation -->
      <div class="px-6 pt-4 border-b border-gray-700 flex gap-1">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          @click="activeTab = tab.id"
          type="button"
          class="px-4 py-2 text-sm rounded-t-lg transition-colors -mb-px border-b-2"
          :class="activeTab === tab.id ? 'text-primary border-primary bg-darker' : 'text-gray-400 border-transparent hover:text-gray-200'"
        >
          {{ tab.label }}
        </button>
      </div>

      <div class="flex-1 overflow-y-auto p-6">
        <!-- Tab 1: Basic Settings -->
        <BasicSettings
          v-if="activeTab === 'basic'"
          :settings="basicSettings"
          @update:settings="updateBasicSettings"
          @show-create-model-modal="showCreateModelModal = true"
          @show-fine-tuning-modal="showFineTuningModal = true"
          @show-copy-model-modal="openCopyModelModal"
        />

        <!-- Tab 2: AI Settings -->
        <AISettings
          v-if="activeTab === 'ai'"
          :settings="aiSettings"
          :tag-categories-text="tagCategoriesText"
          @update:settings="updateAISettings"
          @update:tag-categories-text="tagCategoriesText = $event"
          @show-default-prompt-modal="openDefaultPromptModal"
          @show-default-summary-prompt-modal="openDefaultSummaryPromptModal"
          @show-template-library-modal="showTemplateLibraryModal = true"
          @show-default-tag-categories-modal="openDefaultTagCategoriesModal"
        />

        <!-- Tab 3: Capture Settings -->
        <CaptureSettings
          v-if="activeTab === 'capture'"
          :settings="captureSettings"
          :whitelist-tags="whitelistTags"
          :blacklist-tags="blacklistTags"
          :monitors="monitors"
          @update:settings="updateCaptureSettings"
          @update:whitelist-tags="whitelistTags = $event"
          @update:blacklist-tags="blacklistTags = $event"
        />

        <!-- Tab 4: Output Settings -->
        <OutputSettings
          v-if="activeTab === 'output'"
          :settings="outputSettings"
          :vaults="vaults"
          :graphs="graphs"
          @update:settings="updateOutputSettings"
          @update:vaults="vaults = $event"
          @update:graphs="graphs = $event"
        />
      </div>

      <!-- Footer -->
      <div class="px-6 py-4 border-t border-gray-700 flex justify-between items-center">
        <div class="text-sm">
          <span v-if="saveStatus === 'ok'" class="text-green-400">{{ $t('settings.saved') }}</span>
          <span v-else-if="saveStatus === 'err'" class="text-red-400">{{ saveError }}</span>
        </div>
        <div class="flex gap-3">
          <button
            @click="handleClose"
            class="px-4 py-2 rounded-lg text-sm text-gray-300 hover:bg-gray-700 hover:text-white transition-colors"
          >
            {{ $t('settings.cancel') }}
          </button>
          <button
            @click="saveSettings"
            :disabled="isSaving"
            class="px-4 py-2 bg-primary rounded-lg text-sm font-medium text-white hover:bg-blue-600 disabled:opacity-50 transition-colors"
          >
            {{ isSaving ? $t('settings.saving') : $t('settings.save') }}
          </button>
        </div>
      </div>
    </div>

    <!-- Close confirmation dialog -->
    <div
      v-if="showCloseConfirm"
      class="fixed inset-0 bg-black/50 flex items-center justify-center z-60"
    >
      <div class="bg-dark rounded-xl p-6 max-w-sm border border-gray-700">
        <h3 class="text-lg font-semibold mb-4">{{ $t('settings.unsavedChanges') }}</h3>
        <p class="text-gray-400 mb-6">{{ $t('settings.unsavedChangesMessage') }}</p>
        <div class="flex justify-end gap-3">
          <button
            @click="showCloseConfirm = false"
            class="px-4 py-2 bg-gray-600 text-white rounded hover:bg-gray-500 transition-colors"
          >
            {{ $t('common.cancel') }}
          </button>
          <button
            @click="confirmClose"
            class="px-4 py-2 bg-red-500 text-white rounded hover:bg-red-400 transition-colors"
          >
            {{ $t('settings.discardAndClose') }}
          </button>
        </div>
      </div>
    </div>

    <!-- Default Prompt Modal -->
    <div v-if="showDefaultPromptModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-60">
      <div class="bg-dark rounded-xl p-6 max-w-2xl max-h-[80vh] overflow-auto border border-gray-700">
        <h3 class="text-lg font-semibold mb-4">{{ $t('settings.defaultPrompt') }}</h3>
        <pre class="text-sm text-gray-300 whitespace-pre-wrap bg-darker p-4 rounded-lg">{{ defaultPromptContent }}</pre>
        <div class="mt-4 flex justify-end">
          <button @click="showDefaultPromptModal = false" class="px-4 py-2 bg-gray-600 text-white rounded hover:bg-gray-500 transition-colors">
            {{ $t('common.close') }}
          </button>
        </div>
      </div>
    </div>

    <!-- Default Summary Prompt Modal -->
    <div v-if="showDefaultSummaryPromptModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-60">
      <div class="bg-dark rounded-xl p-6 max-w-2xl max-h-[80vh] overflow-auto border border-gray-700">
        <h3 class="text-lg font-semibold mb-4">{{ $t('settings.defaultReportPrompt') }}</h3>
        <pre class="text-sm text-gray-300 whitespace-pre-wrap bg-darker p-4 rounded-lg">{{ defaultSummaryPromptContent }}</pre>
        <div class="mt-4 flex justify-end">
          <button @click="showDefaultSummaryPromptModal = false" class="px-4 py-2 bg-gray-600 text-white rounded hover:bg-gray-500 transition-colors">
            {{ $t('common.close') }}
          </button>
        </div>
      </div>
    </div>

    <!-- Template Library Modal -->
    <div v-if="showTemplateLibraryModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-60">
      <div class="bg-dark rounded-xl p-6 max-w-2xl max-h-[80vh] overflow-auto border border-gray-700">
        <h3 class="text-lg font-semibold mb-4">{{ $t('common.templateLibrary') }}</h3>
        <div class="space-y-3">
          <div
            v-for="template in presetTemplates"
            :key="template.id"
            class="bg-darker p-4 rounded-lg border border-gray-700 hover:border-primary cursor-pointer transition-colors"
            @click="applyTemplate(template)"
          >
            <h4 class="text-sm font-medium text-gray-200">{{ template.name }}</h4>
            <p class="text-xs text-gray-400 mt-1">{{ template.description }}</p>
          </div>
        </div>
        <div class="mt-4 flex justify-end">
          <button @click="showTemplateLibraryModal = false" class="px-4 py-2 bg-gray-600 text-white rounded hover:bg-gray-500 transition-colors">
            {{ $t('common.close') }}
          </button>
        </div>
      </div>
    </div>

    <!-- Default Tag Categories Modal -->
    <div v-if="showDefaultTagCategoriesModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-60">
      <div class="bg-dark rounded-xl p-6 max-w-lg max-h-[80vh] overflow-auto border border-gray-700">
        <h3 class="text-lg font-semibold mb-4">{{ $t('settings.defaultTagCategories') }}</h3>
        <div class="space-y-2">
          <div v-for="category in defaultTagCategoriesContent" :key="category" class="text-sm text-gray-300 bg-darker px-3 py-2 rounded">
            {{ category }}
          </div>
        </div>
        <div class="mt-4 flex justify-end">
          <button @click="showDefaultTagCategoriesModal = false" class="px-4 py-2 bg-gray-600 text-white rounded hover:bg-gray-500 transition-colors">
            {{ $t('common.close') }}
          </button>
        </div>
      </div>
    </div>

    <!-- Create Model Modal (Ollama) -->
    <div v-if="showCreateModelModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-60">
      <div class="bg-dark rounded-xl p-6 max-w-md border border-gray-700">
        <h3 class="text-lg font-semibold mb-4">{{ $t('settings.createCustomModel') }}</h3>
        <div class="space-y-3">
          <div>
            <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.modelName') }}</label>
            <input v-model="createModelParams.name" type="text" class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 focus:border-primary focus:outline-none" />
          </div>
          <div>
            <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.fromModel') }}</label>
            <input v-model="createModelParams.from" type="text" class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 focus:border-primary focus:outline-none" />
          </div>
          <div>
            <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.systemPrompt') }}</label>
            <textarea v-model="createModelParams.system" rows="3" class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 focus:border-primary focus:outline-none resize-y"></textarea>
          </div>
        </div>
        <div class="mt-4 flex justify-end gap-3">
          <button @click="showCreateModelModal = false" class="px-4 py-2 bg-gray-600 text-white rounded hover:bg-gray-500 transition-colors">
            {{ $t('common.cancel') }}
          </button>
          <button @click="createCustomModel" :disabled="isCreatingModel" class="px-4 py-2 bg-primary text-white rounded hover:bg-primary/80 disabled:opacity-50 transition-colors">
            {{ isCreatingModel ? $t('settings.creating') : $t('settings.create') }}
          </button>
        </div>
      </div>
    </div>

    <!-- Copy Model Modal -->
    <div v-if="showCopyModelModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-60">
      <div class="bg-dark rounded-xl p-6 max-w-md border border-gray-700">
        <h3 class="text-lg font-semibold mb-4">{{ $t('settings.copyModel') }}</h3>
        <div class="space-y-3">
          <div>
            <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.sourceModel') }}</label>
            <div class="bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-400">{{ copyModelSource }}</div>
          </div>
          <div>
            <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.newModelName') }}</label>
            <input v-model="copyModelDestination" type="text" class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 focus:border-primary focus:outline-none" />
          </div>
        </div>
        <div class="mt-4 flex justify-end gap-3">
          <button @click="showCopyModelModal = false" class="px-4 py-2 bg-gray-600 text-white rounded hover:bg-gray-500 transition-colors">
            {{ $t('common.cancel') }}
          </button>
          <button @click="copyModel" :disabled="isCopyingModel" class="px-4 py-2 bg-primary text-white rounded hover:bg-primary/80 disabled:opacity-50 transition-colors">
            {{ isCopyingModel ? $t('settings.copying') : $t('settings.copy') }}
          </button>
        </div>
      </div>
    </div>

    <!-- Fine-tuning Modal -->
    <div v-if="showFineTuningModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-60">
      <div class="bg-dark rounded-xl p-6 max-w-md border border-gray-700">
        <h3 class="text-lg font-semibold mb-4">{{ $t('settings.fineTuning') }}</h3>
        <div class="space-y-3">
          <div>
            <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.baseModel') }}</label>
            <input v-model="fineTuningParams.baseModel" type="text" class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 focus:border-primary focus:outline-none" />
          </div>
          <div>
            <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.outputModelName') }}</label>
            <input v-model="fineTuningParams.outputModelName" type="text" class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 focus:border-primary focus:outline-none" />
          </div>
        </div>
        <div class="mt-4 flex justify-end gap-3">
          <button @click="showFineTuningModal = false" class="px-4 py-2 bg-gray-600 text-white rounded hover:bg-gray-500 transition-colors">
            {{ $t('common.cancel') }}
          </button>
          <button @click="startFineTuning" :disabled="isFineTuning" class="px-4 py-2 bg-primary text-white rounded hover:bg-primary/80 disabled:opacity-50 transition-colors">
            {{ isFineTuning ? $t('settings.fineTuning') : $t('settings.start') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { showError, showSuccess } from '../stores/toast'
import { useI18n } from 'vue-i18n'
import { BasicSettings, AISettings, CaptureSettings, OutputSettings } from './settings'
import type { Settings } from '../types/tauri'

// Types
interface Monitor {
  id: number
  name: string
  width: number
  height: number
  is_primary: boolean
  index?: number
  resolution?: string
}

interface Vault {
  name: string
  path: string
  is_default?: boolean
}

interface Graph {
  name: string
  path: string
  is_default?: boolean
}

interface Template {
  id: string
  name: string
  description: string
  content: string | null
}

// Composables
const { t } = useI18n()

// Emits
const emit = defineEmits<{(e: 'close'): void}>()

// Tab navigation
type SettingsTab = 'basic' | 'ai' | 'capture' | 'output'
const activeTab = ref<SettingsTab>('basic')
const tabs: { id: SettingsTab; label: string }[] = [
  { id: 'basic', label: t('settings.tabBasic') },
  { id: 'ai', label: t('settings.tabAI') },
  { id: 'capture', label: t('settings.tabCapture') },
  { id: 'output', label: t('settings.tabOutput') }
]

// Core settings state
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
  include_manual_records: true,
  window_whitelist: '[]',
  window_blacklist: '[]',
  use_whitelist_only: false,
  auto_adjust_silent: true,
  silent_adjustment_paused_until: null,
  auto_detect_work_time: true,
  use_custom_work_time: false,
  custom_work_time_start: '09:00',
  custom_work_time_end: '18:00',
  learned_work_time: null,
  capture_mode: 'primary',
  selected_monitor_index: 0,
  capture_only_mode: false,
  tag_categories: '',
  is_ollama: false,
  obsidian_vaults: '[]',
  logseq_graphs: '[]',
  notion_api_key: null as string | null,
  notion_database_id: null as string | null,
  github_token: null as string | null,
  github_repositories: '[]',
  slack_webhook_url: null as string | null
})

// Derived state for sub-components
const basicSettings = computed(() => ({
  api_base_url: settings.value.api_base_url,
  api_key: settings.value.api_key,
  model_name: settings.value.model_name
}))

const aiSettings = computed(() => ({
  model_name: settings.value.model_name,
  analysis_prompt: settings.value.analysis_prompt,
  summary_model_name: settings.value.summary_model_name,
  summary_prompt: settings.value.summary_prompt,
  summary_title_format: settings.value.summary_title_format,
  include_manual_records: settings.value.include_manual_records,
  api_base_url: settings.value.api_base_url,
  api_key: settings.value.api_key
}))

const captureSettings = computed(() => ({
  screenshot_interval: settings.value.screenshot_interval,
  summary_time: settings.value.summary_time,
  change_threshold: settings.value.change_threshold,
  max_silent_minutes: settings.value.max_silent_minutes,
  auto_adjust_silent: settings.value.auto_adjust_silent,
  auto_detect_work_time: settings.value.auto_detect_work_time,
  use_custom_work_time: settings.value.use_custom_work_time,
  custom_work_time_start: settings.value.custom_work_time_start,
  custom_work_time_end: settings.value.custom_work_time_end,
  use_whitelist_only: settings.value.use_whitelist_only,
  capture_mode: settings.value.capture_mode,
  selected_monitor_index: settings.value.selected_monitor_index,
  capture_only_mode: settings.value.capture_only_mode
}))

const outputSettings = computed(() => ({
  notion_api_key: settings.value.notion_api_key as string | null,
  notion_database_id: settings.value.notion_database_id as string | null,
  github_token: settings.value.github_token as string | null,
  github_repositories: settings.value.github_repositories,
  slack_webhook_url: settings.value.slack_webhook_url as string | null
}))

// Auxiliary state
const whitelistTags = ref<string[]>([])
const blacklistTags = ref<string[]>([])
const tagCategoriesText = ref('')
const vaults = ref<Vault[]>([])
const graphs = ref<Graph[]>([])
const monitors = ref<Monitor[]>([])

// Update handlers for sub-components
function updateBasicSettings(newSettings: typeof basicSettings.value) {
  settings.value.api_base_url = newSettings.api_base_url
  settings.value.api_key = newSettings.api_key
  settings.value.model_name = newSettings.model_name
}

function updateAISettings(newSettings: typeof aiSettings.value) {
  settings.value.model_name = newSettings.model_name
  settings.value.analysis_prompt = newSettings.analysis_prompt
  settings.value.summary_model_name = newSettings.summary_model_name
  settings.value.summary_prompt = newSettings.summary_prompt
  settings.value.summary_title_format = newSettings.summary_title_format
  settings.value.include_manual_records = newSettings.include_manual_records
}

function updateCaptureSettings(newSettings: typeof captureSettings.value) {
  Object.assign(settings.value, newSettings)
}

function updateOutputSettings(newSettings: typeof outputSettings.value) {
  settings.value.notion_api_key = newSettings.notion_api_key
  settings.value.notion_database_id = newSettings.notion_database_id
  settings.value.github_token = newSettings.github_token
  settings.value.github_repositories = newSettings.github_repositories
  settings.value.slack_webhook_url = newSettings.slack_webhook_url
}

// Save state
const isSaving = ref(false)
const saveStatus = ref('')
const saveError = ref('')
const initialSettings = ref('')
const showCloseConfirm = ref(false)

// Modal states
const showDefaultPromptModal = ref(false)
const defaultPromptContent = ref('')
const showDefaultSummaryPromptModal = ref(false)
const defaultSummaryPromptContent = ref('')
const showTemplateLibraryModal = ref(false)
const showDefaultTagCategoriesModal = ref(false)
const defaultTagCategoriesContent = ref<string[]>([])
const showCreateModelModal = ref(false)
const isCreatingModel = ref(false)
const createModelParams = ref({ name: '', from: '', system: '', temperature: null, num_ctx: null, quantize: '' })
const showCopyModelModal = ref(false)
const isCopyingModel = ref(false)
const copyModelSource = ref('')
const copyModelDestination = ref('')
const showFineTuningModal = ref(false)
const isFineTuning = ref(false)
const fineTuningParams = ref({ baseModel: '', outputModelName: '', epochs: 3, systemPrompt: '', temperature: 0.7, numCtx: 4096 })

// Preset templates
const presetTemplates: Template[] = [
  { id: 'default', name: t('settings.templateDefaultName'), description: t('settings.templateDefaultDesc'), content: null },
  { id: 'concise', name: t('settings.templateSimpleName'), description: t('settings.templateSimpleDesc'), content: `请根据以下今日工作记录，生成简洁的工作摘要。\n\n今日记录：\n{records}\n\n要求：\n1. 仅列出 3-5 条主要工作项\n2. 每项不超过 20 字\n3. 格式：• 工作项\n\n请生成摘要：` },
  { id: 'detailed', name: t('settings.templateDetailedName'), description: t('settings.templateDetailedDesc'), content: `请根据以下今日工作记录，生成详细的工作日报。\n\n今日记录：\n{records}\n\n请按以下格式生成日报：\n\n## 📋 今日概览\n- 工作时长估算\n- 主要工作领域\n\n## ✅ 完成事项\n按优先级列出已完成的工作\n\n## 🔄 进行中\n正在处理的事项\n\n## 💡 改进建议\n基于今日工作的改进建议\n\n## 📌 明日计划\n建议的后续事项\n\n请生成日报：` }
]

// Computed for unsaved changes
const hasUnsavedChanges = computed(() => {
  if (!initialSettings.value) return false
  const currentSnapshot = JSON.stringify({
    settings: settings.value,
    vaults: vaults.value,
    graphs: graphs.value,
    whitelistTags: whitelistTags.value,
    blacklistTags: blacklistTags.value,
    tagCategoriesText: tagCategoriesText.value
  })
  return currentSnapshot !== initialSettings.value
})

// Handlers
function handleClose() {
  if (hasUnsavedChanges.value) {
    showCloseConfirm.value = true
  } else {
    emit('close')
  }
}

function confirmClose() {
  showCloseConfirm.value = false
  emit('close')
}

async function loadSettings() {
  try {
    const loaded = await invoke<Partial<Settings>>('get_settings')
    settings.value = { ...settings.value, ...loaded } as typeof settings.value

    // Parse auxiliary data
    try { vaults.value = JSON.parse(settings.value.obsidian_vaults || '[]') } catch { vaults.value = [] }
    if (vaults.value.length === 0 && settings.value.obsidian_path) {
      vaults.value = [{ name: 'Default', path: settings.value.obsidian_path, is_default: true }]
    }
    try { graphs.value = JSON.parse(settings.value.logseq_graphs || '[]') } catch { graphs.value = [] }
    try { whitelistTags.value = JSON.parse(settings.value.window_whitelist || '[]') } catch { whitelistTags.value = [] }
    try { blacklistTags.value = JSON.parse(settings.value.window_blacklist || '[]') } catch { blacklistTags.value = [] }
    try { tagCategoriesText.value = JSON.parse(settings.value.tag_categories || '[]').join('\n') } catch { tagCategoriesText.value = '' }

    // Load monitors
    try { monitors.value = await invoke<Monitor[]>('get_monitors') } catch { /* ignore */ }

    // Load default prompts if custom prompts are empty (Issue #56)
    if (!settings.value.analysis_prompt || settings.value.analysis_prompt.trim() === '') {
      try {
        settings.value.analysis_prompt = await invoke('get_default_analysis_prompt')
      } catch { /* ignore */ }
    }
    if (!settings.value.summary_prompt || settings.value.summary_prompt.trim() === '') {
      try {
        settings.value.summary_prompt = await invoke('get_default_summary_prompt')
      } catch { /* ignore */ }
    }

    // Save initial snapshot
    initialSettings.value = JSON.stringify({
      settings: settings.value,
      vaults: vaults.value,
      graphs: graphs.value,
      whitelistTags: whitelistTags.value,
      blacklistTags: blacklistTags.value,
      tagCategoriesText: tagCategoriesText.value
    })
  } catch (err) {
    console.error('Failed to load settings:', err)
  }
}

async function saveSettings() {
  isSaving.value = true
  saveStatus.value = ''
  saveError.value = ''

  try {
    const settingsToSave = {
      ...settings.value,
      window_whitelist: JSON.stringify(whitelistTags.value),
      window_blacklist: JSON.stringify(blacklistTags.value),
      tag_categories: JSON.stringify(tagCategoriesText.value.split('\n').map(t => t.trim()).filter(t => t.length > 0)),
      obsidian_vaults: JSON.stringify(vaults.value),
      obsidian_path: vaults.value.find(v => v.is_default)?.path || vaults.value[0]?.path || settings.value.obsidian_path || '',
      logseq_graphs: JSON.stringify(graphs.value)
    }

    await invoke('save_settings', { settings: settingsToSave })
    saveStatus.value = 'ok'
    showSuccess(t('settings.settingsSaved'))
    setTimeout(() => emit('close'), 800)
  } catch (err) {
    console.error('Failed to save settings:', err)
    saveStatus.value = 'err'
    saveError.value = t('settings.saveFailed')
    showError(err)
  } finally {
    isSaving.value = false
  }
}

// Modal handlers
async function openDefaultPromptModal() {
  try {
    defaultPromptContent.value = await invoke('get_default_analysis_prompt')
    showDefaultPromptModal.value = true
  } catch (err) {
    showError(err)
  }
}

async function openDefaultSummaryPromptModal() {
  try {
    defaultSummaryPromptContent.value = await invoke('get_default_summary_prompt')
    showDefaultSummaryPromptModal.value = true
  } catch (err) {
    showError(err)
  }
}

async function openDefaultTagCategoriesModal() {
  try {
    defaultTagCategoriesContent.value = await invoke('get_default_tag_categories')
    showDefaultTagCategoriesModal.value = true
  } catch (err) {
    showError(err)
  }
}

function applyTemplate(template: Template) {
  if (template.content) {
    settings.value.summary_prompt = template.content
    showTemplateLibraryModal.value = false
    showSuccess(t('settings.templateApplied', { name: template.name }))
  }
}

function openCopyModelModal(source: string) {
  copyModelSource.value = source
  copyModelDestination.value = ''
  showCopyModelModal.value = true
}

async function copyModel() {
  if (!copyModelDestination.value.trim()) {
    showError(t('settings.modelNameRequired'))
    return
  }

  isCopyingModel.value = true
  try {
    const result = await invoke<{ success: boolean; message: string }>('copy_ollama_model', {
      baseUrl: settings.value.api_base_url,
      source: copyModelSource.value,
      destination: copyModelDestination.value.trim()
    })
    if (result.success) {
      showSuccess(result.message)
      showCopyModelModal.value = false
    } else {
      showError(result.message)
    }
  } catch (err) {
    showError(err)
  } finally {
    isCopyingModel.value = false
  }
}

async function createCustomModel() {
  if (!createModelParams.value.name.trim() || !createModelParams.value.from.trim()) {
    showError(t('settings.modelNameRequired'))
    return
  }

  isCreatingModel.value = true
  try {
    const result = await invoke<{ success: boolean; message: string }>('create_ollama_model', {
      baseUrl: settings.value.api_base_url,
      name: createModelParams.value.name.trim(),
      from: createModelParams.value.from.trim(),
      system: createModelParams.value.system || null,
      temperature: createModelParams.value.temperature,
      num_ctx: createModelParams.value.num_ctx,
      quantize: createModelParams.value.quantize || null
    })
    if (result.success) {
      showSuccess(result.message)
      showCreateModelModal.value = false
    } else {
      showError(result.message)
    }
  } catch (err) {
    showError(err)
  } finally {
    isCreatingModel.value = false
  }
}

async function startFineTuning() {
  if (!fineTuningParams.value.baseModel.trim() || !fineTuningParams.value.outputModelName.trim()) {
    showError(t('settings.modelNameRequired'))
    return
  }

  isFineTuning.value = true
  try {
    const result = await invoke<{ success: boolean; message: string; model_name?: string }>('start_fine_tuning', {
      baseUrl: settings.value.api_base_url,
      config: {
        base_model: fineTuningParams.value.baseModel,
        output_model_name: fineTuningParams.value.outputModelName.trim(),
        epochs: fineTuningParams.value.epochs,
        system_prompt: fineTuningParams.value.systemPrompt || null,
        temperature: fineTuningParams.value.temperature,
        num_ctx: fineTuningParams.value.numCtx
      }
    })
    if (result.success) {
      showSuccess(t('settings.fineTuningSuccess', { model: result.model_name }))
      showFineTuningModal.value = false
    } else {
      showError(t('settings.fineTuningFailed', { error: result.message }))
    }
  } catch (err) {
    showError(t('settings.fineTuningFailed', { error: String(err) }))
  } finally {
    isFineTuning.value = false
  }
}

onMounted(() => {
  loadSettings()
})
</script>