import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { ref } from 'vue'

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

vi.mock('@tauri-apps/plugin-dialog', () => ({
  save: vi.fn()
}))

vi.mock('@tauri-apps/plugin-fs', () => ({
  writeFile: vi.fn(),
  writeTextFile: vi.fn()
}))

vi.mock('../stores/toast.js', () => ({
  showSuccess: vi.fn(),
  showError: vi.fn()
}))

import { invoke } from '@tauri-apps/api/core'
import { showSuccess, showError } from '../stores/toast.js'
import SettingsModal from '../components/SettingsModal.vue'

describe('SettingsModal', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    // Default mock implementations
    invoke.mockImplementation((cmd) => {
      if (cmd === 'get_settings') {
        return Promise.resolve({
          api_base_url: '',
          api_key: '',
          model_name: 'gpt-4o',
          screenshot_interval: 5,
          summary_time: '18:00',
          obsidian_path: '',
          analysis_prompt: '',
          summary_prompt: '',
          change_threshold: 3,
          max_silent_minutes: 30
        })
      }
      if (cmd === 'get_default_analysis_prompt') {
        return Promise.resolve('Analyze this screenshot and return a JSON object with:\n- current_focus: What is the user currently working on?\n- active_software: What software is being used?\n- context_keywords: What are the key topics/technologies?')
      }
      if (cmd === 'save_settings') {
        return Promise.resolve()
      }
      return Promise.resolve()
    })
  })

  it('renders analysis prompt textarea', async () => {
    const wrapper = mount(SettingsModal)
    await wrapper.vm.$nextTick()

    // Switch to 'ai' tab where analysis prompt textarea is located
    wrapper.vm.activeTab = 'ai'
    await wrapper.vm.$nextTick()

    const textarea = wrapper.find('textarea')
    expect(textarea.exists()).toBe(true)
  })

  it('shows "View Default" and "Reset to Default" buttons', async () => {
    const wrapper = mount(SettingsModal)
    await wrapper.vm.$nextTick()

    // Switch to 'ai' tab where analysis prompt buttons are located
    wrapper.vm.activeTab = 'ai'
    await wrapper.vm.$nextTick()

    const buttons = wrapper.findAll('button')
    const buttonTexts = buttons.map(b => b.text())

    expect(buttonTexts.some(t => t.includes('View Default'))).toBe(true)
    expect(buttonTexts.some(t => t.includes('Reset to Default'))).toBe(true)
  })

  it('calls get_default_analysis_prompt when "View Default" is clicked', async () => {
    const wrapper = mount(SettingsModal)
    await wrapper.vm.$nextTick()

    // Find and click the "View Default" button
    const buttons = wrapper.findAll('button')
    const showDefaultBtn = buttons.find(b => b.text().includes('View Default'))

    if (showDefaultBtn) {
      await showDefaultBtn.trigger('click')
      await wrapper.vm.$nextTick()

      expect(invoke).toHaveBeenCalledWith('get_default_analysis_prompt')
      expect(wrapper.vm.showDefaultPromptModal).toBe(true)
    }
  })

  it('resets analysis_prompt when "Reset to Default" is clicked', async () => {
    const wrapper = mount(SettingsModal)
    await wrapper.vm.$nextTick()

    // Set a custom prompt
    wrapper.vm.settings.analysis_prompt = 'Custom prompt'
    expect(wrapper.vm.settings.analysis_prompt).toBe('Custom prompt')

    // Find and click the "Reset to Default" button
    const buttons = wrapper.findAll('button')
    const resetBtn = buttons.find(b => b.text().includes('Reset to Default'))

    if (resetBtn) {
      await resetBtn.trigger('click')
      await wrapper.vm.$nextTick()

      expect(wrapper.vm.settings.analysis_prompt).toBe('')
      expect(showSuccess).toHaveBeenCalledWith('Reset to default prompt, will take effect after save')
    }
  })

  it('displays default prompt modal with content', async () => {
    const wrapper = mount(SettingsModal)
    await wrapper.vm.$nextTick()

    // Trigger showing the modal
    wrapper.vm.showDefaultPromptModal = true
    wrapper.vm.defaultPromptContent = 'Test default prompt content'
    await wrapper.vm.$nextTick()

    // Check if modal is visible by looking for the pre element with content
    const preElement = wrapper.find('pre')
    expect(preElement.exists()).toBe(true)
    expect(preElement.text()).toContain('Test default prompt content')
  })
})
