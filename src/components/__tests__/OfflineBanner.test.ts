import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import OfflineBanner from '../OfflineBanner.vue'
import { createI18n } from 'vue-i18n'
import en from '../../locales/en.json'

// Create i18n instance for testing
const createTestI18n = () => {
  return createI18n({
    legacy: false,
    locale: 'en',
    messages: {
      en
    }
  })
}

describe('OfflineBanner', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  describe('initial state', () => {
    it('shows banner when initially offline', () => {
      const i18n = createTestI18n()
      const wrapper = mount(OfflineBanner, {
        props: {
          isOnline: false
        },
        global: {
          plugins: [i18n]
        }
      })

      const banner = wrapper.find('.fixed')
      expect(banner.exists()).toBe(true)
      expect(banner.classes()).toContain('bg-yellow-600')
    })

    it('hides banner when initially online', () => {
      const i18n = createTestI18n()
      const wrapper = mount(OfflineBanner, {
        props: {
          isOnline: true
        },
        global: {
          plugins: [i18n]
        }
      })

      expect(wrapper.find('.fixed').exists()).toBe(false)
    })
  })

  describe('offline state', () => {
    it('shows yellow banner when offline', () => {
      const i18n = createTestI18n()
      const wrapper = mount(OfflineBanner, {
        props: {
          isOnline: false
        },
        global: {
          plugins: [i18n]
        }
      })

      const banner = wrapper.find('.fixed')
      expect(banner.exists()).toBe(true)
      expect(banner.classes()).toContain('bg-yellow-600')
    })

    it('displays offline message', () => {
      const i18n = createTestI18n()
      const wrapper = mount(OfflineBanner, {
        props: {
          isOnline: false
        },
        global: {
          plugins: [i18n]
        }
      })

      expect(wrapper.text()).toContain('You are offline')
    })

    it('shows warning icon when offline', () => {
      const i18n = createTestI18n()
      const wrapper = mount(OfflineBanner, {
        props: {
          isOnline: false
        },
        global: {
          plugins: [i18n]
        }
      })

      expect(wrapper.text()).toContain('⚠')
    })
  })

  describe('reconnection', () => {
    it('shows green banner when reconnecting', async () => {
      const i18n = createTestI18n()
      const wrapper = mount(OfflineBanner, {
        props: {
          isOnline: false
        },
        global: {
          plugins: [i18n]
        }
      })

      // Initially offline - yellow banner
      expect(wrapper.find('.fixed').classes()).toContain('bg-yellow-600')

      // Reconnect
      await wrapper.setProps({ isOnline: true })
      await nextTick()

      // Should now show green banner
      expect(wrapper.find('.fixed').classes()).toContain('bg-green-600')
    })

    it('displays reconnected message when reconnecting', async () => {
      const i18n = createTestI18n()
      const wrapper = mount(OfflineBanner, {
        props: {
          isOnline: false
        },
        global: {
          plugins: [i18n]
        }
      })

      await wrapper.setProps({ isOnline: true })
      await nextTick()

      expect(wrapper.text()).toContain('Connection restored')
    })

    it('shows checkmark icon when reconnected', async () => {
      const i18n = createTestI18n()
      const wrapper = mount(OfflineBanner, {
        props: {
          isOnline: false
        },
        global: {
          plugins: [i18n]
        }
      })

      await wrapper.setProps({ isOnline: true })
      await nextTick()

      expect(wrapper.text()).toContain('✓')
    })

    it('hides banner after 3 seconds when reconnected', async () => {
      const i18n = createTestI18n()
      const wrapper = mount(OfflineBanner, {
        props: {
          isOnline: false
        },
        global: {
          plugins: [i18n]
        }
      })

      // Reconnect
      await wrapper.setProps({ isOnline: true })
      await nextTick()

      // Banner should still be visible
      expect(wrapper.find('.fixed').exists()).toBe(true)

      // Advance time by 3 seconds
      vi.advanceTimersByTime(3000)
      await nextTick()

      // Banner should now be hidden
      expect(wrapper.find('.fixed').exists()).toBe(false)
    })

    it('does not show banner when already online', async () => {
      const i18n = createTestI18n()
      const wrapper = mount(OfflineBanner, {
        props: {
          isOnline: true
        },
        global: {
          plugins: [i18n]
        }
      })

      // No banner when already online
      expect(wrapper.find('.fixed').exists()).toBe(false)

      // Stay online
      await wrapper.setProps({ isOnline: true })
      await nextTick()

      // Still no banner
      expect(wrapper.find('.fixed').exists()).toBe(false)
    })
  })

  describe('state transitions', () => {
    it('shows yellow banner again when going offline after reconnection', async () => {
      const i18n = createTestI18n()
      const wrapper = mount(OfflineBanner, {
        props: {
          isOnline: false
        },
        global: {
          plugins: [i18n]
        }
      })

      // Reconnect
      await wrapper.setProps({ isOnline: true })
      await nextTick()
      expect(wrapper.find('.fixed').classes()).toContain('bg-green-600')

      // Go offline again
      await wrapper.setProps({ isOnline: false })
      await nextTick()

      // Should show yellow banner again
      expect(wrapper.find('.fixed').classes()).toContain('bg-yellow-600')
      expect(wrapper.text()).toContain('You are offline')
    })

    it('cancels reconnection timer when going offline during reconnection display', async () => {
      const i18n = createTestI18n()
      const wrapper = mount(OfflineBanner, {
        props: {
          isOnline: false
        },
        global: {
          plugins: [i18n]
        }
      })

      // Reconnect
      await wrapper.setProps({ isOnline: true })
      await nextTick()

      // Advance time by 1 second (before 3 second timeout)
      vi.advanceTimersByTime(1000)

      // Go offline again
      await wrapper.setProps({ isOnline: false })
      await nextTick()

      // Should show yellow banner
      expect(wrapper.find('.fixed').classes()).toContain('bg-yellow-600')

      // Advance time by another 2 seconds (would have been 3 seconds total)
      vi.advanceTimersByTime(2000)
      await nextTick()

      // Banner should still be visible (timer was cancelled)
      expect(wrapper.find('.fixed').exists()).toBe(true)
      expect(wrapper.find('.fixed').classes()).toContain('bg-yellow-600')
    })
  })

  describe('styling', () => {
    it('has correct positioning classes', () => {
      const i18n = createTestI18n()
      const wrapper = mount(OfflineBanner, {
        props: {
          isOnline: false
        },
        global: {
          plugins: [i18n]
        }
      })

      const banner = wrapper.find('.fixed')
      expect(banner.classes()).toContain('fixed')
      expect(banner.classes()).toContain('top-0')
      expect(banner.classes()).toContain('left-0')
      expect(banner.classes()).toContain('right-0')
      expect(banner.classes()).toContain('z-50')
    })

    it('has correct text styling', () => {
      const i18n = createTestI18n()
      const wrapper = mount(OfflineBanner, {
        props: {
          isOnline: false
        },
        global: {
          plugins: [i18n]
        }
      })

      const banner = wrapper.find('.fixed')
      expect(banner.classes()).toContain('text-white')
      expect(banner.classes()).toContain('text-sm')
    })
  })
})