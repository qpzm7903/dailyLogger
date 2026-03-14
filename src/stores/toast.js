/**
 * Toast notification state management
 * Provides centralized toast notification queue management
 */
import { ref, computed } from 'vue'
import { parseError, getErrorMessage, getSuggestedAction } from '../utils/errors.js'

// Global toast queue
const toasts = ref([])
let toastId = 0

/**
 * Toast store composable
 */
export function useToastStore() {
  const add = (toast) => {
    const id = ++toastId
    const defaultDuration = toast.type === 'error' ? 0 : 5000 // Error toasts don't auto-dismiss

    const newToast = {
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

  const remove = (id) => {
    const index = toasts.value.findIndex(t => t.id === id)
    if (index !== -1) {
      toasts.value.splice(index, 1)
    }
  }

  const clear = () => {
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
 * @param {string} message - The message to display
 * @param {Object} options - Toast options
 * @param {string} [options.type='info'] - Toast type: success, error, warning, info
 * @param {string} [options.suggestion] - Suggested action
 * @param {Function} [options.retryCallback] - Callback for retry button
 * @param {number} [options.duration] - Auto-dismiss duration in ms (0 for no auto-dismiss)
 * @returns {number} Toast ID
 */
export function showToast(message, options = {}) {
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
 * @param {Error|string} error - The error to display
 * @param {Function} [retryCallback] - Optional retry callback
 * @returns {number} Toast ID
 */
export function showError(error, retryCallback = null) {
  const errorType = parseError(error)
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
 * @param {string} message - The success message
 * @returns {number} Toast ID
 */
export function showSuccess(message) {
  return showToast(message, { type: 'success' })
}

/**
 * Show a warning toast
 * @param {string} message - The warning message
 * @returns {number} Toast ID
 */
export function showWarning(message) {
  return showToast(message, { type: 'warning' })
}

/**
 * Show an info toast
 * @param {string} message - The info message
 * @returns {number} Toast ID
 */
export function showInfo(message) {
  return showToast(message, { type: 'info' })
}