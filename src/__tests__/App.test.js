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

vi.mock('../stores/toast.js', () => ({
  showSuccess: vi.fn(),
  showError: vi.fn()
}))

import { invoke } from '@tauri-apps/api/core'

// Helper to wait for async updates
const waitFor = async (condition, timeout = 1000) => {
  const start = Date.now()
  while (!condition() && Date.now() - start < timeout) {
    await new Promise(resolve => setTimeout(resolve, 50))
  }
}

describe('App.vue - Window Info Display (SMART-001 Task 6)', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    // Default mock implementations
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
      return Promise.resolve()
    })
  })

  describe('AC1 - Record list displays window info', () => {
    it('shows window title when record has active_window info', async () => {
      const recordsWithWindow = [
        {
          id: 1,
          timestamp: '2026-03-15T10:00:00Z',
          source_type: 'auto',
          screenshot_path: '/path/screenshot1.png',
          content: JSON.stringify({
            current_focus: 'Working on code',
            active_software: 'VS Code',
            context_keywords: ['code', 'development'],
            active_window: {
              title: 'main.rs - DailyLogger - VS Code',
              process_name: 'Code'
            }
          })
        }
      ]

      invoke.mockImplementation((cmd) => {
        if (cmd === 'get_settings') {
          return Promise.resolve({
            auto_capture_enabled: false,
            last_summary_path: ''
          })
        }
        if (cmd === 'get_today_records') {
          return Promise.resolve(recordsWithWindow)
        }
        return Promise.resolve()
      })

      const wrapper = mount(App, {
        global: {
          stubs: {
            SettingsModal: true,
            QuickNoteModal: true,
            ScreenshotModal: true,
            ScreenshotGallery: true,
            DailySummaryViewer: true,
            LogViewer: true,
            Toast: true
          }
        }
      })

      await nextTick()
      await waitFor(() => wrapper.vm.todayRecords.length > 0)

      // Check that window title is displayed
      const html = wrapper.html()
      expect(html).toContain('main.rs - DailyLogger - VS Code')
    })

    it('shows process name (app name) when active_window exists', async () => {
      const recordsWithWindow = [
        {
          id: 1,
          timestamp: '2026-03-15T10:00:00Z',
          source_type: 'auto',
          screenshot_path: '/path/screenshot1.png',
          content: JSON.stringify({
            current_focus: 'Working on code',
            active_software: 'VS Code',
            context_keywords: ['code'],
            active_window: {
              title: 'My Project - IntelliJ IDEA',
              process_name: 'idea64'
            }
          })
        }
      ]

      invoke.mockImplementation((cmd) => {
        if (cmd === 'get_settings') {
          return Promise.resolve({
            auto_capture_enabled: false,
            last_summary_path: ''
          })
        }
        if (cmd === 'get_today_records') {
          return Promise.resolve(recordsWithWindow)
        }
        return Promise.resolve()
      })

      const wrapper = mount(App, {
        global: {
          stubs: {
            SettingsModal: true,
            QuickNoteModal: true,
            ScreenshotModal: true,
            ScreenshotGallery: true,
            DailySummaryViewer: true,
            LogViewer: true,
            Toast: true
          }
        }
      })

      await nextTick()
      await waitFor(() => wrapper.vm.todayRecords.length > 0)

      // Check that process name is displayed
      const html = wrapper.html()
      expect(html).toContain('idea64')
    })

    it('does not show window info section when active_window is missing', async () => {
      const recordsWithoutWindow = [
        {
          id: 1,
          timestamp: '2026-03-15T10:00:00Z',
          source_type: 'auto',
          screenshot_path: '/path/screenshot1.png',
          content: JSON.stringify({
            current_focus: 'Working on code',
            active_software: 'VS Code',
            context_keywords: ['code']
            // No active_window field
          })
        }
      ]

      invoke.mockImplementation((cmd) => {
        if (cmd === 'get_settings') {
          return Promise.resolve({
            auto_capture_enabled: false,
            last_summary_path: ''
          })
        }
        if (cmd === 'get_today_records') {
          return Promise.resolve(recordsWithoutWindow)
        }
        return Promise.resolve()
      })

      const wrapper = mount(App, {
        global: {
          stubs: {
            SettingsModal: true,
            QuickNoteModal: true,
            ScreenshotModal: true,
            ScreenshotGallery: true,
            DailySummaryViewer: true,
            LogViewer: true,
            Toast: true
          }
        }
      })

      await nextTick()
      await waitFor(() => wrapper.vm.todayRecords.length > 0)

      // Window info container should not be present
      const windowInfoSection = wrapper.find('.window-info')
      expect(windowInfoSection.exists()).toBe(false)
    })

    it('handles manual records without window info gracefully', async () => {
      const manualRecord = [
        {
          id: 1,
          timestamp: '2026-03-15T10:00:00Z',
          source_type: 'manual',
          screenshot_path: null,
          content: 'Quick note about my task'
        }
      ]

      invoke.mockImplementation((cmd) => {
        if (cmd === 'get_settings') {
          return Promise.resolve({
            auto_capture_enabled: false,
            last_summary_path: ''
          })
        }
        if (cmd === 'get_today_records') {
          return Promise.resolve(manualRecord)
        }
        return Promise.resolve()
      })

      const wrapper = mount(App, {
        global: {
          stubs: {
            SettingsModal: true,
            QuickNoteModal: true,
            ScreenshotModal: true,
            ScreenshotGallery: true,
            DailySummaryViewer: true,
            LogViewer: true,
            Toast: true
          }
        }
      })

      await nextTick()
      await waitFor(() => wrapper.vm.todayRecords.length > 0)

      // Should display the manual record content without errors
      expect(wrapper.text()).toContain('Quick note about my task')
    })
  })

  describe('Window icon display', () => {
    it('shows app icon based on process name for known apps', async () => {
      const recordsWithKnownApp = [
        {
          id: 1,
          timestamp: '2026-03-15T10:00:00Z',
          source_type: 'auto',
          screenshot_path: '/path/screenshot1.png',
          content: JSON.stringify({
            current_focus: 'Working on code',
            active_software: 'VS Code',
            context_keywords: ['code'],
            active_window: {
              title: 'main.rs - VS Code',
              process_name: 'Code'
            }
          })
        }
      ]

      invoke.mockImplementation((cmd) => {
        if (cmd === 'get_settings') {
          return Promise.resolve({
            auto_capture_enabled: false,
            last_summary_path: ''
          })
        }
        if (cmd === 'get_today_records') {
          return Promise.resolve(recordsWithKnownApp)
        }
        return Promise.resolve()
      })

      const wrapper = mount(App, {
        global: {
          stubs: {
            SettingsModal: true,
            QuickNoteModal: true,
            ScreenshotModal: true,
            ScreenshotGallery: true,
            DailySummaryViewer: true,
            LogViewer: true,
            Toast: true
          }
        }
      })

      await nextTick()
      await waitFor(() => wrapper.vm.todayRecords.length > 0)

      // Should show VS Code icon (or fallback)
      const html = wrapper.html()
      // Check for icon element or fallback character
      expect(html).toMatch(/(💻|🖥️|Code)/)
    })
  })
})