import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { nextTick, h, ref } from 'vue'
import ErrorBoundary from '../ErrorBoundary.vue'

// Mock systemActions
vi.mock('../../features/system/actions', () => ({
  systemActions: {
    logFrontendError: vi.fn().mockResolvedValue(undefined)
  }
}))

describe('ErrorBoundary', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  // Helper to mount ErrorBoundary with Teleport stubbed
  const mountWithTeleport = (slots = {}) => {
    return mount(ErrorBoundary, {
      slots,
      global: {
        stubs: {
          Teleport: {
            template: '<div><slot /></div>'
          }
        },
        mocks: {
          // Mock i18n
          $t: (key: string) => key
        }
      }
    })
  }

  describe('rendering', () => {
    it('renders slot content normally when no error', () => {
      const wrapper = mountWithTeleport({
        default: h('div', { class: 'test-content' }, 'Test Content')
      })
      expect(wrapper.find('.test-content').exists()).toBe(true)
      expect(wrapper.text()).toContain('Test Content')
    })

    it('does not show error alert when no error', () => {
      const wrapper = mountWithTeleport({
        default: h('div', 'Normal content')
      })
      expect(wrapper.find('div[role="alert"]').exists()).toBe(false)
    })
  })

  describe('error capture', () => {
    it('captures error from child component', async () => {
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})

      const ChildComponent = {
        setup() {
          throw new Error('Test error')
        },
        template: '<div>Child</div>'
      }

      mountWithTeleport({
        default: h(ChildComponent)
      })

      await flushPromises()

      // Error should be captured and logged
      expect(consoleSpy).toHaveBeenCalled()
      consoleSpy.mockRestore()
    })

    it('displays error alert when error is captured', async () => {
      const ChildComponent = {
        setup() {
          throw new Error('Test error')
        },
        template: '<div>Child</div>'
      }

      const wrapper = mountWithTeleport({
        default: h(ChildComponent)
      })

      await flushPromises()

      // Error alert should be shown
      const alertDiv = wrapper.find('div[role="alert"]')
      expect(alertDiv.exists()).toBe(true)
    })

    it('shows error message from captured error', async () => {
      const ChildComponent = {
        setup() {
          throw new Error('Specific error message')
        },
        template: '<div>Child</div>'
      }

      const wrapper = mountWithTeleport({
        default: h(ChildComponent)
      })

      await flushPromises()

      // Error message should be displayed
      expect(wrapper.text()).toContain('Specific error message')
    })
  })

  describe('error logging', () => {
    it('logs error to backend when error is captured', async () => {
      const { systemActions } = await import('../../features/system/actions')

      const ChildComponent = {
        setup() {
          throw new Error('Backend error')
        },
        template: '<div>Child</div>'
      }

      mountWithTeleport({
        default: h(ChildComponent)
      })

      await flushPromises()

      expect(systemActions.logFrontendError).toHaveBeenCalledWith(
        'Backend error',
        expect.any(String)
      )
    })

    it('handles backend logging failure gracefully', async () => {
      const { systemActions } = await import('../../features/system/actions')
      ;(systemActions.logFrontendError as ReturnType<typeof vi.fn>).mockRejectedValueOnce(
        new Error('Backend unavailable')
      )

      const consoleSpy = vi.spyOn(console, 'warn').mockImplementation(() => {})

      const ChildComponent = {
        setup() {
          throw new Error('Test error')
        },
        template: '<div>Child</div>'
      }

      const wrapper = mountWithTeleport({
        default: h(ChildComponent)
      })

      await flushPromises()

      // Should still show error alert even if backend logging fails
      expect(wrapper.find('div[role="alert"]').exists()).toBe(true)
      expect(consoleSpy).toHaveBeenCalledWith(
        '[ErrorBoundary] Failed to log error to backend:',
        expect.any(Error)
      )

      consoleSpy.mockRestore()
    })
  })

  describe('resetError', () => {
    it('hides error alert when resetError is called', async () => {
      const ChildComponent = {
        setup() {
          throw new Error('Test error')
        },
        template: '<div>Child</div>'
      }

      const wrapper = mountWithTeleport({
        default: h(ChildComponent)
      })

      await flushPromises()

      // Verify error alert is shown
      expect(wrapper.find('div[role="alert"]').exists()).toBe(true)

      // Call resetError
      const instance = wrapper.vm as { resetError: () => void }
      instance.resetError()

      await nextTick()

      // Error alert should be hidden
      expect(wrapper.find('div[role="alert"]').exists()).toBe(false)
    })

    it('exposes resetError via defineExpose', async () => {
      const wrapper = mountWithTeleport({
        default: h('div', 'Content')
      })

      expect(wrapper.vm.resetError).toBeDefined()
      expect(typeof wrapper.vm.resetError).toBe('function')
    })
  })
})