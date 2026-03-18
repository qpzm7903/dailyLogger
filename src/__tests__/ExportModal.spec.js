import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import ExportModal from '../components/ExportModal.vue'

// Mock @tauri-apps/api/core
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

import { invoke } from '@tauri-apps/api/core'

describe('ExportModal', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('renders with default 7-day date range', () => {
    const wrapper = mount(ExportModal)

    const dateInputs = wrapper.findAll('input[type="date"]')
    expect(dateInputs).toHaveLength(2)

    // Start date should be 6 days ago, end date should be today
    const today = new Date()
    const endValue = dateInputs[1].element.value
    const expectedEnd = `${today.getFullYear()}-${String(today.getMonth() + 1).padStart(2, '0')}-${String(today.getDate()).padStart(2, '0')}`
    expect(endValue).toBe(expectedEnd)
  })

  it('defaults to JSON format', () => {
    const wrapper = mount(ExportModal)

    // JSON button should have primary styling
    const formatButtons = wrapper.findAll('button').filter(b =>
      b.text().includes('JSON') || b.text().includes('Markdown')
    )
    expect(formatButtons.length).toBe(2)

    // First button (JSON) should be active
    const jsonBtn = formatButtons[0]
    expect(jsonBtn.classes()).toContain('border-primary')
  })

  it('switches format on click', async () => {
    const wrapper = mount(ExportModal)

    const mdBtn = wrapper.findAll('button').find(b => b.text().includes('Markdown'))
    await mdBtn.trigger('click')

    // Markdown button should now be active
    expect(mdBtn.classes()).toContain('border-primary')
  })

  it('shows date error when start > end', async () => {
    const wrapper = mount(ExportModal)

    const dateInputs = wrapper.findAll('input[type="date"]')
    await dateInputs[0].setValue('2026-03-20')
    await dateInputs[1].setValue('2026-03-10')

    expect(wrapper.text()).toContain('Start date cannot be after end date')
  })

  it('disables export button when dates invalid', async () => {
    const wrapper = mount(ExportModal)

    const dateInputs = wrapper.findAll('input[type="date"]')
    await dateInputs[0].setValue('2026-03-20')
    await dateInputs[1].setValue('2026-03-10')

    const exportBtn = wrapper.findAll('button').find(b => b.text().includes('Export'))
    expect(exportBtn.attributes('disabled')).toBeDefined()
  })

  it('calls invoke with correct params on export', async () => {
    invoke.mockResolvedValue({
      path: '/tmp/export.json',
      record_count: 5,
      file_size: 1024,
    })

    const wrapper = mount(ExportModal)

    const dateInputs = wrapper.findAll('input[type="date"]')
    await dateInputs[0].setValue('2026-03-07')
    await dateInputs[1].setValue('2026-03-14')

    const exportBtn = wrapper.findAll('button').find(b =>
      b.text().includes('Export')
    )
    await exportBtn.trigger('click')

    // Wait for async
    await vi.waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('export_records', {
        request: {
          start_date: '2026-03-07',
          end_date: '2026-03-14',
          format: 'json',
        }
      })
    })
  })

  it('shows success result after export', async () => {
    invoke.mockResolvedValue({
      path: '/home/user/exports/dailylogger-export-2026-03-14.json',
      record_count: 42,
      file_size: 2048,
    })

    const wrapper = mount(ExportModal)

    const exportBtn = wrapper.findAll('button').find(b =>
      b.text().includes('Export')
    )
    await exportBtn.trigger('click')

    await vi.waitFor(() => {
      expect(wrapper.text()).toContain('Export successful')
      expect(wrapper.text()).toContain('Records: 42')
    })
  })

  it('shows error on export failure', async () => {
    invoke.mockRejectedValue('Database error')

    const wrapper = mount(ExportModal)

    const exportBtn = wrapper.findAll('button').find(b =>
      b.text().includes('Export')
    )
    await exportBtn.trigger('click')

    await vi.waitFor(() => {
      expect(wrapper.text()).toContain('Database error')
    })
  })

  it('emits close on close button', async () => {
    const wrapper = mount(ExportModal)

    const closeBtn = wrapper.findAll('button').find(b => b.text() === 'Close')
    await closeBtn.trigger('click')

    expect(wrapper.emitted('close')).toBeTruthy()
  })

  it('emits close on backdrop click', async () => {
    const wrapper = mount(ExportModal)

    // Click the backdrop (outer div)
    await wrapper.find('.fixed').trigger('click')

    expect(wrapper.emitted('close')).toBeTruthy()
  })

  it('shows open directory button after successful export', async () => {
    invoke.mockResolvedValue({
      path: '/home/user/exports/dailylogger-export-2026-03-14.json',
      record_count: 10,
      file_size: 512,
    })

    const wrapper = mount(ExportModal)

    const exportBtn = wrapper.findAll('button').find(b =>
      b.text().includes('Export')
    )
    await exportBtn.trigger('click')

    await vi.waitFor(() => {
      expect(wrapper.text()).toContain('Open containing folder')
    })
  })

  it('calls open_export_dir when open directory button is clicked', async () => {
    invoke.mockResolvedValueOnce({
      path: '/home/user/exports/dailylogger-export-2026-03-14.json',
      record_count: 10,
      file_size: 512,
    })
    invoke.mockResolvedValueOnce(undefined) // for open_export_dir

    const wrapper = mount(ExportModal)

    const exportBtn = wrapper.findAll('button').find(b =>
      b.text().includes('Export')
    )
    await exportBtn.trigger('click')

    await vi.waitFor(() => {
      expect(wrapper.text()).toContain('Open containing folder')
    })

    const openDirBtn = wrapper.findAll('button').find(b => b.text().includes('Open containing folder'))
    await openDirBtn.trigger('click')

    await vi.waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('open_export_dir', {
        path: '/home/user/exports/dailylogger-export-2026-03-14.json',
      })
    })
  })
})
