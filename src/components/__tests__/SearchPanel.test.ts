import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import SearchPanel from '../SearchPanel.vue'

// Mock Tauri APIs
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

// Mock vue-i18n
vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string, params?: Record<string, unknown>) => {
      const translations: Record<string, string> = {
        'searchPanel.title': 'Search Records',
        'searchPanel.placeholder': 'Enter search keywords...',
        'searchPanel.search': 'Search',
        'searchPanel.searching': 'Searching...',
        'searchPanel.sortBy': 'Sort by:',
        'searchPanel.relevance': 'Relevance',
        'searchPanel.time': 'Time',
        'searchPanel.totalResults': `${params?.count || 0} results`,
        'searchPanel.noResults': 'No results found',
        'searchPanel.startHint': 'Enter keywords to search',
        'searchPanel.auto': 'Auto',
        'searchPanel.manual': 'Manual',
        'searchPanel.relevanceScore': `Score: ${params?.rank || 0}`,
        'searchPanel.searchFailed': `Search failed: ${params?.error || 'unknown'}`,
        'emptyState.screenshots': 'No screenshots yet',
        'emptyState.dailyReport': 'No daily report generated yet',
        'emptyState.searchResults': 'No matching records found',
        'emptyState.generic': 'Nothing here yet',
      }
      return translations[key] || key
    },
  }),
}))

// Mock toast store
vi.mock('../../stores/toast', () => ({
  showError: vi.fn(),
}))

// Mock @tanstack/vue-virtual
vi.mock('@tanstack/vue-virtual', () => ({
  useVirtualizer: vi.fn(() => ({
    value: {
      getTotalSize: () => 7200,
      getVirtualItems: () => [
        { index: 0, size: 72, start: 0 },
        { index: 1, size: 72, start: 72 },
      ],
    },
  })),
}))

