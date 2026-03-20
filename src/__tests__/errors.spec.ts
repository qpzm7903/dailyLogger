import { describe, it, expect } from 'vitest'
import { parseError, getErrorMessageKey, getSuggestedActionKey, createErrorInfo, ErrorType } from '../utils/errors.js'

describe('parseError', () => {
  describe('TIMEOUT errors', () => {
    it('should identify timeout error from "timeout" keyword', () => {
      expect(parseError(new Error('request timeout after 30s'))).toBe(ErrorType.TIMEOUT)
    })

    it('should identify timeout error from "timed out"', () => {
      expect(parseError(new Error('operation timed out'))).toBe(ErrorType.TIMEOUT)
    })

    it('should identify timeout error from Chinese "超时"', () => {
      expect(parseError(new Error('请求超时'))).toBe(ErrorType.TIMEOUT)
    })
  })

  describe('SCREENSHOT errors', () => {
    it('should identify screenshot error from "screenshot" keyword', () => {
      expect(parseError(new Error('screenshot capture failed'))).toBe(ErrorType.SCREENSHOT)
    })

    it('should identify screenshot error from "capture"', () => {
      expect(parseError(new Error('screen capture permission denied'))).toBe(ErrorType.SCREENSHOT)
    })

    it('should identify screenshot error from Chinese "截图"', () => {
      expect(parseError(new Error('截图失败'))).toBe(ErrorType.SCREENSHOT)
    })
  })

  describe('FILE_IO errors', () => {
    it('should identify file error from "file not found"', () => {
      expect(parseError(new Error('file not found: config.json'))).toBe(ErrorType.FILE_IO)
    })

    it('should identify file error from "ENOENT"', () => {
      expect(parseError(new Error('Error: ENOENT: no such file'))).toBe(ErrorType.FILE_IO)
    })

    it('should identify file error from "failed to write"', () => {
      expect(parseError(new Error('failed to write file'))).toBe(ErrorType.FILE_IO)
    })

    it('should identify file error from "failed to read"', () => {
      expect(parseError(new Error('failed to read configuration'))).toBe(ErrorType.FILE_IO)
    })
  })

  describe('DATABASE errors', () => {
    it('should identify database error from "database" keyword', () => {
      expect(parseError(new Error('database connection failed'))).toBe(ErrorType.DATABASE)
    })

    it('should identify database error from "sqlite"', () => {
      expect(parseError(new Error('sqlite error: constraint violation'))).toBe(ErrorType.DATABASE)
    })

    it('should identify database error from "db locked"', () => {
      expect(parseError(new Error('database is locked'))).toBe(ErrorType.DATABASE)
    })

    it('should identify database error from Chinese "数据库"', () => {
      expect(parseError(new Error('数据库操作失败'))).toBe(ErrorType.DATABASE)
    })
  })

  describe('NETWORK errors', () => {
    it('should identify network error from "network" keyword', () => {
      expect(parseError(new Error('network connection failed'))).toBe(ErrorType.NETWORK)
    })

    it('should identify network error from "ECONNREFUSED"', () => {
      expect(parseError(new Error('Error: ECONNREFUSED 127.0.0.1:80'))).toBe(ErrorType.NETWORK)
    })

    it('should identify network error from "ENOTFOUND"', () => {
      expect(parseError(new Error('getaddrinfo ENOTFOUND api.example.com'))).toBe(ErrorType.NETWORK)
    })

    it('should identify network error from "fetch failed"', () => {
      expect(parseError(new Error('TypeError: fetch failed'))).toBe(ErrorType.NETWORK)
    })
  })

  describe('AUTH errors', () => {
    it('should identify auth error from "401" status code', () => {
      expect(parseError(new Error('HTTP 401 Unauthorized'))).toBe(ErrorType.AUTH)
    })

    it('should identify auth error from "403" status code', () => {
      expect(parseError(new Error('Request failed with status 403'))).toBe(ErrorType.AUTH)
    })

    it('should identify auth error from "unauthorized" keyword', () => {
      expect(parseError(new Error('User is unauthorized'))).toBe(ErrorType.AUTH)
    })

    it('should identify auth error from "api key" keyword', () => {
      expect(parseError(new Error('Invalid api key provided'))).toBe(ErrorType.AUTH)
    })
  })

  describe('QUOTA errors', () => {
    it('should identify quota error from "429" status code', () => {
      expect(parseError(new Error('HTTP 429 Too Many Requests'))).toBe(ErrorType.QUOTA)
    })

    it('should identify quota error from "rate limit"', () => {
      expect(parseError(new Error('Rate limit exceeded'))).toBe(ErrorType.QUOTA)
    })

    it('should identify quota error from "quota" keyword', () => {
      expect(parseError(new Error('API quota exceeded for this month'))).toBe(ErrorType.QUOTA)
    })
  })

  describe('VALIDATION errors', () => {
    it('should identify validation error from "invalid" keyword', () => {
      expect(parseError(new Error('Invalid URL format'))).toBe(ErrorType.VALIDATION)
    })

    it('should identify validation error from "validation" keyword', () => {
      expect(parseError(new Error('Validation failed for input'))).toBe(ErrorType.VALIDATION)
    })

    it('should identify validation error from "format" keyword', () => {
      expect(parseError(new Error('Incorrect format for date field'))).toBe(ErrorType.VALIDATION)
    })
  })

  describe('UNKNOWN errors', () => {
    it('should return UNKNOWN for unrecognized errors', () => {
      expect(parseError(new Error('Something went wrong'))).toBe(ErrorType.UNKNOWN)
    })

    it('should return UNKNOWN for empty error', () => {
      expect(parseError(new Error(''))).toBe(ErrorType.UNKNOWN)
    })

    it('should handle string input', () => {
      expect(parseError('network error')).toBe(ErrorType.NETWORK)
    })
  })

  describe('case insensitivity', () => {
    it('should handle uppercase NETWORK', () => {
      expect(parseError(new Error('NETWORK ERROR'))).toBe(ErrorType.NETWORK)
    })

    it('should handle mixed case Timeout', () => {
      expect(parseError(new Error('Request Timeout'))).toBe(ErrorType.TIMEOUT)
    })
  })
})

