/**
 * Shared date formatting utilities
 *
 * Centralizes all date/time formatting to ensure locale consistency
 * and eliminate duplicated format functions across components.
 *
 * Reads locale from document.documentElement.lang (set by the i18n system)
 * to avoid importing from i18n.ts which would trigger createI18n at module load time.
 */

/** Get the JavaScript locale string for the current app locale */
function jsLocale(): string {
  const lang = document.documentElement.lang
    || (typeof localStorage !== 'undefined' && localStorage.getItem('dailylogger-locale'))
    || (typeof navigator !== 'undefined' && navigator.language)
    || 'en'
  return lang.startsWith('zh') ? 'zh-CN' : 'en-US'
}

/**
 * Format a timestamp to time only (HH:MM, 24-hour)
 *
 * Usage: session lists, dashboard time display
 */
export function formatTimeHM(timestamp: string): string {
  const date = new Date(timestamp)
  if (isNaN(date.getTime())) return '--:--'
  return date.toLocaleTimeString(jsLocale(), {
    hour: '2-digit',
    minute: '2-digit',
    hour12: false,
  })
}

/**
 * Format a timestamp to time with seconds (HH:MM:SS, 24-hour)
 *
 * Usage: screenshot gallery time display
 */
export function formatTimeHMS(timestamp: string): string {
  const date = new Date(timestamp)
  const h = String(date.getHours()).padStart(2, '0')
  const m = String(date.getMinutes()).padStart(2, '0')
  const s = String(date.getSeconds()).padStart(2, '0')
  return `${h}:${m}:${s}`
}

/**
 * Format a timestamp to date + time (YYYY/MM/DD HH:MM)
 *
 * Usage: search results, history viewer, offline queue
 */
export function formatDateTime(timestamp: string): string {
  const date = new Date(timestamp)
  return date.toLocaleString(jsLocale(), {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}

/**
 * Format a timestamp to full locale string (delegates to browser locale formatting)
 *
 * Usage: backup dates, screenshot modals
 */
export function formatFull(timestamp: string): string {
  const date = new Date(timestamp)
  return date.toLocaleString(jsLocale())
}

/**
 * Format a Date to ISO date string (YYYY-MM-DD)
 *
 * Usage: date pickers, export templates
 */
export function formatDateISO(date: Date): string {
  const y = date.getFullYear()
  const m = String(date.getMonth() + 1).padStart(2, '0')
  const d = String(date.getDate()).padStart(2, '0')
  return `${y}-${m}-${d}`
}

/**
 * Format current time for display widgets (MM/DD HH:MM)
 *
 * Usage: clock widgets, quick note timestamps
 */
export function formatCurrentTime(): string {
  return new Date().toLocaleString(jsLocale(), {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}

/**
 * Format current time with seconds for display widgets (MM/DD HH:MM:SS)
 *
 * Usage: quick note window timestamp
 */
export function formatCurrentTimeFull(): string {
  return new Date().toLocaleString(jsLocale(), {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  })
}

/**
 * Format a date range for display (MM-DD ~ MM-DD)
 *
 * Usage: statistics panel period display
 */
export function formatDateRange(period: { start: string; end: string }): string {
  return `${period.start.slice(5, 10)} ~ ${period.end.slice(5, 10)}`
}
