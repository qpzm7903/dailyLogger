<template>
  <slot />
</template>

<script setup lang="ts">
import { ref, onErrorCaptured } from 'vue'

// Error info passed to error handler
interface ErrorInfo {
  error: Error | null
  info: string
}

// Emit error to parent components
const emit = defineEmits<{
  (e: 'error-captured', error: ErrorInfo): void
}>()

// Track if we have an error
const hasError = ref(false)

// Capture errors from child components
onErrorCaptured((error, instance, info) => {
  console.error('[ErrorBoundary] Caught error:', error, 'Info:', info)

  hasError.value = true

  // Emit error event for parent to handle
  emit('error-captured', {
    error,
    info,
  })

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

// Reset error state - called by parent when error is acknowledged
function resetError() {
  hasError.value = false
}

// Expose reset function
defineExpose({ resetError })
</script>
