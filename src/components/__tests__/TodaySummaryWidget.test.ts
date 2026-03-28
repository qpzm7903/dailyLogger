import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import TodaySummaryWidget from '../TodaySummaryWidget.vue'

// Mock Tauri APIs
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

// Mock localStorage
const localStorageMock = {
  store: {} as Record<string, string>,
  getItem: vi.fn((key: string) => localStorageMock.store[key] || null),
  setItem: vi.fn((key: string, value: string) => {
    localStorageMock.store[key] = value
  }),
  removeItem: vi.fn((key: string) => {
    delete localStorageMock.store[key]
  }),
  clear: vi.fn(() => {
    localStorageMock.store = {}
  })
}

Object.defineProperty(global, 'localStorage', { value: localStorageMock })

// Mock i18n
vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        'widget.todaySummary': 'Today\'s Summary',
        'widget.loading': 'Loading...',
        'widget.loadFailed': 'Failed to load',
        'widget.noRecordsYet': 'No records yet',
        'widget.totalRecords': 'Total',
        'widget.autoCaptures': 'Auto',
        'widget.manualNotes': 'Manual',
        'widget.busiestHour': 'Busiest hour',
        'widget.records': 'records'
      }
      return translations[key] || key
    }
  })
}))

describe('TodaySummaryWidget', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    localStorageMock.store = {}
    localStorageMock.getItem.mockReturnValue(null)
  })

  describe('rendering', () => {
    it('renders with correct base classes', () => {
      const wrapper = mount(TodaySummaryWidget)
      const div = wrapper.find('div')
      expect(div.exists()).toBe(true)
      expect(div.classes()).toContain('bg-[var(--color-surface-1)]/60')
    })

    it('displays header with emoji and title', () => {
      const wrapper = mount(TodaySummaryWidget)
      expect(wrapper.text()).toContain('📊')
      expect(wrapper.text()).toContain("Today's Summary")
    })

    it('displays collapse toggle indicator', () => {
      const wrapper = mount(TodaySummaryWidget)
      expect(wrapper.text()).toContain('▼')
    })

    it('is clickable for collapse toggle', async () => {
      const wrapper = mount(TodaySummaryWidget)
      const header = wrapper.find('.cursor-pointer')
      await header.trigger('click')
      // Should toggle collapsed state
      expect(wrapper.vm.isCollapsed).toBe(true)
    })
  })

  describe('collapsed state', () => {
    it('starts expanded by default', () => {
      const wrapper = mount(TodaySummaryWidget)
      expect(wrapper.vm.isCollapsed).toBe(false)
    })

    it('starts collapsed if localStorage has collapsed=true', () => {
      localStorageMock.getItem.mockReturnValue('true')
      const wrapper = mount(TodaySummaryWidget)
      expect(wrapper.vm.isCollapsed).toBe(true)
    })

    it('persists collapsed state to localStorage', async () => {
      const wrapper = mount(TodaySummaryWidget)
      const header = wrapper.find('.cursor-pointer')
      await header.trigger('click')
      expect(localStorage.setItem).toHaveBeenCalledWith('today-widget-collapsed', 'true')
    })

    it('hides content when collapsed', async () => {
      const wrapper = mount(TodaySummaryWidget)
      // Initially expanded
      expect(wrapper.find('.mt-4').exists()).toBe(true)

      // Click to collapse
      const header = wrapper.find('.cursor-pointer')
      await header.trigger('click')
      await nextTick()

      // Content should be hidden (Transition with v-if)
      expect(wrapper.find('.mt-4').exists()).toBe(false)
    })
  })

  describe('loading state', () => {
    it('shows loading indicator initially', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      // Never resolve to keep loading state
      ;(invoke as ReturnType<typeof vi.fn>).mockImplementation(() => new Promise(() => {}))
      const wrapper = mount(TodaySummaryWidget)
      expect(wrapper.text()).toContain('Loading...')
    })
  })

  describe('error state', () => {
    it('shows error message when invoke fails', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      ;(invoke as ReturnType<typeof vi.fn>).mockRejectedValue(new Error('Backend error'))
      const wrapper = mount(TodaySummaryWidget)
      await nextTick()
      await nextTick() // Wait for async operations
      expect(wrapper.text()).toContain('Failed to load')
    })
  })

  describe('empty state', () => {
    it('shows no records message when total_count is 0', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      ;(invoke as ReturnType<typeof vi.fn>).mockResolvedValue({
        total_count: 0,
        auto_count: 0,
        manual_count: 0,
        first_record_time: null,
        latest_record_time: null,
        busiest_hour: null,
        busiest_hour_count: 0
      })
      const wrapper = mount(TodaySummaryWidget)
      await nextTick()
      await nextTick()
      expect(wrapper.text()).toContain('No records yet')
    })
  })

  describe('stats content', () => {
    it('displays total count', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      ;(invoke as ReturnType<typeof vi.fn>).mockResolvedValue({
        total_count: 10,
        auto_count: 7,
        manual_count: 3,
        first_record_time: null,
        latest_record_time: null,
        busiest_hour: null,
        busiest_hour_count: 0
      })
      const wrapper = mount(TodaySummaryWidget)
      await nextTick()
      await nextTick()
      expect(wrapper.text()).toContain('10')
      expect(wrapper.text()).toContain('Total')
    })

    it('displays auto capture count', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      ;(invoke as ReturnType<typeof vi.fn>).mockResolvedValue({
        total_count: 10,
        auto_count: 7,
        manual_count: 3,
        first_record_time: null,
        latest_record_time: null,
        busiest_hour: null,
        busiest_hour_count: 0
      })
      const wrapper = mount(TodaySummaryWidget)
      await nextTick()
      await nextTick()
      expect(wrapper.text()).toContain('7')
      expect(wrapper.text()).toContain('Auto')
    })

    it('displays manual notes count', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      ;(invoke as ReturnType<typeof vi.fn>).mockResolvedValue({
        total_count: 10,
        auto_count: 7,
        manual_count: 3,
        first_record_time: null,
        latest_record_time: null,
        busiest_hour: null,
        busiest_hour_count: 0
      })
      const wrapper = mount(TodaySummaryWidget)
      await nextTick()
      await nextTick()
      expect(wrapper.text()).toContain('3')
      expect(wrapper.text()).toContain('Manual')
    })

    it('displays time range when first and latest record times are available', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      ;(invoke as ReturnType<typeof vi.fn>).mockResolvedValue({
        total_count: 10,
        auto_count: 7,
        manual_count: 3,
        first_record_time: '2024-01-15T08:30:00+08:00',
        latest_record_time: '2024-01-15T17:45:00+08:00',
        busiest_hour: null,
        busiest_hour_count: 0
      })
      const wrapper = mount(TodaySummaryWidget)
      await nextTick()
      await nextTick()
      expect(wrapper.text()).toContain('⏱')
    })

    it('displays busiest hour when available', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      ;(invoke as ReturnType<typeof vi.fn>).mockResolvedValue({
        total_count: 10,
        auto_count: 7,
        manual_count: 3,
        first_record_time: null,
        latest_record_time: null,
        busiest_hour: 14,
        busiest_hour_count: 5
      })
      const wrapper = mount(TodaySummaryWidget)
      await nextTick()
      await nextTick()
      expect(wrapper.text()).toContain('🔥')
      expect(wrapper.text()).toContain('14:00')
    })
  })

  describe('refresh', () => {
    it('exposes refresh method via defineExpose', () => {
      const wrapper = mount(TodaySummaryWidget)
      expect(wrapper.vm.refresh).toBeDefined()
      expect(typeof wrapper.vm.refresh).toBe('function')
    })
  })
})