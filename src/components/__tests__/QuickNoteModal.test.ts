import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount } from '@vue/test-utils'
import QuickNoteModal from '../QuickNoteModal.vue'

describe('QuickNoteModal', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    vi.setSystemTime(new Date('2026-03-17T14:30:45Z'))
  })

  afterEach(() => {
    vi.restoreAllMocks()
    vi.useRealTimers()
  })

  it('renders modal title', () => {
    const wrapper = mount(QuickNoteModal)
    expect(wrapper.text()).toContain('Quick Note')
  })

  it('displays keyboard shortcuts hint', () => {
    const wrapper = mount(QuickNoteModal)
    expect(wrapper.text()).toContain('Enter to save')
    expect(wrapper.text()).toContain('Shift+Enter')
  })

  it('displays current time', () => {
    const wrapper = mount(QuickNoteModal)
    expect(wrapper.vm.currentTime).toBeTruthy()
    // Format is MM/DD HH:MM:SS
    expect(wrapper.vm.currentTime).toMatch(/\d{2}\/\d{2}/)
  })

  it('updates time every second', async () => {
    const wrapper = mount(QuickNoteModal)
    const initialTime = wrapper.vm.currentTime

    vi.advanceTimersByTime(1000)
    await wrapper.vm.$nextTick()

    expect(wrapper.vm.currentTime).not.toBe(initialTime)
  })

  it('focuses textarea on mount', async () => {
    const wrapper = mount(QuickNoteModal, {
      attachTo: document.body
    })
    await wrapper.vm.$nextTick()

    const textarea = wrapper.find('textarea')
    expect(document.activeElement).toBe(textarea.element)

    wrapper.unmount()
  })

  it('binds content to textarea', async () => {
    const wrapper = mount(QuickNoteModal)
    const textarea = wrapper.find('textarea')

    await textarea.setValue('Test content')

    expect(wrapper.vm.content).toBe('Test content')
  })

  it('displays placeholder text', () => {
    const wrapper = mount(QuickNoteModal)
    const textarea = wrapper.find('textarea')
    expect(textarea.attributes('placeholder')).toBe('Capture your thoughts...')
  })

  it('emits close event when cancel button clicked', async () => {
    const wrapper = mount(QuickNoteModal)
    const cancelButton = wrapper.findAll('button').find(btn => btn.text() === 'Cancel')

    await cancelButton.trigger('click')

    expect(wrapper.emitted('close')).toBeTruthy()
  })

  it('emits close event when backdrop clicked', async () => {
    const wrapper = mount(QuickNoteModal)
    const backdrop = wrapper.find('.fixed.inset-0')

    await backdrop.trigger('click.self')

    expect(wrapper.emitted('close')).toBeTruthy()
  })

  it('emits save event when save button clicked with content', async () => {
    const wrapper = mount(QuickNoteModal)
    const textarea = wrapper.find('textarea')
    const saveButton = wrapper.findAll('button').find(btn => btn.text() === 'Save')

    await textarea.setValue('Test note')
    await saveButton.trigger('click')

    expect(wrapper.emitted('save')).toBeTruthy()
    expect(wrapper.emitted('save')[0]).toEqual(['Test note'])
  })

  it('trims whitespace when saving', async () => {
    const wrapper = mount(QuickNoteModal)
    const textarea = wrapper.find('textarea')
    const saveButton = wrapper.findAll('button').find(btn => btn.text() === 'Save')

    await textarea.setValue('  Test note  ')
    await saveButton.trigger('click')

    expect(wrapper.emitted('save')[0]).toEqual(['Test note'])
  })

  it('does not emit save when content is empty', async () => {
    const wrapper = mount(QuickNoteModal)
    const saveButton = wrapper.findAll('button').find(btn => btn.text() === 'Save')

    await saveButton.trigger('click')

    expect(wrapper.emitted('save')).toBeFalsy()
  })

  it('does not emit save when content is only whitespace', async () => {
    const wrapper = mount(QuickNoteModal)
    const textarea = wrapper.find('textarea')
    const saveButton = wrapper.findAll('button').find(btn => btn.text() === 'Save')

    await textarea.setValue('   ')
    await saveButton.trigger('click')

    expect(wrapper.emitted('save')).toBeFalsy()
  })

  it('disables save button when content is empty', () => {
    const wrapper = mount(QuickNoteModal)
    const saveButton = wrapper.findAll('button').find(btn => btn.text() === 'Save')

    expect(saveButton.attributes('disabled')).toBeDefined()
  })

  it('disables save button when content is only whitespace', async () => {
    const wrapper = mount(QuickNoteModal)
    const textarea = wrapper.find('textarea')
    const saveButton = wrapper.findAll('button').find(btn => btn.text() === 'Save')

    await textarea.setValue('   ')
    await wrapper.vm.$nextTick()

    expect(saveButton.attributes('disabled')).toBeDefined()
  })

  it('enables save button when content is valid', async () => {
    const wrapper = mount(QuickNoteModal)
    const textarea = wrapper.find('textarea')
    const saveButton = wrapper.findAll('button').find(btn => btn.text() === 'Save')

    await textarea.setValue('Valid content')
    await wrapper.vm.$nextTick()

    expect(saveButton.attributes('disabled')).toBeUndefined()
  })

  it('saves on Enter key press', async () => {
    const wrapper = mount(QuickNoteModal)
    const textarea = wrapper.find('textarea')

    await textarea.setValue('Test note')
    await textarea.trigger('keydown.enter', { key: 'Enter' })

    expect(wrapper.emitted('save')).toBeTruthy()
    expect(wrapper.emitted('save')[0]).toEqual(['Test note'])
  })

  it('does not save on Shift+Enter (allows newline)', async () => {
    const wrapper = mount(QuickNoteModal)
    const textarea = wrapper.find('textarea')

    await textarea.setValue('Test note')
    await textarea.trigger('keydown.enter', { key: 'Enter', shiftKey: true })

    expect(wrapper.emitted('save')).toBeFalsy()
  })

  it('does not save on Enter when content is empty', async () => {
    const wrapper = mount(QuickNoteModal)
    const textarea = wrapper.find('textarea')

    await textarea.trigger('keydown.enter', { key: 'Enter' })

    expect(wrapper.emitted('save')).toBeFalsy()
  })

  it('applies correct styling classes', () => {
    const wrapper = mount(QuickNoteModal)

    expect(wrapper.find('[class*="bg-[var(--color-surface-1)]"]').exists()).toBe(true)
    expect(wrapper.find('.rounded-2xl').exists()).toBe(true)
    expect(wrapper.find('textarea').classes()).toContain('bg-[var(--color-surface-0)]')
    expect(wrapper.find('textarea').classes()).toContain('resize-none')
  })

  it('has correct modal width', () => {
    const wrapper = mount(QuickNoteModal)
    const modal = wrapper.find('[class*="bg-[var(--color-surface-1)]"].rounded-2xl')
    expect(modal.classes()).toContain('w-[600px]')
  })

  it('has correct textarea height', () => {
    const wrapper = mount(QuickNoteModal)
    const textarea = wrapper.find('textarea')
    expect(textarea.classes()).toContain('h-40')
  })
})