import { config } from '@vue/test-utils'
import { createI18n } from 'vue-i18n'
import en from './locales/en.json'
import zhCN from './locales/zh-CN.json'

// Mock window for jsdom environment (vue-i18n requires it)
if (typeof globalThis.window === 'undefined') {
  Object.defineProperty(globalThis, 'window', {
    value: {
      location: { href: '' },
      navigator: { language: 'en' },
      document: { documentElement: { lang: 'en' } }
    },
    writable: true,
    configurable: true
  })
}

// Create i18n instance for tests
const i18n = createI18n({
  legacy: false,
  locale: 'en',
  fallbackLocale: 'en',
  messages: {
    'en': en,
    'zh-CN': zhCN
  }
})

// Register i18n globally for all tests
config.global.plugins = [i18n]