import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import CustomReportModal from '../CustomReportModal.vue'

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

// Mock toast store
vi.mock('../../stores/toast.js', () => ({
  showError: vi.fn(),
  showSuccess: vi.fn()
}))

import { invoke } from '@tauri-apps/api/core'

describe('CustomReportModal - REPORT-003', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('renders with header and close button', () => {
    const wrapper = mount(CustomReportModal)
    expect(wrapper.text()).toContain('Custom Report')
    expect(wrapper.find('button').exists()).toBe(true)
  })

  it('shows three preset buttons: biweekly, quarterly, custom', () => {
    const wrapper = mount(CustomReportModal)
    const text = wrapper.text()
    expect(text).toContain('Biweekly')
    expect(text).toContain('Quarterly')
    expect(text).toContain('Custom')
  })

  it('has date range inputs', () => {
    const wrapper = mount(CustomReportModal)
    const dateInputs = wrapper.findAll('input[type="date"]')
    expect(dateInputs.length).toBe(2)
  })

  it('has report name input', () => {
    const wrapper = mount(CustomReportModal)
    const textInputs = wrapper.findAll('input[type="text"], input:not([type])')
    expect(textInputs.length).toBeGreaterThanOrEqual(1)
  })

  it('biweekly preset sets 14-day range ending today', async () => {
    const wrapper = mount(CustomReportModal)
    const presetButtons = wrapper.findAll('button')
    const biweeklyBtn = presetButtons.find(b => b.text().includes('Biweekly'))

    await biweeklyBtn.trigger('click')
    await nextTick()

    const dateInputs = wrapper.findAll('input[type="date"]')
    const startVal = dateInputs[0].element.value
    const endVal = dateInputs[1].element.value

    const today = new Date().toISOString().split('T')[0]
    expect(endVal).toBe(today)

    // Start should be 13 days before today
    const start = new Date(startVal)
    const end = new Date(endVal)
    const diffDays = Math.round((end - start) / (1000 * 60 * 60 * 24))
    expect(diffDays).toBe(13)
  })

  it('quarterly preset sets current quarter range', async () => {
    const wrapper = mount(CustomReportModal)
    const presetButtons = wrapper.findAll('button')
    const quarterBtn = presetButtons.find(b => b.text().includes('Quarterly'))

    await quarterBtn.trigger('click')
    await nextTick()

    const dateInputs = wrapper.findAll('input[type="date"]')
    const startVal = dateInputs[0].element.value

    // Quarter start should be 1st of quarter month
    const startDate = new Date(startVal)
    expect(startDate.getDate()).toBe(1)
    expect([0, 3, 6, 9]).toContain(startDate.getMonth()) // 0-indexed months
  })

  it('disables generate button when dates are empty', () => {
    const wrapper = mount(CustomReportModal)
    const generateBtn = wrapper.findAll('button').find(b => b.text().includes('Generate Report'))
    expect(generateBtn.attributes('disabled')).toBeDefined()
  })

  it('shows day count when dates are selected', async () => {
    const wrapper = mount(CustomReportModal)
    const dateInputs = wrapper.findAll('input[type="date"]')

    await dateInputs[0].setValue('2026-03-01')
    await dateInputs[1].setValue('2026-03-14')
    await nextTick()

    expect(wrapper.text()).toContain('14 days')
  })

  it('shows error when end date is before start date', async () => {
    const wrapper = mount(CustomReportModal)
    const dateInputs = wrapper.findAll('input[type="date"]')

    await dateInputs[0].setValue('2026-03-14')
    await dateInputs[1].setValue('2026-03-01')
    await nextTick()

    expect(wrapper.text()).toContain('End date cannot be before start date')
  })

  it('calls invoke with correct params on generate', async () => {
    invoke.mockResolvedValue('/path/to/report.md')
    const wrapper = mount(CustomReportModal)

    // Set dates
    const dateInputs = wrapper.findAll('input[type="date"]')
    await dateInputs[0].setValue('2026-03-01')
    await dateInputs[1].setValue('2026-03-14')
    await nextTick()

    // Click generate
    const generateBtn = wrapper.findAll('button').find(b => b.text().includes('Generate Report'))
    await generateBtn.trigger('click')
    await nextTick()

    expect(invoke).toHaveBeenCalledWith('generate_custom_report', {
      startDate: '2026-03-01',
      endDate: '2026-03-14',
      reportName: null,
    })
  })

  it('emits close event when close button clicked', async () => {
    const wrapper = mount(CustomReportModal)
    const closeBtn = wrapper.find('button')
    await closeBtn.trigger('click')
    expect(wrapper.emitted('close')).toBeTruthy()
  })

  it('emits generated event with path on success', async () => {
    invoke.mockResolvedValue('/path/to/custom-report.md')
    const wrapper = mount(CustomReportModal)

    const dateInputs = wrapper.findAll('input[type="date"]')
    await dateInputs[0].setValue('2026-03-01')
    await dateInputs[1].setValue('2026-03-14')
    await nextTick()

    const generateBtn = wrapper.findAll('button').find(b => b.text().includes('Generate Report'))
    await generateBtn.trigger('click')

    // Wait for async invoke
    await new Promise(r => setTimeout(r, 10))
    await nextTick()

    expect(wrapper.emitted('generated')).toBeTruthy()
    expect(wrapper.emitted('generated')[0]).toEqual(['/path/to/custom-report.md'])
  })
})
