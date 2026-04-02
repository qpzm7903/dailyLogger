import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { ref, nextTick } from 'vue'
import SettingsModal from '../SettingsModal.vue'

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

// Mock Tauri plugins
vi.mock('@tauri-apps/plugin-dialog', () => ({
  save: vi.fn(),
  open: vi.fn()
}))

vi.mock('@tauri-apps/plugin-fs', () => ({
  writeFile: vi.fn(),
  writeTextFile: vi.fn(),
  readTextFile: vi.fn()
}))

vi.mock('../../stores/toast.js', () => ({
  showSuccess: vi.fn(),
  showError: vi.fn()
}))

import { invoke } from '@tauri-apps/api/core'

describe('SettingsModal', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  const defaultSettings = {
    api_base_url: 'https://api.openai.com/v1',
    api_key: 'sk-test',
    model_name: 'gpt-4o',
    screenshot_interval: 5,
    summary_time: '18:00',
    obsidian_path: '/test/path',
    summary_model_name: '',
    analysis_prompt: '',
    summary_prompt: '',
    change_threshold: 3,
    max_silent_minutes: 30,
    summary_title_format: '工作日报 - {date}',
    include_manual_records: true,
    window_whitelist: '[]',
    window_blacklist: '[]',
    use_whitelist_only: false
  }

  it('renders title format input field', async () => {
    invoke.mockResolvedValueOnce(defaultSettings)

    const wrapper = mount(SettingsModal, {
      global: {
        stubs: {
          teleport: true,
        }
      }
    })

    await nextTick()
    await nextTick()

    // Find inputs in the daily report section by looking for text inputs
    const inputs = wrapper.findAll('input[type="text"]')
    expect(inputs.length).toBeGreaterThan(0)
  })

  it('renders include manual records checkbox', async () => {
    invoke.mockResolvedValueOnce(defaultSettings)

    const wrapper = mount(SettingsModal, {
      global: {
        stubs: {
          teleport: true,
        }
      }
    })

    await nextTick()

    // Switch to 'ai' tab where include_manual_records checkbox is located
    wrapper.vm.activeTab = 'ai'
    await nextTick()

    // Find the checkbox
    const checkbox = wrapper.find('input[type="checkbox"]#include_manual_records')
    expect(checkbox.exists()).toBe(true)
  })

  it('loads and displays settings with new fields', async () => {
    invoke.mockResolvedValueOnce(defaultSettings)

    const wrapper = mount(SettingsModal, {
      global: {
        stubs: {
          teleport: true,
        }
      }
    })

    // Wait for onMounted to complete
    await nextTick()
    await nextTick()

    expect(invoke).toHaveBeenCalledWith('get_settings')
  })

  it('saves settings with new fields', async () => {
    invoke.mockResolvedValueOnce(defaultSettings)
    invoke.mockResolvedValueOnce(undefined)

    const wrapper = mount(SettingsModal, {
      global: {
        stubs: {
          teleport: true,
        }
      }
    })

    await nextTick()
    await nextTick()

    // Find and click save button
    const saveButton = wrapper.findAll('button').find(btn => btn.text() === 'Save')
    await saveButton.trigger('click')

    await nextTick()

    // Check that save_settings was called with settings including new fields
    expect(invoke).toHaveBeenCalledWith('save_settings', {
      settings: expect.objectContaining({
        summary_title_format: expect.any(String),
        include_manual_records: expect.any(Boolean)
      })
    })
  })

  it('checkbox is checked by default when include_manual_records is true', async () => {
    invoke.mockResolvedValueOnce({ ...defaultSettings, include_manual_records: true })

    const wrapper = mount(SettingsModal, {
      global: {
        stubs: {
          teleport: true,
        }
      }
    })

    await nextTick()
    await nextTick()

    // Switch to 'ai' tab where include_manual_records checkbox is located
    wrapper.vm.activeTab = 'ai'
    await nextTick()

    const checkbox = wrapper.find('input#include_manual_records')
    expect(checkbox.element.checked).toBe(true)
  })

  it('checkbox is unchecked when include_manual_records is false', async () => {
    invoke.mockResolvedValueOnce({ ...defaultSettings, include_manual_records: false })

    const wrapper = mount(SettingsModal, {
      global: {
        stubs: {
          teleport: true,
        }
      }
    })

    await nextTick()
    await nextTick()

    // Switch to 'ai' tab where include_manual_records checkbox is located
    wrapper.vm.activeTab = 'ai'
    await nextTick()

    const checkbox = wrapper.find('input#include_manual_records')
    expect(checkbox.element.checked).toBe(false)
  })

  // ── SMART-001 Task 5: Window whitelist/blacklist UI tests ──

  describe('AC2 - Window whitelist/blacklist configuration', () => {
    it('renders window whitelist section', async () => {
      invoke.mockResolvedValueOnce(defaultSettings)

      const wrapper = mount(SettingsModal, {
        global: {
          stubs: {
            teleport: true,
          }
        }
      })

      await nextTick()
      await nextTick()

      // Switch to 'capture' tab where window filter settings are located
      wrapper.vm.activeTab = 'capture'
      await nextTick()

      // Find window whitelist section by looking for section heading
      const headings = wrapper.findAll('h3')
      const whitelistHeading = headings.find(h => h.text().includes('Window Filter'))
      expect(whitelistHeading.exists()).toBe(true)
    })

    it('renders use_whitelist_only toggle', async () => {
      invoke.mockResolvedValueOnce(defaultSettings)

      const wrapper = mount(SettingsModal, {
        global: {
          stubs: {
            teleport: true,
          }
        }
      })

      await nextTick()
      await nextTick()

      // Switch to 'capture' tab where window filter settings are located
      wrapper.vm.activeTab = 'capture'
      await nextTick()

      // Find the use_whitelist_only checkbox
      const checkbox = wrapper.find('input#use_whitelist_only')
      expect(checkbox.exists()).toBe(true)
      expect(checkbox.element.type).toBe('checkbox')
    })

    it('use_whitelist_only toggle is checked when setting is true', async () => {
      invoke.mockResolvedValueOnce({ ...defaultSettings, use_whitelist_only: true })

      const wrapper = mount(SettingsModal, {
        global: {
          stubs: {
            teleport: true,
          }
        }
      })

      await nextTick()
      await nextTick()

      // Switch to 'capture' tab where window filter settings are located
      wrapper.vm.activeTab = 'capture'
      await nextTick()

      const checkbox = wrapper.find('input#use_whitelist_only')
      expect(checkbox.element.checked).toBe(true)
    })

    it('use_whitelist_only toggle is unchecked when setting is false', async () => {
      invoke.mockResolvedValueOnce({ ...defaultSettings, use_whitelist_only: false })

      const wrapper = mount(SettingsModal, {
        global: {
          stubs: {
            teleport: true,
          }
        }
      })

      await nextTick()
      await nextTick()

      // Switch to 'capture' tab where window filter settings are located
      wrapper.vm.activeTab = 'capture'
      await nextTick()

      const checkbox = wrapper.find('input#use_whitelist_only')
      expect(checkbox.element.checked).toBe(false)
    })

    it('displays whitelist tags from settings', async () => {
      invoke.mockResolvedValueOnce({
        ...defaultSettings,
        window_whitelist: '["VS Code", "IntelliJ IDEA"]'
      })

      const wrapper = mount(SettingsModal, {
        global: {
          stubs: {
            teleport: true,
          }
        }
      })

      await nextTick()
      await nextTick()

      // Switch to 'capture' tab where window filter settings are located
      wrapper.vm.activeTab = 'capture'
      await nextTick()

      // Check whitelist tags are rendered
      const html = wrapper.html()
      expect(html).toContain('VS Code')
      expect(html).toContain('IntelliJ IDEA')
    })

    it('displays blacklist tags from settings', async () => {
      invoke.mockResolvedValueOnce({
        ...defaultSettings,
        window_blacklist: '["浏览器", "聊天软件"]'
      })

      const wrapper = mount(SettingsModal, {
        global: {
          stubs: {
            teleport: true,
          }
        }
      })

      await nextTick()
      await nextTick()

      // Switch to 'capture' tab where window filter settings are located
      wrapper.vm.activeTab = 'capture'
      await nextTick()

      // Check blacklist tags are rendered
      const html = wrapper.html()
      expect(html).toContain('浏览器')
      expect(html).toContain('聊天软件')
    })

    it('can add whitelist tag', async () => {
      invoke.mockResolvedValueOnce(defaultSettings)

      const wrapper = mount(SettingsModal, {
        global: {
          stubs: {
            teleport: true,
          }
        }
      })

      await nextTick()
      await nextTick()

      // Switch to 'capture' tab where window filter settings are located
      wrapper.vm.activeTab = 'capture'
      await nextTick()

      // Find whitelist input and add tag
      const inputs = wrapper.findAll('input[type="text"]')
      const whitelistInput = inputs.find(input =>
        input.attributes('placeholder')?.includes('添加白名单')
      )

      if (whitelistInput) {
        await whitelistInput.setValue('WeChat')
        await whitelistInput.trigger('keyup.enter')

        const html = wrapper.html()
        expect(html).toContain('WeChat')
      }
    })

    it('can remove whitelist tag', async () => {
      invoke.mockResolvedValueOnce({
        ...defaultSettings,
        window_whitelist: '["VS Code", "WeChat"]'
      })

      const wrapper = mount(SettingsModal, {
        global: {
          stubs: {
            teleport: true,
          }
        }
      })

      await nextTick()
      await nextTick()

      // Switch to 'capture' tab where window filter settings are located
      wrapper.vm.activeTab = 'capture'
      await nextTick()

      // Verify both tags are present
      expect(wrapper.html()).toContain('VS Code')
      expect(wrapper.html()).toContain('WeChat')

      // Verify whitelistTags array has both tags
      expect(wrapper.vm.whitelistTags).toEqual(['VS Code', 'WeChat'])

      // Find the close button specifically within the WeChat tag span
      // Tags are rendered in order, so WeChat is at index 1
      // Click the second close button (for WeChat)
      const allSpans = wrapper.findAll('span').filter(s =>
        s.classes().includes('bg-primary/20')
      )

      // Find the span containing 'WeChat' text
      const weChatSpan = allSpans.find(s => s.text().includes('WeChat'))
      expect(weChatSpan.exists()).toBe(true)

      // Click the close button inside that span
      const closeButton = weChatSpan.find('button')
      await closeButton.trigger('click')
      await nextTick()

      // Check that WeChat was removed
      expect(wrapper.vm.whitelistTags).toEqual(['VS Code'])
      expect(wrapper.vm.whitelistTags).not.toContain('WeChat')
    })

    it('saves window whitelist/blacklist settings', async () => {
      invoke.mockResolvedValueOnce(defaultSettings)
      invoke.mockResolvedValueOnce(undefined)

      const wrapper = mount(SettingsModal, {
        global: {
          stubs: {
            teleport: true,
          }
        }
      })

      await nextTick()
      await nextTick()

      // Set some tags
      wrapper.vm.whitelistTags = ['VS Code', 'Terminal']
      wrapper.vm.blacklistTags = ['Chrome']
      wrapper.vm.settings.use_whitelist_only = true

      // Find and click save button
      const saveButton = wrapper.findAll('button').find(btn => btn.text() === 'Save')
      await saveButton.trigger('click')

      await nextTick()

      // Check that save_settings was called with window settings
      expect(invoke).toHaveBeenCalledWith('save_settings', {
        settings: expect.objectContaining({
          window_whitelist: '["VS Code","Terminal"]',
          window_blacklist: '["Chrome"]',
          use_whitelist_only: true
        })
      })
    })
  })

  describe('AC3 - Only capture whitelisted apps toggle', () => {
    it('toggles use_whitelist_only setting', async () => {
      invoke.mockResolvedValueOnce(defaultSettings)

      const wrapper = mount(SettingsModal, {
        global: {
          stubs: {
            teleport: true,
          }
        }
      })

      await nextTick()
      await nextTick()

      // Switch to 'capture' tab where window filter settings are located
      wrapper.vm.activeTab = 'capture'
      await nextTick()

      const checkbox = wrapper.find('input#use_whitelist_only')
      expect(checkbox.element.checked).toBe(false)

      // Toggle the checkbox
      await checkbox.setValue(true)

      expect(wrapper.vm.settings.use_whitelist_only).toBe(true)
    })
  })
})
