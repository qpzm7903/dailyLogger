/**
 * Toast notification state management
 * Provides centralized toast notification queue management
 */
import { ref, computed, type Ref, type ComputedRef } from 'vue'
import { parseError, getErrorMessage, getSuggestedAction, type ErrorTypeValue } from '../utils/errors'

/**
 * Toast notification type
 */
export type ToastType = 'success' | 'error' | 'warning' | 'info'

/**
 * Toast notification item
 */
export interface Toast {
  id: number
  message: string
  type: ToastType
  suggestion: string | null
  retryCallback: (() => void) | null
  duration: number
}

/**
 * Options for adding a toast
 */
export interface ToastOptions {
  type?: ToastType
  suggestion?: string | null
  retryCallback?: (() => void) | null
  duration?: number
}

// Global toast queue
const toasts = ref<Toast[]>([])
let toastId = 0

/**
 * Toast store composable
 */
export function useToastStore(): {
  toasts: Ref<Toast[]>
  add: (toast: ToastOptions & { message: string }) => number
  remove: (id: number) => void
  clear: () => void
} {
  const add = (toast: ToastOptions & { message: string }): number => {
    const id = ++toastId
    const defaultDuration = toast.type === 'error' ? 0 : 5000 // Error toasts don't auto-dismiss

    const newToast: Toast = {
      id,
      message: toast.message,
      type: toast.type || 'info',
      suggestion: toast.suggestion || null,
      retryCallback: toast.retryCallback || null,
      duration: toast.duration ?? defaultDuration
    }

    toasts.value.push(newToast)

    // Auto remove after duration (if duration > 0)
    if (newToast.duration > 0) {
      setTimeout(() => {
        remove(id)
      }, newToast.duration)
    }

    return id
  }

  const remove = (id: number): void => {
    const index = toasts.value.findIndex(t => t.id === id)
    if (index !== -1) {
      toasts.value.splice(index, 1)
    }
  }

  const clear = (): void => {
    toasts.value = []
  }

  return {
    toasts,
    add,
    remove,
    clear
  }
}

/**
 * Show a toast notification
 * @param message - The message to display
 * @param options - Toast options
 * @returns Toast ID
 */
export function showToast(message: string, options: ToastOptions = {}): number {
  const store = useToastStore()
  return store.add({
    message,
    type: options.type || 'info',
    suggestion: options.suggestion,
    retryCallback: options.retryCallback,
    duration: options.duration
  })
}

/**
 * Show an error toast with parsed error info
 * @param error - The error to display
 * @param retryCallback - Optional retry callback
 * @returns Toast ID
 */
export function showError(error: unknown, retryCallback: (() => void) | null = null): number {
  const errorType = parseError(error instanceof Error ? error : String(error))
  const message = getErrorMessage(errorType)
  const suggestion = getSuggestedAction(errorType)

  return showToast(message, {
    type: 'error',
    suggestion: retryCallback ? suggestion : null,
    retryCallback
  })
}

/**
 * Show a success toast
 * @param message - The success message
 * @returns Toast ID
 */
export function showSuccess(message: string): number {
  return showToast(message, { type: 'success' })
}

/**
 * Show a warning toast
 * @param message - The warning message
 * @returns Toast ID
 */
export function showWarning(message: string): number {
  return showToast(message, { type: 'warning' })
}

/**
 * Show an info toast
 * @param message - The info message
 * @returns Toast ID
 */
export function showInfo(message: string): number {
  return showToast(message, { type: 'info' })
}