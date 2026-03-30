import { describe, it, expect } from 'vitest'
import { extractSummary, sanitizeSnippet } from '../utils/contentUtils'

describe('extractSummary', () => {
  it('returns empty string for empty content', () => {
    expect(extractSummary('')).toBe('')
  })

  it('extracts current_focus from valid JSON', () => {
    const content = JSON.stringify({
      current_focus: 'Writing code in VS Code',
      active_software: 'Code',
      tags: ['开发']
    })
    expect(extractSummary(content)).toBe('Writing code in VS Code')
  })

  it('falls back to active_software when current_focus is absent', () => {
    const content = JSON.stringify({
      active_software: 'Chrome Browser',
      tags: ['研究']
    })
    expect(extractSummary(content)).toBe('Chrome Browser')
  })

  it('returns empty string when both current_focus and active_software are absent', () => {
    const content = JSON.stringify({ tags: ['会议'] })
    expect(extractSummary(content)).toBe('')
  })

  it('truncates text longer than maxLength and appends ellipsis', () => {
    const longText = 'A'.repeat(100)
    const content = JSON.stringify({ current_focus: longText })
    const result = extractSummary(content, 80)
    expect(result).toHaveLength(81) // 80 chars + '…'
    expect(result.endsWith('…')).toBe(true)
  })

  it('does not truncate text shorter than or equal to maxLength', () => {
    const text = 'Short text'
    const content = JSON.stringify({ current_focus: text })
    expect(extractSummary(content)).toBe('Short text')
  })

  it('returns empty string when JSON parsing fails', () => {
    expect(extractSummary('not valid json')).toBe('')
    expect(extractSummary('{broken')).toBe('')
  })

  it('respects custom maxLength parameter', () => {
    const text = 'Hello World'
    const content = JSON.stringify({ current_focus: text })
    const result = extractSummary(content, 5)
    expect(result).toBe('Hello…')
  })
})

describe('sanitizeSnippet', () => {
  it('returns empty string for empty input', () => {
    expect(sanitizeSnippet('')).toBe('')
  })

  it('preserves <mark> tags from FTS5 highlighting', () => {
    const input = 'Found <mark>hello</mark> in the text'
    expect(sanitizeSnippet(input)).toBe('Found <mark>hello</mark> in the text')
  })

  it('escapes non-mark HTML tags to prevent XSS', () => {
    const input = '<script>alert("xss")</script><mark>safe</mark>'
    const result = sanitizeSnippet(input)
    expect(result).toContain('<mark>safe</mark>')
    expect(result).not.toContain('<script>')
    expect(result).toContain('&lt;script&gt;')
  })

  it('handles plain text without any tags', () => {
    const input = 'Just plain text'
    expect(sanitizeSnippet(input)).toBe('Just plain text')
  })

  it('escapes img tags', () => {
    const input = '<img src=x onerror=alert(1)><mark>text</mark>'
    const result = sanitizeSnippet(input)
    expect(result).toContain('<mark>text</mark>')
    expect(result).not.toContain('<img')
  })

  it('handles multiple mark tags', () => {
    const input = '<mark>hello</mark> and <mark>world</mark>'
    expect(sanitizeSnippet(input)).toBe('<mark>hello</mark> and <mark>world</mark>')
  })
})
