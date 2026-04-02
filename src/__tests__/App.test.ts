import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import App from '../App.vue'

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {}))
}))

vi.mock('@tauri-apps/plugin-global-shortcut', () => ({
  register: vi.fn(() => Promise.resolve()),
  unregister: vi.fn(() => Promise.resolve())
}))

vi.mock('../stores/toast.js', () => ({
  showSuccess: vi.fn(),
  showError: vi.fn(),
  initToastI18n: vi.fn()
}))

import { invoke } from '@tauri-apps/api/core'

// Helper to wait for async updates
const waitFor = async (condition, timeout = 1000) => {
  const start = Date.now()
  while (!condition() && Date.now() - start < timeout) {
    await new Promise(resolve => setTimeout(resolve, 50))
  }
}

// Common stubs for all tests
const commonStubs = {
  SettingsModal: true,
  QuickNoteModal: true,
  ScreenshotModal: true,
  ScreenshotGallery: true,
  DailySummaryViewer: true,
  LogViewer: true,
  Toast: true,
  Sidebar: true,
  Header: true,
  Dashboard: true,
  OfflineBanner: true
}

describe('App.vue - Layout Structure', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    invoke.mockImplementation((cmd) => {
      if (cmd === 'get_settings') {
        return Promise.resolve({
          auto_capture_enabled: false,
          last_summary_path: ''
        })
      }
      if (cmd === 'get_today_records') {
        return Promise.resolve([])
      }
      if (cmd === 'get_network_status' || cmd === 'check_network_status') {
        return Promise.resolve(true)
      }
      if (cmd === 'get_offline_queue_status') {
        return Promise.resolve({ pending_count: 0 })
      }
      return Promise.resolve()
    })
  })

  it('renders without errors', async () => {
    const wrapper = mount(App, {
      global: { stubs: commonStubs }
    })
    await nextTick()
    expect(wrapper.exists()).toBe(true)
  })

  it('has correct layout structure with Sidebar', async () => {
    const wrapper = mount(App, {
      global: { stubs: commonStubs }
    })
    await nextTick()
    // App should have flex layout
    expect(wrapper.find('.h-screen').exists()).toBe(true)
  })

  it('loads today records on mount', async () => {
    mount(App, {
      global: { stubs: commonStubs }
    })
    await nextTick()
    await new Promise(resolve => setTimeout(resolve, 100))
    expect(invoke).toHaveBeenCalledWith('get_today_records')
  })

  it('loads settings on mount', async () => {
    mount(App, {
      global: { stubs: commonStubs }
    })
    await nextTick()
    await new Promise(resolve => setTimeout(resolve, 100))
    expect(invoke).toHaveBeenCalledWith('get_settings')
  })
})

describe('App.vue - State Management', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    invoke.mockImplementation((cmd) => {
      if (cmd === 'get_settings') {
        return Promise.resolve({
          auto_capture_enabled: false,
          last_summary_path: ''
        })
      }
      if (cmd === 'get_today_records') {
        return Promise.resolve([])
      }
      if (cmd === 'get_network_status' || cmd === 'check_network_status') {
        return Promise.resolve(true)
      }
      if (cmd === 'get_offline_queue_status') {
        return Promise.resolve({ pending_count: 0 })
      }
      return Promise.resolve()
    })
  })

  it('maintains todayRecords state', async () => {
    const mockRecords = [
      { id: 1, timestamp: '2026-03-15T10:00:00Z', source_type: 'auto', content: '{}' }
    ]
    invoke.mockImplementation((cmd) => {
      if (cmd === 'get_today_records') {
        return Promise.resolve(mockRecords)
      }
      if (cmd === 'get_settings') {
        return Promise.resolve({ auto_capture_enabled: false, last_summary_path: '' })
      }
      if (cmd === 'get_network_status' || cmd === 'check_network_status') {
        return Promise.resolve(true)
      }
      if (cmd === 'get_offline_queue_status') {
        return Promise.resolve({ pending_count: 0 })
      }
      return Promise.resolve()
    })

    const wrapper = mount(App, {
      global: { stubs: commonStubs }
    })

    await nextTick()
    await waitFor(() => wrapper.vm.todayRecords.length > 0)

    expect(wrapper.vm.todayRecords).toEqual(mockRecords)
  })

  it('maintains autoCaptureEnabled state from settings', async () => {
    invoke.mockImplementation((cmd) => {
      if (cmd === 'get_settings') {
        return Promise.resolve({
          auto_capture_enabled: true,
          last_summary_path: '/path/to/summary.md'
        })
      }
      if (cmd === 'get_today_records') {
        return Promise.resolve([])
      }
      if (cmd === 'get_network_status' || cmd === 'check_network_status') {
        return Promise.resolve(true)
      }
      if (cmd === 'get_offline_queue_status') {
        return Promise.resolve({ pending_count: 0 })
      }
      return Promise.resolve()
    })

    const wrapper = mount(App, {
      global: { stubs: commonStubs }
    })

    await nextTick()
    await waitFor(() => wrapper.vm.autoCaptureEnabled === true)

    expect(wrapper.vm.autoCaptureEnabled).toBe(true)
    expect(wrapper.vm.summaryPath).toBe('/path/to/summary.md')
  })

  it('resumes auto capture on startup when enabled in settings', async () => {
    invoke.mockImplementation((cmd) => {
      if (cmd === 'get_settings') {
        return Promise.resolve({
          auto_capture_enabled: true,
          last_summary_path: ''
        })
      }
      if (cmd === 'start_auto_capture') {
        return Promise.resolve()
      }
      if (cmd === 'get_today_records') {
        return Promise.resolve([])
      }
      if (cmd === 'get_network_status' || cmd === 'check_network_status') {
        return Promise.resolve(true)
      }
      if (cmd === 'get_offline_queue_status') {
        return Promise.resolve({ pending_count: 0 })
      }
      return Promise.resolve()
    })

    mount(App, {
      global: { stubs: commonStubs }
    })

    await waitFor(() =>
      invoke.mock.calls.some(([cmd]) => cmd === 'start_auto_capture')
    )

    expect(invoke).toHaveBeenCalledWith('start_auto_capture')
  })
})

