import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import TagCloud from '../TagCloud.vue'

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

// Mock toast
vi.mock('../../stores/toast', () => ({
  showSuccess: vi.fn(),
  showError: vi.fn()
}))

import { invoke } from '@tauri-apps/api/core'
import { showSuccess, showError } from '../../stores/toast'

describe('TagCloud', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  const mockTags = [
    { id: 1, name: 'work', color: 'blue', usage_count: 15 },
    { id: 2, name: 'meeting', color: 'green', usage_count: 8 },
    { id: 3, name: 'bug', color: 'red', usage_count: 3 }
  ]

  it('renders tag cloud title', async () => {
    invoke.mockResolvedValue([])
    const wrapper = mount(TagCloud)
    await flushPromises()
    expect(wrapper.text()).toContain('Tag Cloud')
  })

  it('emits close event when close button clicked', async () => {
    invoke.mockResolvedValue([])
    const wrapper = mount(TagCloud)
    await flushPromises()

    const closeButton = wrapper.find('button[class*="text-gray-400"]')
    await closeButton.trigger('click')

    expect(wrapper.emitted('close')).toBeTruthy()
  })

  it('emits close event when backdrop clicked', async () => {
    invoke.mockResolvedValue([])
    const wrapper = mount(TagCloud)
    await flushPromises()

    const backdrop = wrapper.find('.fixed.inset-0')
    await backdrop.trigger('click.self')

    expect(wrapper.emitted('close')).toBeTruthy()
  })

  it('loads tags on mount', async () => {
    invoke.mockResolvedValue(mockTags)
    const wrapper = mount(TagCloud)
    await flushPromises()

    expect(invoke).toHaveBeenCalledWith('get_all_manual_tags')
    expect(wrapper.vm.tags.length).toBe(3)
  })

  it('displays loading state', async () => {
    let resolvePromise
    const promise = new Promise(resolve => { resolvePromise = resolve })
    invoke.mockReturnValue(promise)

    const wrapper = mount(TagCloud)
    await wrapper.vm.$nextTick()

    expect(wrapper.text()).toContain('Loading...')

    resolvePromise([])
    await flushPromises()
  })

  it('displays empty state when no tags', async () => {
    invoke.mockResolvedValue([])
    const wrapper = mount(TagCloud)
    await flushPromises()

    expect(wrapper.text()).toContain('No tags yet')
    expect(wrapper.text()).toContain('add tags to records in history')
  })

  it('displays tags with names and usage counts', async () => {
    invoke.mockResolvedValue(mockTags)
    const wrapper = mount(TagCloud)
    await flushPromises()

    expect(wrapper.text()).toContain('work')
    expect(wrapper.text()).toContain('15')
    expect(wrapper.text()).toContain('meeting')
    expect(wrapper.text()).toContain('8')
    expect(wrapper.text()).toContain('bug')
    expect(wrapper.text()).toContain('3')
  })

  it('applies correct size based on usage count', async () => {
    invoke.mockResolvedValue(mockTags)
    const wrapper = mount(TagCloud)
    await flushPromises()

    const tagButtons = wrapper.findAll('button').filter(btn =>
      btn.text().includes('work') || btn.text().includes('meeting') || btn.text().includes('bug')
    )

    // work (15) should be text-base
    expect(tagButtons[0].classes()).toContain('text-base')
    // meeting (8) should be text-sm
    expect(tagButtons[1].classes()).toContain('text-sm')
    // bug (3) should be text-xs
    expect(tagButtons[2].classes()).toContain('text-xs')
  })

  it('applies correct color classes', async () => {
    invoke.mockResolvedValue(mockTags)
    const wrapper = mount(TagCloud)
    await flushPromises()

    const tagButtons = wrapper.findAll('button').filter(btn =>
      btn.text().includes('work') || btn.text().includes('meeting') || btn.text().includes('bug')
    )

    expect(tagButtons[0].classes()).toContain('bg-blue-500/30')
    expect(tagButtons[1].classes()).toContain('bg-green-500/30')
    expect(tagButtons[2].classes()).toContain('bg-red-500/30')
  })

  it('selects tag when clicked', async () => {
    invoke.mockResolvedValue(mockTags)
    const wrapper = mount(TagCloud)
    await flushPromises()

    const workTag = wrapper.findAll('button').find(btn => btn.text().includes('work'))
    await workTag.trigger('click')

    expect(wrapper.vm.selectedTag).toEqual(mockTags[0])
    expect(wrapper.emitted('tagSelected')).toBeTruthy()
    expect(wrapper.emitted('tagSelected')[0]).toEqual([mockTags[0]])
  })

  it('highlights selected tag with ring', async () => {
    invoke.mockResolvedValue(mockTags)
    const wrapper = mount(TagCloud)
    await flushPromises()

    const workTag = wrapper.findAll('button').find(btn => btn.text().includes('work'))
    await workTag.trigger('click')
    await wrapper.vm.$nextTick()

    expect(workTag.classes()).toContain('ring-2')
    expect(workTag.classes()).toContain('ring-white')
  })

  it('deselects tag when clicked again', async () => {
    invoke.mockResolvedValue(mockTags)
    const wrapper = mount(TagCloud)
    await flushPromises()

    const workTag = wrapper.findAll('button').find(btn => btn.text().includes('work'))
    await workTag.trigger('click')
    await wrapper.vm.$nextTick()

    expect(wrapper.vm.selectedTag).toBeTruthy()

    await workTag.trigger('click')
    await wrapper.vm.$nextTick()

    expect(wrapper.vm.selectedTag).toBeNull()
    expect(wrapper.emitted('tagSelected')[1]).toEqual([null])
  })

  it('shows clear filter button when tag selected', async () => {
    invoke.mockResolvedValue(mockTags)
    const wrapper = mount(TagCloud)
    await flushPromises()

    expect(wrapper.text()).not.toContain('Clear Filter')

    const workTag = wrapper.findAll('button').find(btn => btn.text().includes('work'))
    await workTag.trigger('click')
    await wrapper.vm.$nextTick()

    expect(wrapper.text()).toContain('Clear Filter')
  })

  it('clears selection when clear filter clicked', async () => {
    invoke.mockResolvedValue(mockTags)
    const wrapper = mount(TagCloud)
    await flushPromises()

    const workTag = wrapper.findAll('button').find(btn => btn.text().includes('work'))
    await workTag.trigger('click')
    await wrapper.vm.$nextTick()

    const clearButton = wrapper.findAll('button').find(btn => btn.text() === 'Clear Filter')
    await clearButton.trigger('click')

    expect(wrapper.vm.selectedTag).toBeNull()
  })

  it('handles load error gracefully', async () => {
    invoke.mockRejectedValue(new Error('Load failed'))
    const wrapper = mount(TagCloud)
    await flushPromises()

    expect(showError).toHaveBeenCalledWith(expect.stringContaining('Load failed'))
  })

  it('exposes loadTags method', async () => {
    invoke.mockResolvedValue(mockTags)
    const wrapper = mount(TagCloud)
    await flushPromises()

    expect(wrapper.vm.loadTags).toBeDefined()
    expect(typeof wrapper.vm.loadTags).toBe('function')
  })

  it('exposes requestDelete method', async () => {
    invoke.mockResolvedValue(mockTags)
    const wrapper = mount(TagCloud)
    await flushPromises()

    expect(wrapper.vm.requestDelete).toBeDefined()
    expect(typeof wrapper.vm.requestDelete).toBe('function')
  })

  it('shows delete confirmation modal when requestDelete called', async () => {
    invoke.mockResolvedValue(mockTags)
    const wrapper = mount(TagCloud)
    await flushPromises()

    wrapper.vm.requestDelete(mockTags[0])
    await wrapper.vm.$nextTick()

    expect(wrapper.vm.tagToDelete).toEqual(mockTags[0])
    expect(wrapper.text()).toContain('Delete Tag')
    expect(wrapper.text()).toContain('Are you sure you want to delete tag "work"?')
  })

  it('cancels delete operation', async () => {
    invoke.mockResolvedValue(mockTags)
    const wrapper = mount(TagCloud)
    await flushPromises()

    wrapper.vm.tagToDelete = mockTags[0]
    await wrapper.vm.$nextTick()

    const cancelButton = wrapper.findAll('button').find(btn => btn.text() === 'Cancel')
    await cancelButton.trigger('click')

    expect(wrapper.vm.tagToDelete).toBeNull()
  })

  it('deletes tag successfully', async () => {
    invoke.mockResolvedValueOnce(mockTags).mockResolvedValueOnce(undefined)
    const wrapper = mount(TagCloud)
    await flushPromises()

    wrapper.vm.tagToDelete = mockTags[0]
    await wrapper.vm.$nextTick()

    await wrapper.vm.confirmDelete()
    await flushPromises()

    expect(invoke).toHaveBeenCalledWith('delete_manual_tag', { id: 1 })
    expect(wrapper.vm.tags.length).toBe(2)
    expect(wrapper.vm.tagToDelete).toBeNull()
    expect(showSuccess).toHaveBeenCalledWith('Tag deleted')
  })

  it('clears selection when deleting selected tag', async () => {
    invoke.mockResolvedValueOnce(mockTags).mockResolvedValueOnce(undefined)
    const wrapper = mount(TagCloud)
    await flushPromises()

    wrapper.vm.selectedTag = mockTags[0]
    wrapper.vm.tagToDelete = mockTags[0]
    await wrapper.vm.$nextTick()

    await wrapper.vm.confirmDelete()
    await flushPromises()

    expect(wrapper.vm.selectedTag).toBeNull()
    expect(wrapper.emitted('tagSelected')).toBeTruthy()
  })

  it('handles delete error', async () => {
    invoke.mockResolvedValueOnce(mockTags).mockRejectedValueOnce(new Error('Delete failed'))
    const wrapper = mount(TagCloud)
    await flushPromises()

    wrapper.vm.tagToDelete = mockTags[0]
    await wrapper.vm.confirmDelete()
    await flushPromises()

    expect(showError).toHaveBeenCalledWith(expect.stringContaining('Delete failed'))
    expect(wrapper.vm.tags.length).toBe(3) // Tag not removed
  })

  it('disables delete button while deleting', async () => {
    invoke.mockResolvedValue(mockTags)
    const wrapper = mount(TagCloud)
    await flushPromises()

    wrapper.vm.tagToDelete = mockTags[0]
    wrapper.vm.isDeleting = true
    await wrapper.vm.$nextTick()

    // Find the modal delete button (red button in the modal)
    const redButtons = wrapper.findAll('button').filter(btn =>
      btn.classes().includes('bg-red-500')
    )
    const modalDeleteButton = redButtons.find(btn => btn.text() === 'Deleting...')
    expect(modalDeleteButton).toBeTruthy()
    expect(modalDeleteButton.attributes('disabled')).toBeDefined()
  })

  it('uses default gray color for unknown colors', () => {
    invoke.mockResolvedValue([])
    const wrapper = mount(TagCloud)

    const colorClass = wrapper.vm.getTagColor('unknown')
    // Unknown colors should return the default gray style
    expect(colorClass).toBe('bg-gray-500/30 text-gray-300 hover:bg-gray-500/50')
  })

  it('calculates tag size correctly for edge cases', () => {
    invoke.mockResolvedValue([])
    const wrapper = mount(TagCloud)

    expect(wrapper.vm.getTagSize({ usage_count: 0 })).toBe('text-xs')
    expect(wrapper.vm.getTagSize({ usage_count: 4 })).toBe('text-xs')
    expect(wrapper.vm.getTagSize({ usage_count: 5 })).toBe('text-sm')
    expect(wrapper.vm.getTagSize({ usage_count: 9 })).toBe('text-sm')
    expect(wrapper.vm.getTagSize({ usage_count: 10 })).toBe('text-base')
    expect(wrapper.vm.getTagSize({ usage_count: 100 })).toBe('text-base')
  })

  it('handles missing usage_count gracefully', () => {
    invoke.mockResolvedValue([])
    const wrapper = mount(TagCloud)

    const size = wrapper.vm.getTagSize({})
    expect(size).toBe('text-xs')
  })
})