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

// Polyfill window.performance for @intlify/core-base in jsdom environment
// Without this, intlify's message resolution crashes with "window is not defined"
// because it accesses window.performance.now() during non-production builds
if (typeof window !== 'undefined' && !window.performance) {
  window.performance = {
    now: () => Date.now(),
    mark: () => {},
    measure: () => {},
    getEntriesByType: () => [],
    getEntriesByName: () => [],
    clearMarks: () => {},
    clearMeasures: () => {},
    setResourceTimingBufferSize: () => {},
    onresourcetimingbufferfull: null,
    timeOrigin: 0,
    eventCounts: { getEntriesByType: () => [], getEntriesByName: () => [], size: 0, clear: () => {}, toArray: () => [] }
  } as unknown as Performance
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
