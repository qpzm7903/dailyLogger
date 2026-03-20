import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import SearchPanel from '../SearchPanel.vue'

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

// Mock toast store
vi.mock('../../stores/toast', () => ({
  showError: vi.fn()
}))

describe('SearchPanel', () => {
  let invokeMock
  let showErrorMock

  beforeEach(async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    const { showError } = await import('../../stores/toast')
    invokeMock = invoke
    showErrorMock = showError
    invokeMock.mockClear()
    showErrorMock.mockClear()
  })

  it('renders modal with title', () => {
    const wrapper = mount(SearchPanel)
    expect(wrapper.text()).toContain('Full-text Search')
  })

  it('shows initial empty state', () => {
    const wrapper = mount(SearchPanel)
    expect(wrapper.text()).toContain('Enter keywords to start search')
  })

  it('emits close event when close button is clicked', async () => {
    const wrapper = mount(SearchPanel)
    const closeButton = wrapper.findAll('button').find(btn => btn.text() === '✕')
    await closeButton.trigger('click')
    expect(wrapper.emitted('close')).toBeTruthy()
  })

  it('emits close event when clicking backdrop', async () => {
    const wrapper = mount(SearchPanel)
    const backdrop = wrapper.find('.fixed')
    await backdrop.trigger('click.self')
    expect(wrapper.emitted('close')).toBeTruthy()
  })

  it('updates search query when typing', async () => {
    const wrapper = mount(SearchPanel)
    const input = wrapper.find('input[type="text"]')
    await input.setValue('test query')
    expect(wrapper.vm.searchQuery).toBe('test query')
  })

  it('shows clear button when search query is not empty', async () => {
    const wrapper = mount(SearchPanel)
    const input = wrapper.find('input[type="text"]')

    // Initially no clear button
    expect(wrapper.findAll('button').filter(btn => btn.text() === '✕' && btn.classes().includes('absolute')).length).toBe(0)

    // After typing, clear button appears
    await input.setValue('test')
    await wrapper.vm.$nextTick()
    expect(wrapper.findAll('button').filter(btn => btn.text() === '✕' && btn.classes().includes('absolute')).length).toBe(1)
  })

  it('clears search when clear button is clicked', async () => {
    const wrapper = mount(SearchPanel)
    const input = wrapper.find('input[type="text"]')
    await input.setValue('test query')

    const clearButton = wrapper.findAll('button').find(btn => btn.text() === '✕' && btn.classes().includes('absolute'))
    await clearButton.trigger('click')

    expect(wrapper.vm.searchQuery).toBe('')
    expect(wrapper.vm.results).toEqual([])
    expect(wrapper.vm.hasSearched).toBe(false)
  })

  it('disables search button when query is empty', () => {
    const wrapper = mount(SearchPanel)
    const searchButton = wrapper.findAll('button').find(btn => btn.text().includes('Search'))
    expect(searchButton.attributes('disabled')).toBeDefined()
  })

  it('enables search button when query is not empty', async () => {
    const wrapper = mount(SearchPanel)
    const input = wrapper.find('input[type="text"]')
    await input.setValue('test')
    await wrapper.vm.$nextTick()

    const searchButton = wrapper.findAll('button').find(btn => btn.text().includes('Search'))
    expect(searchButton.attributes('disabled')).toBeUndefined()
  })

  it('performs search when search button is clicked', async () => {
    const mockResults = [
      {
        record: {
          id: 1,
          timestamp: '2024-01-01T10:00:00Z',
          source_type: 'auto',
          content: 'Test content'
        },
        rank: 0.95,
        snippet: 'Test <mark>content</mark>'
      }
    ]
    invokeMock.mockResolvedValue(mockResults)

    const wrapper = mount(SearchPanel)
    const input = wrapper.find('input[type="text"]')
    await input.setValue('test')

    const searchButton = wrapper.findAll('button').find(btn => btn.text().includes('Search'))
    await searchButton.trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 10))

    expect(invokeMock).toHaveBeenCalledWith('search_records', {
      query: 'test',
      orderBy: 'rank',
      limit: 200
    })
    expect(wrapper.vm.results).toEqual(mockResults)
  })

  it('performs search when Enter key is pressed', async () => {
    const mockResults = []
    invokeMock.mockResolvedValue(mockResults)

    const wrapper = mount(SearchPanel)
    const input = wrapper.find('input[type="text"]')
    await input.setValue('test')
    await input.trigger('keyup.enter')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 10))

    expect(invokeMock).toHaveBeenCalled()
  })

  it('shows loading state during search', async () => {
    invokeMock.mockImplementation(() => new Promise(resolve => setTimeout(() => resolve([]), 100)))

    const wrapper = mount(SearchPanel)
    const input = wrapper.find('input[type="text"]')
    await input.setValue('test')

    const searchButton = wrapper.findAll('button').find(btn => btn.text().includes('Search'))
    await searchButton.trigger('click')

    expect(wrapper.text()).toContain('Searching...')
  })

  it('shows no results message when search returns empty', async () => {
    invokeMock.mockResolvedValue([])

    const wrapper = mount(SearchPanel)
    const input = wrapper.find('input[type="text"]')
    await input.setValue('test')

    const searchButton = wrapper.findAll('button').find(btn => btn.text().includes('Search'))
    await searchButton.trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 10))

    expect(wrapper.text()).toContain('No matching records found')
  })

  it('displays search results', async () => {
    const mockResults = [
      {
        record: {
          id: 1,
          timestamp: '2024-01-01T10:00:00Z',
          source_type: 'auto',
          content: 'Test content'
        },
        rank: 0.95,
        snippet: 'Test <mark>content</mark>'
      }
    ]
    invokeMock.mockResolvedValue(mockResults)

    const wrapper = mount(SearchPanel)
    const input = wrapper.find('input[type="text"]')
    await input.setValue('test')

    const searchButton = wrapper.findAll('button').find(btn => btn.text().includes('Search'))
    await searchButton.trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 10))

    expect(wrapper.text()).toContain('1 results')
    expect(wrapper.html()).toContain('Test <mark>content</mark>')
  })

  it('shows error message when search fails', async () => {
    invokeMock.mockRejectedValue(new Error('Search failed'))

    const wrapper = mount(SearchPanel)
    const input = wrapper.find('input[type="text"]')
    await input.setValue('test')

    const searchButton = wrapper.findAll('button').find(btn => btn.text().includes('Search'))
    await searchButton.trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 10))

    expect(showErrorMock).toHaveBeenCalledWith(expect.stringContaining('Search failed'))
  })

  it('shows sort options when results exist', async () => {
    const mockResults = [
      {
        record: {
          id: 1,
          timestamp: '2024-01-01T10:00:00Z',
          source_type: 'auto',
          content: 'Test'
        },
        rank: 0.95,
        snippet: 'Test'
      }
    ]
    invokeMock.mockResolvedValue(mockResults)

    const wrapper = mount(SearchPanel)
    const input = wrapper.find('input[type="text"]')
    await input.setValue('test')

    const searchButton = wrapper.findAll('button').find(btn => btn.text().includes('Search'))
    await searchButton.trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 10))

    expect(wrapper.text()).toContain('Sort by:')
    expect(wrapper.text()).toContain('Relevance')
    expect(wrapper.text()).toContain('Time')
  })

  it('switches sort order when clicking sort buttons', async () => {
    const mockResults = [
      {
        record: {
          id: 1,
          timestamp: '2024-01-01T10:00:00Z',
          source_type: 'auto',
          content: 'Test'
        },
        rank: 0.95,
        snippet: 'Test'
      }
    ]
    invokeMock.mockResolvedValue(mockResults)

    const wrapper = mount(SearchPanel)
    const input = wrapper.find('input[type="text"]')
    await input.setValue('test')

    const searchButton = wrapper.findAll('button').find(btn => btn.text().includes('Search'))
    await searchButton.trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 10))

    // Initially sorted by rank
    expect(wrapper.vm.orderBy).toBe('rank')

    // Click time sort button
    const timeButton = wrapper.findAll('button').find(btn => btn.text() === 'Time')
    await timeButton.trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 10))

    expect(wrapper.vm.orderBy).toBe('time')
    // Should trigger a new search
    expect(invokeMock).toHaveBeenCalledTimes(2)
  })

  it('formats timestamp correctly', () => {
    const wrapper = mount(SearchPanel)
    const timestamp = '2024-01-15T14:30:00Z'
    const formatted = wrapper.vm.formatTime(timestamp)
    expect(formatted).toMatch(/2024/)
    expect(formatted).toMatch(/01/)
    expect(formatted).toMatch(/15/)
  })

  it('displays source type badge correctly for auto records', async () => {
    const mockResults = [
      {
        record: {
          id: 1,
          timestamp: '2024-01-01T10:00:00Z',
          source_type: 'auto',
          content: 'Test'
        },
        rank: 0.95,
        snippet: 'Test'
      }
    ]
    invokeMock.mockResolvedValue(mockResults)

    const wrapper = mount(SearchPanel)
    const input = wrapper.find('input[type="text"]')
    await input.setValue('test')

    const searchButton = wrapper.findAll('button').find(btn => btn.text().includes('Search'))
    await searchButton.trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 10))

    expect(wrapper.text()).toContain('Auto')
  })

  it('displays source type badge correctly for manual records', async () => {
    const mockResults = [
      {
        record: {
          id: 1,
          timestamp: '2024-01-01T10:00:00Z',
          source_type: 'manual',
          content: 'Test'
        },
        rank: 0.95,
        snippet: 'Test'
      }
    ]
    invokeMock.mockResolvedValue(mockResults)

    const wrapper = mount(SearchPanel)
    const input = wrapper.find('input[type="text"]')
    await input.setValue('test')

    const searchButton = wrapper.findAll('button').find(btn => btn.text().includes('Search'))
    await searchButton.trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 10))

    expect(wrapper.text()).toContain('Manual')
  })

  it('shows rank score when sorted by rank', async () => {
    const mockResults = [
      {
        record: {
          id: 1,
          timestamp: '2024-01-01T10:00:00Z',
          source_type: 'auto',
          content: 'Test'
        },
        rank: 0.95,
        snippet: 'Test'
      }
    ]
    invokeMock.mockResolvedValue(mockResults)

    const wrapper = mount(SearchPanel)
    const input = wrapper.find('input[type="text"]')
    await input.setValue('test')

    const searchButton = wrapper.findAll('button').find(btn => btn.text().includes('Search'))
    await searchButton.trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 10))

    expect(wrapper.text()).toContain('Relevance: 0.95')
  })
})
