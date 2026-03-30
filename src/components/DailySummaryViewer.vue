<template>
  <BaseModal content-class="w-[90vw] h-[90vh] max-w-4xl overflow-hidden flex flex-col" @close="$emit('close')">
    <div class="px-6 py-4 border-b border-[var(--color-border)] flex items-center justify-between">
      <h2 class="text-lg font-semibold">📝 {{ t('dailySummaryViewer.title') }}</h2>
      <div class="flex items-center gap-2">
        <button
          @click="openInObsidian"
          class="px-3 py-1 text-sm bg-[var(--color-surface-2)] hover:bg-[var(--color-action-neutral)] rounded transition-colors"
        >
          {{ t('dailySummaryViewer.showInFinder') }}
        </button>
        <button @click="$emit('close')" class="text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]">✕</button>
      </div>
    </div>

      <div class="flex-1 overflow-auto p-6">
        <div v-if="loading" class="text-center py-8 text-[var(--color-text-muted)]">
          {{ t('dailySummaryViewer.loading') }}
        </div>
        <div v-else-if="error" class="text-center py-8 text-red-500">
          {{ error }}
        </div>
        <div v-else class="prose prose-invert max-w-none">
          <div class="text-sm text-[var(--color-text-secondary)] mb-4">
            {{ t('dailySummaryViewer.filePath') }} {{ summaryPath }}
          </div>
          <div class="whitespace-pre-wrap text-[var(--color-text-secondary)] leading-relaxed">
            {{ content }}
          </div>
        </div>
      </div>
  </BaseModal>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-shell'
import { useI18n } from 'vue-i18n'
import BaseModal from './BaseModal.vue'

const { t } = useI18n()

const props = defineProps<{
  summaryPath: string
}>()

const emit = defineEmits<{(e: 'close'): void}>()

const content = ref('')
const loading = ref(true)
const error = ref('')

const loadSummary = async () => {
  if (!props.summaryPath) {
    error.value = t('dailySummaryViewer.pathEmpty')
    loading.value = false
    return
  }

  try {
    content.value = await invoke<string>('read_file', { path: props.summaryPath })
  } catch (err) {
    error.value = t('dailySummaryViewer.loadFailed', { error: err })
    console.error('Failed to load summary:', err)
  } finally {
    loading.value = false
  }
}

const openInObsidian = async () => {
  try {
    // Extract directory from path - handle both / and \ as separators
    const normalizedPath = props.summaryPath.replace(/\\/g, '/')
    const lastSlashIndex = normalizedPath.lastIndexOf('/')
    const dirPath = lastSlashIndex > 0 ? normalizedPath.slice(0, lastSlashIndex) : normalizedPath
    await open(dirPath)
  } catch (err) {
    console.error('Failed to open in Finder:', err)
  }
}

onMounted(() => {
  loadSummary()
})
</script>
