import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { nextTick } from 'vue'
import TimelineVisualization from '../TimelineVisualization.vue'
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

describe('TimelineVisualization', () => {
  let i18n: ReturnType<typeof createTestI18n>

  beforeEach(() => {
    i18n = createTestI18n()
    vi.clearAllMocks()
    // Mock current date
    vi.useFakeTimers()
    vi.setSystemTime(new Date('2024-01-15T12:00:00Z'))
  })

  afterEach(() => {
    vi.useRealTimers()
    vi.clearAllMocks()
  })

  describe('initial state', () => {
    it('calls get_timeline_for_date with today date on mount', async () => {
      mockInvoke.mockResolvedValue({ total_events: 0, active_hours: 0, work_time_estimate: 0, hour_groups: [] })

      mount(TimelineVisualization, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()

      expect(mockInvoke).toHaveBeenCalledWith('get_timeline_for_date', { date: '2024-01-15' })
    })

    it('uses initialDate prop when provided', async () => {
      mockInvoke.mockResolvedValue({ total_events: 0, active_hours: 0, work_time_estimate: 0, hour_groups: [] })

      mount(TimelineVisualization, {
        props: {
          initialDate: '2024-01-10'
        },
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()

      expect(mockInvoke).toHaveBeenCalledWith('get_timeline_for_date', { date: '2024-01-10' })
    })
  })

  describe('stats display', () => {
    it('displays total events count', async () => {
      mockInvoke.mockResolvedValue(mockTimelineData)

      const wrapper = mount(TimelineVisualization, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      expect(wrapper.text()).toContain('Total Events')
      expect(wrapper.text()).toContain('5')
    })

    it('displays active hours', async () => {
      mockInvoke.mockResolvedValue(mockTimelineData)

      const wrapper = mount(TimelineVisualization, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      expect(wrapper.text()).toContain('Active Hours')
      expect(wrapper.text()).toContain('4')
    })

    it('displays work time estimate', async () => {
      mockInvoke.mockResolvedValue(mockTimelineData)

      const wrapper = mount(TimelineVisualization, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      expect(wrapper.text()).toContain('Work Time')
      expect(wrapper.text()).toContain('3.5h')
    })
  })

  describe('hour groups display', () => {
    it('displays hour group labels', async () => {
      mockInvoke.mockResolvedValue(mockTimelineData)

      const wrapper = mount(TimelineVisualization, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      expect(wrapper.text()).toContain('09:00 - 10:00')
      expect(wrapper.text()).toContain('10:00 - 11:00')
    })

    it('displays event count per hour group', async () => {
      mockInvoke.mockResolvedValue(mockTimelineData)

      const wrapper = mount(TimelineVisualization, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      expect(wrapper.text()).toContain('2 events')
      expect(wrapper.text()).toContain('3 events')
    })

    it('displays event previews', async () => {
      mockInvoke.mockResolvedValue(mockTimelineData)

      const wrapper = mount(TimelineVisualization, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      expect(wrapper.text()).toContain('Working on feature A')
      expect(wrapper.text()).toContain('Quick note about meeting')
      expect(wrapper.text()).toContain('Code review')
    })

    it('displays time strings for events', async () => {
      mockInvoke.mockResolvedValue(mockTimelineData)

      const wrapper = mount(TimelineVisualization, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      expect(wrapper.text()).toContain('09:30')
      expect(wrapper.text()).toContain('09:45')
      expect(wrapper.text()).toContain('10:00')
    })
  })

  describe('hour expansion', () => {
    it('auto-expands all hours on load', async () => {
      mockInvoke.mockResolvedValue(mockTimelineData)

      const wrapper = mount(TimelineVisualization, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      // Events should be visible since hours are auto-expanded
      expect(wrapper.text()).toContain('Working on feature A')
    })

    it('toggles hour expansion when clicking hour header', async () => {
      mockInvoke.mockResolvedValue(mockTimelineData)

      const wrapper = mount(TimelineVisualization, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      // Find the first hour header and click it to collapse
      const hourHeaders = wrapper.findAll('[class*="cursor-pointer"]')
      await hourHeaders[0].trigger('click')
      await nextTick()

      // Should show collapsed indicator
      expect(wrapper.html()).toContain('▶')
    })
  })

  describe('date navigation', () => {
    it('navigates to previous day', async () => {
      mockInvoke.mockResolvedValue(mockTimelineData)

      const wrapper = mount(TimelineVisualization, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      // Clear previous calls
      mockInvoke.mockClear()
      mockInvoke.mockResolvedValue(mockTimelineData)

      const prevBtn = wrapper.findAll('button').find(btn => btn.text().includes('Previous'))
      await prevBtn?.trigger('click')
      await flushPromises()

      expect(mockInvoke).toHaveBeenCalledWith('get_timeline_for_date', { date: '2024-01-14' })
    })

    it('navigates to next day', async () => {
      mockInvoke.mockResolvedValue(mockTimelineData)

      const wrapper = mount(TimelineVisualization, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      // Clear previous calls
      mockInvoke.mockClear()
      mockInvoke.mockResolvedValue(mockTimelineData)

      const nextBtn = wrapper.findAll('button').find(btn => btn.text().includes('Next'))
      await nextBtn?.trigger('click')
      await flushPromises()

      expect(mockInvoke).toHaveBeenCalledWith('get_timeline_for_date', { date: '2024-01-16' })
    })

    it('navigates to today', async () => {
      mockInvoke.mockResolvedValue(mockTimelineData)

      const wrapper = mount(TimelineVisualization, {
        props: {
          initialDate: '2024-01-10'
        },
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      // Clear previous calls
      mockInvoke.mockClear()
      mockInvoke.mockResolvedValue(mockTimelineData)

      const todayBtn = wrapper.findAll('button').find(btn => btn.text().includes('Today'))
      await todayBtn?.trigger('click')
      await flushPromises()

      expect(mockInvoke).toHaveBeenCalledWith('get_timeline_for_date', { date: '2024-01-15' })
    })

    it('updates when date input changes', async () => {
      mockInvoke.mockResolvedValue(mockTimelineData)

      const wrapper = mount(TimelineVisualization, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      // Clear previous calls
      mockInvoke.mockClear()
      mockInvoke.mockResolvedValue(mockTimelineData)

      const dateInput = wrapper.find('input[type="date"]')
      await dateInput.setValue('2024-01-20')
      await flushPromises()

      expect(mockInvoke).toHaveBeenCalledWith('get_timeline_for_date', { date: '2024-01-20' })
    })
  })

  describe('event interactions', () => {
    it('emits viewScreenshot when clicking event with screenshot', async () => {
      mockInvoke.mockResolvedValue(mockTimelineData)

      const wrapper = mount(TimelineVisualization, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      // Find event with screenshot (has 📷 icon)
      const events = wrapper.findAll('.hover\\:bg-gray-800\\/30')
      const eventWithScreenshot = events.find(e => e.text().includes('📷'))

      await eventWithScreenshot?.trigger('click')

      expect(wrapper.emitted('viewScreenshot')).toBeTruthy()
    })

    it('does not emit viewScreenshot when clicking event without screenshot', async () => {
      mockInvoke.mockResolvedValue(mockTimelineData)

      const wrapper = mount(TimelineVisualization, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      // Find event without screenshot
      const events = wrapper.findAll('.hover\\:bg-gray-800\\/30')
      const eventWithoutScreenshot = events.find(e => !e.text().includes('📷'))

      await eventWithoutScreenshot?.trigger('click')

      expect(wrapper.emitted('viewScreenshot')).toBeFalsy()
    })
  })

  describe('error and empty states', () => {
    it('shows error message when loading fails', async () => {
      mockInvoke.mockRejectedValue(new Error('Network error'))

      const wrapper = mount(TimelineVisualization, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      expect(wrapper.find('.text-red-400').exists()).toBe(true)
      expect(wrapper.text()).toContain('Failed to load timeline')
    })

    it('shows no events message when timeline is empty', async () => {
      mockInvoke.mockResolvedValue({
        total_events: 0,
        active_hours: 0,
        work_time_estimate: 0,
        hour_groups: []
      })

      const wrapper = mount(TimelineVisualization, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      expect(wrapper.text()).toContain('No events for this day')
    })
  })

  describe('close action', () => {
    it('emits close when close button is clicked', async () => {
      mockInvoke.mockResolvedValue(mockTimelineData)

      const wrapper = mount(TimelineVisualization, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      const closeBtn = wrapper.findAll('button').find(b => b.text() === '✕')
      await closeBtn?.trigger('click')

      expect(wrapper.emitted('close')).toBeTruthy()
    })
  })

  describe('hour icons', () => {
    it('shows correct icons for different times of day', async () => {
      const timelineWithDifferentHours = {
        total_events: 4,
        active_hours: 4,
        work_time_estimate: 4,
        hour_groups: [
          { hour: 6, label: '06:00', count: 1, events: [] },
          { hour: 12, label: '12:00', count: 1, events: [] },
          { hour: 15, label: '15:00', count: 1, events: [] },
          { hour: 20, label: '20:00', count: 1, events: [] }
        ]
      }
      mockInvoke.mockResolvedValue(timelineWithDifferentHours)

      const wrapper = mount(TimelineVisualization, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      const html = wrapper.html()
      // Morning (6-12): 🌅
      expect(html).toContain('🌅')
      // Noon (12-14): ☀️
      expect(html).toContain('☀️')
      // Afternoon (14-18): 🌤️
      expect(html).toContain('🌤️')
      // Evening (18-22): 🌆
      expect(html).toContain('🌆')
    })
  })

  describe('event type icons', () => {
    it('shows correct icon for auto events', async () => {
      mockInvoke.mockResolvedValue(mockTimelineData)

      const wrapper = mount(TimelineVisualization, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      const html = wrapper.html()
      // Auto events use 🖥️
      expect(html).toContain('🖥️')
    })

    it('shows correct icon for manual events', async () => {
      mockInvoke.mockResolvedValue(mockTimelineData)

      const wrapper = mount(TimelineVisualization, {
        global: {
          plugins: [i18n]
        }
      })

      await flushPromises()
      await nextTick()

      const html = wrapper.html()
      // Manual events use ⚡
      expect(html).toContain('⚡')
    })
  })
})