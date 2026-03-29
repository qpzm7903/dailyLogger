import { config } from '@vue/test-utils'
import { createI18n } from 'vue-i18n'
import en from './locales/en.json'
import zhCN from './locales/zh-CN.json'

// Mock window for jsdom environment (vue-i18n requires it)
// This handles cases where jsdom might not be fully initialized or window is missing
const windowMock = {
  location: { href: '' },
  navigator: { language: 'en' },
  document: { documentElement: { lang: 'en' } },
  alert: () => {},
  confirm: () => true,
  prompt: () => null,
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

// Ensure window is available on all global object references for Node.js ESM compatibility
// Modules may access 'window' via 'globalThis', 'global', or direct 'window' reference
const setupWindow = () => {
  const windowRef = typeof window !== 'undefined' ? window : windowMock

  // Set on globalThis
  if (typeof globalThis !== 'undefined') {
    ;(globalThis as any).window = windowRef
    ;(globalThis as any).self = windowRef
  }

  // Set on global (Node.js ESM compatibility)
  if (typeof global !== 'undefined') {
    ;(global as any).window = windowRef
    ;(global as any).self = windowRef
  }

  // Also try direct assignment for maximum compatibility
  try {
    // @ts-ignore
    window = windowRef
  } catch (e) {
    // Ignore errors from read-only properties
  }
}

setupWindow()

// Ensure window.performance is available
if (typeof window !== 'undefined' && !window.performance) {
  window.performance = windowMock.performance
}

// Ensure window.matchMedia is available
if (typeof window !== 'undefined' && !window.matchMedia) {
  window.matchMedia = windowMock.matchMedia as any
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
