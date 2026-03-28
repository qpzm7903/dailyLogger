import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { nextTick, defineComponent, h } from 'vue'
import Toast from '../Toast.vue'
import { useToastStore, showToast, showError, showSuccess, showWarning, showInfo } from '../../stores/toast.js'

// Helper to create a wrapper that includes the Toast component
const mountWithToast = () => {
  return mount(Toast, {
    global: {
      stubs: {
        Teleport: {
          template: '<div><slot /></div>'
        }
      }
    }
  })
}

describe('Toast', () => {
  let toastStore

  beforeEach(() => {
    // Clear toasts before each test
    toastStore = useToastStore()
    toastStore.clear()
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
    toastStore.clear()
  })

  describe('empty state', () => {
    it('does not render toast container when no toasts', () => {
      const wrapper = mountWithToast()
      expect(wrapper.find('.toast-container').exists()).toBe(false)
    })
  })

  describe('toast display', () => {
    it('renders toast container when toasts exist', async () => {
      const wrapper = mountWithToast()
      toastStore.add({ message: 'Test toast', type: 'info' })
      await nextTick()
      expect(wrapper.find('.toast-container').exists()).toBe(true)
    })

    it('displays toast message', async () => {
      const wrapper = mountWithToast()
      toastStore.add({ message: 'Hello World', type: 'info' })
      await nextTick()
      expect(wrapper.text()).toContain('Hello World')
    })

    it('displays multiple toasts', async () => {
      const wrapper = mountWithToast()
      toastStore.add({ message: 'First toast', type: 'info' })
      toastStore.add({ message: 'Second toast', type: 'success' })
      await nextTick()
      expect(wrapper.text()).toContain('First toast')
      expect(wrapper.text()).toContain('Second toast')
      expect(wrapper.findAll('.border').length).toBe(2)
    })

    it('displays suggestion when provided', async () => {
      const wrapper = mountWithToast()
      toastStore.add({
        message: 'Error occurred',
        type: 'error',
        suggestion: 'Please try again'
      })
      await nextTick()
      expect(wrapper.text()).toContain('Suggestion: Please try again')
    })

    it('does not display suggestion when not provided', async () => {
      const wrapper = mountWithToast()
      toastStore.add({ message: 'Simple toast', type: 'info' })
      await nextTick()
      expect(wrapper.text()).not.toContain('Suggestion')
    })
  })

  describe('toast types and styling', () => {
    it('applies error styling for error type', async () => {
      const wrapper = mountWithToast()
      toastStore.add({ message: 'Error', type: 'error' })
      await nextTick()
      const toastEl = wrapper.find('.border')
      expect(toastEl.attributes('class')).toMatch(/border-red-700|border-\[var\(--color-error\)/)
      expect(toastEl.classes()).toContain('bg-red-900/20')
    })

    it('applies success styling for success type', async () => {
      const wrapper = mountWithToast()
      toastStore.add({ message: 'Success', type: 'success' })
      await nextTick()
      const toastEl = wrapper.find('.border')
      expect(toastEl.attributes('class')).toMatch(/border-green-700|border-\[var\(--color-success\)/)
      expect(toastEl.classes()).toContain('bg-green-900/20')
    })

    it('applies warning styling for warning type', async () => {
      const wrapper = mountWithToast()
      toastStore.add({ message: 'Warning', type: 'warning' })
      await nextTick()
      const toastEl = wrapper.find('.border')
      expect(toastEl.attributes('class')).toMatch(/border-yellow-700|border-\[var\(--color-warning\)/)
      expect(toastEl.classes()).toContain('bg-yellow-900/20')
    })

    it('applies info styling for info type', async () => {
      const wrapper = mountWithToast()
      toastStore.add({ message: 'Info', type: 'info' })
      await nextTick()
      const toastEl = wrapper.find('.border')
      expect(toastEl.attributes('class')).toMatch(/border-gray-700|border-\[var\(--color-border\)/)
    })

    it('defaults to info type when type is not specified', async () => {
      const wrapper = mountWithToast()
      toastStore.add({ message: 'Default' })
      await nextTick()
      const toastEl = wrapper.find('.border')
      expect(toastEl.attributes('class')).toMatch(/border-gray-700|border-\[var\(--color-border\)/)
    })
  })

  describe('toast icons', () => {
    it('shows error icon for error type', async () => {
      const wrapper = mountWithToast()
      toastStore.add({ message: 'Error', type: 'error' })
      await nextTick()
      expect(wrapper.text()).toContain('⚠️')
    })

    it('shows success icon for success type', async () => {
      const wrapper = mountWithToast()
      toastStore.add({ message: 'Success', type: 'success' })
      await nextTick()
      expect(wrapper.text()).toContain('✓')
    })

    it('shows warning icon for warning type', async () => {
      const wrapper = mountWithToast()
      toastStore.add({ message: 'Warning', type: 'warning' })
      await nextTick()
      expect(wrapper.text()).toContain('⚡')
    })

    it('shows info icon for info type', async () => {
      const wrapper = mountWithToast()
      toastStore.add({ message: 'Info', type: 'info' })
      await nextTick()
      expect(wrapper.text()).toContain('ℹ️')
    })
  })

  describe('retry functionality', () => {
    it('shows retry and close buttons when retryCallback is provided', async () => {
      const wrapper = mountWithToast()
      toastStore.add({
        message: 'Error',
        type: 'error',
        retryCallback: () => {}
      })
      await nextTick()
      expect(wrapper.find('.btn-retry').exists()).toBe(true)
      expect(wrapper.findAll('.btn-close').length).toBe(1)
    })

    it('does not show retry button when retryCallback is not provided', async () => {
      const wrapper = mountWithToast()
      toastStore.add({ message: 'Info', type: 'info' })
      await nextTick()
      expect(wrapper.find('.btn-retry').exists()).toBe(false)
    })

    it('calls retryCallback and removes toast when retry is clicked', async () => {
      const retryCallback = vi.fn()
      const wrapper = mountWithToast()
      toastStore.add({
        message: 'Error',
        type: 'error',
        retryCallback
      })
      await nextTick()
      await wrapper.find('.btn-retry').trigger('click')
      expect(retryCallback).toHaveBeenCalledOnce()
      expect(toastStore.toasts.value.length).toBe(0)
    })
  })

  describe('close functionality', () => {
    it('removes toast when close button is clicked', async () => {
      const wrapper = mountWithToast()
      toastStore.add({ message: 'Test', type: 'info' })
      await nextTick()
      expect(toastStore.toasts.value.length).toBe(1)
      await wrapper.find('.btn-close').trigger('click')
      expect(toastStore.toasts.value.length).toBe(0)
    })

    it('removes correct toast when multiple toasts exist', async () => {
      const wrapper = mountWithToast()
      toastStore.add({ message: 'First', type: 'info' })
      toastStore.add({ message: 'Second', type: 'info' })
      await nextTick()
      expect(toastStore.toasts.value.length).toBe(2)
      // Click the first close button
      const closeButtons = wrapper.findAll('.btn-close')
      await closeButtons[0].trigger('click')
      expect(toastStore.toasts.value.length).toBe(1)
    })
  })

  describe('auto-dismiss', () => {
    it('auto-dismisses toast after duration', async () => {
      const wrapper = mountWithToast()
      toastStore.add({ message: 'Auto dismiss', type: 'info', duration: 3000 })
      await nextTick()
      expect(toastStore.toasts.value.length).toBe(1)

      // Advance time by 3 seconds
      vi.advanceTimersByTime(3000)
      await nextTick()

      expect(toastStore.toasts.value.length).toBe(0)
    })

    it('auto-dismisses error toasts after 5000ms', async () => {
      const wrapper = mountWithToast()
      toastStore.add({ message: 'Error', type: 'error' })
      await nextTick()
      expect(toastStore.toasts.value.length).toBe(1)

      // Error toasts should still be present at 4999ms
      vi.advanceTimersByTime(4999)
      await nextTick()
      expect(toastStore.toasts.value.length).toBe(1)

      // Auto-dismiss after 5000ms
      vi.advanceTimersByTime(1)
      await nextTick()
      expect(toastStore.toasts.value.length).toBe(0)
    })

    it('auto-dismisses success toast after default duration (3000ms)', async () => {
      const wrapper = mountWithToast()
      toastStore.add({ message: 'Success', type: 'success' })
      await nextTick()
      expect(toastStore.toasts.value.length).toBe(1)

      // Default success duration is 3000ms
      vi.advanceTimersByTime(3000)
      await nextTick()

      expect(toastStore.toasts.value.length).toBe(0)
    })
  })
})

describe('Toast Store Helper Functions', () => {
  let toastStore

  beforeEach(() => {
    toastStore = useToastStore()
    toastStore.clear()
  })

  afterEach(() => {
    toastStore.clear()
  })

  describe('showToast', () => {
    it('adds toast with message', () => {
      const id = showToast('Test message')
      expect(toastStore.toasts.value.length).toBe(1)
      expect(toastStore.toasts.value[0].message).toBe('Test message')
    })

    it('uses default info type when type not specified', () => {
      showToast('Test')
      expect(toastStore.toasts.value[0].type).toBe('info')
    })

    it('accepts custom options', () => {
      showToast('Test', { type: 'success', suggestion: 'Suggestion' })
      expect(toastStore.toasts.value[0].type).toBe('success')
      expect(toastStore.toasts.value[0].suggestion).toBe('Suggestion')
    })
  })

  describe('showSuccess', () => {
    it('adds success toast', () => {
      showSuccess('Operation completed')
      expect(toastStore.toasts.value.length).toBe(1)
      expect(toastStore.toasts.value[0].type).toBe('success')
      expect(toastStore.toasts.value[0].message).toBe('Operation completed')
    })
  })

  describe('showWarning', () => {
    it('adds warning toast', () => {
      showWarning('Warning message')
      expect(toastStore.toasts.value.length).toBe(1)
      expect(toastStore.toasts.value[0].type).toBe('warning')
    })
  })

  describe('showInfo', () => {
    it('adds info toast', () => {
      showInfo('Info message')
      expect(toastStore.toasts.value.length).toBe(1)
      expect(toastStore.toasts.value[0].type).toBe('info')
    })
  })

  describe('showError', () => {
    it('adds error toast', () => {
      showError('Something went wrong')
      expect(toastStore.toasts.value.length).toBe(1)
      expect(toastStore.toasts.value[0].type).toBe('error')
    })

    it('includes retry callback when provided', () => {
      const callback = vi.fn()
      showError('Error', callback)
      expect(toastStore.toasts.value[0].retryCallback).toBe(callback)
    })
  })

  describe('store methods', () => {
    it('remove removes specific toast', () => {
      const id1 = toastStore.add({ message: 'First', type: 'info' })
      toastStore.add({ message: 'Second', type: 'info' })
      expect(toastStore.toasts.value.length).toBe(2)

      toastStore.remove(id1)
      expect(toastStore.toasts.value.length).toBe(1)
      expect(toastStore.toasts.value[0].message).toBe('Second')
    })

    it('clear removes all toasts', () => {
      toastStore.add({ message: 'First', type: 'info' })
      toastStore.add({ message: 'Second', type: 'info' })
      toastStore.add({ message: 'Third', type: 'info' })
      expect(toastStore.toasts.value.length).toBe(3)

      toastStore.clear()
      expect(toastStore.toasts.value.length).toBe(0)
    })
  })
})