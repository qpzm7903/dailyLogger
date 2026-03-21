<template>
  <div class="space-y-6">
    <!-- Screenshot Analysis Settings -->
    <div v-if="isDesktop">
      <h3 class="text-sm font-medium text-gray-300 mb-3">{{ $t('settings.screenshotAnalysis') }}</h3>
      <div class="space-y-3">
        <div>
          <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.analysisModel') }}</label>
          <div class="flex items-center gap-2">
            <input
              v-model="localSettings.model_name"
              type="text"
              placeholder="gpt-4o"
              class="flex-1 bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
            />
            <button
              @click="getModelInfo('analysis')"
              :disabled="isLoadingModelInfo || !localSettings.model_name"
              type="button"
              class="text-gray-400 hover:text-primary disabled:opacity-50 disabled:cursor-not-allowed transition-colors px-2"
              :title="$t('settings.contextWindow', { size: '' })"
            >ℹ️</button>
          </div>
          <span v-if="analysisModelInfo?.context_window" class="text-xs text-gray-500 mt-1 block">
            {{ $t('settings.contextWindow', { size: analysisModelInfo.context_window / 1000 }) }}
          </span>
          <span v-else class="text-xs text-gray-500 mt-1 block">{{ $t('settings.visionRequired') }}</span>
        </div>
        <div>
          <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.analysisPrompt') }}</label>
          <textarea
            v-model="localSettings.analysis_prompt"
            rows="4"
            :placeholder="$t('settings.analysisPromptPlaceholder')"
            class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none resize-y"
          />
          <div class="flex gap-3 mt-2">
            <button
              type="button"
              @click="showDefaultPrompt"
              class="text-xs text-gray-400 hover:text-primary transition-colors"
            >
              {{ $t('common.viewDefault') }}
            </button>
            <button
              type="button"
              @click="resetPrompt"
              class="text-xs text-gray-400 hover:text-primary transition-colors"
            >
              {{ $t('common.resetDefault') }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Daily Report Settings -->
    <div>
      <h3 class="text-sm font-medium text-gray-300 mb-3">{{ $t('settings.dailyReport') }}</h3>
      <div class="space-y-3">
        <div>
          <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.reportTitleFormat') }}</label>
          <input
            v-model="localSettings.summary_title_format"
            type="text"
            :placeholder="$t('settings.reportTitlePlaceholder')"
            class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
          />
          <span class="text-xs text-gray-500 mt-1 block">{{ $t('settings.reportTitleHint') }}</span>
        </div>
        <div>
          <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.reportModel') }}</label>
          <div class="flex items-center gap-2">
            <input
              v-model="localSettings.summary_model_name"
              type="text"
              :placeholder="$t('settings.reportModelPlaceholder')"
              class="flex-1 bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
            />
            <button
              @click="getModelInfo('summary')"
              :disabled="isLoadingModelInfo || !localSettings.summary_model_name"
              type="button"
              class="text-gray-400 hover:text-primary disabled:opacity-50 disabled:cursor-not-allowed transition-colors px-2"
              :title="$t('settings.contextWindow', { size: '' })"
            >ℹ️</button>
          </div>
          <span v-if="summaryModelInfo?.context_window" class="text-xs text-gray-500 mt-1 block">
            {{ $t('settings.contextWindow', { size: summaryModelInfo.context_window / 1000 }) }}
          </span>
          <span v-else class="text-xs text-gray-500 mt-1 block">{{ $t('settings.textModelHint') }}</span>
        </div>
        <div>
          <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.reportPrompt') }}</label>
          <textarea
            v-model="localSettings.summary_prompt"
            rows="4"
            :placeholder="$t('settings.reportPromptPlaceholder')"
            class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none resize-y"
          />
          <div class="flex gap-3 mt-2">
            <button
              type="button"
              @click="showDefaultSummaryPrompt"
              class="text-xs text-gray-400 hover:text-primary transition-colors"
            >
              {{ $t('common.viewDefault') }}
            </button>
            <button
              type="button"
              @click="resetSummaryPrompt"
              class="text-xs text-gray-400 hover:text-primary transition-colors"
            >
              {{ $t('common.resetDefault') }}
            </button>
            <button
              type="button"
              @click="showTemplateLibrary"
              class="text-xs text-gray-400 hover:text-primary transition-colors"
            >
              {{ $t('common.templateLibrary') }}
            </button>
            <button
              type="button"
              @click="exportTemplate"
              class="text-xs text-gray-400 hover:text-primary transition-colors"
            >
              {{ $t('common.exportTemplate') }}
            </button>
            <button
              type="button"
              @click="importTemplate"
              class="text-xs text-gray-400 hover:text-primary transition-colors"
            >
              {{ $t('common.importTemplate') }}
            </button>
          </div>
        </div>
        <div class="flex items-center gap-2 pt-1">
          <input
            v-model="localSettings.include_manual_records"
            type="checkbox"
            id="include_manual_records"
            class="w-4 h-4 rounded border-gray-600 bg-darker text-primary focus:ring-primary focus:ring-offset-0 cursor-pointer"
          />
          <label for="include_manual_records" class="text-xs text-gray-300 cursor-pointer">
            {{ $t('settings.includeQuickNotes') }}
          </label>
          <span class="text-xs text-gray-500">{{ $t('settings.includeQuickNotesHint') }}</span>
        </div>
      </div>
    </div>

    <!-- Tag Categories -->
    <div>
      <h3 class="text-sm font-medium text-gray-300 mb-3">{{ $t('settings.tagCategories') }}</h3>
      <div class="space-y-3">
        <div>
          <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.customTagCategories') }}</label>
          <textarea
            v-model="localTagCategoriesText"
            rows="4"
            :placeholder="$t('settings.tagCategoriesPlaceholder')"
            class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none resize-y font-mono"
          />
          <span class="text-xs text-gray-500 mt-1 block">{{ $t('settings.tagCategoriesHint') }}</span>
          <div class="flex gap-3 mt-2">
            <button
              type="button"
              @click="showDefaultTagCategories"
              class="text-xs text-gray-400 hover:text-primary transition-colors"
            >
              {{ $t('settings.viewDefaultTags') }}
            </button>
            <button
              type="button"
              @click="resetTagCategories"
              class="text-xs text-gray-400 hover:text-primary transition-colors"
            >
              {{ $t('common.resetDefault') }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import { showError, showSuccess } from '../../stores/toast'
import { usePlatform } from '../../composables/usePlatform'
import { type ModelInfo } from './shared/types'

// Props
interface Props {
  settings: {
    model_name: string
    analysis_prompt: string
    summary_model_name: string
    summary_prompt: string
    summary_title_format: string
    include_manual_records: boolean
    api_base_url: string
    api_key: string
  }
  tagCategoriesText: string
}

const props = defineProps<Props>()

// Emits
const emit = defineEmits<{
  (e: 'update:settings', value: Props['settings']): void
  (e: 'update:tagCategoriesText', value: string): void
  (e: 'show-default-prompt-modal'): void
  (e: 'show-default-summary-prompt-modal'): void
  (e: 'show-template-library-modal'): void
  (e: 'show-default-tag-categories-modal'): void
}>()

// Composables
const { t } = useI18n()
const { isDesktop } = usePlatform()

// Local state
const localSettings = ref({ ...props.settings })
const localTagCategoriesText = ref(props.tagCategoriesText)

// Watch for external changes
watch(() => props.settings, (newVal) => {
  localSettings.value = { ...newVal }
}, { deep: true })

watch(() => props.tagCategoriesText, (newVal) => {
  localTagCategoriesText.value = newVal
})

// Watch for local changes and emit
watch(localSettings, (newVal) => {
  emit('update:settings', newVal)
}, { deep: true })

watch(localTagCategoriesText, (newVal) => {
  emit('update:tagCategoriesText', newVal)
})

// UI State
const isLoadingModelInfo = ref(false)
const analysisModelInfo = ref<ModelInfo | null>(null)
const summaryModelInfo = ref<ModelInfo | null>(null)

// Methods
async function getModelInfo(type: 'analysis' | 'summary') {
  const modelName = type === 'analysis' ? localSettings.value.model_name : localSettings.value.summary_model_name
  if (!modelName) {
    showError(t('settings.modelNameRequired'))
    return
  }

  isLoadingModelInfo.value = true

  try {
    const result = await invoke<ModelInfo | { error: string; context_window?: number }>('get_model_info', {
      apiBaseUrl: localSettings.value.api_base_url,
      apiKey: localSettings.value.api_key,
      modelName: modelName
    })

    if (type === 'analysis') {
      analysisModelInfo.value = result as ModelInfo
    } else {
      summaryModelInfo.value = result as ModelInfo
    }

    if ('error' in result && result.error) {
      showError(result.error)
    } else if ('context_window' in result && result.context_window) {
      showSuccess(t('settings.modelContextWindow', { model: modelName, size: result.context_window / 1000 }))
    } else {
      showSuccess(t('settings.modelInfoUnavailable'))
    }
  } catch (err) {
    console.error('Failed to get model info:', err)
    showError(err)
  } finally {
    isLoadingModelInfo.value = false
  }
}

function showDefaultPrompt() {
  emit('show-default-prompt-modal')
}

async function resetPrompt() {
  try {
    localSettings.value.analysis_prompt = await invoke('get_default_analysis_prompt')
  } catch (err) {
    console.error('Failed to get default analysis prompt:', err)
    localSettings.value.analysis_prompt = ''
  }
}

function showDefaultSummaryPrompt() {
  emit('show-default-summary-prompt-modal')
}

async function resetSummaryPrompt() {
  try {
    localSettings.value.summary_prompt = await invoke('get_default_summary_prompt')
  } catch (err) {
    console.error('Failed to get default summary prompt:', err)
    localSettings.value.summary_prompt = ''
  }
}

function showTemplateLibrary() {
  emit('show-template-library-modal')
}

async function exportTemplate() {
  // This would be implemented with Tauri file dialog
  console.log('Export template')
}

async function importTemplate() {
  // This would be implemented with Tauri file dialog
  console.log('Import template')
}

function showDefaultTagCategories() {
  emit('show-default-tag-categories-modal')
}

function resetTagCategories() {
  localTagCategoriesText.value = ''
}
</script>