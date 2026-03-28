<template>
  <div ref="containerRef" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="$emit('close')">
    <div class="bg-[var(--color-surface-1)] rounded-2xl w-[600px] border border-[var(--color-border)] shadow-2xl">
      <div class="px-6 py-4 border-b border-[var(--color-border)] flex items-center justify-between">
        <div class="flex items-center gap-2">
          <span class="text-xl">⚡</span>
          <h2 class="text-lg font-semibold">{{ $t('quickNote.title') }}</h2>
        </div>
        <span v-if="isDesktop" class="text-xs text-[var(--color-text-muted)]">{{ $t('quickNote.shortcutHint') }}</span>
      </div>

      <div class="p-6">
        <textarea
          ref="inputRef"
          v-model="content"
          @keydown.enter.exact.prevent="save"
          :placeholder="$t('quickNote.placeholder')"
          class="w-full h-40 bg-[var(--color-surface-0)] border border-[var(--color-border)] rounded-lg px-4 py-3 text-sm focus:border-primary focus:outline-none resize-none"
          autofocus
        ></textarea>
      </div>

      <div class="px-6 py-4 border-t border-[var(--color-border)] flex justify-between items-center">
        <span class="text-xs text-[var(--color-text-muted)]">{{ currentTime }}</span>
        <div class="flex gap-3">
          <button
            @click="$emit('close')"
            class="px-4 py-2 rounded-lg text-sm hover:bg-[var(--color-action-neutral)] transition-colors"
          >
            {{ $t('common.cancel') }}
          </button>
          <button
            @click="save"
            :disabled="!content.trim()"
            class="px-4 py-2 bg-primary rounded-lg text-sm font-medium hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            {{ $t('common.save') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, nextTick, onBeforeUnmount } from 'vue'
import { useI18n } from 'vue-i18n'
import { usePlatform } from '../composables/usePlatform'
import { useFocusTrap } from '../composables/useFocusTrap'

const { locale } = useI18n()
const { isDesktop } = usePlatform()
const emit = defineEmits<{(e: 'close'): void; (e: 'save', content: string): void}>()

const content = ref('')
const inputRef = ref<HTMLTextAreaElement | null>(null)
const currentTime = ref('')

// Focus trap for accessibility (UX-5)
const containerRef = ref<HTMLElement | null>(null)
const { activate: activateFocusTrap, deactivate: deactivateFocusTrap } = useFocusTrap(containerRef)

const updateTime = () => {
  currentTime.value = new Date().toLocaleString(locale.value === 'zh-CN' ? 'zh-CN' : 'en-US', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  })
}

const save = () => {
  if (!content.value.trim()) return
  emit('save', content.value.trim())
}

let timeInterval: ReturnType<typeof setInterval> | null = null

onMounted(() => {
  updateTime()
  timeInterval = setInterval(updateTime, 1000)
  nextTick(() => {
    inputRef.value?.focus()
  })
  activateFocusTrap()
})

onBeforeUnmount(() => {
  if (timeInterval) clearInterval(timeInterval)
  deactivateFocusTrap()
})
</script>