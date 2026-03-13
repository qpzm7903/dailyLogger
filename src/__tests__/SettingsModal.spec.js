/**
 * SettingsModal.vue 测试
 *
 * 覆盖行为：
 *  1. 保存成功时显示"✓ 已保存"
 *  2. 保存失败时显示"✗ 保存失败"
 *  3. 保存时传入正确的命令和参数
 *  4. API Key 显示/隐藏切换
 */
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))

import { invoke } from '@tauri-apps/api/core'
import SettingsModal from '../components/SettingsModal.vue'

function setupInvoke(overrides = {}) {
  invoke.mockImplementation((cmd) => {
    if (cmd in overrides) {
      const val = overrides[cmd]
      return val instanceof Error ? Promise.reject(val) : Promise.resolve(val)
    }
    if (cmd === 'get_settings') return Promise.resolve({})
    return Promise.resolve()
  })
}

describe('SettingsModal.vue - saveSettings', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    setupInvoke()
  })

  it('保存成功时显示"✓ 已保存"', async () => {
    const wrapper = mount(SettingsModal)
    await flushPromises()

    // Updated selector to match new gradient button class
    await wrapper.find('button.bg-gradient-to-r').trigger('click')
    await flushPromises()

    // Updated selector to match new green text class
    expect(wrapper.find('.text-green-400').text()).toBe('✓ 已保存')
  })

  it('保存失败时显示"✗ 保存失败"', async () => {
    setupInvoke({ save_settings: new Error('network error') })
    const wrapper = mount(SettingsModal)
    await flushPromises()

    // Updated selector to match new gradient button class
    await wrapper.find('button.bg-gradient-to-r').trigger('click')
    await flushPromises()

    // Updated selector to match new red text class
    expect(wrapper.find('.text-red-400').text()).toBe('✗ 保存失败')
  })

  it('调用 save_settings 命令并传入 settings 对象', async () => {
    const wrapper = mount(SettingsModal)
    await flushPromises()

    // Updated selector to match new gradient button class
    await wrapper.find('button.bg-gradient-to-r').trigger('click')
    await flushPromises()

    expect(invoke).toHaveBeenCalledWith('save_settings', {
      settings: expect.objectContaining({
        api_base_url: expect.any(String),
        api_key: expect.any(String),
        model_name: expect.any(String),
      }),
    })
  })

  it('保存进行中时按钮显示"保存中…"且禁用', async () => {
    let resolveSave
    invoke.mockImplementation((cmd) => {
      if (cmd === 'get_settings') return Promise.resolve({})
      if (cmd === 'save_settings') return new Promise(r => { resolveSave = r })
      return Promise.resolve()
    })

    const wrapper = mount(SettingsModal)
    await flushPromises()

    // Updated selector to match new gradient button class
    const saveBtn = wrapper.find('button.bg-gradient-to-r')
    saveBtn.trigger('click') // 不 await，让保存保持 pending

    await flushPromises()

    // Updated expected text to match new button text with emoji prefix
    expect(saveBtn.text()).toBe('保存中…')
    expect(saveBtn.attributes('disabled')).toBeDefined()

    resolveSave()
    await flushPromises()
  })
})

describe('SettingsModal.vue - API Key 可见性', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    setupInvoke()
  })

  it('初始状态 API Key 输入框为 password 类型', async () => {
    const wrapper = mount(SettingsModal)
    await flushPromises()

    expect(wrapper.find('input[placeholder="sk-..."]').attributes('type')).toBe('password')
  })

  it('点击"显示"按钮后切换为 text 类型', async () => {
    const wrapper = mount(SettingsModal)
    await flushPromises()

    // Find the toggle button by text content
    const allButtons = wrapper.findAll('button')
    const toggleBtn = allButtons.find(b => b.text().includes('显示') || b.text().includes('隐藏'))

    if (toggleBtn) {
      await toggleBtn.trigger('click')
    }

    expect(wrapper.find('input[placeholder="sk-..."]').attributes('type')).toBe('text')
  })

  it('再次点击"隐藏"按钮恢复为 password 类型', async () => {
    const wrapper = mount(SettingsModal)
    await flushPromises()

    // Find the toggle button by text content
    const allButtons = wrapper.findAll('button')
    const toggleBtn = allButtons.find(b => b.text().includes('显示') || b.text().includes('隐藏'))

    // First click to show
    if (toggleBtn) {
      await toggleBtn.trigger('click')
    }

    // Find the toggle button again (text may have changed)
    const toggleBtn2 = wrapper.findAll('button').find(b => b.text().includes('显示') || b.text().includes('隐藏'))

    // Second click to hide
    if (toggleBtn2) {
      await toggleBtn2.trigger('click')
    }

    expect(wrapper.find('input[placeholder="sk-..."]').attributes('type')).toBe('password')
  })
})
