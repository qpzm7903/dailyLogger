import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount } from '@vue/test-utils'
import OfflineBanner from '../OfflineBanner.vue'

// Mock useI18n
const mockT = (key: string) => {
  const translations: Record<string, string> = {
    'offlineBanner.offline': '当前处于离线状态，AI 功能暂不可用',
    'offlineBanner.reconnected': '网络已恢复'
  }
  return translations[key] || key
}

vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: mockT
  })
}))

describe('OfflineBanner.vue', () => {
  describe('Initial state', () => {
    it('should show banner when initially offline', () => {
      const wrapper = mount(OfflineBanner, {
        props: { isOnline: false }
      })
      expect(wrapper.find('.bg-yellow-600').exists()).toBe(true)
      expect(wrapper.text()).toContain('当前处于离线状态')
    })

    it('should not show banner when initially online', () => {
      const wrapper = mount(OfflineBanner, {
        props: { isOnline: true }
      })
      expect(wrapper.find('.bg-yellow-600').exists()).toBe(false)
      expect(wrapper.find('.bg-green-600').exists()).toBe(false)
    })
  })

  describe('Network status transitions', () => {
    it('should show yellow banner when going offline', async () => {
      const wrapper = mount(OfflineBanner, {
        props: { isOnline: true }
      })
      expect(wrapper.find('.bg-yellow-600').exists()).toBe(false)

      await wrapper.setProps({ isOnline: false })
      expect(wrapper.find('.bg-yellow-600').exists()).toBe(true)
      expect(wrapper.text()).toContain('当前处于离线状态')
    })

    it('should show green banner when coming back online', async () => {
      vi.useFakeTimers()
      const wrapper = mount(OfflineBanner, {
        props: { isOnline: false }
      })
      expect(wrapper.find('.bg-yellow-600').exists()).toBe(true)

      await wrapper.setProps({ isOnline: true })
      expect(wrapper.find('.bg-green-600').exists()).toBe(true)
      expect(wrapper.text()).toContain('网络已恢复')

      // Fast-forward timer to auto-hide
      vi.advanceTimersByTime(3000)
      await wrapper.vm.$nextTick()

      vi.useRealTimers()
    })

    it('should hide banner after reconnection timeout', async () => {
      vi.useFakeTimers()
      const wrapper = mount(OfflineBanner, {
        props: { isOnline: false }
      })
      expect(wrapper.find('.bg-yellow-600').exists()).toBe(true)

      await wrapper.setProps({ isOnline: true })
      expect(wrapper.find('.bg-green-600').exists()).toBe(true)

      // Fast-forward past the 3s timeout
      vi.advanceTimersByTime(3000)
      await wrapper.vm.$nextTick()

      expect(wrapper.find('.bg-yellow-600').exists()).toBe(false)
      expect(wrapper.find('.bg-green-600').exists()).toBe(false)

      vi.useRealTimers()
    })
  })

  describe('Reconnection timer management', () => {
    it('should clear previous timer when going offline then online quickly', async () => {
      vi.useFakeTimers()
      const wrapper = mount(OfflineBanner, {
        props: { isOnline: false }
      })

      // Go online briefly
      await wrapper.setProps({ isOnline: true })
      expect(wrapper.find('.bg-green-600').exists()).toBe(true)

      // Go offline again before timer fires
      vi.advanceTimersByTime(1000) // Only 1s passed
      await wrapper.setProps({ isOnline: false })

      // Timer should be cleared, now show yellow
      expect(wrapper.find('.bg-yellow-600').exists()).toBe(true)
      expect(wrapper.find('.bg-green-600').exists()).toBe(false)

      vi.useRealTimers()
    })
  })
})
