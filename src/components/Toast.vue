<template>
  <Teleport to="body">
    <div v-if="toasts.length > 0" class="toast-container fixed bottom-4 right-4 z-50 flex flex-col gap-2 max-w-sm">
      <TransitionGroup name="toast">
        <div
          v-for="toast in toasts"
          :key="toast.id"
          :class="[
            'bg-[var(--color-surface-1)] border rounded-lg px-4 py-3 shadow-lg',
            toast.type === 'error' ? 'border-red-700 bg-red-900/20' : '',
            toast.type === 'success' ? 'border-green-700 bg-green-900/20' : '',
            toast.type === 'warning' ? 'border-yellow-700 bg-yellow-900/20' : '',
            toast.type === 'info' ? 'border-[var(--color-border)]' : ''
          ]"
        >
          <div class="flex items-start gap-2">
            <!-- Type icon -->
            <span v-if="toast.type === 'error'" class="text-red-400 text-sm flex-shrink-0">⚠️</span>
            <span v-else-if="toast.type === 'success'" class="text-green-400 text-sm flex-shrink-0">✓</span>
            <span v-else-if="toast.type === 'warning'" class="text-yellow-400 text-sm flex-shrink-0">⚡</span>
            <span v-else class="text-blue-400 text-sm flex-shrink-0">ℹ️</span>

            <div class="flex-1 min-w-0">
              <!-- Message -->
              <p class="text-sm text-[var(--color-text-secondary)]">{{ toast.message }}</p>

              <!-- Suggestion -->
              <p v-if="toast.suggestion" class="toast-suggestion text-xs text-[var(--color-text-secondary)] mt-1">
                {{ t('toast.suggestion') }} {{ toast.suggestion }}
              </p>
            </div>
          </div>

          <!-- Action buttons -->
          <div v-if="toast.retryCallback" class="mt-2 flex gap-2 justify-end">
            <button
              @click="handleRetry(toast)"
              class="btn-retry px-3 py-1 text-xs bg-primary hover:bg-blue-600 rounded text-[var(--color-text-primary)] transition-colors"
            >
              {{ t('toast.retry') }}
            </button>
            <button
              @click="remove(toast.id)"
              class="btn-close px-3 py-1 text-xs bg-[var(--color-action-neutral)] hover:bg-[var(--color-action-neutral)] rounded text-[var(--color-text-secondary)] transition-colors"
            >
              {{ t('toast.close') }}
            </button>
          </div>

          <!-- Close button for non-retryable toasts -->
          <button
            v-if="!toast.retryCallback"
            @click="remove(toast.id)"
            class="btn-close absolute top-2 right-2 text-[var(--color-text-muted)] hover:text-[var(--color-text-primary)] text-xs"
          >
            ✕
          </button>
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { useToastStore } from '../stores/toast'
import type { Toast } from '../stores/toast'

const { t } = useI18n()
const { toasts, remove } = useToastStore()

const handleRetry = (toast: Toast): void => {
  if (toast.retryCallback) {
    toast.retryCallback()
  }
  remove(toast.id)
}
</script>

<style scoped>
.toast-enter-active,
.toast-leave-active {
  transition: all 0.3s ease;
}

.toast-enter-from {
  opacity: 0;
  transform: translateX(100%);
}

.toast-leave-to {
  opacity: 0;
  transform: translateX(100%);
}
</style>