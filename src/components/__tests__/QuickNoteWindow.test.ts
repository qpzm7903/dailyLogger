import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount } from '@vue/test-utils'
import QuickNoteWindow from '../QuickNoteWindow.vue'

// Mock Tauri APIs
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: vi.fn(() => ({
    close: vi.fn()
  }))
}))

// Mock usePlatform composable
vi.mock('../../composables/usePlatform', () => ({
  usePlatform: () => ({
    isDesktop: ref(true)
  })
}))

import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'

const mockInvoke = vi.mocked(invoke)
const mockGetCurrentWindow = vi.mocked(getCurrentWindow)

describe('QuickNoteWindow', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    vi.setSystemTime(new Date('2026-03-20T14:30:45Z'))
    mockInvoke.mockClear()
    mockGetCurrentWindow.mockClear()
  })

  afterEach(() => {
    vi.restoreAllMocks()
    vi.useRealTimers()
  })

  it('renders window title', () => {
    const wrapper = mount(QuickNoteWindow)
    expect(wrapper.text()).toContain('Quick Note')
  })

  it('displays current time', () => {
    const wrapper = mount(QuickNoteWindow)
    expect(wrapper.vm.currentTime).toBeTruthy()
    // Format is MM/DD HH:MM:SS
    expect(wrapper.vm.currentTime).toMatch(/\d{2}\/\d{2}/)
  })

  it('updates time every second', async () => {
    const wrapper = mount(QuickNoteWindow)
    const initialTime = wrapper.vm.currentTime

    vi.advanceTimersByTime(1000)
    await wrapper.vm.$nextTick()

    expect(wrapper.vm.currentTime).not.toBe(initialTime)
  })

  it('focuses textarea on mount', async () => {
    const wrapper = mount(QuickNoteWindow, {
      attachTo: document.body
    })
    await wrapper.vm.$nextTick()

    const textarea = wrapper.find('textarea')
    expect(document.activeElement).toBe(textarea.element)

    wrapper.unmount()
  })

  it('binds content to textarea', async () => {
    const wrapper = mount(QuickNoteWindow)
    const textarea = wrapper.find('textarea')

    await textarea.setValue('Test content')

    expect(wrapper.vm.content).toBe('Test content')
  })

  it('displays placeholder text', () => {
    const wrapper = mount(QuickNoteWindow)
    const textarea = wrapper.find('textarea')
    expect(textarea.attributes('placeholder')).toBeTruthy()
  })

  it('disables save button when content is empty', () => {
    const wrapper = mount(QuickNoteWindow)
    const saveButton = wrapper.findAll('button').find(btn => btn.text() === 'Save')

    expect(saveButton.attributes('disabled')).toBeDefined()
  })

  it('disables save button when content is only whitespace', async () => {
    const wrapper = mount(QuickNoteWindow)
    const textarea = wrapper.find('textarea')
    const saveButton = wrapper.findAll('button').find(btn => btn.text() === 'Save')

    await textarea.setValue('   ')
    await wrapper.vm.$nextTick()

    expect(saveButton.attributes('disabled')).toBeDefined()
  })

  it('enables save button when content is valid', async () => {
    const wrapper = mount(QuickNoteWindow)
    const textarea = wrapper.find('textarea')
    const saveButton = wrapper.findAll('button').find(btn => btn.text() === 'Save')

    await textarea.setValue('Valid content')
    await wrapper.vm.$nextTick()

    expect(saveButton.attributes('disabled')).toBeUndefined()
  })

  it('calls invoke and closes window on save', async () => {
    const mockClose = vi.fn()
    mockGetCurrentWindow.mockReturnValue({
      close: mockClose
    } as any)

    const wrapper = mount(QuickNoteWindow)
    const textarea = wrapper.find('textarea')
    const saveButton = wrapper.findAll('button').find(btn => btn.text() === 'Save')

    await textarea.setValue('Test note')
    await saveButton.trigger('click')

    expect(mockInvoke).toHaveBeenCalledWith('tray_quick_note', { content: 'Test note' })
    expect(mockClose).toHaveBeenCalled()
  })

  it('trims whitespace when saving', async () => {
    const mockClose = vi.fn()
    mockGetCurrentWindow.mockReturnValue({
      close: mockClose
    } as any)

    const wrapper = mount(QuickNoteWindow)
    const textarea = wrapper.find('textarea')
    const saveButton = wrapper.findAll('button').find(btn => btn.text() === 'Save')

    await textarea.setValue('  Test note  ')
    await saveButton.trigger('click')

    expect(mockInvoke).toHaveBeenCalledWith('tray_quick_note', { content: 'Test note' })
  })

  it('saves on Enter key press', async () => {
    const mockClose = vi.fn()
    mockGetCurrentWindow.mockReturnValue({
      close: mockClose
    } as any)

    const wrapper = mount(QuickNoteWindow)
    const textarea = wrapper.find('textarea')

    await textarea.setValue('Test note')
    await textarea.trigger('keydown.enter', { key: 'Enter' })

    expect(mockInvoke).toHaveBeenCalledWith('tray_quick_note', { content: 'Test note' })
    expect(mockClose).toHaveBeenCalled()
  })

  it('does not save on Enter when content is empty', async () => {
    const wrapper = mount(QuickNoteWindow)
    const textarea = wrapper.find('textarea')

    await textarea.trigger('keydown.enter', { key: 'Enter' })

    expect(mockInvoke).not.toHaveBeenCalled()
  })

  it('closes window when cancel button clicked', async () => {
    const mockClose = vi.fn()
    mockGetCurrentWindow.mockReturnValue({
      close: mockClose
    } as any)

    const wrapper = mount(QuickNoteWindow)
    const cancelButton = wrapper.findAll('button').find(btn => btn.text() === 'Cancel')

    await cancelButton.trigger('click')

    expect(mockClose).toHaveBeenCalled()
  })

  it('closes window on Escape key press', async () => {
    const mockClose = vi.fn()
    mockGetCurrentWindow.mockReturnValue({
      close: mockClose
    } as any)

    const wrapper = mount(QuickNoteWindow)
    const textarea = wrapper.find('textarea')

    await textarea.trigger('keydown.esc')

    expect(mockClose).toHaveBeenCalled()
  })

  it('shows saving state while saving', async () => {
    mockInvoke.mockImplementation(() => new Promise(resolve => setTimeout(resolve, 100)))

    const wrapper = mount(QuickNoteWindow)
    const textarea = wrapper.find('textarea')
    const saveButton = wrapper.findAll('button').find(btn => btn.text() === 'Save')

    await textarea.setValue('Test note')
    await saveButton.trigger('click')

    // Button should show "Saving..." text during save
    expect(wrapper.vm.isSaving).toBe(true)
  })

  it('handles save error gracefully', async () => {
    vi.useRealTimers()
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
    mockInvoke.mockRejectedValue(new Error('Network error'))

    const wrapper = mount(QuickNoteWindow)
    const textarea = wrapper.find('textarea')
    const saveButton = wrapper.findAll('button').find(btn => btn.text() === 'Save')

    await textarea.setValue('Test note')
    await saveButton.trigger('click')

    // Wait for promise to settle
    await new Promise(resolve => setTimeout(resolve, 10))
    await wrapper.vm.$nextTick()

    expect(consoleSpy).toHaveBeenCalled()
    expect(wrapper.vm.isSaving).toBe(false)

    consoleSpy.mockRestore()
  })

  it('applies correct styling classes', () => {
    const wrapper = mount(QuickNoteWindow)

    expect(wrapper.find('[class*="bg-[var(--color-surface-0)]"]').exists()).toBe(true)
    expect(wrapper.find('textarea').classes()).toContain('bg-[var(--color-surface-0)]')
    expect(wrapper.find('textarea').classes()).toContain('resize-none')
  })

  it('has header with correct structure', () => {
    const wrapper = mount(QuickNoteWindow)

    const header = wrapper.find('header')
    expect(header.exists()).toBe(true)
    expect(header.find('h1').exists()).toBe(true)
  })

  it('has footer with buttons', () => {
    const wrapper = mount(QuickNoteWindow)

    const footer = wrapper.find('footer')
    expect(footer.exists()).toBe(true)
    const buttons = footer.findAll('button')
    expect(buttons.length).toBe(2)
  })

  it('shows shortcut hint when isDesktop is true', () => {
    const wrapper = mount(QuickNoteWindow)

    // Should show keyboard hints for desktop
    expect(wrapper.text()).toContain('Enter to save')
    expect(wrapper.text()).toContain('Esc to close')
  })

  it('clears time interval on unmount', async () => {
    const clearIntervalSpy = vi.spyOn(global, 'clearInterval')

    const wrapper = mount(QuickNoteWindow)
    wrapper.unmount()

    expect(clearIntervalSpy).toHaveBeenCalled()

    clearIntervalSpy.mockRestore()
  })
})