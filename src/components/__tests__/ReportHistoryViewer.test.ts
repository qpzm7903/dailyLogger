import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { nextTick } from 'vue'
import ReportHistoryViewer from '../ReportHistoryViewer.vue'
import { createI18n } from 'vue-i18n'
import en from '../../locales/en.json'

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

import { invoke } from '@tauri-apps/api/core'

const mockInvoke = vi.mocked(invoke)

// Create i18n instance for testing
const createTestI18n = () => {
  return createI18n({
    legacy: false,
    locale: 'en',
    messages: {
      en
    }
  })
}

// Mock report file data
const mockFiles = [
  {
    name: 'daily-report-2024-01-15.md',
    path: '/path/to/daily-report-2024-01-15.md',
    modified_time: '2024-01-15 18:00:00',
    size_bytes: 2048
  },
  {
    name: 'daily-report-2024-01-14.md',
    path: '/path/to/daily-report-2024-01-14.md',
    modified_time: '2024-01-14 18:00:00',
    size_bytes: 1536
  }
]

describe('ReportHistoryViewer', () => {
  let i18n: ReturnType<typeof createTestI18n>

  beforeEach(() => {
    i18n = createTestI18n()
    vi.clearAllMocks()
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  describe('initial loading state', () => {
    it('shows loading state initially', () => {
      mockInvoke.mockImplementation(() => new Promise(() => {})) // Never resolves

      const wrapper = mount(ReportHistoryViewer, {
        global: {
          plugins: [i18n]
        }
      })

      expect(wrapper.text()).toContain('Loading...')
    })

    it('calls list_report_files on mount', () => {
      mockInvoke.mockResolvedValue([])

      mount(ReportHistoryViewer, {
        global: {
          plugins: [i18n]
        }
      })

      expect(mockInvoke).toHaveBeenCalledWith('list_report_files')
    })
  })

  describe('file list display', () => {
    it('displays files when loaded successfully', async () => {
      mockInvoke.mockResolvedValue(mockFiles)

      const wrapper = mount(ReportHistoryViewer, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      expect(wrapper.text()).toContain('daily-report-2024-01-15.md')
      expect(wrapper.text()).toContain('daily-report-2024-01-14.md')
    })

    it('displays file modified time', async () => {
      mockInvoke.mockResolvedValue(mockFiles)

      const wrapper = mount(ReportHistoryViewer, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      expect(wrapper.text()).toContain('Modified:')
      expect(wrapper.text()).toContain('2024-01-15 18:00:00')
    })

    it('formats file sizes correctly', async () => {
      const filesWithDifferentSizes = [
        { name: 'small.md', path: '/small.md', modified_time: '2024-01-15', size_bytes: 512 },
        { name: 'medium.md', path: '/medium.md', modified_time: '2024-01-15', size_bytes: 2048 },
        { name: 'large.md', path: '/large.md', modified_time: '2024-01-15', size_bytes: 2 * 1024 * 1024 }
      ]
      mockInvoke.mockResolvedValue(filesWithDifferentSizes)

      const wrapper = mount(ReportHistoryViewer, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      expect(wrapper.text()).toContain('512 B')
      expect(wrapper.text()).toContain('2.0 KB')
      expect(wrapper.text()).toContain('2.0 MB')
    })
  })

  describe('empty state', () => {
    it('shows no files message when list is empty', async () => {
      mockInvoke.mockResolvedValue([])

      const wrapper = mount(ReportHistoryViewer, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      expect(wrapper.text()).toContain('No report files found')
    })
  })

  describe('error handling', () => {
    it('shows error message when loading fails', async () => {
      mockInvoke.mockRejectedValue(new Error('Failed to load files'))

      const wrapper = mount(ReportHistoryViewer, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      expect(wrapper.find('.text-red-500').exists()).toBe(true)
      expect(wrapper.text()).toContain('Error')
    })
  })

  describe('file selection', () => {
    it('selects file when clicked', async () => {
      mockInvoke.mockResolvedValue(mockFiles)

      const wrapper = mount(ReportHistoryViewer, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      const fileItems = wrapper.findAll('.bg-darker')
      await fileItems[0].trigger('click')

      // Selected file should have primary border
      expect(fileItems[0].classes()).toContain('border-primary')
    })

    it('updates selected file styling', async () => {
      mockInvoke.mockResolvedValue(mockFiles)

      const wrapper = mount(ReportHistoryViewer, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      const fileItems = wrapper.findAll('.bg-darker')

      // Click first file
      await fileItems[0].trigger('click')
      expect(fileItems[0].classes()).toContain('border-primary')
      expect(fileItems[1].classes()).not.toContain('border-primary')

      // Click second file
      await fileItems[1].trigger('click')
      await nextTick()
      expect(fileItems[0].classes()).not.toContain('border-primary')
      expect(fileItems[1].classes()).toContain('border-primary')
    })
  })

  describe('view file actions', () => {
    it('emits viewFile event when view button is clicked', async () => {
      mockInvoke.mockResolvedValue(mockFiles)

      const wrapper = mount(ReportHistoryViewer, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      const viewButtons = wrapper.findAll('button').filter(btn => btn.text() === 'View')
      await viewButtons[0].trigger('click')

      expect(wrapper.emitted('viewFile')).toBeTruthy()
      expect(wrapper.emitted('viewFile')![0]).toEqual(['/path/to/daily-report-2024-01-15.md'])
    })

    it('emits viewFile event when View Selected button is clicked', async () => {
      mockInvoke.mockResolvedValue(mockFiles)

      const wrapper = mount(ReportHistoryViewer, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      // Select first file
      const fileItems = wrapper.findAll('.bg-darker')
      await fileItems[0].trigger('click')

      // Click View Selected button
      const viewSelectedBtn = wrapper.findAll('button').find(btn => btn.text() === 'View Selected')
      await viewSelectedBtn?.trigger('click')

      expect(wrapper.emitted('viewFile')).toBeTruthy()
      expect(wrapper.emitted('viewFile')![0]).toEqual(['/path/to/daily-report-2024-01-15.md'])
    })

    it('View Selected button is disabled when no file selected', async () => {
      mockInvoke.mockResolvedValue(mockFiles)

      const wrapper = mount(ReportHistoryViewer, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      const viewSelectedBtn = wrapper.findAll('button').find(btn => btn.text() === 'View Selected')
      expect(viewSelectedBtn?.attributes('disabled')).toBeDefined()
    })

    it('View Selected button is enabled when file is selected', async () => {
      mockInvoke.mockResolvedValue(mockFiles)

      const wrapper = mount(ReportHistoryViewer, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      // Select first file
      const fileItems = wrapper.findAll('.bg-darker')
      await fileItems[0].trigger('click')
      await nextTick()

      const viewSelectedBtn = wrapper.findAll('button').find(btn => btn.text() === 'View Selected')
      expect(viewSelectedBtn?.attributes('disabled')).toBeUndefined()
    })
  })

  describe('close actions', () => {
    it('emits close event when close button is clicked', async () => {
      mockInvoke.mockResolvedValue(mockFiles)

      const wrapper = mount(ReportHistoryViewer, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      const closeBtn = wrapper.find('button.text-gray-400')
      await closeBtn.trigger('click')

      expect(wrapper.emitted('close')).toBeTruthy()
    })

    it('emits close event when backdrop is clicked', async () => {
      mockInvoke.mockResolvedValue(mockFiles)

      const wrapper = mount(ReportHistoryViewer, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      // Click the backdrop (the outer div)
      await wrapper.find('.bg-black\\/80').trigger('click')

      expect(wrapper.emitted('close')).toBeTruthy()
    })

    it('emits close event when Close button in footer is clicked', async () => {
      mockInvoke.mockResolvedValue(mockFiles)

      const wrapper = mount(ReportHistoryViewer, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      // Find the close button in footer by its class
      const footerCloseBtn = wrapper.find('.px-6.py-4 button.bg-gray-700')
      await footerCloseBtn.trigger('click')

      expect(wrapper.emitted('close')).toBeTruthy()
    })
  })

  describe('modal structure', () => {
    it('has correct title', async () => {
      mockInvoke.mockResolvedValue(mockFiles)

      const wrapper = mount(ReportHistoryViewer, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      expect(wrapper.find('h2').text()).toContain('Report History')
    })

    it('has correct modal dimensions', () => {
      mockInvoke.mockResolvedValue([])

      const wrapper = mount(ReportHistoryViewer, {
        global: {
          plugins: [i18n]
        }
      })

      const modal = wrapper.find('.bg-dark')
      expect(modal.classes()).toContain('max-w-3xl')
    })
  })
})