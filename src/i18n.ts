import { createI18n } from 'vue-i18n'
import en from './locales/en.json'
import zhCN from './locales/zh-CN.json'

// Type for locale
export type Locale = 'en' | 'zh-CN'

// Detect system language
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