describe('getErrorMessageKey', () => {
  it('should return correct i18n key for NETWORK error', () => {
    expect(getErrorMessageKey(ErrorType.NETWORK)).toBe('errors.messages.network')
  })

  it('should return correct i18n key for AUTH error', () => {
    expect(getErrorMessageKey(ErrorType.AUTH)).toBe('errors.messages.auth')
  })

  it('should return correct i18n key for TIMEOUT error', () => {
    expect(getErrorMessageKey(ErrorType.TIMEOUT)).toBe('errors.messages.timeout')
  })

  it('should return correct i18n key for DATABASE error', () => {
    expect(getErrorMessageKey(ErrorType.DATABASE)).toBe('errors.messages.database')
  })

  it('should return correct i18n key for SCREENSHOT error', () => {
    expect(getErrorMessageKey(ErrorType.SCREENSHOT)).toBe('errors.messages.screenshot')
  })

  it('should return correct i18n key for UNKNOWN error', () => {
    expect(getErrorMessageKey(ErrorType.UNKNOWN)).toBe('errors.messages.unknown')
  })
})

describe('getSuggestedActionKey', () => {
  it('should return correct i18n key for NETWORK suggestion', () => {
    expect(getSuggestedActionKey(ErrorType.NETWORK)).toBe('errors.suggestions.network')
  })

  it('should return correct i18n key for AUTH suggestion', () => {
    expect(getSuggestedActionKey(ErrorType.AUTH)).toBe('errors.suggestions.auth')
  })

  it('should return correct i18n key for TIMEOUT suggestion', () => {
    expect(getSuggestedActionKey(ErrorType.TIMEOUT)).toBe('errors.suggestions.timeout')
  })

  it('should return correct i18n key for UNKNOWN suggestion', () => {
    expect(getSuggestedActionKey(ErrorType.UNKNOWN)).toBe('errors.suggestions.unknown')
  })
})

describe('createErrorInfo', () => {
  it('should create error info with correct type and keys', () => {
    const errorInfo = createErrorInfo(new Error('network connection failed'))
    expect(errorInfo.type).toBe(ErrorType.NETWORK)
    expect(errorInfo.messageKey).toBe('errors.messages.network')
    expect(errorInfo.suggestionKey).toBe('errors.suggestions.network')
    expect(errorInfo.originalError).toBe('network connection failed')
  })

  it('should handle string errors', () => {
    const errorInfo = createErrorInfo('timeout occurred')
    expect(errorInfo.type).toBe(ErrorType.TIMEOUT)
    expect(errorInfo.messageKey).toBe('errors.messages.timeout')
  })

  it('should handle unknown errors', () => {
    const errorInfo = createErrorInfo(new Error('something unknown'))
    expect(errorInfo.type).toBe(ErrorType.UNKNOWN)
    expect(errorInfo.messageKey).toBe('errors.messages.unknown')
  })
})