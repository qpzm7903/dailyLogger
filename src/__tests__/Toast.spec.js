import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount, config } from '@vue/test-utils'
import { ref, nextTick } from 'vue'

// Stub Teleport for testing
config.global.stubs = {
  Teleport: {
    template: '<div><slot /></div>'
  }
}

// Mock the toast store
const mockToastStore = {
  toasts: ref([]),
  remove: vi.fn()
}

vi.mock('../stores/toast.js', () => ({
  useToastStore: () => mockToastStore
}))

import Toast from '../components/Toast.vue'

describe('Toast.vue', () => {
  beforeEach(() => {
    mockToastStore.toasts.value = []
    mockToastStore.remove.mockClear()
  })

  describe('rendering', () => {
    it('should not render when no toasts', () => {
      const wrapper = mount(Toast)
      expect(wrapper.find('.toast-container').exists()).toBe(false)
    })

    it('should render toast with message', async () => {
      mockToastStore.toasts.value = [{
        id: 1,
        message: 'Test message',
        type: 'info'
      }]
      const wrapper = mount(Toast)
      await nextTick()
      expect(wrapper.text()).toContain('Test message')
    })

    it('should render error type with warning icon', async () => {
      mockToastStore.toasts.value = [{
        id: 1,
        message: 'Error message',
        type: 'error'
      }]
      const wrapper = mount(Toast)
      await nextTick()
      expect(wrapper.find('.text-red-400').exists()).toBe(true)
    })

    it('should render success type with check icon', async () => {
      mockToastStore.toasts.value = [{
        id: 1,
        message: 'Success message',
        type: 'success'
      }]
      const wrapper = mount(Toast)
      await nextTick()
      expect(wrapper.find('.text-green-400').exists()).toBe(true)
    })

    it('should render warning type', async () => {
      mockToastStore.toasts.value = [{
        id: 1,
        message: 'Warning message',
        type: 'warning'
      }]
      const wrapper = mount(Toast)
      await nextTick()
      expect(wrapper.find('.text-yellow-400').exists()).toBe(true)
    })
  })

  describe('suggestion display', () => {
    it('should show suggestion when provided', async () => {
      mockToastStore.toasts.value = [{
        id: 1,
        message: 'Error message',
        type: 'error',
        suggestion: 'Check settings'
      }]
      const wrapper = mount(Toast)
      await nextTick()
      expect(wrapper.text()).toContain('Check settings')
    })

    it('should not show suggestion when not provided', async () => {
      mockToastStore.toasts.value = [{
        id: 1,
        message: 'Error message',
        type: 'error'
      }]
      const wrapper = mount(Toast)
      await nextTick()
      expect(wrapper.find('.toast-suggestion').exists()).toBe(false)
    })
  })

  describe('retry button', () => {
    it('should show retry button when retryCallback is provided', async () => {
      mockToastStore.toasts.value = [{
        id: 1,
        message: 'Network error',
        type: 'error',
        retryCallback: vi.fn()
      }]
      const wrapper = mount(Toast)
      await nextTick()
      expect(wrapper.text()).toContain('重试')
    })

    it('should not show retry button when no retryCallback', async () => {
      mockToastStore.toasts.value = [{
        id: 1,
        message: 'Error message',
        type: 'error'
      }]
      const wrapper = mount(Toast)
      await nextTick()
      expect(wrapper.text()).not.toContain('重试')
    })

    it('should call retryCallback when retry button clicked', async () => {
      const retryCallback = vi.fn()
      mockToastStore.toasts.value = [{
        id: 1,
        message: 'Network error',
        type: 'error',
        retryCallback
      }]
      const wrapper = mount(Toast)
      await nextTick()
      await wrapper.find('.btn-retry').trigger('click')
      expect(retryCallback).toHaveBeenCalled()
    })
  })

  describe('close functionality', () => {
    it('should call remove when close button clicked', async () => {
      mockToastStore.toasts.value = [{
        id: 1,
        message: 'Test message',
        type: 'info'
      }]
      const wrapper = mount(Toast)
      await nextTick()
      await wrapper.find('.btn-close').trigger('click')
      expect(mockToastStore.remove).toHaveBeenCalledWith(1)
    })
  })

  describe('multiple toasts', () => {
    it('should render multiple toasts', async () => {
      mockToastStore.toasts.value = [
        { id: 1, message: 'First message', type: 'info' },
        { id: 2, message: 'Second message', type: 'error' }
      ]
      const wrapper = mount(Toast)
      await nextTick()
      expect(wrapper.text()).toContain('First message')
      expect(wrapper.text()).toContain('Second message')
    })
  })
})