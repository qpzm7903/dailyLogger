import { describe, it, expect } from 'vitest'
import { parseError, getErrorMessage, getSuggestedAction, ErrorType } from '../utils/errors.js'

describe('parseError', () => {
  describe('NETWORK errors', () => {
    it('should identify network error from "network" keyword', () => {
      expect(parseError(new Error('network connection failed'))).toBe(ErrorType.NETWORK)
    })

    it('should identify network error from "timeout" keyword', () => {
      expect(parseError(new Error('request timeout after 30s'))).toBe(ErrorType.NETWORK)
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

    it('should identify auth error from "authentication failed"', () => {
      expect(parseError(new Error('authentication failed'))).toBe(ErrorType.AUTH)
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
      expect(parseError(new Error('Request Timeout'))).toBe(ErrorType.NETWORK)
    })
  })
})

describe('getErrorMessage', () => {
  it('should return Chinese message for NETWORK error', () => {
    expect(getErrorMessage(ErrorType.NETWORK)).toBe('网络连接失败，请检查网络设置')
  })

  it('should return Chinese message for AUTH error', () => {
    expect(getErrorMessage(ErrorType.AUTH)).toBe('API Key 无效或已过期')
  })

  it('should return Chinese message for QUOTA error', () => {
    expect(getErrorMessage(ErrorType.QUOTA)).toBe('API 调用次数已达上限')
  })

  it('should return Chinese message for VALIDATION error', () => {
    expect(getErrorMessage(ErrorType.VALIDATION)).toBe('输入内容格式不正确')
  })

  it('should return Chinese message for UNKNOWN error', () => {
    expect(getErrorMessage(ErrorType.UNKNOWN)).toBe('操作失败，请稍后重试')
  })
})

describe('getSuggestedAction', () => {
  it('should suggest retry for NETWORK error', () => {
    expect(getSuggestedAction(ErrorType.NETWORK)).toBe('重试')
  })

  it('should suggest checking settings for AUTH error', () => {
    expect(getSuggestedAction(ErrorType.AUTH)).toBe('检查设置')
  })

  it('should suggest checking account for QUOTA error', () => {
    expect(getSuggestedAction(ErrorType.QUOTA)).toBe('检查账户')
  })

  it('should suggest modifying input for VALIDATION error', () => {
    expect(getSuggestedAction(ErrorType.VALIDATION)).toBe('修改输入')
  })

  it('should suggest retry for UNKNOWN error', () => {
    expect(getSuggestedAction(ErrorType.UNKNOWN)).toBe('重试')
  })
})