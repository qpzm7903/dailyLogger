/**
 * Error handling utility for DailyLogger
 * Provides unified error classification and user-friendly messages
 */

/**
 * Error type enumeration
 */
export const ErrorType = {
  NETWORK: 'network',
  AUTH: 'auth',
  QUOTA: 'quota',
  VALIDATION: 'validation',
  DATABASE: 'database',
  FILE_IO: 'fileIO',
  SCREENSHOT: 'screenshot',
  TIMEOUT: 'timeout',
  UNKNOWN: 'unknown'
} as const

export type ErrorTypeValue = typeof ErrorType[keyof typeof ErrorType]

/**
 * Patterns for identifying error types
 * Order matters: more specific patterns should come first
 */
const ERROR_PATTERNS: Record<ErrorTypeValue, string[]> = {
  [ErrorType.TIMEOUT]: [
    'timeout',
    'timed out',
    '请求超时',
    '超时'
  ],
  [ErrorType.SCREENSHOT]: [
    'screenshot',
    'capture',
    '截图',
    '屏幕捕获',
    'permission denied',
    '权限被拒绝'
  ],
  [ErrorType.FILE_IO]: [
    'file not found',
    'enoent',
    'eacces',
    'permission denied',
    '文件不存在',
    '文件读写',
    'cannot read',
    'cannot write',
    'failed to write',
    'failed to read'
  ],
  [ErrorType.DATABASE]: [
    'database',
    'sqlite',
    'sql',
    '数据库',
    'db locked',
    'constraint',
    'unique constraint',
    'foreign key'
  ],
  [ErrorType.NETWORK]: [
    'network',
    'econnrefused',
    'enotfound',
    'econnreset',
    'fetch failed',
    'networkerror',
    '网络连接',
    '连接失败'
  ],
  [ErrorType.AUTH]: [
    '401',
    '403',
    'unauthorized',
    'api key',
    'authentication failed',
    'invalid api key',
    'access denied',
    '认证失败',
    'api key 无效',
    'api key 过期'
  ],
  [ErrorType.QUOTA]: [
    '429',
    'rate limit',
    'quota',
    'too many requests',
    'usage limit',
    '配额',
    '调用次数',
    'rate_limit'
  ],
  [ErrorType.VALIDATION]: [
    'invalid',
    'validation',
    'format',
    'required',
    'empty',
    'cannot be empty',
    '验证失败',
    '格式不正确',
    '不能为空'
  ],
  [ErrorType.UNKNOWN]: []
}

/**
 * Parse an error and return its type
 * @param error - The error to parse
 * @returns The error type from ErrorType enum
 */
export function parseError(error: Error | string): ErrorTypeValue {
  const errorMessage = error instanceof Error
    ? error.message.toLowerCase()
    : String(error).toLowerCase()

  for (const [type, patterns] of Object.entries(ERROR_PATTERNS)) {
    for (const pattern of patterns) {
      if (errorMessage.includes(pattern.toLowerCase())) {
        return type as ErrorTypeValue
      }
    }
  }

  return ErrorType.UNKNOWN
}

/**
 * Get i18n key for error message
 * @param errorType - The error type from ErrorType enum
 * @returns The i18n key for the error message
 */
export function getErrorMessageKey(errorType: ErrorTypeValue): string {
  return `errors.messages.${errorType}`
}

/**
 * Get i18n key for suggested action
 * @param errorType - The error type from ErrorType enum
 * @returns The i18n key for the suggested action
 */
export function getSuggestedActionKey(errorType: ErrorTypeValue): string {
  return `errors.suggestions.${errorType}`
}

/**
 * Structured error info object
 */
export interface ErrorInfo {
  type: ErrorTypeValue
  messageKey: string
  suggestionKey: string
  originalError: string
}

/**
 * Create a structured error info object
 * @param error - The original error
 * @returns Structured error information with i18n keys
 */
export function createErrorInfo(error: Error | string): ErrorInfo {
  const type = parseError(error)
  return {
    type,
    messageKey: getErrorMessageKey(type),
    suggestionKey: getSuggestedActionKey(type),
    originalError: error instanceof Error ? error.message : String(error)
  }
}