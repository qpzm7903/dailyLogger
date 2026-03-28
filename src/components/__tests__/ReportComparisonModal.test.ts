import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount } from '@vue/test-utils'
import ReportComparisonModal from '../ReportComparisonModal.vue'

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

// Mock toast store
vi.mock('../../stores/toast.js', () => ({
  showError: vi.fn(),
  showSuccess: vi.fn(),
}))

import { invoke } from '@tauri-apps/api/core'
import { showError, showSuccess } from '../../stores/toast.js'

describe('ReportComparisonModal', () => {
  let wrapper

  const mountComponent = (props = {}) => {
    return mount(ReportComparisonModal, {
      props,
      global: {
        stubs: {},
      },
    })
  }

  beforeEach(() => {
    vi.clearAllMocks()
    // Mock Date for consistent preset testing
    vi.useFakeTimers()
    vi.setSystemTime(new Date('2026-03-18T12:00:00Z')) // Wednesday, March 18, 2026
  })

  afterEach(() => {
    vi.useRealTimers()
    if (wrapper) {
      wrapper.unmount()
    }
  })

  describe('Rendering', () => {
    it('renders modal with title', () => {
      wrapper = mountComponent()
      expect(wrapper.find('h2').text()).toBe('Comparison Report')
    })

    it('renders close button', () => {
      wrapper = mountComponent()
      expect(wrapper.find('button').text()).toContain('✕')
    })

    it('renders period A inputs', () => {
      wrapper = mountComponent()
      const inputs = wrapper.findAll('input[type="date"]')
      expect(inputs.length).toBe(4) // 2 for each period
      expect(wrapper.findAll('label')[0].text()).toBe('Period A')
    })

    it('renders period B inputs', () => {
      wrapper = mountComponent()
      expect(wrapper.findAll('label')[1].text()).toBe('Period B')
    })

    it('renders preset buttons', () => {
      wrapper = mountComponent()
      const presetButtons = wrapper.findAll('button').filter(btn =>
        btn.text().includes('This Week vs Last Week') || btn.text().includes('This Month vs Last Month')
      )
      expect(presetButtons.length).toBe(2)
    })

    it('renders generate button', () => {
      wrapper = mountComponent()
      const generateBtn = wrapper.find('button.bg-primary')
      expect(generateBtn.text()).toContain('Generate Comparison Report')
    })

    it('renders close button in footer', () => {
      wrapper = mountComponent()
      const closeBtn = wrapper.findAll('button').find(btn => btn.text() === 'Close')
      expect(closeBtn.exists()).toBe(true)
    })
  })

  describe('Close Events', () => {
    it('emits close when close button clicked', async () => {
      wrapper = mountComponent()
      const closeBtn = wrapper.findAll('button').find(btn => btn.text() === '✕')
      await closeBtn.trigger('click')
      expect(wrapper.emitted('close')).toBeTruthy()
    })

    it('emits close when footer close button clicked', async () => {
      wrapper = mountComponent()
      const closeBtn = wrapper.findAll('button').find(btn => btn.text() === 'Close')
      await closeBtn.trigger('click')
      expect(wrapper.emitted('close')).toBeTruthy()
    })

    it('emits close when clicking backdrop', async () => {
      wrapper = mountComponent()
      const backdrop = wrapper.find('.fixed.inset-0')
      await backdrop.trigger('click.self')
      expect(wrapper.emitted('close')).toBeTruthy()
    })
  })

  describe('Date Input', () => {
    it('binds startDateA correctly', async () => {
      wrapper = mountComponent()
      const inputs = wrapper.findAll('input[type="date"]')
      await inputs[0].setValue('2026-03-01')
      expect(wrapper.vm.startDateA).toBe('2026-03-01')
    })

    it('binds endDateA correctly', async () => {
      wrapper = mountComponent()
      const inputs = wrapper.findAll('input[type="date"]')
      await inputs[1].setValue('2026-03-07')
      expect(wrapper.vm.endDateA).toBe('2026-03-07')
    })

    it('binds startDateB correctly', async () => {
      wrapper = mountComponent()
      const inputs = wrapper.findAll('input[type="date"]')
      await inputs[2].setValue('2026-03-08')
      expect(wrapper.vm.startDateB).toBe('2026-03-08')
    })

    it('binds endDateB correctly', async () => {
      wrapper = mountComponent()
      const inputs = wrapper.findAll('input[type="date"]')
      await inputs[3].setValue('2026-03-14')
      expect(wrapper.vm.endDateB).toBe('2026-03-14')
    })
  })

  describe('Day Count Display', () => {
    it('calculates day count for period A correctly', async () => {
      wrapper = mountComponent()
      const inputs = wrapper.findAll('input[type="date"]')
      await inputs[0].setValue('2026-03-01')
      await inputs[1].setValue('2026-03-07')

      // 7 days (Mar 1-7 inclusive)
      const dayCountText = wrapper.findAll('p.text-xs')[0]
      expect(dayCountText.text()).toBe('7 days')
    })

    it('calculates day count for period B correctly', async () => {
      wrapper = mountComponent()
      const inputs = wrapper.findAll('input[type="date"]')
      await inputs[2].setValue('2026-03-08')
      await inputs[3].setValue('2026-03-14')
      await wrapper.vm.$nextTick()

      // The second period shows 7 days
      expect(wrapper.vm.dayCountB).toBe(7)

      // Find the day count text elements
      const allSmallText = wrapper.findAll('p.text-xs')
      const dayCountElements = allSmallText.filter(p => p.text().includes('days'))

      // Since only period B dates are set, there should be one day count
      expect(dayCountElements.length).toBe(1)
      expect(dayCountElements[0].text()).toBe('7 days')
    })

    it('does not show day count when dates not set', () => {
      wrapper = mountComponent()
      const dayCountTexts = wrapper.findAll('p.text-xs')
      // Only the day count elements, not error messages
      const dayCounts = dayCountTexts.filter(p => p.text().includes('days'))
      expect(dayCounts.length).toBe(0)
    })

    it('shows 0 days for invalid date range', async () => {
      wrapper = mountComponent()
      const inputs = wrapper.findAll('input[type="date"]')
      await inputs[0].setValue('2026-03-07')
      await inputs[1].setValue('2026-03-01') // End before start

      // Should show 0 days (or no display)
      const dayCountTexts = wrapper.findAll('p.text-xs')
      const dayCounts = dayCountTexts.filter(p => p.text().includes('days'))
      expect(dayCounts.length).toBe(0)
    })
  })

  describe('Date Validation', () => {
    it('shows error when period A end date is before start date', async () => {
      wrapper = mountComponent()
      const inputs = wrapper.findAll('input[type="date"]')
      await inputs[0].setValue('2026-03-07')
      await inputs[1].setValue('2026-03-01')

      const errorEl = wrapper.find('p.text-red-400.text-xs')
      expect(errorEl.text()).toBe('Period A end date cannot be before start date')
    })

    it('shows error when period B end date is before start date', async () => {
      wrapper = mountComponent()
      const inputs = wrapper.findAll('input[type="date"]')
      await inputs[2].setValue('2026-03-14')
      await inputs[3].setValue('2026-03-08')

      const errorEl = wrapper.find('p.text-red-400.text-xs')
      expect(errorEl.text()).toBe('Period B end date cannot be before start date')
    })

    it('clears error when dates are valid', async () => {
      wrapper = mountComponent()
      const inputs = wrapper.findAll('input[type="date"]')

      // Set invalid first
      await inputs[0].setValue('2026-03-07')
      await inputs[1].setValue('2026-03-01')
      expect(wrapper.find('p.text-red-400.text-xs').exists()).toBe(true)

      // Fix the dates
      await inputs[0].setValue('2026-03-01')
      await inputs[1].setValue('2026-03-07')
      expect(wrapper.find('p.text-red-400.text-xs').exists()).toBe(false)
    })
  })

  describe('Generate Button State', () => {
    it('is disabled when no dates are set', () => {
      wrapper = mountComponent()
      const generateBtn = wrapper.find('button.bg-primary')
      expect(generateBtn.attributes('disabled')).toBeDefined()
    })

    it('is disabled when only some dates are set', async () => {
      wrapper = mountComponent()
      const inputs = wrapper.findAll('input[type="date"]')
      await inputs[0].setValue('2026-03-01')
      await inputs[1].setValue('2026-03-07')
      // Only period A set

      const generateBtn = wrapper.find('button.bg-primary')
      expect(generateBtn.attributes('disabled')).toBeDefined()
    })

    it('is disabled when date error exists', async () => {
      wrapper = mountComponent()
      const inputs = wrapper.findAll('input[type="date"]')
      await inputs[0].setValue('2026-03-07')
      await inputs[1].setValue('2026-03-01') // Invalid
      await inputs[2].setValue('2026-03-08')
      await inputs[3].setValue('2026-03-14')

      const generateBtn = wrapper.find('button.bg-primary')
      expect(generateBtn.attributes('disabled')).toBeDefined()
    })

    it('is enabled when all dates are valid', async () => {
      wrapper = mountComponent()
      const inputs = wrapper.findAll('input[type="date"]')
      await inputs[0].setValue('2026-03-01')
      await inputs[1].setValue('2026-03-07')
      await inputs[2].setValue('2026-03-08')
      await inputs[3].setValue('2026-03-14')

      const generateBtn = wrapper.find('button.bg-primary')
      expect(generateBtn.attributes('disabled')).toBeUndefined()
    })

    it('is disabled while generating', async () => {
      wrapper = mountComponent()
      wrapper.vm.isGenerating = true
      const generateBtn = wrapper.find('button.bg-primary')
      expect(generateBtn.attributes('disabled')).toBeDefined()
    })
  })

  describe('Preset: This Week vs Last Week', () => {
    it('sets correct dates for week preset', async () => {
      wrapper = mountComponent()
      // Date is mocked to Wednesday, March 18, 2026
      const weekBtn = wrapper.findAll('button').find(btn => btn.text().includes('This Week vs Last Week'))
      await weekBtn.trigger('click')

      // Week 12 of 2026: Mar 16-22 (this week), Mar 9-15 (last week)
      expect(wrapper.vm.startDateA).toBe('2026-03-09') // Last Monday
      expect(wrapper.vm.endDateA).toBe('2026-03-15')   // Last Sunday
      expect(wrapper.vm.startDateB).toBe('2026-03-16') // This Monday
      expect(wrapper.vm.endDateB).toBe('2026-03-22')   // This Sunday
    })

    it('clears error and result when preset applied', async () => {
      wrapper = mountComponent()
      wrapper.vm.errorMsg = 'Previous error'
      wrapper.vm.resultPath = '/some/path'

      const weekBtn = wrapper.findAll('button').find(btn => btn.text().includes('This Week vs Last Week'))
      await weekBtn.trigger('click')

      expect(wrapper.vm.errorMsg).toBe('')
      expect(wrapper.vm.resultPath).toBe('')
    })
  })

  describe('Preset: This Month vs Last Month', () => {
    it('sets correct dates for month preset', async () => {
      wrapper = mountComponent()
      // Date is mocked to March 18, 2026
      const monthBtn = wrapper.findAll('button').find(btn => btn.text().includes('This Month vs Last Month'))
      await monthBtn.trigger('click')

      // March 2026 and February 2026
      expect(wrapper.vm.startDateA).toBe('2026-02-01') // Feb 1
      expect(wrapper.vm.endDateA).toBe('2026-02-28')   // Feb 28
      expect(wrapper.vm.startDateB).toBe('2026-03-01') // Mar 1
      expect(wrapper.vm.endDateB).toBe('2026-03-31')   // Mar 31
    })
  })

  describe('Generate Comparison', () => {
    it('calls invoke with correct parameters', async () => {
      invoke.mockResolvedValueOnce('/path/to/report.md')

      wrapper = mountComponent()
      const inputs = wrapper.findAll('input[type="date"]')
      await inputs[0].setValue('2026-03-01')
      await inputs[1].setValue('2026-03-07')
      await inputs[2].setValue('2026-03-08')
      await inputs[3].setValue('2026-03-14')

      const generateBtn = wrapper.find('button.bg-primary')
      await generateBtn.trigger('click')

      expect(invoke).toHaveBeenCalledWith('compare_reports', {
        startDateA: '2026-03-01',
        endDateA: '2026-03-07',
        startDateB: '2026-03-08',
        endDateB: '2026-03-14',
      })
    })

    it('shows loading state while generating', async () => {
      let resolvePromise
      invoke.mockImplementation(() => new Promise(resolve => {
        resolvePromise = resolve
      }))

      wrapper = mountComponent()
      const inputs = wrapper.findAll('input[type="date"]')
      await inputs[0].setValue('2026-03-01')
      await inputs[1].setValue('2026-03-07')
      await inputs[2].setValue('2026-03-08')
      await inputs[3].setValue('2026-03-14')

      // Start the generation
      const generatePromise = wrapper.vm.generateComparison()

      // Check loading state immediately after starting
      expect(wrapper.vm.isGenerating).toBe(true)

      // Resolve and wait for completion
      resolvePromise('/path/to/report.md')
      await generatePromise

      expect(wrapper.vm.isGenerating).toBe(false)
    })

    it('shows success message on successful generation', async () => {
      invoke.mockResolvedValueOnce('/path/to/comparison-report.md')

      wrapper = mountComponent()
      const inputs = wrapper.findAll('input[type="date"]')
      await inputs[0].setValue('2026-03-01')
      await inputs[1].setValue('2026-03-07')
      await inputs[2].setValue('2026-03-08')
      await inputs[3].setValue('2026-03-14')

      const generateBtn = wrapper.find('button.bg-primary')
      await generateBtn.trigger('click')

      expect(wrapper.vm.resultPath).toBe('/path/to/comparison-report.md')
      expect(showSuccess).toHaveBeenCalledWith('Comparison report generated successfully')
    })

    it('emits generated event with path', async () => {
      invoke.mockResolvedValueOnce('/path/to/report.md')

      wrapper = mountComponent()
      const inputs = wrapper.findAll('input[type="date"]')
      await inputs[0].setValue('2026-03-01')
      await inputs[1].setValue('2026-03-07')
      await inputs[2].setValue('2026-03-08')
      await inputs[3].setValue('2026-03-14')

      const generateBtn = wrapper.find('button.bg-primary')
      await generateBtn.trigger('click')

      expect(wrapper.emitted('generated')).toBeTruthy()
      expect(wrapper.emitted('generated')[0]).toEqual(['/path/to/report.md'])
    })

    it('shows result path in UI', async () => {
      invoke.mockResolvedValueOnce('/path/to/report.md')

      wrapper = mountComponent()
      const inputs = wrapper.findAll('input[type="date"]')
      await inputs[0].setValue('2026-03-01')
      await inputs[1].setValue('2026-03-07')
      await inputs[2].setValue('2026-03-08')
      await inputs[3].setValue('2026-03-14')

      const generateBtn = wrapper.find('button.bg-primary')
      await generateBtn.trigger('click')

      const successBox = wrapper.find('.border-green-700\\/50')
      expect(successBox.exists()).toBe(true)
      expect(successBox.text()).toContain('Comparison report generated successfully')
      expect(successBox.text()).toContain('/path/to/report.md')
    })

    it('handles error from API', async () => {
      invoke.mockRejectedValueOnce('API error: Rate limit exceeded')

      wrapper = mountComponent()
      const inputs = wrapper.findAll('input[type="date"]')
      await inputs[0].setValue('2026-03-01')
      await inputs[1].setValue('2026-03-07')
      await inputs[2].setValue('2026-03-08')
      await inputs[3].setValue('2026-03-14')

      const generateBtn = wrapper.find('button.bg-primary')
      await generateBtn.trigger('click')

      expect(wrapper.vm.errorMsg).toBe('API error: Rate limit exceeded')
      expect(showError).toHaveBeenCalled()
    })

    it('shows error message in UI', async () => {
      invoke.mockRejectedValueOnce('Network error')

      wrapper = mountComponent()
      const inputs = wrapper.findAll('input[type="date"]')
      await inputs[0].setValue('2026-03-01')
      await inputs[1].setValue('2026-03-07')
      await inputs[2].setValue('2026-03-08')
      await inputs[3].setValue('2026-03-14')

      const generateBtn = wrapper.find('button.bg-primary')
      await generateBtn.trigger('click')

      const errorBox = wrapper.find('.bg-red-900\\/20')
      expect(errorBox.exists()).toBe(true)
      expect(errorBox.text()).toContain('Network error')
    })

    it('clears previous error and result on new generation', async () => {
      invoke.mockResolvedValueOnce('/path/to/report.md')

      wrapper = mountComponent()
      wrapper.vm.errorMsg = 'Previous error'
      wrapper.vm.resultPath = '/old/path'

      const inputs = wrapper.findAll('input[type="date"]')
      await inputs[0].setValue('2026-03-01')
      await inputs[1].setValue('2026-03-07')
      await inputs[2].setValue('2026-03-08')
      await inputs[3].setValue('2026-03-14')

      const generateBtn = wrapper.find('button.bg-primary')
      await generateBtn.trigger('click')

      expect(wrapper.vm.errorMsg).toBe('')
      expect(wrapper.vm.resultPath).toBe('/path/to/report.md')
    })

    it('resets isGenerating after success', async () => {
      invoke.mockResolvedValueOnce('/path/to/report.md')

      wrapper = mountComponent()
      const inputs = wrapper.findAll('input[type="date"]')
      await inputs[0].setValue('2026-03-01')
      await inputs[1].setValue('2026-03-07')
      await inputs[2].setValue('2026-03-08')
      await inputs[3].setValue('2026-03-14')

      const generateBtn = wrapper.find('button.bg-primary')
      await generateBtn.trigger('click')

      expect(wrapper.vm.isGenerating).toBe(false)
    })

    it('resets isGenerating after error', async () => {
      invoke.mockRejectedValueOnce('Error')

      wrapper = mountComponent()
      const inputs = wrapper.findAll('input[type="date"]')
      await inputs[0].setValue('2026-03-01')
      await inputs[1].setValue('2026-03-07')
      await inputs[2].setValue('2026-03-08')
      await inputs[3].setValue('2026-03-14')

      const generateBtn = wrapper.find('button.bg-primary')
      await generateBtn.trigger('click')

      expect(wrapper.vm.isGenerating).toBe(false)
    })

    it('does not call invoke when dateError exists', async () => {
      wrapper = mountComponent()
      // Set dates that create an error (end before start)
      wrapper.vm.startDateA = '2026-03-07'
      wrapper.vm.endDateA = '2026-03-01'

      await wrapper.vm.$nextTick()
      expect(wrapper.vm.dateError).toBeTruthy()

      await wrapper.vm.generateComparison()

      expect(invoke).not.toHaveBeenCalled()
    })

    it('does not call invoke when already generating', async () => {
      wrapper = mountComponent()
      wrapper.vm.isGenerating = true

      await wrapper.vm.generateComparison()

      expect(invoke).not.toHaveBeenCalled()
    })
  })

  describe('Helper Functions', () => {
    it('calcDays returns correct count', () => {
      wrapper = mountComponent()
      expect(wrapper.vm.calcDays('2026-03-01', '2026-03-07')).toBe(7)
      expect(wrapper.vm.calcDays('2026-03-01', '2026-03-01')).toBe(1)
    })

    it('calcDays returns 0 for invalid dates', () => {
      wrapper = mountComponent()
      expect(wrapper.vm.calcDays('', '')).toBe(0)
      expect(wrapper.vm.calcDays('invalid', '2026-03-07')).toBe(0)
    })

    it('calcDays returns 0 when end before start', () => {
      wrapper = mountComponent()
      expect(wrapper.vm.calcDays('2026-03-07', '2026-03-01')).toBe(0)
    })

    it('formatDate returns ISO date string', () => {
      wrapper = mountComponent()
      const date = new Date('2026-03-18T12:00:00Z')
      expect(wrapper.vm.formatDate(date)).toBe('2026-03-18')
    })
  })

  describe('Edge Cases', () => {
    it('handles Error object from invoke', async () => {
      invoke.mockRejectedValueOnce(new Error('Custom error object'))

      wrapper = mountComponent()
      const inputs = wrapper.findAll('input[type="date"]')
      await inputs[0].setValue('2026-03-01')
      await inputs[1].setValue('2026-03-07')
      await inputs[2].setValue('2026-03-08')
      await inputs[3].setValue('2026-03-14')

      const generateBtn = wrapper.find('button.bg-primary')
      await generateBtn.trigger('click')

      expect(wrapper.vm.errorMsg).toBe('Error: Custom error object')
    })

    it('handles same day for start and end', async () => {
      wrapper = mountComponent()
      const inputs = wrapper.findAll('input[type="date"]')
      await inputs[0].setValue('2026-03-01')
      await inputs[1].setValue('2026-03-01')
      await inputs[2].setValue('2026-03-02')
      await inputs[3].setValue('2026-03-02')

      // Should show 1 day
      const dayCountTexts = wrapper.findAll('p.text-xs')
      expect(dayCountTexts[0].text()).toBe('1 days')
      expect(dayCountTexts[1].text()).toBe('1 days')
    })

    it('week preset works on Sunday', async () => {
      // Set date to Sunday, March 22, 2026
      vi.setSystemTime(new Date('2026-03-22T12:00:00Z'))

      wrapper = mountComponent()
      const weekBtn = wrapper.findAll('button').find(btn => btn.text().includes('This Week vs Last Week'))
      await weekBtn.trigger('click')

      // Still should work correctly (Sunday is day 7, treated as end of week)
      expect(wrapper.vm.startDateB).toBe('2026-03-16') // Monday of this week
      expect(wrapper.vm.endDateB).toBe('2026-03-22')   // Sunday (today)
    })

    it('month preset works in January', async () => {
      // Set date to January 15, 2026
      vi.setSystemTime(new Date('2026-01-15T12:00:00Z'))

      wrapper = mountComponent()
      const monthBtn = wrapper.findAll('button').find(btn => btn.text().includes('This Month vs Last Month'))
      await monthBtn.trigger('click')

      // Should show December 2025 for last month
      expect(wrapper.vm.startDateA).toBe('2025-12-01')
      expect(wrapper.vm.endDateA).toBe('2025-12-31')
      expect(wrapper.vm.startDateB).toBe('2026-01-01')
      expect(wrapper.vm.endDateB).toBe('2026-01-31')
    })
  })
})