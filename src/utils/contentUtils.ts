/**
 * Content extraction utilities for DailyLogger
 * Provides functions for parsing and summarizing AI analysis content
 */

import type { ScreenAnalysis } from '../features/capture/actions'

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

/**
 * Sanitize a search snippet for safe v-html rendering.
 *
 * FTS5 snippets contain `<mark>...</mark>` for highlighting matched terms.
 * All other HTML is escaped to prevent XSS. This function:
 * 1. Replaces `<mark>`/`</mark>` with placeholders
 * 2. HTML-escapes the remaining text
 * 3. Restores the `<mark>` tags
 */
export function sanitizeSnippet(snippet: string): string {
  if (!snippet) return ''
  const OPEN_PLACEHOLDER = '\x00MARK_OPEN\x00'
  const CLOSE_PLACEHOLDER = '\x00MARK_CLOSE\x00'
  const div = document.createElement('div')

  // Preserve <mark> tags, escape everything else
  let text = snippet
  text = text.replace(/<mark>/gi, OPEN_PLACEHOLDER)
  text = text.replace(/<\/mark>/gi, CLOSE_PLACEHOLDER)

  // Use DOM textContent for proper HTML escaping
  div.textContent = text
  const escaped = div.innerHTML

  // Restore <mark> tags
  return escaped
    .replaceAll(OPEN_PLACEHOLDER, '<mark>')
    .replaceAll(CLOSE_PLACEHOLDER, '</mark>')
}
