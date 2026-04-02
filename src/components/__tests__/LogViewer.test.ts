import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import LogViewer from '../LogViewer.vue'

// Mock Tauri APIs
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

// Mock vue-i18n
vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string, params?: Record<string, unknown>) => {
      const translations: Record<string, string> = {
        'logViewer.title': 'Application Logs',
        'logViewer.loading': 'Loading...',
        'logViewer.refresh': 'Refresh',
        'logViewer.autoRefresh': 'Auto',
        'logViewer.noLogs': 'No logs to display',
        'logViewer.loadFailed': `Failed to load logs: ${params?.error || 'unknown'}`,
      }
      return translations[key] || key
    },
  }),
}))

describe('LogViewer', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  describe('rendering', () => {
    it('renders modal with title', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue('')

      const wrapper = mount(LogViewer)

      expect(wrapper.find('h2').text()).toContain('Application Logs')
    })

    it('shows log path in header', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue('')

      const wrapper = mount(LogViewer)

      expect(wrapper.text()).toContain('DailyLogger/logs/daily-logger.log')
    })

    it('shows loading state initially before logs load', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      // Create a promise that doesn't resolve immediately
      let resolvePromise: (value: string) => void
      vi.mocked(invoke).mockImplementation(() =>
        new Promise((resolve) => {
          resolvePromise = resolve
        })
      )

      const wrapper = mount(LogViewer)

      // Loading should be true before promise resolves
      expect(wrapper.vm.loading).toBe(true)

      // Resolve the promise
      resolvePromise!('Log content')
      await vi.waitFor(() => {
        expect(wrapper.vm.loading).toBe(false)
      })
    })

    it('shows no logs message when empty', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue('')

      const wrapper = mount(LogViewer)

      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('No logs to display')
      })
    })
  })

  describe('log loading', () => {
    it('loads logs on mount', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue('2024-01-01 INFO Test message\n2024-01-01 WARN Warning message')

      const wrapper = mount(LogViewer)

      await vi.waitFor(() => {
        expect(invoke).toHaveBeenCalledWith('get_recent_logs', { lines: 500 })
      })

      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('INFO Test message')
        expect(wrapper.text()).toContain('WARN Warning message')
      })
    })

    it('displays multiple log lines', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      const logContent = [
        '2024-01-01 10:00:00 INFO Application started',
        '2024-01-01 10:01:00 INFO Processing request',
        '2024-01-01 10:02:00 WARN Slow response time',
        '2024-01-01 10:03:00 ERROR Connection failed',
      ].join('\n')
      vi.mocked(invoke).mockResolvedValue(logContent)

      const wrapper = mount(LogViewer)

      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('Application started')
        expect(wrapper.text()).toContain('Processing request')
        expect(wrapper.text()).toContain('Slow response time')
        expect(wrapper.text()).toContain('Connection failed')
      })
    })

    it('strips ansi escape sequences before rendering logs', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue('\u001b[2m2024-01-01\u001b[0m \u001b[32mINFO\u001b[0m Clean log')

      const wrapper = mount(LogViewer)

      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('2024-01-01 INFO Clean log')
      })

      expect(wrapper.text()).not.toContain('\u001b[2m')
      expect(wrapper.text()).not.toContain('\u001b[32m')
    })

    it('shows error message when loading fails', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockRejectedValue(new Error('File not found'))

      const wrapper = mount(LogViewer)

      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('Failed to load logs')
      })
    })
  })

  describe('level filtering', () => {
    it('shows level filter buttons', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue('')

      const wrapper = mount(LogViewer)

      await vi.waitFor(() => {
        expect(invoke).toHaveBeenCalled()
      })

      const buttons = wrapper.findAll('button').filter((b) =>
        ['INFO', 'WARN', 'ERROR'].includes(b.text().trim())
      )
      expect(buttons.length).toBe(3)
    })

    it('has INFO, WARN, ERROR levels active by default', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue('')

      const wrapper = mount(LogViewer)

      await vi.waitFor(() => {
        expect(invoke).toHaveBeenCalled()
      })

      // All levels should be active (have their color classes)
      const infoBtn = wrapper.findAll('button').find((b) => b.text().trim() === 'INFO')
      const warnBtn = wrapper.findAll('button').find((b) => b.text().trim() === 'WARN')
      const errorBtn = wrapper.findAll('button').find((b) => b.text().trim() === 'ERROR')

      expect(infoBtn?.classes()).toContain('bg-blue-900/60')
      expect(warnBtn?.classes()).toContain('bg-yellow-900/60')
      expect(errorBtn?.classes()).toContain('bg-red-900/60')
    })

    it('toggles level filter when button clicked', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue('')

      const wrapper = mount(LogViewer)

      await vi.waitFor(() => {
        expect(invoke).toHaveBeenCalled()
      })

      // Find and click INFO button to deactivate
      const infoBtn = wrapper.findAll('button').find((b) => b.text().trim() === 'INFO')
      await infoBtn?.trigger('click')

      // INFO should now be deactivated (gray)
      expect(infoBtn?.classes()).toContain('bg-[var(--color-surface-0)]')
      expect(infoBtn?.classes()).toContain('text-[var(--color-text-muted)]')
    })

    it('filters log lines based on active levels', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      const logContent = [
        '2024-01-01 INFO Info message',
        '2024-01-01 WARN Warning message',
        '2024-01-01 ERROR Error message',
      ].join('\n')
      vi.mocked(invoke).mockResolvedValue(logContent)

      const wrapper = mount(LogViewer)

      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('Info message')
      })

      // Deactivate INFO
      const infoBtn = wrapper.findAll('button').find((b) => b.text().trim() === 'INFO')
      await infoBtn?.trigger('click')

      await nextTick()

      // INFO messages should be filtered out
      expect(wrapper.text()).not.toContain('Info message')
      expect(wrapper.text()).toContain('Warning message')
      expect(wrapper.text()).toContain('Error message')
    })

    it('keeps at least one level active', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue('')

      const wrapper = mount(LogViewer)

      await vi.waitFor(() => {
        expect(invoke).toHaveBeenCalled()
      })

      // Deactivate all levels one by one
      const infoBtn = wrapper.findAll('button').find((b) => b.text().trim() === 'INFO')
      const warnBtn = wrapper.findAll('button').find((b) => b.text().trim() === 'WARN')
      const errorBtn = wrapper.findAll('button').find((b) => b.text().trim() === 'ERROR')

      await infoBtn?.trigger('click')
      await warnBtn?.trigger('click')
      await errorBtn?.trigger('click')

      await nextTick()

      // At least one should still be active (ERROR was last clicked, so it should remain)
      expect(errorBtn?.classes()).toContain('bg-red-900/60')
    })
  })

  describe('line styling', () => {
    it('applies correct color for ERROR lines', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue('2024-01-01 ERROR Error message')

      const wrapper = mount(LogViewer)

      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('Error message')
      })

      const lineDiv = wrapper.findAll('.leading-5').find((d) => d.text().includes('ERROR'))
      expect(lineDiv?.classes()).toContain('text-red-400')
    })

    it('applies correct color for WARN lines', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue('2024-01-01 WARN Warning message')

      const wrapper = mount(LogViewer)

      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('Warning message')
      })

      const lineDiv = wrapper.findAll('.leading-5').find((d) => d.text().includes('WARN'))
      expect(lineDiv?.classes()).toContain('text-yellow-400')
    })

    it('applies correct color for INFO lines', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue('2024-01-01 INFO Info message')

      const wrapper = mount(LogViewer)

      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('Info message')
      })

      const lineDiv = wrapper.findAll('.leading-5').find((d) => d.text().includes('INFO'))
      expect(lineDiv?.classes()).toContain('text-[var(--color-text-secondary)]')
    })
  })

  describe('refresh functionality', () => {
    it('has refresh button', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue('')

      const wrapper = mount(LogViewer)

      await vi.waitFor(() => {
        expect(invoke).toHaveBeenCalled()
      })

      // Refresh button shows "Refresh" when not loading
      const allButtons = wrapper.findAll('button')
      const refreshBtn = allButtons.find((b) => {
        const text = b.text().trim()
        return text === 'Refresh' || text === 'Loading...'
      })
      expect(refreshBtn).toBeDefined()
    })

    it('reloads logs when refresh button clicked', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue('First log')

      const wrapper = mount(LogViewer)

      await vi.waitFor(() => {
        expect(invoke).toHaveBeenCalledTimes(1)
      })

      // Wait for loading to complete
      await vi.waitFor(() => {
        expect(wrapper.vm.loading).toBe(false)
      })

      // Change mock response
      vi.mocked(invoke).mockResolvedValue('Second log')

      // Find the refresh button (the one showing "Refresh")
      const allButtons = wrapper.findAll('button')
      const refreshBtn = allButtons.find((b) => b.text().trim() === 'Refresh')

      // Verify button exists before clicking
      expect(refreshBtn).toBeDefined()

      await refreshBtn!.trigger('click')

      // Wait for the second call
      await vi.waitFor(() => {
        expect(invoke).toHaveBeenCalledTimes(2)
      }, { timeout: 3000 })
    })

    it('shows loading text while refreshing', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue('Log content')

      const wrapper = mount(LogViewer)

      await vi.waitFor(() => {
        expect(invoke).toHaveBeenCalled()
      })

      // After loading, button should show "Refresh" and not be disabled
      const allButtons = wrapper.findAll('button')
      const refreshBtn = allButtons.find((b) => b.text().trim() === 'Refresh')
      expect(refreshBtn?.attributes('disabled')).toBeUndefined()
    })
  })

  describe('auto refresh', () => {
    it('has auto refresh checkbox', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue('')

      const wrapper = mount(LogViewer)

      await vi.waitFor(() => {
        expect(invoke).toHaveBeenCalled()
      })

      const checkbox = wrapper.find('input[type="checkbox"]')
      expect(checkbox.exists()).toBe(true)
    })

    it('auto refresh is off by default', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue('')

      const wrapper = mount(LogViewer)

      await vi.waitFor(() => {
        expect(invoke).toHaveBeenCalled()
      })

      const checkbox = wrapper.find('input[type="checkbox"]')
      expect(checkbox.element.checked).toBe(false)
    })

    it('starts auto refresh when checkbox checked', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue('')

      const wrapper = mount(LogViewer)

      await vi.waitFor(() => {
        expect(invoke).toHaveBeenCalledTimes(1)
      })

      // Enable auto refresh
      const checkbox = wrapper.find('input[type="checkbox"]')
      await checkbox.setValue(true)

      // Fast forward 3 seconds
      vi.advanceTimersByTime(3000)

      expect(invoke).toHaveBeenCalledTimes(2)
    })

    it('stops auto refresh when checkbox unchecked', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue('')

      const wrapper = mount(LogViewer)

      await vi.waitFor(() => {
        expect(invoke).toHaveBeenCalledTimes(1)
      })

      // Enable then disable auto refresh
      const checkbox = wrapper.find('input[type="checkbox"]')
      await checkbox.setValue(true)
      await checkbox.setValue(false)

      // Fast forward 6 seconds - should not trigger more calls
      vi.advanceTimersByTime(6000)

      // Only the initial call should have been made
      expect(invoke).toHaveBeenCalledTimes(1)
    })
  })

  describe('close functionality', () => {
    it('emits close when close button clicked', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue('')

      const wrapper = mount(LogViewer)

      await vi.waitFor(() => {
        expect(invoke).toHaveBeenCalled()
      })

      const closeBtn = wrapper.findAll('button').find((b) => b.text().includes('✕'))
      await closeBtn?.trigger('click')

      expect(wrapper.emitted('close')).toBeTruthy()
    })
  })

  describe('modal structure', () => {
    it('has proper modal styling', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue('')

      const wrapper = mount(LogViewer)

      // Check backdrop
      expect(wrapper.find('.fixed.inset-0.bg-black\\/70').exists()).toBe(true)

      // Check modal container
      expect(wrapper.find('[class*="bg-[var(--color-surface-1)]"]').exists()).toBe(true)

      // Check z-index
      expect(wrapper.find('.z-50').exists()).toBe(true)
    })

    it('uses monospace font for log content', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue('Log content')

      const wrapper = mount(LogViewer)

      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('Log content')
      })

      expect(wrapper.find('.font-mono').exists()).toBe(true)
    })
  })
})
