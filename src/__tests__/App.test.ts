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

describe('App.vue - AI-004 Tag functionality', () => {
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
      return Promise.resolve()
    })
  })

  describe('AC1 - Record displays tag badges', () => {
    it('shows tag badges when record has tags field', async () => {
      const recordsWithTags = [
        {
          id: 1,
          timestamp: '2026-03-15T10:00:00Z',
          source_type: 'auto',
          screenshot_path: '/path/screenshot1.png',
          content: JSON.stringify({
            current_focus: 'Working on code',
            active_software: 'VS Code',
            context_keywords: ['code']
          }),
          tags: JSON.stringify(['开发', '测试'])
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
          return Promise.resolve(recordsWithTags)
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

      const html = wrapper.html()
      expect(html).toContain('开发')
      expect(html).toContain('测试')
    })

    it('shows tags from content field for auto records without tags field', async () => {
      const recordsWithContentTags = [
        {
          id: 1,
          timestamp: '2026-03-15T10:00:00Z',
          source_type: 'auto',
          screenshot_path: '/path/screenshot1.png',
          content: JSON.stringify({
            current_focus: 'In a meeting',
            active_software: 'Zoom',
            context_keywords: ['meeting'],
            tags: ['会议', '沟通']
          }),
          tags: null
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
          return Promise.resolve(recordsWithContentTags)
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

      const html = wrapper.html()
      expect(html).toContain('会议')
      expect(html).toContain('沟通')
    })

    it('limits display to 3 tags per record', async () => {
      const recordsWithManyTags = [
        {
          id: 1,
          timestamp: '2026-03-15T10:00:00Z',
          source_type: 'auto',
          screenshot_path: '/path/screenshot1.png',
          content: JSON.stringify({ current_focus: 'Work' }),
          tags: JSON.stringify(['开发', '测试', '文档', '会议', '设计']) // 5 tags
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
          return Promise.resolve(recordsWithManyTags)
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

      const tags = wrapper.vm.getRecordTags(wrapper.vm.todayRecords[0])
      expect(tags.length).toBeLessThanOrEqual(3)
    })

    it('does not show tag section when record has no tags', async () => {
      const recordsWithoutTags = [
        {
          id: 1,
          timestamp: '2026-03-15T10:00:00Z',
          source_type: 'manual',
          screenshot_path: null,
          content: 'Quick note',
          tags: null
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
          return Promise.resolve(recordsWithoutTags)
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

      const tags = wrapper.vm.getRecordTags(wrapper.vm.todayRecords[0])
      expect(tags).toEqual([])
    })
  })

  describe('AC3 - Tag filtering functionality', () => {
    it('shows tag filter when records have tags', async () => {
      const recordsWithTags = [
        {
          id: 1,
          timestamp: '2026-03-15T10:00:00Z',
          source_type: 'auto',
          content: JSON.stringify({ current_focus: 'Work' }),
          tags: JSON.stringify(['开发'])
        },
        {
          id: 2,
          timestamp: '2026-03-15T11:00:00Z',
          source_type: 'auto',
          content: JSON.stringify({ current_focus: 'Meeting' }),
          tags: JSON.stringify(['会议'])
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
          return Promise.resolve(recordsWithTags)
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

      // Should show "全部" button
      const html = wrapper.html()
      expect(html).toContain('全部')
    })

    it('filters records by selected tag', async () => {
      const recordsWithTags = [
        {
          id: 1,
          timestamp: '2026-03-15T10:00:00Z',
          source_type: 'auto',
          content: JSON.stringify({ current_focus: 'Dev work' }),
          tags: JSON.stringify(['开发'])
        },
        {
          id: 2,
          timestamp: '2026-03-15T11:00:00Z',
          source_type: 'auto',
          content: JSON.stringify({ current_focus: 'Meeting' }),
          tags: JSON.stringify(['会议'])
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
          return Promise.resolve(recordsWithTags)
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

      // Initially all records should be visible
      expect(wrapper.vm.filteredRecords.length).toBe(2)

      // Select a tag to filter
      wrapper.vm.selectedTagFilter = '开发'
      await nextTick()

      // Only records with '开发' tag should be visible
      expect(wrapper.vm.filteredRecords.length).toBe(1)
      expect(wrapper.vm.filteredRecords[0].id).toBe(1)
    })

    it('shows correct tag counts', async () => {
      const recordsWithTags = [
        {
          id: 1,
          timestamp: '2026-03-15T10:00:00Z',
          source_type: 'auto',
          content: JSON.stringify({ current_focus: 'Dev work' }),
          tags: JSON.stringify(['开发'])
        },
        {
          id: 2,
          timestamp: '2026-03-15T11:00:00Z',
          source_type: 'auto',
          content: JSON.stringify({ current_focus: 'Dev work 2' }),
          tags: JSON.stringify(['开发', '测试'])
        },
        {
          id: 3,
          timestamp: '2026-03-15T12:00:00Z',
          source_type: 'auto',
          content: JSON.stringify({ current_focus: 'Meeting' }),
          tags: JSON.stringify(['会议'])
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
          return Promise.resolve(recordsWithTags)
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

      const counts = wrapper.vm.tagCounts
      expect(counts['开发']).toBe(2)
      expect(counts['测试']).toBe(1)
      expect(counts['会议']).toBe(1)
    })

    it('clears filter when selecting "全部"', async () => {
      const recordsWithTags = [
        {
          id: 1,
          timestamp: '2026-03-15T10:00:00Z',
          source_type: 'auto',
          content: JSON.stringify({ current_focus: 'Dev work' }),
          tags: JSON.stringify(['开发'])
        },
        {
          id: 2,
          timestamp: '2026-03-15T11:00:00Z',
          source_type: 'auto',
          content: JSON.stringify({ current_focus: 'Meeting' }),
          tags: JSON.stringify(['会议'])
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
          return Promise.resolve(recordsWithTags)
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

      // Apply filter
      wrapper.vm.selectedTagFilter = '开发'
      await nextTick()
      expect(wrapper.vm.filteredRecords.length).toBe(1)

      // Clear filter by setting to empty string
      wrapper.vm.selectedTagFilter = ''
      await nextTick()
      expect(wrapper.vm.filteredRecords.length).toBe(2)
    })
  })
})