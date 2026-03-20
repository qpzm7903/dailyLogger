/**
 * Toast notification state management
 * Provides centralized toast notification queue management
 */
import { ref, type Ref } from 'vue'
import { parseError, getErrorMessageKey, getSuggestedActionKey, type ErrorTypeValue } from '../utils/errors'
import { useI18n } from 'vue-i18n'

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
      suggestion: toast.suggestion,
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
 * Get i18n instance for error messages
 * This is a workaround since we can't use useI18n() outside of setup()
 */
let i18nInstance: ReturnType<typeof useI18n> | null = null

/**
 * Initialize the i18n instance for toast errors
 * Called from App.vue setup
 */
export function initToastI18n(i18n: ReturnType<typeof useI18n>): void {
  i18nInstance = i18n
}

/**
 * Get translated error message
 */
function getTranslatedErrorMessage(errorType: ErrorTypeValue): string {
  if (i18nInstance) {
    return i18nInstance.t(getErrorMessageKey(errorType))
  }
  // Fallback messages (Chinese)
  const fallbackMessages: Record<ErrorTypeValue, string> = {
    network: '网络连接失败，请检查网络设置',
    auth: 'API Key 无效或已过期',
    quota: 'API 调用次数已达上限',
    validation: '输入内容格式不正确',
    database: '数据库操作失败',
    fileIO: '文件读写失败',
    screenshot: '截图捕获失败',
    timeout: '请求超时，请稍后重试',
    unknown: '操作失败，请稍后重试'
  }
  return fallbackMessages[errorType] || fallbackMessages.unknown
}

/**
 * Get translated suggestion
 */
function getTranslatedSuggestion(errorType: ErrorTypeValue): string {
  if (i18nInstance) {
    return i18nInstance.t(getSuggestedActionKey(errorType))
  }
  // Fallback suggestions (Chinese)
  const fallbackSuggestions: Record<ErrorTypeValue, string> = {
    network: '检查网络连接后重试',
    auth: '前往设置检查 API Key',
    quota: '检查账户余额或升级套餐',
    validation: '检查输入内容格式',
    database: '尝试重启应用',
    fileIO: '检查文件路径和权限',
    screenshot: '检查截图权限设置',
    timeout: '检查网络状况后重试',
    unknown: '重试或联系支持'
  }
  return fallbackSuggestions[errorType] || fallbackSuggestions.unknown
}

/**
 * Show an error toast with parsed error info
 * @param error - The error to display
 * @param retryCallback - Optional retry callback
 * @returns Toast ID
 */
export function showError(error: unknown, retryCallback: (() => void) | null = null): number {
  const errorType = parseError(error instanceof Error ? error : String(error))
  const message = getTranslatedErrorMessage(errorType)
  const suggestion = getTranslatedSuggestion(errorType)

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