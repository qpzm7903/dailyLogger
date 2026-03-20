import { describe, it, expect } from 'vitest'
import { extractSummary } from '../utils/contentUtils'

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
