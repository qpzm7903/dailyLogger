import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { h } from 'vue'
import Header from '../Header.vue'

// Mock i18n
vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string, params?: Record<string, string | number>) => {
      const translations: Record<string, string> = {
        'header.running': 'Running',
        'header.paused': 'Paused',
        'header.records': 'records',
        'header.pendingSync': 'Pending sync ({{ count }})'
      }
      let result = translations[key] || key
      if (params) {
        Object.entries(params).forEach(([k, v]) => {
          result = result.replace(`{{ ${k} }}`, String(v))
        })
      }
      return result
    }
  })
}))

describe('Header', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('rendering', () => {
    it('renders with correct base classes', () => {
      const wrapper = mount(Header, {
        props: {
          isOnline: true,
          offlineQueueCount: 0,
          currentTime: '12:00:00',
          autoCaptureEnabled: true,
          todayRecordsCount: 5
        }
      })
      const header = wrapper.find('header')
      expect(header.exists()).toBe(true)
      expect(header.classes()).toContain('flex')
      expect(header.classes()).toContain('items-center')
    })

    it('displays DailyLogger title', () => {
      const wrapper = mount(Header, {
        props: {
          isOnline: true,
          offlineQueueCount: 0,
          currentTime: '12:00:00',
          autoCaptureEnabled: true,
          todayRecordsCount: 5
        }
      })
      expect(wrapper.find('h1').text()).toBe('DailyLogger')
    })

    it('displays current time', () => {
      const wrapper = mount(Header, {
        props: {
          isOnline: true,
          offlineQueueCount: 0,
          currentTime: '14:30:00',
          autoCaptureEnabled: true,
          todayRecordsCount: 5
        }
      })
      expect(wrapper.text()).toContain('14:30:00')
    })

    it('displays today records count', () => {
      const wrapper = mount(Header, {
        props: {
          isOnline: true,
          offlineQueueCount: 0,
          currentTime: '12:00:00',
          autoCaptureEnabled: true,
          todayRecordsCount: 10
        }
      })
      expect(wrapper.text()).toContain('10')
      expect(wrapper.text()).toContain('records')
    })
  })

  describe('auto capture status', () => {
    it('shows running status when autoCaptureEnabled is true', () => {
      const wrapper = mount(Header, {
        props: {
          isOnline: true,
          offlineQueueCount: 0,
          currentTime: '12:00:00',
          autoCaptureEnabled: true,
          todayRecordsCount: 5
        }
      })
      expect(wrapper.text()).toContain('Running')
    })

    it('shows paused status when autoCaptureEnabled is false', () => {
      const wrapper = mount(Header, {
        props: {
          isOnline: true,
          offlineQueueCount: 0,
          currentTime: '12:00:00',
          autoCaptureEnabled: false,
          todayRecordsCount: 5
        }
      })
      expect(wrapper.text()).toContain('Paused')
    })

    it('shows green indicator when autoCaptureEnabled is true', () => {
      const wrapper = mount(Header, {
        props: {
          isOnline: true,
          offlineQueueCount: 0,
          currentTime: '12:00:00',
          autoCaptureEnabled: true,
          todayRecordsCount: 5
        }
      })
      const indicator = wrapper.find('.bg-status-success')
      expect(indicator.exists()).toBe(true)
    })

    it('shows gray indicator when autoCaptureEnabled is false', () => {
      const wrapper = mount(Header, {
        props: {
          isOnline: true,
          offlineQueueCount: 0,
          currentTime: '12:00:00',
          autoCaptureEnabled: false,
          todayRecordsCount: 5
        }
      })
      const indicator = wrapper.find('.bg-\\[var\\(--color-action-neutral\\)\\]')
      expect(indicator.exists()).toBe(true)
    })
  })

  describe('online status', () => {
    it('applies mt-9 class when isOnline is false', () => {
      const wrapper = mount(Header, {
        props: {
          isOnline: false,
          offlineQueueCount: 0,
          currentTime: '12:00:00',
          autoCaptureEnabled: true,
          todayRecordsCount: 5
        }
      })
      const header = wrapper.find('header')
      expect(header.classes()).toContain('mt-9')
    })

    it('does not apply mt-9 class when isOnline is true', () => {
      const wrapper = mount(Header, {
        props: {
          isOnline: true,
          offlineQueueCount: 0,
          currentTime: '12:00:00',
          autoCaptureEnabled: true,
          todayRecordsCount: 5
        }
      })
      const header = wrapper.find('header')
      expect(header.classes()).not.toContain('mt-9')
    })
  })

  describe('offline queue', () => {
    it('does not show pending sync badge when offlineQueueCount is 0', () => {
      const wrapper = mount(Header, {
        props: {
          isOnline: true,
          offlineQueueCount: 0,
          currentTime: '12:00:00',
          autoCaptureEnabled: true,
          todayRecordsCount: 5
        }
      })
      expect(wrapper.text()).not.toContain('Pending sync')
    })

    it('shows pending sync badge when offlineQueueCount is greater than 0', () => {
      const wrapper = mount(Header, {
        props: {
          isOnline: true,
          offlineQueueCount: 3,
          currentTime: '12:00:00',
          autoCaptureEnabled: true,
          todayRecordsCount: 5
        }
      })
      expect(wrapper.text()).toContain('Pending sync')
      expect(wrapper.text()).toContain('3')
    })

    it('emits showOfflineQueue when pending sync badge is clicked', async () => {
      const wrapper = mount(Header, {
        props: {
          isOnline: true,
          offlineQueueCount: 3,
          currentTime: '12:00:00',
          autoCaptureEnabled: true,
          todayRecordsCount: 5
        }
      })
      const badge = wrapper.find('button')
      await badge.trigger('click')
      expect(wrapper.emitted('showOfflineQueue')).toBeTruthy()
    })

    it('shows animated yellow dot for pending sync', () => {
      const wrapper = mount(Header, {
        props: {
          isOnline: true,
          offlineQueueCount: 3,
          currentTime: '12:00:00',
          autoCaptureEnabled: true,
          todayRecordsCount: 5
        }
      })
      const yellowDot = wrapper.find('.bg-yellow-400')
      expect(yellowDot.exists()).toBe(true)
      expect(yellowDot.classes()).toContain('animate-pulse')
    })
  })
})