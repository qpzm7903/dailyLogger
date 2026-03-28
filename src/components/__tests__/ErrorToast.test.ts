import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { nextTick } from 'vue'
import ErrorToast from '../ErrorToast.vue'

// Helper to stub Teleport
const mountWithTeleportStub = (props = {}) => {
  return mount(ErrorToast, {
    props: { title: 'Error', ...props },
    global: {
      stubs: {
        Teleport: {
          template: '<div><slot /></div>'
        }
      }
    }
  })
}

describe('ErrorToast', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  describe('rendering', () => {
    it('renders with required props', () => {
      const wrapper = mountWithTeleportStub({ title: 'Error occurred' })
      expect(wrapper.find('div[role="alert"]').exists()).toBe(true)
      expect(wrapper.text()).toContain('Error occurred')
    })

    it('displays message when provided', () => {
      const wrapper = mountWithTeleportStub({
        title: 'Error',
        message: 'Something went wrong'
      })
      expect(wrapper.text()).toContain('Something went wrong')
    })

    it('displays timestamp', () => {
      const wrapper = mountWithTeleportStub({ title: 'Error' })
      // The timestamp element exists with opacity class
      const timestampEl = wrapper.find('.text-xs')
      expect(timestampEl.exists()).toBe(true)
    })
  })

  describe('type variations', () => {
    it('applies error styling for error type', () => {
      const wrapper = mountWithTeleportStub({ title: 'Error', type: 'error' })
      const alertDiv = wrapper.find('div[role="alert"]')
      expect(alertDiv.classes()).toContain('bg-red-900/95')
      expect(alertDiv.classes()).toContain('border-red-700')
    })

    it('applies warning styling for warning type', () => {
      const wrapper = mountWithTeleportStub({ title: 'Warning', type: 'warning' })
      const alertDiv = wrapper.find('div[role="alert"]')
      expect(alertDiv.classes()).toContain('bg-yellow-900/95')
      expect(alertDiv.classes()).toContain('border-yellow-700')
    })

    it('applies success styling for success type', () => {
      const wrapper = mountWithTeleportStub({ title: 'Success', type: 'success' })
      const alertDiv = wrapper.find('div[role="alert"]')
      expect(alertDiv.classes()).toContain('bg-green-900/95')
      expect(alertDiv.classes()).toContain('border-green-700')
    })

    it('applies info styling for info type', () => {
      const wrapper = mountWithTeleportStub({ title: 'Info', type: 'info' })
      const alertDiv = wrapper.find('div[role="alert"]')
      expect(alertDiv.classes()).toContain('bg-blue-900/95')
      expect(alertDiv.classes()).toContain('border-blue-700')
    })
  })

  describe('icons', () => {
    it('displays warning icon for error type', () => {
      const wrapper = mountWithTeleportStub({ title: 'Error', type: 'error' })
      expect(wrapper.text()).toContain('⚠')
    })

    it('displays lightning icon for warning type', () => {
      const wrapper = mountWithTeleportStub({ title: 'Warning', type: 'warning' })
      expect(wrapper.text()).toContain('⚡')
    })

    it('displays checkmark icon for success type', () => {
      const wrapper = mountWithTeleportStub({ title: 'Success', type: 'success' })
      expect(wrapper.text()).toContain('✓')
    })

    it('displays info icon for info type', () => {
      const wrapper = mountWithTeleportStub({ title: 'Info', type: 'info' })
      expect(wrapper.text()).toContain('ℹ')
    })
  })

  describe('dismiss functionality', () => {
    it('shows dismiss button when dismissible is true', () => {
      const wrapper = mountWithTeleportStub({ title: 'Error', dismissible: true })
      expect(wrapper.find('button[aria-label="Dismiss"]').exists()).toBe(true)
    })

    it('hides dismiss button when dismissible is false', () => {
      const wrapper = mountWithTeleportStub({ title: 'Error', dismissible: false })
      expect(wrapper.find('button[aria-label="Dismiss"]').exists()).toBe(false)
    })

    it('emits dismiss event when button clicked', async () => {
      const wrapper = mountWithTeleportStub({ title: 'Error', dismissible: true })
      await wrapper.find('button[aria-label="Dismiss"]').trigger('click')
      expect(wrapper.emitted('dismiss')).toBeTruthy()
    })
  })

  describe('auto-dismiss', () => {
    it('auto-dismisses after specified duration', async () => {
      const wrapper = mountWithTeleportStub({ title: 'Error', duration: 5000 })
      expect(wrapper.emitted('dismiss')).toBeFalsy()

      vi.advanceTimersByTime(5000)
      await nextTick()

      expect(wrapper.emitted('dismiss')).toBeTruthy()
    })

    it('does not auto-dismiss when duration is 0', async () => {
      const wrapper = mountWithTeleportStub({ title: 'Error', duration: 0 })

      vi.advanceTimersByTime(10000)
      await nextTick()

      expect(wrapper.emitted('dismiss')).toBeFalsy()
    })
  })

  describe('Teleport stub', () => {
    it('renders with Teleport stub applied', () => {
      const wrapper = mountWithTeleportStub({ title: 'Error' })
      // The stub renders the content directly
      expect(wrapper.find('.fixed.bottom-4.right-4').exists()).toBe(true)
    })
  })
})
