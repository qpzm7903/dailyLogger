<template>
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="$emit('close')">
    <div class="bg-dark rounded-2xl w-[600px] border border-gray-700 shadow-2xl">
      <div class="px-6 py-4 border-b border-gray-700 flex items-center justify-between">
        <div class="flex items-center gap-2">
          <span class="text-xl">⚡</span>
          <h2 class="text-lg font-semibold">{{ $t('quickNote.title') }}</h2>
        </div>
        <span v-if="isDesktop" class="text-xs text-gray-500">{{ $t('quickNote.shortcutHint') }}</span>
      </div>

      <div class="p-6">
        <textarea
          ref="inputRef"
          v-model="content"
          @keydown.enter.exact.prevent="save"
          :placeholder="$t('quickNote.placeholder')"
          class="w-full h-40 bg-darker border border-gray-700 rounded-lg px-4 py-3 text-sm focus:border-primary focus:outline-none resize-none"
          autofocus
        ></textarea>
      </div>

      <div class="px-6 py-4 border-t border-gray-700 flex justify-between items-center">
        <span class="text-xs text-gray-500">{{ currentTime }}</span>
        <div class="flex gap-3">
          <button
            @click="$emit('close')"
            class="px-4 py-2 rounded-lg text-sm hover:bg-gray-700 transition-colors"
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
import { ref, onMounted, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import { usePlatform } from '../composables/usePlatform'

const { locale } = useI18n()
const { isDesktop } = usePlatform()
const emit = defineEmits<{(e: 'close'): void; (e: 'save', content: string): void}>()

const content = ref('')
const inputRef = ref<HTMLTextAreaElement | null>(null)
const currentTime = ref('')

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

onMounted(() => {
  updateTime()
  setInterval(updateTime, 1000)
  nextTick(() => {
    inputRef.value?.focus()
  })
})
</script>