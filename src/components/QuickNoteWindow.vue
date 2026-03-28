<template>
  <div class="h-screen bg-[var(--color-surface-0)] text-[var(--color-text-primary)] flex flex-col">
    <header class="bg-[var(--color-surface-1)] border-b border-[var(--color-border)] px-4 py-3 flex items-center justify-between">
      <div class="flex items-center gap-2">
        <span class="text-lg">⚡</span>
        <h1 class="text-sm font-medium">{{ t('quickNoteWindow.title') }}</h1>
      </div>
      <span v-if="isDesktop" class="text-xs text-[var(--color-text-muted)]">{{ t('quickNoteWindow.shortcutHint') }}</span>
    </header>

    <main class="flex-1 p-4">
      <textarea
        ref="inputRef"
        v-model="content"
        @keydown.enter.exact.prevent="save"
        @keydown.esc="closeWindow"
        :placeholder="t('quickNoteWindow.placeholder')"
        class="w-full h-full bg-[var(--color-surface-0)] border border-[var(--color-border)] rounded-lg px-4 py-3 text-sm focus:border-primary focus:outline-none resize-none"
        autofocus
      ></textarea>
    </main>

    <footer class="bg-[var(--color-surface-1)] border-t border-[var(--color-border)] px-4 py-3 flex justify-between items-center">
      <span class="text-xs text-[var(--color-text-muted)]">{{ currentTime }}</span>
      <div class="flex gap-2">
        <button
          @click="closeWindow"
          class="px-3 py-1.5 rounded-lg text-xs hover:bg-[var(--color-action-neutral)] transition-colors"
        >
          {{ t('quickNoteWindow.cancel') }}
        </button>
        <button
          @click="save"
          :disabled="!content.trim() || isSaving"
          class="px-3 py-1.5 bg-primary rounded-lg text-xs font-medium hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          {{ isSaving ? t('quickNoteWindow.saving') : t('quickNoteWindow.save') }}
        </button>
      </div>
    </footer>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useI18n } from 'vue-i18n'
import { usePlatform } from '../composables/usePlatform'
import { systemActions } from '../features/system/actions'

const { t } = useI18n()
const { isDesktop } = usePlatform()

const content = ref('')
const inputRef = ref<HTMLTextAreaElement | null>(null)
const currentTime = ref('')
const isSaving = ref(false)

const updateTime = () => {
  currentTime.value = new Date().toLocaleString('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  })
}

const closeWindow = async () => {
  const window = getCurrentWindow()
  await window.close()
}

const save = async () => {
  if (!content.value.trim() || isSaving.value) return

  isSaving.value = true
  try {
    await systemActions.trayQuickNote(content.value.trim())
    await closeWindow()
  } catch (err) {
    console.error('Failed to save quick note:', err)
    isSaving.value = false
  }
}

let timeInterval: ReturnType<typeof setInterval> | null = null

onMounted(() => {
  updateTime()
  timeInterval = setInterval(updateTime, 1000)
  inputRef.value?.focus()
})

onUnmounted(() => {
  if (timeInterval) clearInterval(timeInterval)
})
</script>