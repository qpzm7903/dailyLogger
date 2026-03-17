import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import HistoryViewer from '../HistoryViewer.vue'

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

describe('HistoryViewer', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.useFakeTimers()
    vi.setSystemTime(new Date('2026-03-17T12:00:00Z'))
  })

  afterEach(() => {
    vi.restoreAllMocks()
    vi.useRealTimers()
  })

  const mockRecords = [
    {
      id: 1,
      timestamp: '2026-03-15T10:00:00Z',
      source_type: 'auto',
      content: JSON.stringify({ summary: 'Working on Rust code' }),
      screenshot_path: '/path/to/screenshot.png'
    },
    {
      id: 2,
      timestamp: '2026-03-16T14:30:00Z',
      source_type: 'manual',
      content: 'Manual note about meeting',
      screenshot_path: null
    }
  ]

  const mountComponent = (props = {}) => {
    return mount(HistoryViewer, {
      props,
      global: {
        stubs: {
          TagFilter: {
            template: '<div class="tag-filter-stub"></div>',
            props: ['modelValue'],
            emits: ['update:modelValue']
          },
          TagBadge: {
            template: '<span class="tag-badge-stub">{{ tag.name }}</span>',
            props: ['tag']
          }
        }
      }
    })
  }

  it('renders history viewer title', async () => {
    invoke.mockResolvedValue([])
    const wrapper = mountComponent()
    await flushPromises()
    expect(wrapper.text()).toContain('历史记录')
  })

  it('emits close event when close button clicked', async () => {
    invoke.mockResolvedValue([])
    const wrapper = mountComponent()
    await flushPromises()

    const closeButton = wrapper.find('button[class*="text-gray-400"]')
    await closeButton.trigger('click')

    expect(wrapper.emitted('close')).toBeTruthy()
  })

  it('initializes with last 7 days date range', async () => {
    invoke.mockResolvedValue([])
    const wrapper = mountComponent()
    await flushPromises()

    expect(wrapper.vm.startDate).toBe('2026-03-10')
    expect(wrapper.vm.endDate).toBe('2026-03-17')
  })

  it('loads records on mount', async () => {
    invoke.mockResolvedValueOnce(mockRecords).mockResolvedValueOnce({})
    const wrapper = mountComponent()
    await flushPromises()

    expect(invoke).toHaveBeenCalledWith('get_history_records', expect.objectContaining({
      startDate: '2026-03-10',
      endDate: '2026-03-17',
      sourceType: null,
      page: 0,
      pageSize: 50
    }))
  })

  it('displays loading state', async () => {
    let resolvePromise
    const promise = new Promise(resolve => { resolvePromise = resolve })
    invoke.mockReturnValue(promise)

    const wrapper = mountComponent()
    await wrapper.vm.$nextTick()

    expect(wrapper.text()).toContain('加载中...')

    resolvePromise([])
    await flushPromises()
  })

  it('displays empty state when no records', async () => {
    invoke.mockResolvedValue([])
    const wrapper = mountComponent()
    await flushPromises()

    expect(wrapper.text()).toContain('暂无记录')
  })

  it('displays records list', async () => {
    invoke.mockResolvedValueOnce(mockRecords).mockResolvedValueOnce({})
    const wrapper = mountComponent()
    await flushPromises()

    expect(wrapper.vm.records.length).toBe(2)
    expect(wrapper.text()).toContain('Working on Rust code')
    expect(wrapper.text()).toContain('Manual note about meeting')
  })

  it('displays source type badges correctly', async () => {
    invoke.mockResolvedValueOnce(mockRecords).mockResolvedValueOnce({})
    const wrapper = mountComponent()
    await flushPromises()

    const badges = wrapper.findAll('.px-2.py-0\\.5.rounded.text-xs')
    expect(badges[0].text()).toBe('自动')
    expect(badges[0].classes()).toContain('bg-blue-500/20')
    expect(badges[1].text()).toBe('手动')
    expect(badges[1].classes()).toContain('bg-green-500/20')
  })

  it('displays record count', async () => {
    invoke.mockResolvedValueOnce(mockRecords).mockResolvedValueOnce({})
    const wrapper = mountComponent()
    await flushPromises()

    expect(wrapper.text()).toContain('共 2 条')
  })

  it('filters by source type', async () => {
    invoke.mockResolvedValue([])
    const wrapper = mountComponent()
    await flushPromises()

    vi.clearAllMocks()

    const select = wrapper.find('select')
    await select.setValue('auto')
    await wrapper.find('button[class*="bg-primary"]').trigger('click')
    await flushPromises()

    expect(invoke).toHaveBeenCalledWith('get_history_records', expect.objectContaining({
      sourceType: 'auto'
    }))
  })

  it('filters by date range', async () => {
    invoke.mockResolvedValue([])
    const wrapper = mountComponent()
    await flushPromises()

    vi.clearAllMocks()

    const dateInputs = wrapper.findAll('input[type="date"]')
    await dateInputs[0].setValue('2026-03-01')
    await dateInputs[1].setValue('2026-03-10')
    await wrapper.find('button[class*="bg-primary"]').trigger('click')
    await flushPromises()

    expect(invoke).toHaveBeenCalledWith('get_history_records', expect.objectContaining({
      startDate: '2026-03-01',
      endDate: '2026-03-10'
    }))
  })

  it('disables query button while loading', async () => {
    let resolvePromise
    const promise = new Promise(resolve => { resolvePromise = resolve })
    invoke.mockReturnValue(promise)

    const wrapper = mountComponent()
    await wrapper.vm.$nextTick()

    const queryButton = wrapper.find('button[class*="bg-primary"]')
    expect(queryButton.attributes('disabled')).toBeDefined()
    expect(queryButton.text()).toBe('加载中...')

    resolvePromise([])
    await flushPromises()
  })

  it('formats timestamp correctly', async () => {
    invoke.mockResolvedValue([])
    const wrapper = mountComponent()
    await flushPromises()

    const formatted = wrapper.vm.formatTime('2026-03-17T14:30:00Z')
    expect(formatted).toMatch(/2026/)
    expect(formatted).toMatch(/03/)
    expect(formatted).toMatch(/17/)
  })

  it('truncates long content', async () => {
    invoke.mockResolvedValue([])
    const wrapper = mountComponent()
    await flushPromises()

    const longContent = 'a'.repeat(150)
    const truncated = wrapper.vm.truncateContent(longContent)
    expect(truncated.length).toBe(103) // 100 + '...'
    expect(truncated.endsWith('...')).toBe(true)
  })

  it('parses JSON content summary', async () => {
    invoke.mockResolvedValue([])
    const wrapper = mountComponent()
    await flushPromises()

    const content = JSON.stringify({ summary: 'Test summary' })
    const parsed = wrapper.vm.truncateContent(content)
    expect(parsed).toBe('Test summary')
  })

  it('parses JSON content note', async () => {
    invoke.mockResolvedValue([])
    const wrapper = mountComponent()
    await flushPromises()

    const content = JSON.stringify({ note: 'Test note' })
    const parsed = wrapper.vm.truncateContent(content)
    expect(parsed).toBe('Test note')
  })

  it('handles non-JSON content gracefully', async () => {
    invoke.mockResolvedValue([])
    const wrapper = mountComponent()
    await flushPromises()

    const plainText = 'Plain text content'
    const parsed = wrapper.vm.truncateContent(plainText)
    expect(parsed).toBe(plainText)
  })

  it('loads tags for records (batch query)', async () => {
    const tagsMap = {
      1: [{ id: 1, name: 'work', color: 'blue' }],
      2: [{ id: 2, name: 'meeting', color: 'green' }]
    }

    invoke.mockResolvedValueOnce(mockRecords).mockResolvedValueOnce(tagsMap)
    const wrapper = mountComponent()
    await flushPromises()

    expect(invoke).toHaveBeenCalledWith('get_tags_for_records', {
      recordIds: [1, 2]
    })
    expect(wrapper.vm.recordTags).toEqual(tagsMap)
  })

  it('filters by selected tags', async () => {
    const tag = { id: 1, name: 'work', color: 'blue' }
    invoke.mockResolvedValue([])

    const wrapper = mountComponent()
    await flushPromises()

    vi.clearAllMocks()

    wrapper.vm.selectedTags = [tag]
    await wrapper.vm.$nextTick()
    await flushPromises()

    expect(invoke).toHaveBeenCalledWith('get_records_by_manual_tags', expect.objectContaining({
      tagIds: [1]
    }))
  })

  it('applies initial tag filter from prop (FIX-003)', async () => {
    const initialTag = { id: 1, name: 'work', color: 'blue' }
    invoke.mockResolvedValue([])

    const wrapper = mountComponent({ initialTag })
    await flushPromises()

    expect(wrapper.vm.selectedTags).toEqual([initialTag])
    expect(invoke).toHaveBeenCalledWith('get_records_by_manual_tags', expect.objectContaining({
      tagIds: [1]
    }))
  })

  it('loads more records on scroll to bottom', async () => {
    const page1 = Array.from({ length: 50 }, (_, i) => ({
      id: i + 1,
      timestamp: '2026-03-15T10:00:00Z',
      source_type: 'auto',
      content: `Record ${i + 1}`,
      screenshot_path: null
    }))

    const page2 = Array.from({ length: 20 }, (_, i) => ({
      id: i + 51,
      timestamp: '2026-03-15T11:00:00Z',
      source_type: 'auto',
      content: `Record ${i + 51}`,
      screenshot_path: null
    }))

    invoke.mockResolvedValueOnce(page1)
      .mockResolvedValueOnce({})
      .mockResolvedValueOnce(page2)
      .mockResolvedValueOnce({})

    const wrapper = mountComponent()
    await flushPromises()

    expect(wrapper.vm.records.length).toBe(50)
    expect(wrapper.vm.hasMore).toBe(true)

    // Simulate scroll to bottom
    wrapper.vm.scrollContainer = {
      scrollTop: 900,
      scrollHeight: 1000,
      clientHeight: 50
    }
    wrapper.vm.handleScroll()
    await flushPromises()

    expect(invoke).toHaveBeenCalledWith('get_history_records', expect.objectContaining({
      page: 1
    }))
    expect(wrapper.vm.records.length).toBe(70)
  })

  it('displays loading indicator when loading more', async () => {
    invoke.mockResolvedValueOnce(mockRecords).mockResolvedValueOnce({})
    const wrapper = mountComponent()
    await flushPromises()

    wrapper.vm.isLoadingMore = true
    await wrapper.vm.$nextTick()

    expect(wrapper.text()).toContain('加载更多...')
  })

  it('does not load more when already loading', async () => {
    invoke.mockResolvedValueOnce(mockRecords).mockResolvedValueOnce({})
    const wrapper = mountComponent()
    await flushPromises()

    wrapper.vm.isLoadingMore = true
    const initialCallCount = invoke.mock.calls.length

    await wrapper.vm.loadMoreRecords()

    expect(invoke.mock.calls.length).toBe(initialCallCount)
  })

  it('does not load more when no more pages', async () => {
    invoke.mockResolvedValueOnce(mockRecords).mockResolvedValueOnce({})
    const wrapper = mountComponent()
    await flushPromises()

    wrapper.vm.hasMore = false
    const initialCallCount = invoke.mock.calls.length

    await wrapper.vm.loadMoreRecords()

    expect(invoke.mock.calls.length).toBe(initialCallCount)
  })

  it('does not load more when tag filter is active', async () => {
    const tag = { id: 1, name: 'work', color: 'blue' }
    invoke.mockResolvedValue([])

    const wrapper = mountComponent()
    await flushPromises()

    wrapper.vm.selectedTags = [tag]
    await flushPromises()

    const initialCallCount = invoke.mock.calls.length

    await wrapper.vm.loadMoreRecords()

    expect(invoke.mock.calls.length).toBe(initialCallCount)
  })

  it('shows delete confirmation modal', async () => {
    invoke.mockResolvedValueOnce(mockRecords).mockResolvedValueOnce({})
    const wrapper = mountComponent()
    await flushPromises()

    const deleteButton = wrapper.findAll('button').find(btn => btn.text() === '删除')
    await deleteButton.trigger('click')

    expect(wrapper.vm.recordToDelete).toBeTruthy()
    expect(wrapper.text()).toContain('确认删除')
    expect(wrapper.text()).toContain('确定要删除这条记录吗？')
  })

  it('cancels delete operation', async () => {
    invoke.mockResolvedValueOnce(mockRecords).mockResolvedValueOnce({})
    const wrapper = mountComponent()
    await flushPromises()

    wrapper.vm.recordToDelete = mockRecords[0]
    await wrapper.vm.$nextTick()

    const cancelButton = wrapper.findAll('button').find(btn => btn.text() === '取消')
    await cancelButton.trigger('click')

    expect(wrapper.vm.recordToDelete).toBeNull()
  })

  it('deletes record successfully', async () => {
    invoke.mockResolvedValueOnce(mockRecords).mockResolvedValueOnce({}).mockResolvedValueOnce(undefined)
    const wrapper = mountComponent()
    await flushPromises()

    wrapper.vm.recordToDelete = mockRecords[0]
    await wrapper.vm.$nextTick()

    // Call deleteRecord directly
    await wrapper.vm.deleteRecord()
    await flushPromises()

    expect(invoke).toHaveBeenCalledWith('delete_record', { id: 1 })
    expect(wrapper.vm.records.length).toBe(1)
    expect(wrapper.vm.recordToDelete).toBeNull()
    expect(showSuccess).toHaveBeenCalledWith('记录已删除')
  })

  it('handles delete error', async () => {
    invoke.mockResolvedValueOnce(mockRecords)
      .mockResolvedValueOnce({})
      .mockRejectedValueOnce(new Error('Delete failed'))

    const wrapper = mountComponent()
    await flushPromises()

    wrapper.vm.recordToDelete = mockRecords[0]
    await wrapper.vm.deleteRecord()
    await flushPromises()

    expect(showError).toHaveBeenCalledWith(expect.stringContaining('Delete failed'))
    expect(wrapper.vm.records.length).toBe(2) // Record not removed
  })

  it('disables delete button while deleting', async () => {
    invoke.mockResolvedValueOnce(mockRecords).mockResolvedValueOnce({})
    const wrapper = mountComponent()
    await flushPromises()

    wrapper.vm.recordToDelete = mockRecords[0]
    wrapper.vm.isDeleting = true
    await wrapper.vm.$nextTick()

    const deleteButtons = wrapper.findAll('button').filter(btn => btn.text().includes('删除'))
    const modalDeleteButton = deleteButtons.find(btn => btn.text() === '删除中...')
    expect(modalDeleteButton).toBeTruthy()
    expect(modalDeleteButton.attributes('disabled')).toBeDefined()
  })

  it('handles load error gracefully', async () => {
    invoke.mockRejectedValueOnce(new Error('Load failed'))
    const wrapper = mountComponent()
    await flushPromises()

    expect(showError).toHaveBeenCalledWith(expect.stringContaining('Load failed'))
  })

  it('reverts page increment on load more error', async () => {
    invoke.mockResolvedValueOnce(mockRecords)
      .mockResolvedValueOnce({})
      .mockRejectedValueOnce(new Error('Load more failed'))

    const wrapper = mountComponent()
    await flushPromises()

    wrapper.vm.hasMore = true
    const initialPage = wrapper.vm.page

    await wrapper.vm.loadMoreRecords()
    await flushPromises()

    expect(wrapper.vm.page).toBe(initialPage)
    expect(showError).toHaveBeenCalledWith(expect.stringContaining('Load more failed'))
  })

  it('resets pagination when reloading records', async () => {
    invoke.mockResolvedValue([])
    const wrapper = mountComponent()
    await flushPromises()

    wrapper.vm.page = 5
    await wrapper.vm.loadRecords()
    await flushPromises()

    expect(wrapper.vm.page).toBe(0)
  })

  it('clears records when reloading', async () => {
    invoke.mockResolvedValueOnce(mockRecords)
      .mockResolvedValueOnce({})
      .mockResolvedValueOnce([])
      .mockResolvedValueOnce({})

    const wrapper = mountComponent()
    await flushPromises()

    expect(wrapper.vm.records.length).toBe(2)

    await wrapper.vm.loadRecords()
    await flushPromises()

    expect(wrapper.vm.records.length).toBe(0)
  })
})
