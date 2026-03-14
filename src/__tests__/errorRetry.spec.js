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
  add: vi.fn((toast) => {
    const id = Date.now()
    mockToastStore.toasts.value.push({ ...toast, id })
    return id
  }),
  remove: vi.fn((id) => {
    const index = mockToastStore.toasts.value.findIndex(t => t.id === id)
    if (index !== -1) {
      mockToastStore.toasts.value.splice(index, 1)
    }
  })
}

vi.mock('../stores/toast.js', () => ({
  useToastStore: () => mockToastStore,
  showError: vi.fn((error, retryCallback) => {
    return mockToastStore.add({
      message: 'Network error',
      type: 'error',
      suggestion: 'Retry',
      retryCallback
    })
  }),
  showSuccess: vi.fn((message) => {
    return mockToastStore.add({
      message,
      type: 'success'
    })
  })
}))

// Mock Tauri API
const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (cmd, args) => mockInvoke(cmd, args)
}))

import Toast from '../components/Toast.vue'

describe('Error State and Retry', () => {
  beforeEach(() => {
    mockToastStore.toasts.value = []
    mockToastStore.add.mockClear()
    mockToastStore.remove.mockClear()
    mockInvoke.mockReset()
  })

  describe('Toast with retry', () => {
    it('should show retry button when retryCallback is provided', async () => {
      const retryCallback = vi.fn()
      mockToastStore.toasts.value = [{
        id: 1,
        message: 'Network error',
        type: 'error',
        suggestion: 'Retry',
        retryCallback
      }]

      const wrapper = mount(Toast)
      await nextTick()

      expect(wrapper.find('.btn-retry').exists()).toBe(true)
      expect(wrapper.text()).toContain('重试')
    })

    it('should call retryCallback when retry button is clicked', async () => {
      const retryCallback = vi.fn()
      mockToastStore.toasts.value = [{
        id: 1,
        message: 'Network error',
        type: 'error',
        suggestion: 'Retry',
        retryCallback
      }]

      const wrapper = mount(Toast)
      await nextTick()

      await wrapper.find('.btn-retry').trigger('click')
      expect(retryCallback).toHaveBeenCalled()
    })

    it('should remove toast after retry', async () => {
      const retryCallback = vi.fn()
      mockToastStore.toasts.value = [{
        id: 1,
        message: 'Network error',
        type: 'error',
        suggestion: 'Retry',
        retryCallback
      }]

      const wrapper = mount(Toast)
      await nextTick()

      await wrapper.find('.btn-retry').trigger('click')
      expect(mockToastStore.remove).toHaveBeenCalledWith(1)
    })
  })

  describe('Error display', () => {
    it('should show error message with red styling', async () => {
      mockToastStore.toasts.value = [{
        id: 1,
        message: 'API Key 无效',
        type: 'error'
      }]

      const wrapper = mount(Toast)
      await nextTick()

      expect(wrapper.find('.border-red-700').exists()).toBe(true)
      expect(wrapper.text()).toContain('API Key 无效')
    })

    it('should show suggestion when provided', async () => {
      mockToastStore.toasts.value = [{
        id: 1,
        message: 'Network error',
        type: 'error',
        suggestion: 'Check network settings'
      }]

      const wrapper = mount(Toast)
      await nextTick()

      expect(wrapper.text()).toContain('Check network settings')
    })
  })

  describe('Multiple error toasts', () => {
    it('should handle multiple error toasts in queue', async () => {
      mockToastStore.toasts.value = [
        { id: 1, message: 'First error', type: 'error' },
        { id: 2, message: 'Second error', type: 'error' }
      ]

      const wrapper = mount(Toast)
      await nextTick()

      expect(wrapper.text()).toContain('First error')
      expect(wrapper.text()).toContain('Second error')
    })
  })
})