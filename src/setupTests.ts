import { config } from '@vue/test-utils'
import { createI18n } from 'vue-i18n'
import en from './locales/en.json'
import zhCN from './locales/zh-CN.json'

// Mock window for jsdom environment (vue-i18n requires it)
// Use Object.defineProperty to ensure window is properly set on globalThis
// This handles cases where jsdom might not be fully initialized or window is missing
const windowMock = {
  location: { href: '' },
  navigator: { language: 'en' },
  document: { documentElement: { lang: 'en' } },
  performance: {
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
    eventCounts: { getEntriesByType: () => [], getEntriesByName: () => [], size: 0, clear: () => {}, toArray: () => [] },
    navigation: { type: 0, redirectCount: 0 } as PerformanceNavigation,
    timing: { navigationStart: 0, unloadEventStart: 0, unloadEventEnd: 0, redirectStart: 0, redirectEnd: 0, fetchStart: 0, domainLookupStart: 0, domainLookupEnd: 0, connectStart: 0, connectEnd: 0, secureConnectionStart: 0, requestStart: 0, responseStart: 0, responseEnd: 0, domLoading: 0, domInteractive: 0, domContentLoadedEventStart: 0, domContentLoadedEventEnd: 0, domComplete: 0, loadEventStart: 0, loadEventEnd: 0 } as PerformanceTiming,
    clearResourceTimings: () => {},
    getEntries: () => []
  } as unknown as Performance,
  matchMedia: (query: string) => ({
    matches: query === '(prefers-color-scheme: light)',
    media: query,
    onchange: null,
    addListener: () => {},
    removeListener: () => {},
    addEventListener: () => {},
    removeEventListener: () => {},
    dispatchEvent: () => true
  })
}

// Ensure window is available on globalThis
if (typeof globalThis.window === 'undefined') {
  Object.defineProperty(globalThis, 'window', {
    value: windowMock,
    writable: true,
    configurable: true
  })
}

// Ensure window.performance is available
if (typeof window !== 'undefined' && !window.performance) {
  window.performance = windowMock.performance
}

// Ensure window.matchMedia is available
if (typeof window !== 'undefined' && !window.matchMedia) {
  Object.defineProperty(window, 'matchMedia', {
    value: windowMock.matchMedia,
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
