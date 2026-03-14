import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { ref, nextTick } from 'vue'
import ScreenshotGallery from '../ScreenshotGallery.vue'

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

import { invoke } from '@tauri-apps/api/core'

describe('ScreenshotGallery - Date Filtering', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  const mockRecords = [
    {
      id: 1,
      timestamp: '2026-03-10T09:00:00Z',
      source_type: 'auto',
      content: JSON.stringify({ current_focus: 'Working on task 1' }),
      screenshot_path: '/path/screenshot1.png'
    },
    {
      id: 2,
      timestamp: '2026-03-11T10:00:00Z',
      source_type: 'auto',
      content: JSON.stringify({ current_focus: 'Working on task 2' }),
      screenshot_path: '/path/screenshot2.png'
    },
    {
      id: 3,
      timestamp: '2026-03-12T11:00:00Z',
      source_type: 'auto',
      content: JSON.stringify({ current_focus: 'Working on task 3' }),
      screenshot_path: '/path/screenshot3.png'
    }
  ]

  it('renders date filter inputs', async () => {
    invoke.mockResolvedValueOnce(mockRecords)
    invoke.mockResolvedValue('base64thumbnail')

    const wrapper = mount(ScreenshotGallery, {
      global: {
        stubs: {
          ScreenshotModal: true,
          teleport: true
        }
      }
    })

    await nextTick()
    await nextTick()

    // Should have start date and end date inputs
    const dateInputs = wrapper.findAll('input[type="date"]')
    expect(dateInputs.length).toBe(2)
  })

  it('renders filter and reset buttons', async () => {
    invoke.mockResolvedValueOnce(mockRecords)
    invoke.mockResolvedValue('base64thumbnail')

    const wrapper = mount(ScreenshotGallery, {
      global: {
        stubs: {
          ScreenshotModal: true,
          teleport: true
        }
      }
    })

    await nextTick()
    await nextTick()

    const buttons = wrapper.findAll('button')
    const filterButton = buttons.find(btn => btn.text().includes('筛选'))
    const resetButton = buttons.find(btn => btn.text().includes('重置'))

    expect(filterButton).toBeDefined()
    expect(resetButton).toBeDefined()
  })

  it('calls get_records_by_date_range when filter is clicked', async () => {
    invoke.mockResolvedValueOnce(mockRecords)
    invoke.mockResolvedValue('base64thumbnail')

    const wrapper = mount(ScreenshotGallery, {
      global: {
        stubs: {
          ScreenshotModal: true,
          teleport: true
        }
      }
    })

    await nextTick()
    await nextTick()

    // Set date range
    const dateInputs = wrapper.findAll('input[type="date"]')
    await dateInputs[0].setValue('2026-03-10')
    await dateInputs[1].setValue('2026-03-11')

    // Clear previous calls
    invoke.mockClear()

    // Mock the date range query
    invoke.mockResolvedValueOnce([mockRecords[0], mockRecords[1]])
    invoke.mockResolvedValue('base64thumbnail')

    // Click filter button
    const buttons = wrapper.findAll('button')
    const filterButton = buttons.find(btn => btn.text().includes('筛选'))
    await filterButton.trigger('click')

    await nextTick()

    expect(invoke).toHaveBeenCalledWith('get_records_by_date_range', {
      startDate: '2026-03-10',
      endDate: '2026-03-11'
    })
  })

  it('displays filtered result count', async () => {
    invoke.mockResolvedValueOnce(mockRecords)
    invoke.mockResolvedValue('base64thumbnail')

    const wrapper = mount(ScreenshotGallery, {
      global: {
        stubs: {
          ScreenshotModal: true,
          teleport: true
        }
      }
    })

    await nextTick()
    await nextTick()

    // Set date range
    const dateInputs = wrapper.findAll('input[type="date"]')
    await dateInputs[0].setValue('2026-03-10')
    await dateInputs[1].setValue('2026-03-11')

    // Clear previous calls
    invoke.mockClear()

    // Mock the date range query
    invoke.mockResolvedValueOnce([mockRecords[0], mockRecords[1]])
    invoke.mockResolvedValue('base64thumbnail')

    // Click filter button
    const buttons = wrapper.findAll('button')
    const filterButton = buttons.find(btn => btn.text().includes('筛选'))
    await filterButton.trigger('click')

    await nextTick()
    await nextTick()

    // Should display count
    const countText = wrapper.text()
    expect(countText).toMatch(/共.*条/)
  })

  it('resets to today records when reset is clicked', async () => {
    invoke.mockResolvedValueOnce(mockRecords)
    invoke.mockResolvedValue('base64thumbnail')

    const wrapper = mount(ScreenshotGallery, {
      global: {
        stubs: {
          ScreenshotModal: true,
          teleport: true
        }
      }
    })

    await nextTick()
    await nextTick()

    // Set date range
    const dateInputs = wrapper.findAll('input[type="date"]')
    await dateInputs[0].setValue('2026-03-10')
    await dateInputs[1].setValue('2026-03-11')

    // Clear previous calls
    invoke.mockClear()

    // Mock the reset query
    invoke.mockResolvedValueOnce(mockRecords)
    invoke.mockResolvedValue('base64thumbnail')

    // Click reset button
    const buttons = wrapper.findAll('button')
    const resetButton = buttons.find(btn => btn.text().includes('重置'))
    await resetButton.trigger('click')

    await nextTick()

    expect(invoke).toHaveBeenCalledWith('get_today_records')
  })

  it('shows empty state when no records match filter', async () => {
    invoke.mockResolvedValueOnce(mockRecords)
    invoke.mockResolvedValue('base64thumbnail')

    const wrapper = mount(ScreenshotGallery, {
      global: {
        stubs: {
          ScreenshotModal: true,
          teleport: true
        }
      }
    })

    await nextTick()
    await nextTick()

    // Set date range that returns no records
    const dateInputs = wrapper.findAll('input[type="date"]')
    await dateInputs[0].setValue('2025-01-01')
    await dateInputs[1].setValue('2025-01-02')

    // Clear previous calls
    invoke.mockClear()

    // Mock empty result
    invoke.mockResolvedValueOnce([])

    // Click filter button
    const buttons = wrapper.findAll('button')
    const filterButton = buttons.find(btn => btn.text().includes('筛选'))
    await filterButton.trigger('click')

    await nextTick()
    await nextTick()

    // Should show empty state
    expect(wrapper.text()).toContain('暂无截图记录')
  })

  it('clears date inputs on reset', async () => {
    invoke.mockResolvedValueOnce(mockRecords)
    invoke.mockResolvedValue('base64thumbnail')

    const wrapper = mount(ScreenshotGallery, {
      global: {
        stubs: {
          ScreenshotModal: true,
          teleport: true
        }
      }
    })

    await nextTick()
    await nextTick()

    // Set date range
    const dateInputs = wrapper.findAll('input[type="date"]')
    await dateInputs[0].setValue('2026-03-10')
    await dateInputs[1].setValue('2026-03-11')

    // Clear previous calls
    invoke.mockClear()

    // Mock the reset query
    invoke.mockResolvedValueOnce(mockRecords)
    invoke.mockResolvedValue('base64thumbnail')

    // Click reset button
    const buttons = wrapper.findAll('button')
    const resetButton = buttons.find(btn => btn.text().includes('重置'))
    await resetButton.trigger('click')

    await nextTick()

    // Date inputs should be cleared
    expect(dateInputs[0].element.value).toBe('')
    expect(dateInputs[1].element.value).toBe('')
  })
})