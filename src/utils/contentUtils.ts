/**
 * Content extraction utilities for DailyLogger
 * Provides functions for parsing and summarizing AI analysis content
 */

interface ScreenAnalysis {
  current_focus?: string
  active_software?: string
  context_keywords?: string[]
  active_window?: { title?: string; process_name?: string }
  tags?: string[]
}

/**
 * Extract a short summary from AI screenshot analysis content.
 * @param content - Raw JSON content from an auto record
 * @param maxLength - Maximum summary length (default 80 chars)
 * @returns Human-readable summary string, or empty string if unavailable
 */
export function extractSummary(content: string, maxLength = 80): string {
  if (!content) return ''
  try {
    const parsed = JSON.parse(content) as ScreenAnalysis
    const text = parsed.current_focus || parsed.active_software || ''
    if (!text) return ''
    return text.length > maxLength ? text.slice(0, maxLength) + '…' : text
  } catch {
    return ''
  }
}
