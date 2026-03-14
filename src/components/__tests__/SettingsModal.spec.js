import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { ref, nextTick } from 'vue'
import SettingsModal from '../SettingsModal.vue'

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
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
    include_manual_records: true
  }

  it('renders title format input field', async () => {
    invoke.mockResolvedValueOnce(defaultSettings)

    const wrapper = mount(SettingsModal, {
      global: {
        stubs: {
          teleport: true
        }
      }
    })

    await nextTick()

    // Find the title format input
    const inputs = wrapper.findAll('input')
    const titleFormatInput = inputs.find(input =>
      input.attributes('placeholder') === '工作日报 - {date}'
    )

    expect(titleFormatInput).toBeDefined()
  })

  it('renders include manual records checkbox', async () => {
    invoke.mockResolvedValueOnce(defaultSettings)

    const wrapper = mount(SettingsModal, {
      global: {
        stubs: {
          teleport: true
        }
      }
    })

    await nextTick()

    // Find the checkbox
    const checkbox = wrapper.find('input[type="checkbox"]')
    expect(checkbox.exists()).toBe(true)
    expect(checkbox.attributes('id')).toBe('include_manual_records')
  })

  it('loads and displays settings with new fields', async () => {
    invoke.mockResolvedValueOnce(defaultSettings)

    const wrapper = mount(SettingsModal, {
      global: {
        stubs: {
          teleport: true
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
          teleport: true
        }
      }
    })

    await nextTick()
    await nextTick()

    // Find and click save button
    const saveButton = wrapper.findAll('button').find(btn => btn.text() === '保存')
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
          teleport: true
        }
      }
    })

    await nextTick()
    await nextTick()

    const checkbox = wrapper.find('input[type="checkbox"]')
    expect(checkbox.element.checked).toBe(true)
  })

  it('checkbox is unchecked when include_manual_records is false', async () => {
    invoke.mockResolvedValueOnce({ ...defaultSettings, include_manual_records: false })

    const wrapper = mount(SettingsModal, {
      global: {
        stubs: {
          teleport: true
        }
      }
    })

    await nextTick()
    await nextTick()

    const checkbox = wrapper.find('input[type="checkbox"]')
    expect(checkbox.element.checked).toBe(false)
  })
})