<template>
  <Teleport to="body">
    <Transition name="toast">
      <div
        v-if="visible"
        :class="[
          'fixed bottom-4 right-4 z-40 max-w-md p-4 rounded-lg shadow-lg border flex items-start gap-3 pointer-events-none',
          typeClasses
        ]"
        role="alert"
      >
        <span class="text-xl flex-shrink-0">{{ icon }}</span>
        <div class="flex-1 min-w-0">
          <p class="font-medium text-sm">{{ title }}</p>
          <p v-if="message" class="text-xs mt-1 opacity-90">{{ message }}</p>
          <p v-if="timestamp" class="text-xs mt-1 opacity-60">{{ timestamp }}</p>
        </div>
        <button
          v-if="dismissible"
          @click="dismiss"
          class="flex-shrink-0 p-1 rounded hover:bg-black/10 transition-colors pointer-events-auto"
          aria-label="Dismiss"
        >
          ✕
        </button>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'

export type ErrorToastType = 'error' | 'warning' | 'info' | 'success'

const props = withDefaults(defineProps<{
  type?: ErrorToastType
  title: string
  message?: string
  duration?: number // Auto-dismiss after ms, 0 = no auto-dismiss
  dismissible?: boolean
}>(), {
  type: 'error',
  duration: 5000,
  dismissible: true,
})

const emit = defineEmits<{
  (e: 'dismiss'): void
  (e: 'click'): void
}>()

const visible = ref(true)
let dismissTimer: ReturnType<typeof setTimeout> | null = null

const typeClasses = computed(() => {
  switch (props.type) {
    case 'error':
      return 'bg-red-900/95 border-red-700 text-red-100'
    case 'warning':
      return 'bg-yellow-900/95 border-yellow-700 text-yellow-100'
    case 'success':
      return 'bg-green-900/95 border-green-700 text-green-100'
    case 'info':
    default:
      return 'bg-blue-900/95 border-blue-700 text-blue-100'
  }
})

const icon = computed(() => {
  switch (props.type) {
    case 'error':
      return '⚠'
    case 'warning':
      return '⚡'
    case 'success':
      return '✓'
    case 'info':
    default:
      return 'ℹ'
  }
})

const timestamp = computed(() => {
  return new Date().toLocaleTimeString()
})

function dismiss() {
  visible.value = false
  emit('dismiss')
}

function scheduleAutoDismiss() {
  if (props.duration > 0) {
    dismissTimer = setTimeout(() => {
      dismiss()
    }, props.duration)
  }
}

function cancelAutoDismiss() {
  if (dismissTimer) {
    clearTimeout(dismissTimer)
    dismissTimer = null
  }
}

// Watch for visibility changes to manage auto-dismiss
watch(visible, (v) => {
  if (v) {
    scheduleAutoDismiss()
  } else {
    cancelAutoDismiss()
  }
}, { immediate: true })
</script>

<style scoped>
.toast-enter-active,
.toast-leave-active {
  transition: transform 0.3s ease, opacity 0.3s ease;
}

.toast-enter-from {
  transform: translateX(100%);
  opacity: 0;
}

.toast-leave-to {
  transform: translateX(100%);
  opacity: 0;
}
</style>
