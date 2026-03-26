// Theme management module
// Supports 'dark' and 'light' themes with localStorage persistence
// and system theme detection

export type Theme = 'dark' | 'light'

const STORAGE_KEY = 'dailylogger-theme'

/**
 * Get the stored theme preference, or detect from system if none stored
 */
export function getTheme(): Theme {
  const stored = localStorage.getItem(STORAGE_KEY)
  if (stored === 'dark' || stored === 'light') {
    return stored
  }
  return detectSystemTheme()
}

/**
 * Set the theme and apply it to the document
 */
export function setTheme(theme: Theme): void {
  localStorage.setItem(STORAGE_KEY, theme)
  document.documentElement.classList.remove('dark', 'light')
  document.documentElement.classList.add(theme)
}

/**
 * Detect system preferred color scheme
 */
export function detectSystemTheme(): Theme {
  if (typeof window === 'undefined' || !window.matchMedia) {
    return 'dark'
  }
  return window.matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark'
}

/**
 * Initialize theme on app startup
 * Call this once at app mount
 */
export function initTheme(): void {
  setTheme(getTheme())
}
