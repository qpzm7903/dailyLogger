import { createI18n } from 'vue-i18n'
import en from './locales/en.json'
import zhCN from './locales/zh-CN.json'
import { invoke } from '@tauri-apps/api/core'
import type { Settings } from './types/tauri'

// Type for locale
export type Locale = 'en' | 'zh-CN'

// Detect system language (fallback when no backend/localStorage)
function detectLanguage(): Locale {
  // Check localStorage first
  const stored = localStorage.getItem('dailylogger-locale')
  if (stored && (stored === 'en' || stored === 'zh-CN')) {
    return stored as Locale
  }

  // Fallback to browser language
  const browserLang = navigator.language || (navigator as { userLanguage?: string }).userLanguage
  if (browserLang && browserLang.startsWith('zh')) {
    return 'zh-CN'
  }
  return 'en'
}

// Load language from backend settings (async)
export async function loadLanguageFromBackend(): Promise<void> {
  try {
    const settings = await invoke<Settings>('get_settings')
    if (settings.language === 'en' || settings.language === 'zh-CN') {
      // Backend setting takes priority
      setLocale(settings.language as Locale)
    }
  } catch (e) {
    console.warn('Failed to load language from backend:', e)
    // Fallback to localStorage/browser detection if backend fails
    const fallback = detectLanguage()
    setLocale(fallback)
  }
}

const i18n = createI18n({
  legacy: false, // Use Composition API
  locale: detectLanguage(),
  fallbackLocale: 'en',
  messages: {
    'en': en,
    'zh-CN': zhCN
  }
})

export default i18n

// Export helper for changing language
export function setLocale(locale: Locale): void {
  if (locale === 'en' || locale === 'zh-CN') {
    i18n.global.locale.value = locale
    localStorage.setItem('dailylogger-locale', locale)
    document.documentElement.lang = locale
  }
}

// Export helper for getting current language
export function getLocale(): Locale {
  return i18n.global.locale.value as Locale
}