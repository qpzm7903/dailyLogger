/**
 * App.vue - triggerCapture 功能测试
 *
 * 覆盖行为：
 *  1. 点击"🤖 分析"按钮时调用后端 trigger_capture 命令
 *  2. 截图成功后自动刷新今日记录（loadTodayRecords）
 *  3. 截图失败时不触发记录刷新
 *  4. 截图失败后按钮恢复可点击状态（isCapturing 重置）
 */
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'

// 必须在 import 组件之前声明 mock
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn(() => Promise.resolve(() => {})) }))

import { invoke } from '@tauri-apps/api/core'
import App from '../App.vue'

// 所有子弹窗存根，避免其内部 invoke 调用干扰
const STUBS = {
  SettingsModal: true,
  QuickNoteModal: true,
  ScreenshotModal: true,
  ScreenshotGallery: true,
  DailySummaryViewer: true,
  LogViewer: true,
}

/**
 * 配置 invoke mock：
 *   overrides 可以传 Error 实例表示该命令应当 reject
 */
function setupInvoke(overrides = {}) {
  invoke.mockImplementation((cmd) => {
    if (cmd in overrides) {
      const val = overrides[cmd]
      return val instanceof Error ? Promise.reject(val) : Promise.resolve(val)
    }
    if (cmd === 'get_settings') return Promise.resolve({ auto_capture_enabled: false, last_summary_path: '' })
    if (cmd === 'get_today_records') return Promise.resolve([])
    return Promise.resolve()
  })
}

describe('App.vue - triggerCapture', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    setupInvoke()
  })

  it('点击"🤖 分析"按钮时调用 trigger_capture 命令', async () => {
    const wrapper = mount(App, { global: { stubs: STUBS } })
    await flushPromises()

    // Use text content to find the analyze button (i18n: "🤖 Analyze" or "🤖 分析")
    const buttons = wrapper.findAll('button')
    const analyzeBtn = buttons.find(btn => btn.text().includes('🤖'))
    await analyzeBtn.trigger('click')
    await flushPromises()

    expect(invoke).toHaveBeenCalledWith('trigger_capture')
  })

  it('截图成功后调用 get_today_records 刷新记录', async () => {
    const wrapper = mount(App, { global: { stubs: STUBS } })
    await flushPromises()

    // 清除 onMounted 产生的调用记录，只统计本次点击之后的行为
    invoke.mockClear()
    setupInvoke()

    const buttons = wrapper.findAll('button')
    const analyzeBtn = buttons.find(btn => btn.text().includes('🤖'))
    await analyzeBtn.trigger('click')
    await flushPromises()

    expect(invoke).toHaveBeenCalledWith('get_today_records')
  })

  it('截图失败时不调用 get_today_records', async () => {
    const wrapper = mount(App, { global: { stubs: STUBS } })
    await flushPromises()

    invoke.mockClear()
    setupInvoke({ trigger_capture: new Error('screenshot failed') })

    const buttons = wrapper.findAll('button')
    const analyzeBtn = buttons.find(btn => btn.text().includes('🤖'))
    await analyzeBtn.trigger('click')
    await flushPromises()

    expect(invoke).not.toHaveBeenCalledWith('get_today_records')
  })

  it('截图失败后按钮文字恢复且不再禁用', async () => {
    setupInvoke({ trigger_capture: new Error('failed') })
    const wrapper = mount(App, { global: { stubs: STUBS } })
    await flushPromises()

    const buttons = wrapper.findAll('button')
    const analyzeBtn = buttons.find(btn => btn.text().includes('🤖'))
    await analyzeBtn.trigger('click')
    await flushPromises()

    expect(analyzeBtn.text()).toMatch(/🤖/)
    expect(analyzeBtn.attributes('disabled')).toBeUndefined()
  })
})
