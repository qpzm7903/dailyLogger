import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { nextTick } from 'vue'
import TimelineWidget from '../TimelineWidget.vue'
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

// Mock timeline data
const mockTimelineData = {
  date: '2024-01-15',
  total_events: 5,
  active_hours: 4,
  work_time_estimate: 3.5,
  hour_groups: [
    {
      hour: 9,
      label: '09:00 - 10:00',
      count: 2,
      events: [
        {
          record: {
            id: 1,
            timestamp: '2024-01-15T09:30:00Z',
            source_type: 'auto',
            content: JSON.stringify({ summary: 'Working on feature A' }),
            screenshot_path: '/path/to/screenshot1.png'
          },
          time_str: '09:30',
          event_type: 'auto' as const,
          preview: 'Working on feature A'
        },
        {
          record: {
            id: 2,
            timestamp: '2024-01-15T09:45:00Z',
            source_type: 'manual',
            content: 'Quick note about meeting',
            screenshot_path: null
          },
          time_str: '09:45',
          event_type: 'manual' as const,
          preview: 'Quick note about meeting'
        }
      ]
    },
    {
      hour: 10,
      label: '10:00 - 11:00',
      count: 3,
      events: [
        {
          record: {
            id: 3,
            timestamp: '2024-01-15T10:00:00Z',
            source_type: 'auto',
            content: JSON.stringify({ summary: 'Code review' }),
            screenshot_path: '/path/to/screenshot2.png'
          },
          time_str: '10:00',
          event_type: 'auto' as const,
          preview: 'Code review'
        }
      ]
    }
  ]
}

describe('TimelineWidget', () => {
  let i18n: ReturnType<typeof createTestI18n>

  beforeEach(() => {
    i18n = createTestI18n()
    vi.clearAllMocks()
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  describe('initial state', () => {
    it('calls get_timeline_today on mount', async () => {
      mockInvoke.mockResolvedValue(mockTimelineData)

      mount(TimelineWidget, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()

      expect(mockInvoke).toHaveBeenCalledWith('get_timeline_today')
    })

    it('shows loading state initially when invoke is slow', async () => {
      let resolvePromise: () => void
      const slowPromise = new Promise((resolve) => {
        resolvePromise = resolve
      })
      mockInvoke.mockImplementation(() => slowPromise)

      const wrapper = mount(TimelineWidget, {
        global: {
          plugins: [i18n]
        }
      })

      // Check loading state before promise resolves
      expect(wrapper.text()).toContain('Loading')
    })
  })

  describe('stats display', () => {
    it('displays work time estimate', async () => {
      mockInvoke.mockResolvedValue(mockTimelineData)

      const wrapper = mount(TimelineWidget, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      expect(wrapper.text()).toContain('3.5h')
    })

    it('displays active hours', async () => {
      mockInvoke.mockResolvedValue(mockTimelineData)

      const wrapper = mount(TimelineWidget, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      expect(wrapper.text()).toContain('4')
    })

    it('displays total events', async () => {
      mockInvoke.mockResolvedValue(mockTimelineData)

      const wrapper = mount(TimelineWidget, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      expect(wrapper.text()).toContain('5')
    })
  })

  describe('heatmap display', () => {
    it('renders 24 hour cells', async () => {
      mockInvoke.mockResolvedValue(mockTimelineData)

      const wrapper = mount(TimelineWidget, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      // Should have 24 hour cells in the grid
      const hourCells = wrapper.findAll('.h-6.rounded-sm')
      expect(hourCells.length).toBe(24)
    })

    it('shows blue color for auto capture hours', async () => {
      mockInvoke.mockResolvedValue(mockTimelineData)

      const wrapper = mount(TimelineWidget, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      // Hour 10 only has auto events
      const html = wrapper.html()
      expect(html).toContain('bg-blue-')
    })

    it('shows green color for manual note hours', async () => {
      const manualOnlyData = {
        ...mockTimelineData,
        hour_groups: [
          {
            hour: 9,
            label: '09:00 - 10:00',
            count: 1,
            events: [
              {
                record: {
                  id: 1,
                  timestamp: '2024-01-15T09:30:00Z',
                  source_type: 'manual',
                  content: 'Quick note',
                  screenshot_path: null
                },
                time_str: '09:30',
                event_type: 'manual' as const,
                preview: 'Quick note'
              }
            ]
          }
        ]
      }
      mockInvoke.mockResolvedValue(manualOnlyData)

      const wrapper = mount(TimelineWidget, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      const html = wrapper.html()
      expect(html).toContain('bg-green-')
    })

    it('shows gray color for inactive hours', async () => {
      mockInvoke.mockResolvedValue(mockTimelineData)

      const wrapper = mount(TimelineWidget, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      // Most hours should be gray (inactive)
      const html = wrapper.html()
      expect(html).toContain('bg-[var(--color-surface-2)]/30')
    })
  })

  describe('hour interaction', () => {
    it('shows tooltip on hover', async () => {
      mockInvoke.mockResolvedValue(mockTimelineData)

      const wrapper = mount(TimelineWidget, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      // Hover over hour 9 (has events)
      const hourCells = wrapper.findAll('.h-6.rounded-sm')
      await hourCells[9].trigger('mouseenter')
      await nextTick()

      // Should show tooltip
      expect(wrapper.html()).toContain('09:00 - 10:00')
    })

    it('expands hour details on click', async () => {
      mockInvoke.mockResolvedValue(mockTimelineData)

      const wrapper = mount(TimelineWidget, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      // Click on hour 9 (has events)
      const hourCells = wrapper.findAll('.h-6.rounded-sm')
      await hourCells[9].trigger('click')
      await nextTick()

      // Should show expanded details
      expect(wrapper.text()).toContain('09:30')
    })
  })

  describe('open full timeline', () => {
    it('emits openFullTimeline when button is clicked', async () => {
      mockInvoke.mockResolvedValue(mockTimelineData)

      const wrapper = mount(TimelineWidget, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      const viewFullBtn = wrapper.findAll('button').find(btn => btn.text().includes('View Full'))
      await viewFullBtn?.trigger('click')

      expect(wrapper.emitted('openFullTimeline')).toBeTruthy()
    })
  })

  describe('error handling', () => {
    it('shows error message when loading fails', async () => {
      mockInvoke.mockRejectedValue(new Error('Network error'))

      const wrapper = mount(TimelineWidget, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      expect(wrapper.text()).toContain('Failed to load')
    })
  })

  describe('refresh method', () => {
    it('exposes refresh method', async () => {
      mockInvoke.mockResolvedValue(mockTimelineData)

      const wrapper = mount(TimelineWidget, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()

      // Clear mock calls from mount
      mockInvoke.mockClear()
      mockInvoke.mockResolvedValue(mockTimelineData)

      // Call refresh
      await wrapper.vm.refresh()
      await flushPromises()

      expect(mockInvoke).toHaveBeenCalledWith('get_timeline_today')
    })
  })

  describe('legend', () => {
    it('displays legend for event types', async () => {
      mockInvoke.mockResolvedValue(mockTimelineData)

      const wrapper = mount(TimelineWidget, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      expect(wrapper.text()).toContain('Auto Capture')
      expect(wrapper.text()).toContain('Manual Note')
      expect(wrapper.text()).toContain('No Activity')
    })
  })
})