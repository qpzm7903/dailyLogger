import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import DailySummaryViewer from '../DailySummaryViewer.vue'

// Mock Tauri APIs
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

vi.mock('@tauri-apps/plugin-shell', () => ({
  open: vi.fn(),
}))

// Mock vue-i18n
vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string, params?: Record<string, unknown>) => {
      const translations: Record<string, string> = {
        'dailySummaryViewer.title': 'Daily Summary',
        'dailySummaryViewer.showInFinder': 'Show in Finder',
        'dailySummaryViewer.loading': 'Loading...',
        'dailySummaryViewer.filePath': 'File path:',
        'dailySummaryViewer.pathEmpty': 'Path is empty',
        'dailySummaryViewer.loadFailed': `Failed to load: ${params?.error || 'unknown'}`,
      }
      return translations[key] || key
    },
  }),
}))

describe('DailySummaryViewer', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('rendering', () => {
    it('renders modal with title', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue('Test content')

      const wrapper = mount(DailySummaryViewer, {
        props: { summaryPath: '/test/path.md' },
      })

      expect(wrapper.find('h2').text()).toContain('Daily Summary')
    })

    it('shows loading state initially', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockImplementation(() => new Promise(() => {})) // Never resolves

      const wrapper = mount(DailySummaryViewer, {
        props: { summaryPath: '/test/path.md' },
      })

      expect(wrapper.text()).toContain('Loading...')
    })

    it('shows file path in the viewer', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue('Test content')

      const wrapper = mount(DailySummaryViewer, {
        props: { summaryPath: '/test/path.md' },
      })

      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('/test/path.md')
      })
    })
  })

  describe('file loading', () => {
    it('loads file content on mount', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue('# Daily Report\n\nThis is the content.')

      const wrapper = mount(DailySummaryViewer, {
        props: { summaryPath: '/test/report.md' },
      })

      await vi.waitFor(() => {
        expect(invoke).toHaveBeenCalledWith('read_file', { path: '/test/report.md' })
      })

      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('# Daily Report')
        expect(wrapper.text()).toContain('This is the content.')
      })
    })

    it('shows error when path is empty', async () => {
      const { invoke } = await import('@tauri-apps/api/core')

      const wrapper = mount(DailySummaryViewer, {
        props: { summaryPath: '' },
      })

      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('Path is empty')
      })

      expect(invoke).not.toHaveBeenCalled()
    })

    it('shows error when file loading fails', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockRejectedValue(new Error('File not found'))

      const wrapper = mount(DailySummaryViewer, {
        props: { summaryPath: '/test/nonexistent.md' },
      })

      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('Failed to load')
      })
    })
  })

  describe('open in finder', () => {
    it('has show in finder button', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue('Test content')

      const wrapper = mount(DailySummaryViewer, {
        props: { summaryPath: '/test/path.md' },
      })

      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('Show in Finder')
      })
    })

    it('calls open with directory path when button clicked', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      const { open } = await import('@tauri-apps/plugin-shell')
      vi.mocked(invoke).mockResolvedValue('Test content')

      const wrapper = mount(DailySummaryViewer, {
        props: { summaryPath: '/vault/daily/2024-01-01.md' },
      })

      await vi.waitFor(() => {
        expect(invoke).toHaveBeenCalled()
      })

      const button = wrapper.findAll('button').find((b) => b.text().includes('Show in Finder'))
      await button?.trigger('click')

      expect(open).toHaveBeenCalledWith('/vault/daily')
    })

    it('handles paths with multiple slashes', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      const { open } = await import('@tauri-apps/plugin-shell')
      vi.mocked(invoke).mockResolvedValue('Test content')

      const wrapper = mount(DailySummaryViewer, {
        props: { summaryPath: '/path/to/deep/folder/file.md' },
      })

      await vi.waitFor(() => {
        expect(invoke).toHaveBeenCalled()
      })

      const button = wrapper.findAll('button').find((b) => b.text().includes('Show in Finder'))
      await button?.trigger('click')

      expect(open).toHaveBeenCalledWith('/path/to/deep/folder')
    })
  })

  describe('close functionality', () => {
    it('emits close when clicking outside modal', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue('Test content')

      const wrapper = mount(DailySummaryViewer, {
        props: { summaryPath: '/test/path.md' },
      })

      await vi.waitFor(() => {
        expect(invoke).toHaveBeenCalled()
      })

      // Click on the backdrop (outer div)
      await wrapper.find('.fixed.inset-0').trigger('click')

      expect(wrapper.emitted('close')).toBeTruthy()
    })

    it('emits close when clicking close button', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue('Test content')

      const wrapper = mount(DailySummaryViewer, {
        props: { summaryPath: '/test/path.md' },
      })

      await vi.waitFor(() => {
        expect(invoke).toHaveBeenCalled()
      })

      // Find the close button (has ✕ text)
      const closeButton = wrapper.findAll('button').find((b) => b.text().includes('✕'))
      await closeButton?.trigger('click')

      expect(wrapper.emitted('close')).toBeTruthy()
    })
  })

  describe('content display', () => {
    it('displays markdown content with preserved whitespace', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      const content = '# Title\n\n- Item 1\n- Item 2\n\nParagraph text.'
      vi.mocked(invoke).mockResolvedValue(content)

      const wrapper = mount(DailySummaryViewer, {
        props: { summaryPath: '/test/report.md' },
      })

      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('# Title')
        expect(wrapper.text()).toContain('- Item 1')
      })

      // Check whitespace-pre-wrap class is applied
      expect(wrapper.find('.whitespace-pre-wrap').exists()).toBe(true)
    })

    it('displays long content correctly', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      const longContent = 'Line 1\n'.repeat(100)
      vi.mocked(invoke).mockResolvedValue(longContent)

      const wrapper = mount(DailySummaryViewer, {
        props: { summaryPath: '/test/long.md' },
      })

      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('Line 1')
      })

      // Check that content is scrollable
      expect(wrapper.find('.overflow-auto').exists()).toBe(true)
    })
  })

  describe('styling', () => {
    it('has correct modal structure', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue('Test content')

      const wrapper = mount(DailySummaryViewer, {
        props: { summaryPath: '/test/path.md' },
      })

      // Check backdrop
      expect(wrapper.find('.fixed.inset-0.bg-black\\/80').exists()).toBe(true)

      // Check modal container
      expect(wrapper.find('[class*="bg-[var(--color-surface-1)]"].rounded-2xl').exists()).toBe(true)
    })

    it('has proper z-index for modal', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue('Test content')

      const wrapper = mount(DailySummaryViewer, {
        props: { summaryPath: '/test/path.md' },
      })

      expect(wrapper.find('.z-50').exists()).toBe(true)
    })
  })
})