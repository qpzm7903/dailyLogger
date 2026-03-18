import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import ScreenshotModal from '../ScreenshotModal.vue'

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

import { invoke } from '@tauri-apps/api/core'

describe('ScreenshotModal - Window Info Display (SMART-001 Task 6)', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    invoke.mockResolvedValue('data:image/png;base64,mockBase64Data')
  })

  describe('AC1 - Window info in record details', () => {
    it('displays window title in details section when active_window exists', async () => {
      const record = {
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

      const wrapper = mount(ScreenshotModal, {
        props: { record },
        global: {
          stubs: {}
        }
      })

      await nextTick()
      await nextTick()

      // Check that window title is displayed in details
      const html = wrapper.html()
      expect(html).toContain('main.rs - DailyLogger - VS Code')
    })

    it('displays process name (app name) in details section', async () => {
      const record = {
        id: 1,
        timestamp: '2026-03-15T10:00:00Z',
        source_type: 'auto',
        screenshot_path: '/path/screenshot1.png',
        content: JSON.stringify({
          current_focus: 'Working on project',
          active_software: 'IntelliJ IDEA',
          context_keywords: ['java'],
          active_window: {
            title: 'MyProject - IntelliJ IDEA',
            process_name: 'idea64'
          }
        })
      }

      const wrapper = mount(ScreenshotModal, {
        props: { record },
        global: {
          stubs: {}
        }
      })

      await nextTick()
      await nextTick()

      // Check that process name is displayed
      const html = wrapper.html()
      expect(html).toContain('idea64')
    })

    it('shows window info section label', async () => {
      const record = {
        id: 1,
        timestamp: '2026-03-15T10:00:00Z',
        source_type: 'auto',
        screenshot_path: '/path/screenshot1.png',
        content: JSON.stringify({
          current_focus: 'Working on code',
          active_software: 'VS Code',
          context_keywords: ['code'],
          active_window: {
            title: 'VS Code Window',
            process_name: 'Code'
          }
        })
      }

      const wrapper = mount(ScreenshotModal, {
        props: { record },
        global: {
          stubs: {}
        }
      })

      await nextTick()
      await nextTick()

      // Check for window info section label
      const html = wrapper.html()
      expect(html).toContain('Window')
    })

    it('does not show window info section when active_window is missing', async () => {
      const record = {
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

      const wrapper = mount(ScreenshotModal, {
        props: { record },
        global: {
          stubs: {}
        }
      })

      await nextTick()
      await nextTick()

      // Window info section should not exist
      const windowInfoSection = wrapper.find('.window-info-section')
      expect(windowInfoSection.exists()).toBe(false)
    })

    it('handles empty active_window gracefully', async () => {
      const record = {
        id: 1,
        timestamp: '2026-03-15T10:00:00Z',
        source_type: 'auto',
        screenshot_path: '/path/screenshot1.png',
        content: JSON.stringify({
          current_focus: 'Working on code',
          active_software: 'VS Code',
          context_keywords: ['code'],
          active_window: {
            title: '',
            process_name: ''
          }
        })
      }

      const wrapper = mount(ScreenshotModal, {
        props: { record },
        global: {
          stubs: {}
        }
      })

      await nextTick()
      await nextTick()

      // Should not crash and window info section should be hidden
      expect(wrapper.html()).toContain('Screenshot Details')
    })

    it('displays window info with icon', async () => {
      const record = {
        id: 1,
        timestamp: '2026-03-15T10:00:00Z',
        source_type: 'auto',
        screenshot_path: '/path/screenshot1.png',
        content: JSON.stringify({
          current_focus: 'Browsing',
          active_software: 'Chrome',
          context_keywords: ['web'],
          active_window: {
            title: 'Google - Chrome',
            process_name: 'chrome'
          }
        })
      }

      const wrapper = mount(ScreenshotModal, {
        props: { record },
        global: {
          stubs: {}
        }
      })

      await nextTick()
      await nextTick()

      const html = wrapper.html()
      // Should have an icon element (emoji or otherwise) with window info
      expect(html).toContain('Google - Chrome')
      expect(html).toContain('chrome')
    })
  })

  describe('parseContent with window info', () => {
    it('parses content JSON correctly with active_window', async () => {
      const record = {
        id: 1,
        timestamp: '2026-03-15T10:00:00Z',
        source_type: 'auto',
        screenshot_path: '/path/screenshot1.png',
        content: JSON.stringify({
          current_focus: 'Writing documentation',
          active_software: 'Typora',
          context_keywords: ['docs', 'markdown'],
          active_window: {
            title: 'README.md - Typora',
            process_name: 'Typora'
          }
        })
      }

      const wrapper = mount(ScreenshotModal, {
        props: { record },
        global: {
          stubs: {}
        }
      })

      await nextTick()
      await nextTick()

      const html = wrapper.html()
      // Should show the parsed content
      expect(html).toContain('Writing documentation')
      expect(html).toContain('Typora')
      expect(html).toContain('README.md - Typora')
    })

    it('handles malformed content JSON gracefully', async () => {
      const record = {
        id: 1,
        timestamp: '2026-03-15T10:00:00Z',
        source_type: 'auto',
        screenshot_path: '/path/screenshot1.png',
        content: 'This is not valid JSON'
      }

      const wrapper = mount(ScreenshotModal, {
        props: { record },
        global: {
          stubs: {}
        }
      })

      await nextTick()
      await nextTick()

      // Should show raw content when JSON parsing fails
      expect(wrapper.html()).toContain('This is not valid JSON')
    })
  })
})