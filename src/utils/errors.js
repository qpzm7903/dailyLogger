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
}

/**
 * Patterns for identifying error types
 * Order matters: more specific patterns should come first
 */
const ERROR_PATTERNS = {
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
  ]
}

/**
 * User-friendly error messages (in Chinese)
 */
const ERROR_MESSAGES = {
  [ErrorType.NETWORK]: '网络连接失败，请检查网络设置',
  [ErrorType.AUTH]: 'API Key 无效或已过期',
  [ErrorType.QUOTA]: 'API 调用次数已达上限',
  [ErrorType.VALIDATION]: '输入内容格式不正确',
  [ErrorType.UNKNOWN]: '操作失败，请稍后重试'
}

/**
 * Suggested actions for each error type
 */
const SUGGESTED_ACTIONS = {
  [ErrorType.NETWORK]: '重试',
  [ErrorType.AUTH]: '检查设置',
  [ErrorType.QUOTA]: '检查账户',
  [ErrorType.VALIDATION]: '修改输入',
  [ErrorType.UNKNOWN]: '重试'
}

/**
 * Parse an error and return its type
 * @param {Error|string} error - The error to parse
 * @returns {string} The error type from ErrorType enum
 */
export function parseError(error) {
  const errorMessage = error instanceof Error
    ? error.message.toLowerCase()
    : String(error).toLowerCase()

  for (const [type, patterns] of Object.entries(ERROR_PATTERNS)) {
    for (const pattern of patterns) {
      if (errorMessage.includes(pattern.toLowerCase())) {
        return type
      }
    }
  }

  return ErrorType.UNKNOWN
}

/**
 * Get user-friendly error message for an error type
 * @param {string} errorType - The error type from ErrorType enum
 * @returns {string} User-friendly error message in Chinese
 */
export function getErrorMessage(errorType) {
  return ERROR_MESSAGES[errorType] || ERROR_MESSAGES[ErrorType.UNKNOWN]
}

/**
 * Get suggested action for an error type
 * @param {string} errorType - The error type from ErrorType enum
 * @returns {string} Suggested action in Chinese
 */
export function getSuggestedAction(errorType) {
  return SUGGESTED_ACTIONS[errorType] || SUGGESTED_ACTIONS[ErrorType.UNKNOWN]
}

/**
 * Create a structured error info object
 * @param {Error|string} error - The original error
 * @returns {{type: string, message: string, suggestion: string, originalError: string}}
 */
export function createErrorInfo(error) {
  const type = parseError(error)
  return {
    type,
    message: getErrorMessage(type),
    suggestion: getSuggestedAction(type),
    originalError: error instanceof Error ? error.message : String(error)
  }
}