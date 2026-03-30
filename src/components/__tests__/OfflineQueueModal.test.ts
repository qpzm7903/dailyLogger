import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { nextTick } from 'vue'
import OfflineQueueModal from '../OfflineQueueModal.vue'

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

// Mock toast store
vi.mock('../../stores/toast', () => ({
  showSuccess: vi.fn(),
  showError: vi.fn()
}))

import { invoke } from '@tauri-apps/api/core'
import { showSuccess, showError } from '../../stores/toast'

describe('OfflineQueueModal', () => {
  const mockTasks = [
    {
      id: 1,
      task_type: 'screenshot_analysis',
      payload: JSON.stringify({ screenshot_path: '/path/to/screenshot.png' }),
      record_id: 123,
      status: 'pending',
      error_message: null,
      created_at: '2026-03-28T09:00:00Z',
      completed_at: null,
      retry_count: 0,
      max_retries: 3
    },
    {
      id: 2,
      task_type: 'daily_summary',
      payload: JSON.stringify({}),
      record_id: null,
      status: 'pending',
      error_message: 'Network error',
      created_at: '2026-03-28T08:00:00Z',
      completed_at: null,
      retry_count: 2,
      max_retries: 3
    },
    {
      id: 3,
      task_type: 'weekly_report',
      payload: JSON.stringify({}),
      record_id: null,
      status: 'pending',
      error_message: null,
      created_at: '2026-03-27T10:00:00Z',
      completed_at: null,
      retry_count: 1,
      max_retries: 3
    }
  ]

  beforeEach(() => {
    vi.clearAllMocks()
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  const mountComponent = (tasks = mockTasks) => {
    vi.mocked(invoke).mockResolvedValue(tasks)
    return mount(OfflineQueueModal, {
      global: {
        stubs: {
          teleport: true
        }
      }
    })
  }

  it('renders modal title', async () => {
    const wrapper = mountComponent()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    expect(wrapper.text()).toContain('Pending Sync Tasks')
  })

  it('renders close button in header', async () => {
    const wrapper = mountComponent()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    const closeButtons = wrapper.findAll('button')
    expect(closeButtons.length).toBeGreaterThan(0)
  })

  it('emits close event when backdrop clicked', async () => {
    const wrapper = mountComponent()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    const backdrop = wrapper.find('.fixed.inset-0')
    await backdrop.trigger('click.self')

    expect(wrapper.emitted('close')).toBeTruthy()
  })

  it('emits close event when header close button clicked', async () => {
    const wrapper = mountComponent()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    const closeButtons = wrapper.findAll('button')
    await closeButtons[0].trigger('click')

    expect(wrapper.emitted('close')).toBeTruthy()
  })

  it('loads tasks on mount', async () => {
    const wrapper = mountComponent()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    expect(invoke).toHaveBeenCalledWith('get_pending_offline_tasks')
  })

  it('displays tasks after loading', async () => {
    const wrapper = mountComponent()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    expect(wrapper.vm.tasks).toEqual(mockTasks)
  })

  it('displays empty state when no tasks', async () => {
    const wrapper = mountComponent([])
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    expect(wrapper.text()).toContain('No pending tasks')
  })

  it('displays task type badge for screenshot_analysis', async () => {
    const wrapper = mountComponent()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    const badge = wrapper.find('.bg-blue-500\\/20')
    expect(badge.exists()).toBe(true)
  })

  it('displays task type badge for daily_summary', async () => {
    const wrapper = mountComponent()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    const badge = wrapper.find('.bg-green-500\\/20')
    expect(badge.exists()).toBe(true)
  })

  it('displays task type badge for weekly_report', async () => {
    const wrapper = mountComponent()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    const badge = wrapper.find('.bg-purple-500\\/20')
    expect(badge.exists()).toBe(true)
  })

  it('displays error message when present', async () => {
    const wrapper = mountComponent()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    expect(wrapper.text()).toContain('Network error')
  })

  it('displays total pending count in footer', async () => {
    const wrapper = mountComponent()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    expect(wrapper.text()).toContain('3 pending tasks')
  })

  it('shows process now button when tasks exist', async () => {
    const wrapper = mountComponent()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    expect(wrapper.text()).toContain('Process Now')
  })

  it('hides process now button when no tasks', async () => {
    const wrapper = mountComponent([])
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    expect(wrapper.text()).not.toContain('Process Now')
  })

  it('processes queue when process now button clicked', async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce(mockTasks) // First call for loadTasks
      .mockResolvedValueOnce('Queue processed') // Second call for processQueue

    const wrapper = mountComponent()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    // Find and click process now button
    const processButton = wrapper.findAll('button').find(btn => btn.text() === 'Process Now')
    await processButton.trigger('click')

    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    expect(invoke).toHaveBeenCalledWith('process_offline_queue')
  })

  it('calls showError on process failure', async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce(mockTasks) // First call for loadTasks
      .mockRejectedValueOnce(new Error('Process failed')) // Second call for processQueue

    const wrapper = mountComponent()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    // Find and click process now button
    const processButton = wrapper.findAll('button').find(btn => btn.text() === 'Process Now')
    await processButton.trigger('click')

    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    expect(showError).toHaveBeenCalled()
  })

  it('calls showSuccess on process success', async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce(mockTasks) // First call for loadTasks
      .mockResolvedValueOnce('Queue processed') // Second call for processQueue

    const wrapper = mountComponent()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    // Find and click process now button
    const processButton = wrapper.findAll('button').find(btn => btn.text() === 'Process Now')
    await processButton.trigger('click')

    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    expect(showSuccess).toHaveBeenCalled()
  })

  it('formatTime returns correct format', async () => {
    const wrapper = mountComponent()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    const formatted = wrapper.vm.formatTime('2026-03-28T09:00:00Z')
    expect(formatted).toContain('03')
    expect(formatted).toContain('28')
  })

  it('getTaskTypeClass returns correct class for screenshot_analysis', async () => {
    const wrapper = mountComponent()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    const className = wrapper.vm.getTaskTypeClass('screenshot_analysis')
    expect(className).toBe('bg-blue-500/20 text-blue-400')
  })

  it('getTaskTypeClass returns correct class for daily_summary', async () => {
    const wrapper = mountComponent()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    const className = wrapper.vm.getTaskTypeClass('daily_summary')
    expect(className).toBe('bg-green-500/20 text-green-400')
  })

  it('getTaskTypeClass returns correct class for weekly_report', async () => {
    const wrapper = mountComponent()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    const className = wrapper.vm.getTaskTypeClass('weekly_report')
    expect(className).toBe('bg-purple-500/20 text-purple-400')
  })

  it('getTaskTypeClass returns correct class for monthly_report', async () => {
    const wrapper = mountComponent()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    const className = wrapper.vm.getTaskTypeClass('monthly_report')
    expect(className).toBe('bg-orange-500/20 text-orange-400')
  })

  it('getTaskTypeClass returns default class for unknown type', async () => {
    const wrapper = mountComponent()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    const className = wrapper.vm.getTaskTypeClass('unknown_type')
    expect(className).toBe('bg-[var(--color-action-neutral)]/20 text-[var(--color-text-muted)]')
  })

  it('getTaskDescription parses screenshot path correctly', async () => {
    const wrapper = mountComponent()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    const task = mockTasks[0]
    const description = wrapper.vm.getTaskDescription(task)
    expect(description).toContain('screenshot.png')
  })
})