describe('App.vue - Modal Management', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    invoke.mockImplementation((cmd) => {
      if (cmd === 'get_settings') {
        return Promise.resolve({
          auto_capture_enabled: false,
          last_summary_path: ''
        })
      }
      if (cmd === 'get_today_records') {
        return Promise.resolve([])
      }
      if (cmd === 'get_network_status' || cmd === 'check_network_status') {
        return Promise.resolve(true)
      }
      if (cmd === 'get_offline_queue_status') {
        return Promise.resolve({ pending_count: 0 })
      }
      return Promise.resolve()
    })
  })

  it('has useModal composable for modal state', async () => {
    const wrapper = mount(App, {
      global: { stubs: commonStubs }
    })
    await nextTick()

    // Should have open and close methods from useModal
    expect(typeof wrapper.vm.open).toBe('function')
    expect(typeof wrapper.vm.close).toBe('function')
    expect(typeof wrapper.vm.isOpen).toBe('function')
  })
})

describe('App.vue - Screenshot Count', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    invoke.mockImplementation((cmd) => {
      if (cmd === 'get_settings') {
        return Promise.resolve({
          auto_capture_enabled: false,
          last_summary_path: ''
        })
      }
      if (cmd === 'get_today_records') {
        return Promise.resolve([])
      }
      if (cmd === 'get_network_status' || cmd === 'check_network_status') {
        return Promise.resolve(true)
      }
      if (cmd === 'get_offline_queue_status') {
        return Promise.resolve({ pending_count: 0 })
      }
      return Promise.resolve()
    })
  })

  it('calculates screenshotCount from todayRecords', async () => {
    const mockRecords = [
      { id: 1, timestamp: '2026-03-15T10:00:00Z', source_type: 'auto', screenshot_path: '/path/1.png', content: '{}' },
      { id: 2, timestamp: '2026-03-15T11:00:00Z', source_type: 'manual', content: 'note' },
      { id: 3, timestamp: '2026-03-15T12:00:00Z', source_type: 'auto', screenshot_path: '/path/2.png', content: '{}' },
      { id: 4, timestamp: '2026-03-15T13:00:00Z', source_type: 'auto', screenshot_path: null, content: '{}' }
    ]
    invoke.mockImplementation((cmd) => {
      if (cmd === 'get_today_records') {
        return Promise.resolve(mockRecords)
      }
      if (cmd === 'get_settings') {
        return Promise.resolve({ auto_capture_enabled: false, last_summary_path: '' })
      }
      if (cmd === 'get_network_status' || cmd === 'check_network_status') {
        return Promise.resolve(true)
      }
      if (cmd === 'get_offline_queue_status') {
        return Promise.resolve({ pending_count: 0 })
      }
      return Promise.resolve()
    })

    const wrapper = mount(App, {
      global: { stubs: commonStubs }
    })

    await nextTick()
    await waitFor(() => wrapper.vm.todayRecords.length > 0)

    // screenshotCount = auto records with screenshot_path
    expect(wrapper.vm.screenshotCount).toBe(2)
  })
})
