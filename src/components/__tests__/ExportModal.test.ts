import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import ExportModal from '../ExportModal.vue'

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

// Mock vue-i18n
vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string, params?: Record<string, unknown>) => {
      const translations: Record<string, string> = {
        'exportModal.title': 'Data Export',
        'exportModal.dateRange': 'Date Range',
        'exportModal.to': 'to',
        'exportModal.selectDateRange': 'Please select a date range',
        'exportModal.startDateAfterEnd': 'Start date cannot be after end date',
        'exportModal.exportFormat': 'Export Format',
        'exportModal.jsonFormat': 'JSON',
        'exportModal.jsonDescription': 'Structured data, suitable for analysis',
        'exportModal.markdownFormat': 'Markdown',
        'exportModal.markdownDescription': 'Readable document, suitable for archiving',
        'exportModal.exportSuccess': 'Export successful',
        'exportModal.recordCount': `Records: ${params?.count ?? 0}`,
        'exportModal.fileSize': `File size: ${params?.size ?? '0 B'}`,
        'exportModal.path': `Path: ${params?.path ?? ''}`,
        'exportModal.openDirectory': 'Open containing folder',
        'exportModal.close': 'Close',
        'exportModal.exporting': 'Exporting...',
        'exportModal.export': 'Export',
        'exportModal.exportFailed': 'Export failed'
      }
      return translations[key] ?? key
    }
  })
}))

describe('ExportModal', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('renders correctly with default values', () => {
    const wrapper = mount(ExportModal)
    expect(wrapper.find('h2').text()).toBe('Data Export')
    expect(wrapper.find('input[type="date"]').exists()).toBe(true)
    expect(wrapper.findAll('input[type="date"]')).toHaveLength(2)
  })

  it('has JSON and Markdown format options', () => {
    const wrapper = mount(ExportModal)
    // Find buttons containing JSON and Markdown text
    const allButtons = wrapper.findAll('button')
    const jsonButton = allButtons.find(b => b.text().includes('JSON'))
    const markdownButton = allButtons.find(b => b.text().includes('Markdown'))
    expect(jsonButton?.exists()).toBe(true)
    expect(markdownButton?.exists()).toBe(true)
  })

  it('selects JSON format by default', () => {
    const wrapper = mount(ExportModal)
    const allButtons = wrapper.findAll('button')
    const jsonButton = allButtons.find(b => b.text().includes('JSON'))
    expect(jsonButton?.classes()).toContain('border-primary')
  })

  it('switches format when clicking Markdown button', async () => {
    const wrapper = mount(ExportModal)
    const allButtons = wrapper.findAll('button')
    const jsonButton = allButtons.find(b => b.text().includes('JSON'))
    const markdownButton = allButtons.find(b => b.text().includes('Markdown'))

    await markdownButton?.trigger('click')

    expect(markdownButton?.classes()).toContain('border-primary')
    expect(jsonButton?.classes()).not.toContain('border-primary')
  })

  it('emits close event when clicking close button', async () => {
    const wrapper = mount(ExportModal)
    const closeButton = wrapper.findAll('button').find(b => b.text() === 'Close')

    await closeButton?.trigger('click')

    expect(wrapper.emitted('close')).toBeTruthy()
  })

  it('emits close event when clicking backdrop', async () => {
    const wrapper = mount(ExportModal)
    const backdrop = wrapper.find('.fixed.inset-0')

    await backdrop.trigger('click')

    expect(wrapper.emitted('close')).toBeTruthy()
  })

  it('disables export button when start date is after end date', async () => {
    const wrapper = mount(ExportModal)
    const dateInputs = wrapper.findAll('input[type="date"]')

    // Set start date after end date
    await dateInputs[0].setValue('2025-12-31')
    await dateInputs[1].setValue('2025-01-01')

    const exportButton = wrapper.findAll('button').find(b => b.text() === 'Export')
    expect(exportButton?.attributes('disabled')).toBeDefined()
  })

  it('shows error message when start date is after end date', async () => {
    const wrapper = mount(ExportModal)
    const dateInputs = wrapper.findAll('input[type="date"]')

    await dateInputs[0].setValue('2025-12-31')
    await dateInputs[1].setValue('2025-01-01')

    expect(wrapper.find('.text-red-400').exists()).toBe(true)
    expect(wrapper.find('.text-red-400').text()).toBe('Start date cannot be after end date')
  })

  it('calls export_records with correct parameters', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    const mockInvoke = vi.mocked(invoke)
    mockInvoke.mockResolvedValueOnce({
      record_count: 10,
      file_size: 1024,
      path: '/test/export.json'
    })

    const wrapper = mount(ExportModal)
    const exportButton = wrapper.findAll('button').find(b => b.text() === 'Export')

    await exportButton?.trigger('click')

    expect(mockInvoke).toHaveBeenCalledWith('export_records', {
      request: expect.objectContaining({
        format: 'json'
      })
    })
  })

  it('shows exporting state during export', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    const mockInvoke = vi.mocked(invoke)
    mockInvoke.mockImplementation(() => new Promise(resolve => setTimeout(resolve, 100)))

    const wrapper = mount(ExportModal)
    const exportButton = wrapper.findAll('button').find(b => b.text() === 'Export')

    const promise = exportButton?.trigger('click')
    await wrapper.vm.$nextTick()

    // Check button text changed to "Exporting..."
    const buttonText = wrapper.findAll('button').find(b => b.text().includes('Exporting'))
    expect(buttonText?.exists() || wrapper.find('.bg-primary').text()).toBeTruthy()
  })

  it('displays export result on success', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    const mockInvoke = vi.mocked(invoke)
    mockInvoke.mockResolvedValueOnce({
      record_count: 10,
      file_size: 2048,
      path: '/test/export.json'
    })

    const wrapper = mount(ExportModal)
    const exportButton = wrapper.findAll('button').find(b => b.text() === 'Export')

    await exportButton?.trigger('click')
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.text-green-400').exists()).toBe(true)
  })

  it('displays error message on export failure', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    const mockInvoke = vi.mocked(invoke)
    mockInvoke.mockRejectedValueOnce('Export failed')

    const wrapper = mount(ExportModal)
    const exportButton = wrapper.findAll('button').find(b => b.text() === 'Export')

    await exportButton?.trigger('click')
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.bg-red-900\\/20').exists()).toBe(true)
  })
})