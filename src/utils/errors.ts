/**
 * Error handling utility for DailyLogger
 * Provides unified error classification and user-friendly messages
 */

/**
 * Error type enumeration
 */
export const ErrorType = {
  NETWORK: 'NETWORK',
  AUTH: 'AUTH',
  QUOTA: 'QUOTA',
  VALIDATION: 'VALIDATION',
  UNKNOWN: 'UNKNOWN'
} as const

export type ErrorTypeValue = typeof ErrorType[keyof typeof ErrorType]

/**
 * Patterns for identifying error types
 * Order matters: more specific patterns should come first
 */
const ERROR_PATTERNS: Record<ErrorTypeValue, string[]> = {
  [ErrorType.NETWORK]: [
    'network',
    'timeout',
    'ECONNREFUSED',
    'ENOTFOUND',
    'ECONNRESET',
    'fetch failed',
    'networkerror'
  ],
  [ErrorType.AUTH]: [
    '401',
    '403',
    'unauthorized',
    'api key',
    'authentication failed',
    'invalid api key',
    'access denied'
  ],
  [ErrorType.QUOTA]: [
    '429',
    'rate limit',
    'quota',
    'too many requests',
    'usage limit'
  ],
  [ErrorType.VALIDATION]: [
    'invalid',
    'validation',
    'format',
    'required',
    'empty',
    'cannot be empty'
  ],
  [ErrorType.UNKNOWN]: []
}

/**
 * User-friendly error messages (in Chinese)
 */
const ERROR_MESSAGES: Record<ErrorTypeValue, string> = {
  [ErrorType.NETWORK]: '网络连接失败，请检查网络设置',
  [ErrorType.AUTH]: 'API Key 无效或已过期',
  [ErrorType.QUOTA]: 'API 调用次数已达上限',
  [ErrorType.VALIDATION]: '输入内容格式不正确',
  [ErrorType.UNKNOWN]: '操作失败，请稍后重试'
}

/**
 * Suggested actions for each error type
 */
const SUGGESTED_ACTIONS: Record<ErrorTypeValue, string> = {
  [ErrorType.NETWORK]: '重试',
  [ErrorType.AUTH]: '检查设置',
  [ErrorType.QUOTA]: '检查账户',
  [ErrorType.VALIDATION]: '修改输入',
  [ErrorType.UNKNOWN]: '重试'
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
 * Get user-friendly error message for an error type
 * @param errorType - The error type from ErrorType enum
 * @returns User-friendly error message in Chinese
 */
export function getErrorMessage(errorType: ErrorTypeValue): string {
  return ERROR_MESSAGES[errorType] || ERROR_MESSAGES[ErrorType.UNKNOWN]
}

/**
 * Get suggested action for an error type
 * @param errorType - The error type from ErrorType enum
 * @returns Suggested action in Chinese
 */
export function getSuggestedAction(errorType: ErrorTypeValue): string {
  return SUGGESTED_ACTIONS[errorType] || SUGGESTED_ACTIONS[ErrorType.UNKNOWN]
}

/**
 * Structured error info object
 */
export interface ErrorInfo {
  type: ErrorTypeValue
  message: string
  suggestion: string
  originalError: string
}

/**
 * Create a structured error info object
 * @param error - The original error
 * @returns Structured error information
 */
export function createErrorInfo(error: Error | string): ErrorInfo {
  const type = parseError(error)
  return {
    type,
    message: getErrorMessage(type),
    suggestion: getSuggestedAction(type),
    originalError: error instanceof Error ? error.message : String(error)
  }
}