import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { ref, nextTick } from 'vue'
import QuickNoteWindow from '../components/QuickNoteWindow.vue'

// Mock Tauri APIs
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: vi.fn(() => ({
    close: vi.fn().mockResolvedValue(undefined)
  }))
}))

// Helper to wait for async operations
const waitFor = (condition, timeout = 1000) => {
  return new Promise((resolve, reject) => {
    const startTime = Date.now()
    const interval = setInterval(() => {
      if (condition()) {
        clearInterval(interval)
        resolve()
      } else if (Date.now() - startTime > timeout) {
        clearInterval(interval)
        reject(new Error('Timeout waiting for condition'))
      }
    }, 50)
  })
}

describe('QuickNoteWindow.vue', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('renders correctly', () => {
    const wrapper = mount(QuickNoteWindow)
    expect(wrapper.find('h1').text()).toBe('快速记录')
    expect(wrapper.find('textarea').exists()).toBe(true)
    expect(wrapper.find('button').exists()).toBe(true)
  })

  it('has save button disabled initially', () => {
    const wrapper = mount(QuickNoteWindow)
    const saveButton = wrapper.findAll('button').find(b => b.text() === '保存')
    expect(saveButton?.attributes('disabled')).toBeDefined()
  })

  it('enables save button when content is entered', async () => {
    const wrapper = mount(QuickNoteWindow)
    const textarea = wrapper.find('textarea')
    const saveButton = wrapper.findAll('button').find(b => b.text() === '保存')

    await textarea.setValue('Test note content')
    await nextTick()

    expect(saveButton?.attributes('disabled')).toBeUndefined()
  })

  it('displays current time', async () => {
    const wrapper = mount(QuickNoteWindow)

    // Trigger the time update
    vi.advanceTimersByTime(100)
    await nextTick()

    const timeElement = wrapper.find('footer span')
    expect(timeElement.text()).not.toBe('')
  })

  it('calls invoke when saving', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    invoke.mockResolvedValue(undefined)

    const wrapper = mount(QuickNoteWindow)
    const textarea = wrapper.find('textarea')
    const saveButton = wrapper.findAll('button').find(b => b.text() === '保存')

    await textarea.setValue('Test note content')
    await nextTick()
    await saveButton?.trigger('click')

    expect(invoke).toHaveBeenCalledWith('tray_quick_note', {
      content: 'Test note content'
    })
  })

  it('saves on Enter key press', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    invoke.mockResolvedValue(undefined)

    const wrapper = mount(QuickNoteWindow)
    const textarea = wrapper.find('textarea')

    await textarea.setValue('Test note content')
    await nextTick()
    await textarea.trigger('keydown.enter')

    expect(invoke).toHaveBeenCalledWith('tray_quick_note', {
      content: 'Test note content'
    })
  })

  it('does not save empty content', async () => {
    const { invoke } = await import('@tauri-apps/api/core')

    const wrapper = mount(QuickNoteWindow)
    const textarea = wrapper.find('textarea')

    await textarea.setValue('   ')
    await nextTick()
    await textarea.trigger('keydown.enter')

    expect(invoke).not.toHaveBeenCalled()
  })

  it('shows saving state while saving', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    let resolveInvoke
    invoke.mockReturnValue(new Promise(resolve => { resolveInvoke = resolve }))

    const wrapper = mount(QuickNoteWindow)
    const textarea = wrapper.find('textarea')
    const saveButton = wrapper.findAll('button').find(b => b.text() === '保存')

    await textarea.setValue('Test note content')
    await nextTick()
    await saveButton?.trigger('click')
    await nextTick()

    // Button should show saving state
    expect(wrapper.vm.isSaving).toBe(true)

    // Resolve the invoke
    resolveInvoke(undefined)
  })

  it('closes window on cancel button click', async () => {
    const { getCurrentWindow } = await import('@tauri-apps/api/window')
    const mockClose = vi.fn().mockResolvedValue(undefined)
    getCurrentWindow.mockReturnValue({ close: mockClose })

    const wrapper = mount(QuickNoteWindow)
    const cancelButton = wrapper.findAll('button').find(b => b.text() === '取消')

    await cancelButton?.trigger('click')

    expect(mockClose).toHaveBeenCalled()
  })
})