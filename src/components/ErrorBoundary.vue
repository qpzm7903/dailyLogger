<template>
  <div class="error-boundary-wrapper">
    <slot />
    <ErrorToast
      v-if="hasError && errorInfo"
      type="error"
      :title="t('errorBoundary.title')"
      :message="errorInfo.error?.message || 'Unknown error'"
      :duration="0"
      :dismissible="true"
      @dismiss="resetError"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onErrorCaptured } from 'vue'
import { useI18n } from 'vue-i18n'
import ErrorToast from './ErrorToast.vue'

const { t } = useI18n()

// Error info passed to error handler
interface ErrorInfo {
  error: Error | null
  info: string
}

// Track error state and info
const hasError = ref(false)
const errorInfo = ref<ErrorInfo | null>(null)

// Capture errors from child components
onErrorCaptured((error, instance, info) => {
  console.error('[ErrorBoundary] Caught error:', error, 'Info:', info)

  hasError.value = true
  errorInfo.value = { error, info }

  // Log to file via backend if available
  logErrorToBackend(error, info)

  // Return false to prevent error from propagating further
  return false
})

// Send error to backend for logging
async function logErrorToBackend(error: Error | null, info: string) {
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    await invoke('log_frontend_error', {
      message: error?.message || 'Unknown error',
      stack: error?.stack || '',
      source: info,
    })
  } catch (e) {
    console.warn('[ErrorBoundary] Failed to log error to backend:', e)
  }
}

// Reset error state - called when user dismisses the error toast
function resetError() {
  hasError.value = false
  errorInfo.value = null
}

defineExpose({ resetError })
</script>