describe('SearchPanel', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('rendering', () => {
    it('renders modal with title', () => {
      const wrapper = mount(SearchPanel)

      expect(wrapper.find('h2').text()).toContain('Search Records')
    })

    it('shows search input', () => {
      const wrapper = mount(SearchPanel)

      expect(wrapper.find('input[type="text"]').exists()).toBe(true)
    })

    it('shows search button', () => {
      const wrapper = mount(SearchPanel)

      const buttons = wrapper.findAll('button')
      const searchBtn = buttons.find((b) => b.text().includes('Search'))
      expect(searchBtn).toBeDefined()
    })

    it('shows start hint initially', () => {
      const wrapper = mount(SearchPanel)

      expect(wrapper.text()).toContain('Enter keywords to search')
    })

    it('shows close button', () => {
      const wrapper = mount(SearchPanel)

      const closeBtn = wrapper.findAll('button').find((b) => b.text().includes('✕'))
      expect(closeBtn).toBeDefined()
    })
  })

  describe('search input', () => {
    it('updates searchQuery when typing', async () => {
      const wrapper = mount(SearchPanel)

      const input = wrapper.find('input[type="text"]')
      await input.setValue('test query')

      expect(wrapper.vm.searchQuery).toBe('test query')
    })

    it('shows clear button when input has value', async () => {
      const wrapper = mount(SearchPanel)

      // No clear button initially
      expect(wrapper.findAll('button').filter((b) => b.classes().includes('absolute')).length).toBe(0)

      await wrapper.find('input[type="text"]').setValue('test')

      // Clear button should appear
      expect(wrapper.find('.absolute.right-3').exists()).toBe(true)
    })

    it('clears search when clear button clicked', async () => {
      const wrapper = mount(SearchPanel)

      await wrapper.find('input[type="text"]').setValue('test')
      await wrapper.find('.absolute.right-3').trigger('click')

      expect(wrapper.vm.searchQuery).toBe('')
      expect(wrapper.vm.results).toEqual([])
      expect(wrapper.vm.hasSearched).toBe(false)
    })
  })

  describe('search functionality', () => {
    it('calls search on button click', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue([])

      const wrapper = mount(SearchPanel)

      await wrapper.find('input[type="text"]').setValue('test query')
      const searchBtn = wrapper.findAll('button').find((b) => b.text().includes('Search'))
      await searchBtn?.trigger('click')

      expect(invoke).toHaveBeenCalledWith('search_records', {
        query: 'test query',
        orderBy: 'rank',
        limit: 200,
      })
    })

    it('calls search on Enter key', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue([])

      const wrapper = mount(SearchPanel)

      await wrapper.find('input[type="text"]').setValue('test')
      await wrapper.find('input[type="text"]').trigger('keyup.enter')

      expect(invoke).toHaveBeenCalledWith('search_records', {
        query: 'test',
        orderBy: 'rank',
        limit: 200,
      })
    })

    it('disables search button when query is empty', () => {
      const wrapper = mount(SearchPanel)

      const searchBtn = wrapper.findAll('button').find((b) => b.text().includes('Search'))
      expect(searchBtn?.attributes('disabled')).toBeDefined()
    })

    it('disables search button when query is only whitespace', async () => {
      const wrapper = mount(SearchPanel)

      await wrapper.find('input[type="text"]').setValue('   ')

      const searchBtn = wrapper.findAll('button').find((b) => b.text().includes('Search'))
      expect(searchBtn?.attributes('disabled')).toBeDefined()
    })

    it('shows loading state while searching', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockImplementation(() => new Promise(() => {})) // Never resolves

      const wrapper = mount(SearchPanel)

      await wrapper.find('input[type="text"]').setValue('test')
      const searchBtn = wrapper.findAll('button').find((b) => b.text().includes('Search'))
      await searchBtn?.trigger('click')

      expect(wrapper.text()).toContain('Searching...')
    })

    it('shows no results when search returns empty', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue([])

      const wrapper = mount(SearchPanel)

      await wrapper.find('input[type="text"]').setValue('test')
      await wrapper.find('input[type="text"]').trigger('keyup.enter')

      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('No matching records found')
      })
    })
  })

  describe('results display', () => {
    it('displays search results', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      const mockResults = [
        {
          record: {
            id: 1,
            timestamp: '2024-01-01T10:00:00Z',
            source_type: 'auto',
            content: 'Test content 1',
          },
          snippet: 'Test <mark>content</mark> 1',
          rank: 0.95,
        },
        {
          record: {
            id: 2,
            timestamp: '2024-01-01T11:00:00Z',
            source_type: 'manual',
            content: 'Test content 2',
          },
          snippet: 'Test <mark>content</mark> 2',
          rank: 0.85,
        },
      ]
      vi.mocked(invoke).mockResolvedValue(mockResults)

      const wrapper = mount(SearchPanel)

      await wrapper.find('input[type="text"]').setValue('test')
      await wrapper.find('input[type="text"]').trigger('keyup.enter')

      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('Test content 1')
        expect(wrapper.text()).toContain('Test content 2')
      })
    })

    it('shows source type badge for results', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      const mockResults = [
        {
          record: {
            id: 1,
            timestamp: '2024-01-01T10:00:00Z',
            source_type: 'auto',
            content: 'Auto content',
          },
          snippet: 'Auto content',
          rank: 0.9,
        },
        {
          record: {
            id: 2,
            timestamp: '2024-01-01T11:00:00Z',
            source_type: 'manual',
            content: 'Manual content',
          },
          snippet: 'Manual content',
          rank: 0.8,
        },
      ]
      vi.mocked(invoke).mockResolvedValue(mockResults)

      const wrapper = mount(SearchPanel)

      await wrapper.find('input[type="text"]').setValue('test')
      await wrapper.find('input[type="text"]').trigger('keyup.enter')

      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('Auto')
        expect(wrapper.text()).toContain('Manual')
      })
    })

    it('shows total results count', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue([
        {
          record: { id: 1, timestamp: '2024-01-01T10:00:00Z', source_type: 'auto', content: 'Content' },
          snippet: 'Content',
          rank: 0.9,
        },
      ])

      const wrapper = mount(SearchPanel)

      await wrapper.find('input[type="text"]').setValue('test')
      await wrapper.find('input[type="text"]').trigger('keyup.enter')

      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('1 results')
      })
    })
  })

  describe('sort functionality', () => {
    it('shows sort options after search', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue([
        {
          record: { id: 1, timestamp: '2024-01-01T10:00:00Z', source_type: 'auto', content: 'Content' },
          snippet: 'Content',
          rank: 0.9,
        },
      ])

      const wrapper = mount(SearchPanel)

      await wrapper.find('input[type="text"]').setValue('test')
      await wrapper.find('input[type="text"]').trigger('keyup.enter')

      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('Sort by:')
        expect(wrapper.text()).toContain('Relevance')
        expect(wrapper.text()).toContain('Time')
      })
    })

    it('has relevance selected by default', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue([
        {
          record: { id: 1, timestamp: '2024-01-01T10:00:00Z', source_type: 'auto', content: 'Content' },
          snippet: 'Content',
          rank: 0.9,
        },
      ])

      const wrapper = mount(SearchPanel)

      await wrapper.find('input[type="text"]').setValue('test')
      await wrapper.find('input[type="text"]').trigger('keyup.enter')

      await vi.waitFor(() => {
        expect(wrapper.vm.orderBy).toBe('rank')
      })
    })

    it('switches sort order when clicking time button', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue([
        {
          record: { id: 1, timestamp: '2024-01-01T10:00:00Z', source_type: 'auto', content: 'Content' },
          snippet: 'Content',
          rank: 0.9,
        },
      ])

      const wrapper = mount(SearchPanel)

      await wrapper.find('input[type="text"]').setValue('test')
      await wrapper.find('input[type="text"]').trigger('keyup.enter')

      await vi.waitFor(() => {
        expect(invoke).toHaveBeenCalled()
      })

      // Click Time sort button
      const timeBtn = wrapper.findAll('button').find((b) => b.text().trim() === 'Time')
      await timeBtn?.trigger('click')

      expect(wrapper.vm.orderBy).toBe('time')
      // Should re-search with new order
      expect(invoke).toHaveBeenCalledWith('search_records', expect.objectContaining({
        orderBy: 'time',
      }))
    })
  })

  describe('close functionality', () => {
    it('emits close when close button clicked', async () => {
      const wrapper = mount(SearchPanel)

      const closeBtn = wrapper.findAll('button').find((b) => b.text().includes('✕') && b.classes().includes('text-gray-400'))
      await closeBtn?.trigger('click')

      expect(wrapper.emitted('close')).toBeTruthy()
    })

    it('emits close when clicking backdrop', async () => {
      const wrapper = mount(SearchPanel)

      await wrapper.find('.fixed.inset-0').trigger('click')

      expect(wrapper.emitted('close')).toBeTruthy()
    })
  })

  describe('modal structure', () => {
    it('has proper modal styling', () => {
      const wrapper = mount(SearchPanel)

      expect(wrapper.find('.fixed.inset-0.bg-black\\/80').exists()).toBe(true)
      expect(wrapper.find('.bg-dark.rounded-2xl').exists()).toBe(true)
      expect(wrapper.find('.z-50').exists()).toBe(true)
    })

    it('has proper overflow handling', () => {
      const wrapper = mount(SearchPanel)

      expect(wrapper.find('.overflow-auto').exists()).toBe(true)
    })
  })

  describe('formatTime helper', () => {
    it('formats timestamp correctly', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue([
        {
          record: { id: 1, timestamp: '2024-01-15T10:30:00Z', source_type: 'auto', content: 'Content' },
          snippet: 'Content',
          rank: 0.9,
        },
      ])

      const wrapper = mount(SearchPanel)

      await wrapper.find('input[type="text"]').setValue('test')
      await wrapper.find('input[type="text"]').trigger('keyup.enter')

      await vi.waitFor(() => {
        // The formatTime function uses zh-CN locale
        expect(wrapper.text()).toMatch(/2024/)
      })
    })
  })

  describe('error handling', () => {
    it('shows error toast when search fails', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      const { showError } = await import('../../stores/toast')
      vi.mocked(invoke).mockRejectedValue(new Error('Network error'))

      const wrapper = mount(SearchPanel)

      await wrapper.find('input[type="text"]').setValue('test')
      await wrapper.find('input[type="text"]').trigger('keyup.enter')

      await vi.waitFor(() => {
        expect(showError).toHaveBeenCalled()
      })
    })
  })
})