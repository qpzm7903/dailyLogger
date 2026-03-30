import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount } from '@vue/test-utils'
import LogViewer from '../LogViewer.vue'

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

describe('LogViewer', () => {
  let invokeMock

  beforeEach(async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    invokeMock = invoke
    invokeMock.mockClear()
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.restoreAllMocks()
    vi.useRealTimers()
  })

  it('renders modal with title', () => {
    invokeMock.mockResolvedValue('')
    const wrapper = mount(LogViewer)
    expect(wrapper.text()).toContain('Runtime Logs')
  })

  it('displays log path', () => {
    invokeMock.mockResolvedValue('')
    const wrapper = mount(LogViewer)
    expect(wrapper.text()).toContain('DailyLogger/logs/daily-logger.log')
  })

  it('emits close event when close button is clicked', async () => {
    invokeMock.mockResolvedValue('')
    const wrapper = mount(LogViewer)

    const closeButton = wrapper.findAll('button').find(btn => btn.text() === '✕')
    await closeButton.trigger('click')

    expect(wrapper.emitted('close')).toBeTruthy()
  })

  it('loads logs on mount', async () => {
    const mockLogs = 'INFO Log line 1\nWARN Log line 2\nERROR Log line 3'
    invokeMock.mockResolvedValue(mockLogs)

    mount(LogViewer)

    await vi.runAllTimersAsync()

    expect(invokeMock).toHaveBeenCalledWith('get_recent_logs', { lines: 500 })
  })

  it('displays loaded log lines', async () => {
    const mockLogs = 'INFO Log line 1\nWARN Log line 2'
    invokeMock.mockResolvedValue(mockLogs)

    const wrapper = mount(LogViewer)

    await vi.runAllTimersAsync()
    await wrapper.vm.$nextTick()

    expect(wrapper.text()).toContain('Log line 1')
    expect(wrapper.text()).toContain('Log line 2')
  })

  it('shows loading state initially', async () => {
    invokeMock.mockImplementation(() => new Promise(() => {}))
    const wrapper = mount(LogViewer)

    // Wait for component to mount and start loading
    await wrapper.vm.$nextTick()

    // The loading state might not show "加载中..." in the log container
    // because the component shows it only when filteredLines.length === 0
    // Let's check the loading flag instead
    expect(wrapper.vm.loading).toBe(true)
  })

  it('shows empty state when no logs', async () => {
    invokeMock.mockResolvedValue('')

    const wrapper = mount(LogViewer)

    await vi.runAllTimersAsync()
    await wrapper.vm.$nextTick()

    expect(wrapper.text()).toContain('No logs')
  })

  it('shows error message when loading fails', async () => {
    invokeMock.mockRejectedValue(new Error('Failed to load'))

    const wrapper = mount(LogViewer)

    await vi.runAllTimersAsync()
    await wrapper.vm.$nextTick()

    expect(wrapper.text()).toContain('[Log load failed]')
  })

  it('refreshes logs when refresh button is clicked', async () => {
    invokeMock.mockResolvedValue('Initial logs')

    const wrapper = mount(LogViewer)

    await vi.runAllTimersAsync()

    invokeMock.mockResolvedValue('Refreshed logs')

    const refreshButton = wrapper.findAll('button').find(btn => btn.text().includes('Refresh'))
    await refreshButton.trigger('click')

    await vi.runAllTimersAsync()
    await wrapper.vm.$nextTick()

    expect(invokeMock).toHaveBeenCalledTimes(2)
  })

  it('disables refresh button while loading', async () => {
    invokeMock.mockImplementation(() => new Promise(() => {}))

    const wrapper = mount(LogViewer)

    await wrapper.vm.$nextTick()

    // Find the refresh button by checking if it's disabled
    const buttons = wrapper.findAll('button')
    const refreshButton = buttons.find(btn => {
      const text = btn.text()
      return text.includes('Refresh') || text.includes('Loading')
    })

    expect(refreshButton).toBeDefined()
    expect(refreshButton.attributes('disabled')).toBeDefined()
  })

  it('has all log levels active by default', () => {
    invokeMock.mockResolvedValue('')
    const wrapper = mount(LogViewer)

    expect(wrapper.vm.activelevels.has('INFO')).toBe(true)
    expect(wrapper.vm.activelevels.has('WARN')).toBe(true)
    expect(wrapper.vm.activelevels.has('ERROR')).toBe(true)
  })

  it('toggles log level when level button is clicked', async () => {
    invokeMock.mockResolvedValue('')

    const wrapper = mount(LogViewer)

    await vi.runAllTimersAsync()

    const infoButton = wrapper.findAll('button').find(btn => btn.text() === 'INFO')
    await infoButton.trigger('click')

    expect(wrapper.vm.activelevels.has('INFO')).toBe(false)
    expect(wrapper.vm.activelevels.has('WARN')).toBe(true)
    expect(wrapper.vm.activelevels.has('ERROR')).toBe(true)
  })

  it('prevents disabling all log levels', async () => {
    invokeMock.mockResolvedValue('')

    const wrapper = mount(LogViewer)

    await vi.runAllTimersAsync()

    // Disable INFO and WARN
    const infoButton = wrapper.findAll('button').find(btn => btn.text() === 'INFO')
    await infoButton.trigger('click')

    const warnButton = wrapper.findAll('button').find(btn => btn.text() === 'WARN')
    await warnButton.trigger('click')

    // Try to disable ERROR (last one)
    const errorButton = wrapper.findAll('button').find(btn => btn.text() === 'ERROR')
    await errorButton.trigger('click')

    // ERROR should still be active
    expect(wrapper.vm.activelevels.has('ERROR')).toBe(true)
    expect(wrapper.vm.activelevels.size).toBe(1)
  })

  it('filters logs by active levels', async () => {
    const mockLogs = '2024-01-01 INFO Log 1\n2024-01-01 WARN Log 2\n2024-01-01 ERROR Log 3'
    invokeMock.mockResolvedValue(mockLogs)

    const wrapper = mount(LogViewer)

    await vi.runAllTimersAsync()
    await wrapper.vm.$nextTick()

    // Disable INFO
    const infoButton = wrapper.findAll('button').find(btn => btn.text() === 'INFO')
    await infoButton.trigger('click')
    await wrapper.vm.$nextTick()

    const visibleText = wrapper.text()
    expect(visibleText).not.toContain('INFO Log 1')
    expect(visibleText).toContain('WARN Log 2')
    expect(visibleText).toContain('ERROR Log 3')
  })

  it('applies correct color class for ERROR lines', async () => {
    const mockLogs = '2024-01-01 ERROR Test error'
    invokeMock.mockResolvedValue(mockLogs)

    const wrapper = mount(LogViewer)

    await vi.runAllTimersAsync()
    await wrapper.vm.$nextTick()

    const logLine = wrapper.find('.text-red-400')
    expect(logLine.exists()).toBe(true)
  })

  it('applies correct color class for WARN lines', async () => {
    const mockLogs = '2024-01-01 WARN Test warning'
    invokeMock.mockResolvedValue(mockLogs)

    const wrapper = mount(LogViewer)

    await vi.runAllTimersAsync()
    await wrapper.vm.$nextTick()

    const logLine = wrapper.find('.text-yellow-400')
    expect(logLine.exists()).toBe(true)
  })

  it('applies correct color class for INFO lines', async () => {
    const mockLogs = '2024-01-01 INFO Test info'
    invokeMock.mockResolvedValue(mockLogs)

    const wrapper = mount(LogViewer)

    await vi.runAllTimersAsync()
    await wrapper.vm.$nextTick()

    const logLine = wrapper.find('.text-\\[var\\(--color-text-secondary\\)\\]')
    expect(logLine.exists()).toBe(true)
  })

  it('enables auto-refresh when checkbox is checked', async () => {
    invokeMock.mockResolvedValue('Logs')

    const wrapper = mount(LogViewer)

    await vi.runAllTimersAsync()

    const checkbox = wrapper.find('input[type="checkbox"]')
    await checkbox.setValue(true)

    expect(wrapper.vm.autoRefresh).toBe(true)
  })

  it('auto-refreshes logs every 3 seconds when enabled', async () => {
    invokeMock.mockResolvedValue('Logs')

    const wrapper = mount(LogViewer)

    await vi.runAllTimersAsync()

    const checkbox = wrapper.find('input[type="checkbox"]')
    await checkbox.setValue(true)

    // Initial load
    expect(invokeMock).toHaveBeenCalledTimes(1)

    // Advance 3 seconds
    await vi.advanceTimersByTimeAsync(3000)
    expect(invokeMock).toHaveBeenCalledTimes(2)

    // Advance another 3 seconds
    await vi.advanceTimersByTimeAsync(3000)
    expect(invokeMock).toHaveBeenCalledTimes(3)
  })

  it('stops auto-refresh when checkbox is unchecked', async () => {
    invokeMock.mockResolvedValue('Logs')

    const wrapper = mount(LogViewer)

    await vi.runAllTimersAsync()

    const checkbox = wrapper.find('input[type="checkbox"]')
    await checkbox.setValue(true)

    // Initial load
    expect(invokeMock).toHaveBeenCalledTimes(1)

    // Advance 3 seconds
    await vi.advanceTimersByTimeAsync(3000)
    expect(invokeMock).toHaveBeenCalledTimes(2)

    // Disable auto-refresh
    await checkbox.setValue(false)

    // Advance 3 seconds - should not refresh
    await vi.advanceTimersByTimeAsync(3000)
    expect(invokeMock).toHaveBeenCalledTimes(2)
  })

  it('clears auto-refresh timer on unmount', async () => {
    invokeMock.mockResolvedValue('Logs')

    const wrapper = mount(LogViewer)

    await vi.runAllTimersAsync()

    const checkbox = wrapper.find('input[type="checkbox"]')
    await checkbox.setValue(true)

    wrapper.unmount()

    // Advance time - should not refresh after unmount
    await vi.advanceTimersByTimeAsync(3000)
    expect(invokeMock).toHaveBeenCalledTimes(1) // Only initial load
  })

  it('keeps continuation lines when filtering', async () => {
    const mockLogs = '2024-01-01 INFO Start\n  continuation line\n2024-01-01 WARN Warning'
    invokeMock.mockResolvedValue(mockLogs)

    const wrapper = mount(LogViewer)

    await vi.runAllTimersAsync()
    await wrapper.vm.$nextTick()

    // Disable WARN
    const warnButton = wrapper.findAll('button').find(btn => btn.text() === 'WARN')
    await warnButton.trigger('click')
    await wrapper.vm.$nextTick()

    const visibleText = wrapper.text()
    expect(visibleText).toContain('Start')
    expect(visibleText).toContain('continuation line')
    expect(visibleText).not.toContain('Warning')
  })
})
