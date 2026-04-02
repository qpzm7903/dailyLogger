import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount, flushPromises, config as vtuConfig } from '@vue/test-utils'
import TagCloud from '../TagCloud.vue'
import BaseModal from '../BaseModal.vue'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

vi.mock('../../stores/toast', () => ({
  showError: vi.fn()
}))

import { invoke } from '@tauri-apps/api/core'
import { showError } from '../../stores/toast'

describe('TagCloud', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vtuConfig.global.components = { ...vtuConfig.global.components, BaseModal }
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  const mockTags = [
    { id: 1, name: '开发', color: 'blue', usage_count: 15 },
    { id: 2, name: '会议', color: 'purple', usage_count: 8 },
    { id: 3, name: '未知标签', color: 'red', usage_count: 3 }
  ]

  it('renders tag cloud title', async () => {
    vi.mocked(invoke).mockResolvedValue([])
    const wrapper = mount(TagCloud)
    await flushPromises()

    expect(wrapper.text()).toContain('Tag Cloud')
  })

  it('emits close event when close button clicked', async () => {
    vi.mocked(invoke).mockResolvedValue([])
    const wrapper = mount(TagCloud)
    await flushPromises()

    const closeButton = wrapper.findAll('button').find(b => b.text() === '✕')
    await closeButton?.trigger('click')

    expect(wrapper.emitted('close')).toBeTruthy()
  })

  it('loads record tags on mount', async () => {
    vi.mocked(invoke).mockResolvedValue(mockTags)
    const wrapper = mount(TagCloud)
    await flushPromises()

    expect(invoke).toHaveBeenCalledWith('get_tag_cloud_tags')
    expect(wrapper.vm.tags).toEqual(mockTags)
  })

  it('displays loading state', async () => {
    let resolvePromise: (value: unknown) => void
    vi.mocked(invoke).mockImplementation(() =>
      new Promise((resolve) => {
        resolvePromise = resolve
      })
    )

    const wrapper = mount(TagCloud)
    await wrapper.vm.$nextTick()

    expect(wrapper.text()).toContain('Loading...')

    resolvePromise!([])
    await flushPromises()
  })

  it('displays empty state when no tags', async () => {
    vi.mocked(invoke).mockResolvedValue([])
    const wrapper = mount(TagCloud)
    await flushPromises()

    expect(wrapper.text()).toContain('No tags yet')
    expect(wrapper.text()).toContain('AI analysis or manual tagging')
  })

  it('displays tags with usage counts', async () => {
    vi.mocked(invoke).mockResolvedValue(mockTags)
    const wrapper = mount(TagCloud)
    await flushPromises()

    expect(wrapper.text()).toContain('开发')
    expect(wrapper.text()).toContain('15')
    expect(wrapper.text()).toContain('会议')
    expect(wrapper.text()).toContain('8')
  })

  it('applies correct size based on usage count', async () => {
    vi.mocked(invoke).mockResolvedValue(mockTags)
    const wrapper = mount(TagCloud)
    await flushPromises()

    const tagButtons = wrapper.findAll('button').filter(btn =>
      btn.text().includes('开发') || btn.text().includes('会议') || btn.text().includes('未知标签')
    )

    expect(tagButtons[0].classes()).toContain('text-base')
    expect(tagButtons[1].classes()).toContain('text-sm')
    expect(tagButtons[2].classes()).toContain('text-xs')
  })

  it('uses tag-name-based colors', async () => {
    vi.mocked(invoke).mockResolvedValue(mockTags)
    const wrapper = mount(TagCloud)
    await flushPromises()

    const devTag = wrapper.findAll('button').find(btn => btn.text().includes('开发'))
    const meetingTag = wrapper.findAll('button').find(btn => btn.text().includes('会议'))

    expect(devTag?.classes()).toContain('bg-blue-500/20')
    expect(meetingTag?.classes()).toContain('bg-purple-500/20')
  })

  it('selects tag when clicked and emits normalized tag object', async () => {
    vi.mocked(invoke).mockResolvedValue(mockTags)
    const wrapper = mount(TagCloud)
    await flushPromises()

    const devTag = wrapper.findAll('button').find(btn => btn.text().includes('开发'))
    await devTag?.trigger('click')

    expect(wrapper.vm.selectedTag).toEqual(mockTags[0])
    expect(wrapper.emitted('tagSelected')).toBeTruthy()
    expect(wrapper.emitted('tagSelected')?.[0]).toEqual([{
      id: 1,
      name: '开发',
      color: 'blue',
      category_id: null
    }])
  })

  it('clears selection when selected tag is clicked again', async () => {
    vi.mocked(invoke).mockResolvedValue(mockTags)
    const wrapper = mount(TagCloud)
    await flushPromises()

    const devTag = wrapper.findAll('button').find(btn => btn.text().includes('开发'))
    await devTag?.trigger('click')
    await devTag?.trigger('click')

    expect(wrapper.vm.selectedTag).toBeNull()
    expect(wrapper.emitted('tagSelected')?.[1]).toEqual([null])
  })

  it('clears selection via clear button', async () => {
    vi.mocked(invoke).mockResolvedValue(mockTags)
    const wrapper = mount(TagCloud)
    await flushPromises()

    const devTag = wrapper.findAll('button').find(btn => btn.text().includes('开发'))
    await devTag?.trigger('click')
    await wrapper.vm.$nextTick()

    const clearButton = wrapper.findAll('button').find(btn => btn.text() === 'Clear Filter')
    await clearButton?.trigger('click')

    expect(wrapper.vm.selectedTag).toBeNull()
    expect(wrapper.emitted('tagSelected')?.[1]).toEqual([null])
  })

  it('exposes loadTags method', async () => {
    vi.mocked(invoke).mockResolvedValue(mockTags)
    const wrapper = mount(TagCloud)
    await flushPromises()

    expect(wrapper.vm.loadTags).toBeDefined()
    expect(typeof wrapper.vm.loadTags).toBe('function')
  })

  it('handles load error gracefully', async () => {
    vi.mocked(invoke).mockRejectedValue(new Error('Load failed'))
    const wrapper = mount(TagCloud)
    await flushPromises()

    expect(showError).toHaveBeenCalledWith(expect.stringContaining('Load failed'))
    expect(wrapper.text()).toContain('No tags yet')
  })
})
