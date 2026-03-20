<template>
  <div class="fixed inset-0 bg-black/80 flex items-center justify-center z-50" @click.self="$emit('close')">
    <div class="bg-dark rounded-2xl w-[90vw] h-[80vh] max-w-3xl overflow-hidden border border-gray-700 flex flex-col">
      <div class="px-6 py-4 border-b border-gray-700 flex items-center justify-between">
        <h2 class="text-lg font-semibold">📁 {{ t('reportHistory.title') }}</h2>
        <button @click="$emit('close')" class="text-gray-400 hover:text-white">✕</button>
      </div>

      <div class="flex-1 overflow-auto p-6">
        <div v-if="loading" class="text-center py-8 text-gray-500">
          {{ t('reportHistory.loading') }}
        </div>
        <div v-else-if="error" class="text-center py-8 text-red-500">
          {{ error }}
        </div>
        <div v-else-if="files.length === 0" class="text-center py-8 text-gray-500">
          {{ t('reportHistory.noFiles') }}
        </div>
        <div v-else class="space-y-2">
          <div
            v-for="file in files"
            :key="file.path"
            @click="selectFile(file)"
            class="flex items-center gap-3 p-3 bg-darker rounded-lg border border-gray-700 hover:border-primary cursor-pointer transition-colors"
            :class="{ 'border-primary': selectedFile?.path === file.path }"
          >
            <div class="flex-1 min-w-0">
              <div class="text-sm text-gray-200 truncate">{{ file.name }}</div>
              <div class="text-xs text-gray-500 mt-1">
                {{ t('reportHistory.modified') }}: {{ file.modified_time }}
                <span class="ml-2">{{ formatSize(file.size_bytes) }}</span>
              </div>
            </div>
            <button
              @click.stop="viewFile(file)"
              class="px-3 py-1.5 text-xs bg-primary/20 text-primary rounded hover:bg-primary hover:text-white transition-colors"
            >
              {{ t('reportHistory.view') }}
            </button>
          </div>
        </div>
      </div>

      <div class="px-6 py-4 border-t border-gray-700 flex justify-end gap-3">
        <button
          @click="$emit('close')"
          class="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded-lg text-sm text-gray-200 transition-colors"
        >
          {{ t('common.close') }}
        </button>
        <button
          @click="viewSelected"
          :disabled="!selectedFile"
          class="px-4 py-2 bg-primary hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed rounded-lg text-sm font-medium text-white transition-colors"
        >
          {{ t('reportHistory.viewSelected') }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

interface ReportFile {
  name: string
  path: string
  modified_time: string
  size_bytes: number
}

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'viewFile', path: string): void
}>()

const files = ref<ReportFile[]>([])
const selectedFile = ref<ReportFile | null>(null)
const loading = ref(true)
const error = ref('')

const loadFiles = async () => {
  try {
    files.value = await invoke<ReportFile[]>('list_report_files')
  } catch (err) {
    error.value = String(err)
    console.error('Failed to list report files:', err)
  } finally {
    loading.value = false
  }
}

const selectFile = (file: ReportFile) => {
  selectedFile.value = file
}

const viewFile = (file: ReportFile) => {
  emit('viewFile', file.path)
}

const viewSelected = () => {
  if (selectedFile.value) {
    emit('viewFile', selectedFile.value.path)
  }
}

const formatSize = (bytes: number): string => {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`
}

onMounted(() => {
  loadFiles()
})
</script>