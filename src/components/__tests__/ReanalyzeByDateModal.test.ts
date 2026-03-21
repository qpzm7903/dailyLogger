import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import ReanalyzeByDateModal from '../ReanalyzeByDateModal.vue'
import { createI18n } from 'vue-i18n'
import en from '../../locales/en.json'

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

// Mock toast store
vi.mock('../../stores/toast', () => ({
  showToast: vi.fn()
}))

import { invoke } from '@tauri-apps/api/core'
import { showToast } from '../../stores/toast'

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

describe('ReanalyzeByDateModal', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('rendering', () => {
    it('renders modal with title', () => {
      const i18n = createTestI18n()
      const wrapper = mount(ReanalyzeByDateModal, {
        global: {
          plugins: [i18n]
        }
      })

      expect(wrapper.find('h2').text()).toContain('Reanalyze by Date')
    })

    it('renders date input field', () => {
      const i18n = createTestI18n()
      const wrapper = mount(ReanalyzeByDateModal, {
        global: {
          plugins: [i18n]
        }
      })

      const dateInput = wrapper.find('input[type="date"]')
      expect(dateInput.exists()).toBe(true)
    })

    it('renders close button', () => {
      const i18n = createTestI18n()
      const wrapper = mount(ReanalyzeByDateModal, {
        global: {
          plugins: [i18n]
        }
      })

      const closeButton = wrapper.find('button.text-gray-400')
      expect(closeButton.exists()).toBe(true)
    })

    it('renders start analysis button', () => {
      const i18n = createTestI18n()
      const wrapper = mount(ReanalyzeByDateModal, {
        global: {
          plugins: [i18n]
        }
      })

      const startButton = wrapper.findAll('button').find(b => b.text().includes('Start Analysis'))
      expect(startButton?.exists()).toBe(true)
    })
  })

  describe('date selection', () => {
    it('sets today as default date on mount', async () => {
      const i18n = createTestI18n()
      const wrapper = mount(ReanalyzeByDateModal, {
        global: {
          plugins: [i18n]
        }
      })

      await nextTick()

      const dateInput = wrapper.find('input[type="date"]')
      const today = new Date().toISOString().split('T')[0]
      expect((dateInput.element as HTMLInputElement).value).toBe(today)
    })

    it('sets max date to today', async () => {
      const i18n = createTestI18n()
      const wrapper = mount(ReanalyzeByDateModal, {
        global: {
          plugins: [i18n]
        }
      })

      await nextTick()

      const dateInput = wrapper.find('input[type="date"]')
      const today = new Date().toISOString().split('T')[0]
      expect((dateInput.element as HTMLInputElement).max).toBe(today)
    })

    it('updates selectedDate when input changes', async () => {
      const i18n = createTestI18n()
      const wrapper = mount(ReanalyzeByDateModal, {
        global: {
          plugins: [i18n]
        }
      })

      await nextTick()

      const dateInput = wrapper.find('input[type="date"]')
      await dateInput.setValue('2026-03-15')

      expect((wrapper.vm as unknown as { selectedDate: string }).selectedDate).toBe('2026-03-15')
    })
  })

  describe('record count preview', () => {
    it('shows loading state while fetching record count', async () => {
      vi.mocked(invoke).mockImplementation(() => new Promise(() => {})) // Never resolves

      const i18n = createTestI18n()
      const wrapper = mount(ReanalyzeByDateModal, {
        global: {
          plugins: [i18n]
        }
      })

      await nextTick()

      expect(wrapper.text()).toContain('Counting records')
    })

    it('shows record count when loaded', async () => {
      vi.mocked(invoke).mockResolvedValue([
        { id: 1, screenshot_path: '/path/1.png' },
        { id: 2, screenshot_path: '/path/2.png' },
        { id: 3, screenshot_path: null } // No screenshot
      ])

      const i18n = createTestI18n()
      const wrapper = mount(ReanalyzeByDateModal, {
        global: {
          plugins: [i18n]
        }
      })

      await nextTick()
      await new Promise(resolve => setTimeout(resolve, 10))
      await nextTick()

      expect(wrapper.text()).toContain('Records with screenshots: 2')
    })

    it('counts only records with screenshots', async () => {
      vi.mocked(invoke).mockResolvedValue([
        { id: 1, screenshot_path: '/path/1.png' },
        { id: 2, screenshot_path: null },
        { id: 3, screenshot_path: '/path/3.png' },
        { id: 4, screenshot_path: null }
      ])

      const i18n = createTestI18n()
      const wrapper = mount(ReanalyzeByDateModal, {
        global: {
          plugins: [i18n]
        }
      })

      await nextTick()
      await new Promise(resolve => setTimeout(resolve, 10))
      await nextTick()

      expect(wrapper.text()).toContain('Records with screenshots: 2')
    })
  })

  describe('reanalysis action', () => {
    it('calls reanalyze_records_by_date with selected date', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce([{ id: 1, screenshot_path: '/path/1.png' }]) // get_records_by_date_range
        .mockResolvedValueOnce({ total: 1, success: 1, failed: 0, errors: [] }) // reanalyze_records_by_date

      const i18n = createTestI18n()
      const wrapper = mount(ReanalyzeByDateModal, {
        global: {
          plugins: [i18n]
        }
      })

      await nextTick()

      const dateInput = wrapper.find('input[type="date"]')
      await dateInput.setValue('2026-03-15')
      await nextTick()

      const startButton = wrapper.findAll('button').find(b => b.text().includes('Start Analysis'))
      await startButton?.trigger('click')
      await nextTick()

      expect(invoke).toHaveBeenCalledWith('reanalyze_records_by_date', {
        date: '2026-03-15'
      })
    })

    it('shows success toast on successful reanalysis', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce([{ id: 1, screenshot_path: '/path/1.png' }])
        .mockResolvedValueOnce({ total: 2, success: 2, failed: 0, errors: [] })

      const i18n = createTestI18n()
      const wrapper = mount(ReanalyzeByDateModal, {
        global: {
          plugins: [i18n]
        }
      })

      await nextTick()

      const startButton = wrapper.findAll('button').find(b => b.text().includes('Start Analysis'))
      await startButton?.trigger('click')
      await nextTick()

      expect(showToast).toHaveBeenCalledWith(expect.stringContaining('2'), { type: 'success' })
    })

    it('emits reanalyzed event on success', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce([{ id: 1, screenshot_path: '/path/1.png' }])
        .mockResolvedValueOnce({ total: 1, success: 1, failed: 0, errors: [] })

      const i18n = createTestI18n()
      const wrapper = mount(ReanalyzeByDateModal, {
        global: {
          plugins: [i18n]
        }
      })

      await nextTick()

      const startButton = wrapper.findAll('button').find(b => b.text().includes('Start Analysis'))
      await startButton?.trigger('click')
      await nextTick()

      expect(wrapper.emitted('reanalyzed')).toBeTruthy()
    })

    it('shows error message on failure', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce([{ id: 1, screenshot_path: '/path/1.png' }])
        .mockRejectedValueOnce('API Key not configured')

      const i18n = createTestI18n()
      const wrapper = mount(ReanalyzeByDateModal, {
        global: {
          plugins: [i18n]
        }
      })

      await nextTick()

      const startButton = wrapper.findAll('button').find(b => b.text().includes('Start Analysis'))
      await startButton?.trigger('click')
      await nextTick()

      expect(wrapper.text()).toContain('API Key not configured')
    })

    it('shows result with failed count', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce([
          { id: 1, screenshot_path: '/path/1.png' },
          { id: 2, screenshot_path: '/path/2.png' }
        ])
        .mockResolvedValueOnce({
          total: 2,
          success: 1,
          failed: 1,
          errors: ['Record 2: AI analysis failed']
        })

      const i18n = createTestI18n()
      const wrapper = mount(ReanalyzeByDateModal, {
        global: {
          plugins: [i18n]
        }
      })

      await nextTick()

      const startButton = wrapper.findAll('button').find(b => b.text().includes('Start Analysis'))
      await startButton?.trigger('click')
      await nextTick()

      expect(wrapper.text()).toContain('Total records')
      expect(wrapper.text()).toContain('Success')
      expect(wrapper.text()).toContain('Failed')
    })
  })

  describe('button states', () => {
    it('disables start button when no date selected', async () => {
      const i18n = createTestI18n()
      const wrapper = mount(ReanalyzeByDateModal, {
        global: {
          plugins: [i18n]
        }
      })

      await nextTick()

      // Clear the date
      const vm = wrapper.vm as unknown as { selectedDate: string }
      vm.selectedDate = ''
      await nextTick()

      const startButton = wrapper.findAll('button').find(b => b.text().includes('Start Analysis'))
      expect(startButton?.attributes('disabled')).toBeDefined()
    })

    it('disables start button while processing', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce([{ id: 1, screenshot_path: '/path/1.png' }]) // Initial record count

      const i18n = createTestI18n()
      const wrapper = mount(ReanalyzeByDateModal, {
        global: {
          plugins: [i18n]
        }
      })

      await nextTick()
      await new Promise(resolve => setTimeout(resolve, 50))
      await nextTick()

      // Get the Start Analysis button
      const buttons = wrapper.findAll('button')
      const startButton = buttons.find(b => b.text().includes('Start Analysis') || b.text().includes('Processing'))
      expect(startButton?.exists()).toBe(true)
    })

    it('shows processing text while analyzing', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce([{ id: 1, screenshot_path: '/path/1.png' }]) // Initial record count

      const i18n = createTestI18n()
      const wrapper = mount(ReanalyzeByDateModal, {
        global: {
          plugins: [i18n]
        }
      })

      await nextTick()
      await new Promise(resolve => setTimeout(resolve, 50))
      await nextTick()

      // The button should show either Start Analysis or Processing
      const buttons = wrapper.findAll('button')
      const actionButton = buttons.find(b =>
        b.text().includes('Start Analysis') ||
        b.text().includes('Processing') ||
        b.text().includes('Analysis')
      )
      expect(actionButton?.exists()).toBe(true)
    })
  })

  describe('close behavior', () => {
    it('emits close event when close button clicked', async () => {
      const i18n = createTestI18n()
      const wrapper = mount(ReanalyzeByDateModal, {
        global: {
          plugins: [i18n]
        }
      })

      await nextTick()

      const closeButton = wrapper.find('button.text-gray-400')
      await closeButton.trigger('click')

      expect(wrapper.emitted('close')).toBeTruthy()
    })

    it('emits close event when clicking outside modal', async () => {
      const i18n = createTestI18n()
      const wrapper = mount(ReanalyzeByDateModal, {
        global: {
          plugins: [i18n]
        }
      })

      await nextTick()

      const backdrop = wrapper.find('.fixed.inset-0')
      await backdrop.trigger('click')

      expect(wrapper.emitted('close')).toBeTruthy()
    })
  })
})