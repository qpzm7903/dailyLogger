// Theme management module
// PERF-006: Light theme support

export type Theme = 'dark' | 'light'

const STORAGE_KEY = 'dailylogger-theme'

/**
 * Get the stored theme preference, or detect system theme if none stored
 */
export function getTheme(): Theme {
  const stored = localStorage.getItem(STORAGE_KEY)
  if (stored === 'dark' || stored === 'light') {
    return stored
  }
  return detectSystemTheme()
}

/**
 * Set the theme and apply it to the document root
 */
export function setTheme(theme: Theme): void {
  localStorage.setItem(STORAGE_KEY, theme)
  document.documentElement.classList.remove('dark', 'light')
  document.documentElement.classList.add(theme)
}

/**
 * Detect system theme preference
 */
export function detectSystemTheme(): Theme {
  return window.matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark'
}

/**
 * Initialize theme on app startup
 */
export function initTheme(): void {
  setTheme(getTheme())
}

/**
 * Toggle between dark and light themes
 */
export function toggleTheme(): void {
  const current = getTheme()
  setTheme(current === 'dark' ? 'light' : 'dark')
}
