import { describe, it, expect, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import NetworkStatusIndicator from '../NetworkStatusIndicator.vue'

// Mock i18n
vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        'networkStatus.offline': 'Offline'
      }
      return translations[key] || key
    }
  })
}))

describe('NetworkStatusIndicator', () => {
  describe('rendering', () => {
    it('renders offline indicator when isOnline is false', () => {
      const wrapper = mount(NetworkStatusIndicator, {
        props: { isOnline: false }
      })
      expect(wrapper.find('[role="status"]').exists()).toBe(true)
    })

    it('does not render when isOnline is true', () => {
      const wrapper = mount(NetworkStatusIndicator, {
        props: { isOnline: true }
      })
      expect(wrapper.find('[role="status"]').exists()).toBe(false)
      expect(wrapper.findAll('div').length).toBe(0)
    })

    it('displays offline text', () => {
      const wrapper = mount(NetworkStatusIndicator, {
        props: { isOnline: false }
      })
      expect(wrapper.text()).toContain('Offline')
    })
  })

  describe('styling', () => {
    it('has correct container classes', () => {
      const wrapper = mount(NetworkStatusIndicator, {
        props: { isOnline: false }
      })
      const container = wrapper.find('div')
      expect(container.classes()).toContain('flex')
      expect(container.classes()).toContain('items-center')
      expect(container.classes()).toContain('gap-2')
      expect(container.classes()).toContain('px-3')
      expect(container.classes()).toContain('py-1.5')
      expect(container.classes()).toContain('bg-amber-900/80')
      expect(container.classes()).toContain('border')
      expect(container.classes()).toContain('border-amber-700')
      expect(container.classes()).toContain('rounded-lg')
      expect(container.classes()).toContain('text-amber-100')
      expect(container.classes()).toContain('text-xs')
    })

    it('has animate-ping on status dot', () => {
      const wrapper = mount(NetworkStatusIndicator, {
        props: { isOnline: false }
      })
      const pingSpan = wrapper.find('.animate-ping')
      expect(pingSpan.exists()).toBe(true)
    })

    it('has relative positioning on dot container', () => {
      const wrapper = mount(NetworkStatusIndicator, {
        props: { isOnline: false }
      })
      const dotContainer = wrapper.find('.relative')
      expect(dotContainer.exists()).toBe(true)
    })
  })

  describe('accessibility', () => {
    it('has role status', () => {
      const wrapper = mount(NetworkStatusIndicator, {
        props: { isOnline: false }
      })
      expect(wrapper.find('[role="status"]').exists()).toBe(true)
    })

    it('has aria-label for screen readers', () => {
      const wrapper = mount(NetworkStatusIndicator, {
        props: { isOnline: false }
      })
      const statusEl = wrapper.find('[role="status"]')
      expect(statusEl.attributes('aria-label')).toBe('Offline')
    })
  })

  describe('pulse animation elements', () => {
    it('has ping animation span and dot span', () => {
      const wrapper = mount(NetworkStatusIndicator, {
        props: { isOnline: false }
      })
      // There are 4 spans total in the template:
      // 1. The ping animation outer span
      // 2. The ping animation inner span (with animate-ping)
      // 3. The relative dot container span
      // 4. The actual dot span
      const animatePingSpan = wrapper.find('.animate-ping')
      expect(animatePingSpan.exists()).toBe(true)
      const relativeSpan = wrapper.find('.relative')
      expect(relativeSpan.exists()).toBe(true)
    })

    it('both dot elements have h-2 w-2 classes', () => {
      const wrapper = mount(NetworkStatusIndicator, {
        props: { isOnline: false }
      })
      const h2w2Elements = wrapper.findAll('.h-2.w-2')
      expect(h2w2Elements.length).toBe(2)
    })
  })
})
