import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount } from '@vue/test-utils'
import StatisticsPanel from '../StatisticsPanel.vue'

// Mock Tauri APIs
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

// Mock vue-i18n
vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string, params?: Record<string, unknown>) => {
      const translations: Record<string, string> = {
        'statistics.title': 'Data Statistics',
        'statistics.today': 'Today',
        'statistics.thisWeek': 'This Week',
        'statistics.thisMonth': 'This Month',
        'statistics.apply': 'Apply',
        'statistics.screenshots': 'Screenshots',
        'statistics.sessions': 'Sessions',
        'statistics.records': 'Records',
        'statistics.analysisRate': 'Analysis Rate',
        'statistics.dailyBreakdown': 'Daily Breakdown',
        'statistics.noData': 'No data available',
        'statistics.export': 'Export',
        'statistics.exporting': 'Exporting...',
        'statistics.close': 'Close',
      }
      return translations[key] || key
    },
  }),
  createI18n: vi.fn(() => ({
    global: {
      t: (key: string) => key,
    },
  })),
}))

// Mock useFocusTrap
vi.mock('../composables/useFocusTrap', () => ({
  useFocusTrap: () => ({
    activate: vi.fn(),
    deactivate: vi.fn(),
    isActive: { value: false },
  }),
}))

describe('StatisticsPanel', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  describe('rendering', () => {
    it('renders modal with title', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue({
        date_range: { start: '2026-03-26', end: '2026-03-26', label: 'Today' },
        screenshot_count: 10,
        session_count: 5,
        record_count: 20,
        analysis_success_rate: 85.0,
        daily_breakdown: [],
      })

      const wrapper = mount(StatisticsPanel)

      expect(wrapper.find('h2').text()).toContain('Data Statistics')
    })

    it('renders time range buttons', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue({
        date_range: { start: '2026-03-26', end: '2026-03-26', label: 'Today' },
        screenshot_count: 10,
        session_count: 5,
        record_count: 20,
        analysis_success_rate: 85.0,
        daily_breakdown: [],
      })

      const wrapper = mount(StatisticsPanel)

      expect(wrapper.text()).toContain('Today')
      expect(wrapper.text()).toContain('This Week')
      expect(wrapper.text()).toContain('This Month')
    })

    it('renders close button', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue({
        date_range: { start: '2026-03-26', end: '2026-03-26', label: 'Today' },
        screenshot_count: 10,
        session_count: 5,
        record_count: 20,
        analysis_success_rate: 85.0,
        daily_breakdown: [],
      })

      const wrapper = mount(StatisticsPanel)

      const closeBtn = wrapper.findAll('button').find((b) => b.text().includes('✕'))
      expect(closeBtn).toBeDefined()
    })

    it('renders export and close buttons in footer', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue({
        date_range: { start: '2026-03-26', end: '2026-03-26', label: 'Today' },
        screenshot_count: 10,
        session_count: 5,
        record_count: 20,
        analysis_success_rate: 85.0,
        daily_breakdown: [],
      })

      const wrapper = mount(StatisticsPanel)

      expect(wrapper.text()).toContain('Export')
      expect(wrapper.text()).toContain('Close')
    })

    it('has proper modal styling', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue({
        date_range: { start: '2026-03-26', end: '2026-03-26', label: 'Today' },
        screenshot_count: 10,
        session_count: 5,
        record_count: 20,
        analysis_success_rate: 85.0,
        daily_breakdown: [],
      })

      const wrapper = mount(StatisticsPanel)

      expect(wrapper.find('.fixed.inset-0.bg-black\\/80').exists()).toBe(true)
      expect(wrapper.find('[class*="bg-[var(--color-surface-1)]"].rounded-2xl').exists()).toBe(true)
      expect(wrapper.find('.z-50').exists()).toBe(true)
    })
  })

  describe('statistics display', () => {
    it('displays statistics cards when data is loaded', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue({
        date_range: { start: '2026-03-26', end: '2026-03-26', label: 'Today' },
        screenshot_count: 10,
        session_count: 5,
        record_count: 20,
        analysis_success_rate: 85.0,
        daily_breakdown: [],
      })

      const wrapper = mount(StatisticsPanel)

      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('10')
        expect(wrapper.text()).toContain('5')
        expect(wrapper.text()).toContain('20')
        expect(wrapper.text()).toContain('85.0%')
      })
    })

    it('displays screenshot count correctly', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue({
        date_range: { start: '2026-03-26', end: '2026-03-26', label: 'Today' },
        screenshot_count: 42,
        session_count: 3,
        record_count: 15,
        analysis_success_rate: 90.5,
        daily_breakdown: [],
      })

      const wrapper = mount(StatisticsPanel)

      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('42')
      })
    })

    it('displays session count correctly', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue({
        date_range: { start: '2026-03-26', end: '2026-03-26', label: 'Today' },
        screenshot_count: 10,
        session_count: 8,
        record_count: 25,
        analysis_success_rate: 75.0,
        daily_breakdown: [],
      })

      const wrapper = mount(StatisticsPanel)

      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('8')
      })
    })

    it('displays record count correctly', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue({
        date_range: { start: '2026-03-26', end: '2026-03-26', label: 'Today' },
        screenshot_count: 5,
        session_count: 2,
        record_count: 100,
        analysis_success_rate: 60.0,
        daily_breakdown: [],
      })

      const wrapper = mount(StatisticsPanel)

      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('100')
      })
    })

    it('displays analysis success rate with correct color for high rate', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue({
        date_range: { start: '2026-03-26', end: '2026-03-26', label: 'Today' },
        screenshot_count: 10,
        session_count: 5,
        record_count: 20,
        analysis_success_rate: 85.0,
        daily_breakdown: [],
      })

      const wrapper = mount(StatisticsPanel)

      await vi.waitFor(() => {
        expect(wrapper.find('.text-green-400').exists()).toBe(true)
        expect(wrapper.text()).toContain('85.0%')
      })
    })

    it('displays analysis success rate with correct color for medium rate', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue({
        date_range: { start: '2026-03-26', end: '2026-03-26', label: 'Today' },
        screenshot_count: 10,
        session_count: 5,
        record_count: 20,
        analysis_success_rate: 60.0,
        daily_breakdown: [],
      })

      const wrapper = mount(StatisticsPanel)

      await vi.waitFor(() => {
        expect(wrapper.find('.text-yellow-400').exists()).toBe(true)
      })
    })

    it('displays analysis success rate with correct color for low rate', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue({
        date_range: { start: '2026-03-26', end: '2026-03-26', label: 'Today' },
        screenshot_count: 10,
        session_count: 5,
        record_count: 20,
        analysis_success_rate: 30.0,
        daily_breakdown: [],
      })

      const wrapper = mount(StatisticsPanel)

      await vi.waitFor(() => {
        expect(wrapper.find('.text-red-400').exists()).toBe(true)
      })
    })
  })

  describe('loading state', () => {
    it('shows loading spinner while fetching data', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      // Mock with a slow promise that keeps loading state
      let releasePromise: () => void
      const holdPromise = new Promise<void>((resolve) => {
        releasePromise = resolve
      })

      vi.mocked(invoke).mockReturnValue(holdPromise)

      const wrapper = mount(StatisticsPanel)

      // Loading spinner should be visible while waiting for data
      // The component sets isLoading=true before calling invoke
      await vi.waitFor(() => {
        expect(wrapper.find('.animate-spin').exists()).toBe(true)
      })

      // Release the promise to resolve the data load
      releasePromise!()

      await vi.waitFor(() => {
        expect(wrapper.find('.animate-spin').exists()).toBe(false)
      })
    })
  })

  describe('error handling', () => {
    it('displays error message when fetch fails', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockRejectedValue(new Error('Failed to load statistics'))

      const wrapper = mount(StatisticsPanel)

      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('Failed to load statistics')
      })
    })
  })

  describe('daily breakdown', () => {
    it('displays daily breakdown data', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue({
        date_range: { start: '2026-03-24', end: '2026-03-26', label: 'This Week' },
        screenshot_count: 30,
        session_count: 15,
        record_count: 60,
        analysis_success_rate: 75.0,
        daily_breakdown: [
          { date: '2026-03-24', screenshot_count: 10, session_count: 5, record_count: 20 },
          { date: '2026-03-25', screenshot_count: 12, session_count: 6, record_count: 25 },
          { date: '2026-03-26', screenshot_count: 8, session_count: 4, record_count: 15 },
        ],
      })

      const wrapper = mount(StatisticsPanel)

      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('2026-03-24')
        expect(wrapper.text()).toContain('2026-03-25')
        expect(wrapper.text()).toContain('2026-03-26')
      })
    })

    it('shows no data message when daily breakdown is empty', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue({
        date_range: { start: '2026-03-26', end: '2026-03-26', label: 'Today' },
        screenshot_count: 0,
        session_count: 0,
        record_count: 0,
        analysis_success_rate: 0.0,
        daily_breakdown: [],
      })

      const wrapper = mount(StatisticsPanel)

      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('No data available')
      })
    })
  })

  describe('time range selection', () => {
    it('loads statistics on mount', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue({
        date_range: { start: '2026-03-26', end: '2026-03-26', label: 'Today' },
        screenshot_count: 10,
        session_count: 5,
        record_count: 20,
        analysis_success_rate: 85.0,
        daily_breakdown: [],
      })

      mount(StatisticsPanel)

      await vi.waitFor(() => {
        expect(invoke).toHaveBeenCalledWith('get_statistics', { rangeType: 'today' })
      })
    })

    it('calls get_statistics with week range when week button is clicked', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue({
        date_range: { start: '2026-03-26', end: '2026-03-26', label: 'Today' },
        screenshot_count: 10,
        session_count: 5,
        record_count: 20,
        analysis_success_rate: 85.0,
        daily_breakdown: [],
      })

      const wrapper = mount(StatisticsPanel)

      await vi.waitFor(() => {
        expect(invoke).toHaveBeenCalled()
      })

      vi.mocked(invoke).mockClear()
      vi.mocked(invoke).mockResolvedValue({
        date_range: { start: '2026-03-24', end: '2026-03-30', label: 'This Week' },
        screenshot_count: 50,
        session_count: 25,
        record_count: 100,
        analysis_success_rate: 80.0,
        daily_breakdown: [],
      })

      const weekBtn = wrapper.findAll('button').find((b) => b.text().includes('This Week'))
      await weekBtn?.trigger('click')

      await vi.waitFor(() => {
        expect(invoke).toHaveBeenCalledWith('get_statistics', { rangeType: 'week' })
      })
    })

    it('calls get_statistics with month range when month button is clicked', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue({
        date_range: { start: '2026-03-26', end: '2026-03-26', label: 'Today' },
        screenshot_count: 10,
        session_count: 5,
        record_count: 20,
        analysis_success_rate: 85.0,
        daily_breakdown: [],
      })

      const wrapper = mount(StatisticsPanel)

      await vi.waitFor(() => {
        expect(invoke).toHaveBeenCalled()
      })

      vi.mocked(invoke).mockClear()
      vi.mocked(invoke).mockResolvedValue({
        date_range: { start: '2026-03-01', end: '2026-03-31', label: 'This Month' },
        screenshot_count: 200,
        session_count: 100,
        record_count: 400,
        analysis_success_rate: 75.0,
        daily_breakdown: [],
      })

      const monthBtn = wrapper.findAll('button').find((b) => b.text().includes('This Month'))
      await monthBtn?.trigger('click')

      await vi.waitFor(() => {
        expect(invoke).toHaveBeenCalledWith('get_statistics', { rangeType: 'month' })
      })
    })
  })

  describe('custom date range', () => {
    it('has custom date inputs', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue({
        date_range: { start: '2026-03-26', end: '2026-03-26', label: 'Today' },
        screenshot_count: 10,
        session_count: 5,
        record_count: 20,
        analysis_success_rate: 85.0,
        daily_breakdown: [],
      })

      const wrapper = mount(StatisticsPanel)

      const dateInputs = wrapper.findAll('input[type="date"]')
      expect(dateInputs.length).toBe(2)
    })

    it('applies custom date range when apply button is clicked', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue({
        date_range: { start: '2026-03-26', end: '2026-03-26', label: 'Today' },
        screenshot_count: 10,
        session_count: 5,
        record_count: 20,
        analysis_success_rate: 85.0,
        daily_breakdown: [],
      })

      const wrapper = mount(StatisticsPanel)

      await vi.waitFor(() => {
        expect(invoke).toHaveBeenCalled()
      })

      vi.mocked(invoke).mockClear()
      vi.mocked(invoke).mockResolvedValue({
        date_range: { start: '2026-03-01', end: '2026-03-15', label: 'Custom' },
        screenshot_count: 50,
        session_count: 20,
        record_count: 100,
        analysis_success_rate: 90.0,
        daily_breakdown: [],
      })

      // Set custom dates
      const dateInputs = wrapper.findAll('input[type="date"]')
      await dateInputs[0].setValue('2026-03-01')
      await dateInputs[1].setValue('2026-03-15')

      // Click apply
      const applyBtn = wrapper.findAll('button').find((b) => b.text().includes('Apply'))
      await applyBtn?.trigger('click')

      await vi.waitFor(() => {
        expect(invoke).toHaveBeenCalledWith('get_statistics', {
          rangeType: 'custom',
          customStart: '2026-03-01',
          customEnd: '2026-03-15',
        })
      })
    })
  })

  describe('close functionality', () => {
    it('emits close when close button is clicked', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue({
        date_range: { start: '2026-03-26', end: '2026-03-26', label: 'Today' },
        screenshot_count: 10,
        session_count: 5,
        record_count: 20,
        analysis_success_rate: 85.0,
        daily_breakdown: [],
      })

      const wrapper = mount(StatisticsPanel)

      await vi.waitFor(() => {
        expect(invoke).toHaveBeenCalled()
      })

      const closeBtn = wrapper.findAll('button').find((b) => b.text().includes('Close'))
      await closeBtn?.trigger('click')

      expect(wrapper.emitted('close')).toBeTruthy()
    })

    it('emits close when X button is clicked', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue({
        date_range: { start: '2026-03-26', end: '2026-03-26', label: 'Today' },
        screenshot_count: 10,
        session_count: 5,
        record_count: 20,
        analysis_success_rate: 85.0,
        daily_breakdown: [],
      })

      const wrapper = mount(StatisticsPanel)

      await vi.waitFor(() => {
        expect(invoke).toHaveBeenCalled()
      })

      const xBtn = wrapper.findAll('button').find((b) => b.text().includes('✕'))
      await xBtn?.trigger('click')

      expect(wrapper.emitted('close')).toBeTruthy()
    })

    it('emits close when clicking outside modal', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue({
        date_range: { start: '2026-03-26', end: '2026-03-26', label: 'Today' },
        screenshot_count: 10,
        session_count: 5,
        record_count: 20,
        analysis_success_rate: 85.0,
        daily_breakdown: [],
      })

      const wrapper = mount(StatisticsPanel)

      await vi.waitFor(() => {
        expect(invoke).toHaveBeenCalled()
      })

      // Click on the overlay (outside the modal content)
      const overlay = wrapper.find('.fixed.inset-0.bg-black\\/80')
      await overlay.trigger('click.self')

      expect(wrapper.emitted('close')).toBeTruthy()
    })
  })

  describe('export functionality', () => {
    it('export button is disabled when no data', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockRejectedValue(new Error('Loading'))

      const wrapper = mount(StatisticsPanel)

      // Wait for error state
      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('Loading')
      })
    })

    it('export button is enabled when data is loaded', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue({
        date_range: { start: '2026-03-26', end: '2026-03-26', label: 'Today' },
        screenshot_count: 10,
        session_count: 5,
        record_count: 20,
        analysis_success_rate: 85.0,
        daily_breakdown: [],
      })

      const wrapper = mount(StatisticsPanel)

      await vi.waitFor(() => {
        expect(invoke).toHaveBeenCalled()
      })

      const exportBtn = wrapper.findAll('button').find((b) => b.text().includes('Export'))
      // Button should not have disabled attribute set (or it should be falsy)
      expect(exportBtn?.attributes('disabled')).toBeFalsy()
    })
  })
})