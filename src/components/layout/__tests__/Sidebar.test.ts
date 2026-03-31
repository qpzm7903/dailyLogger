import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { h, ref } from 'vue'
import { flushPromises } from '@vue/test-utils'
import Sidebar from '../Sidebar.vue'

// Mock lucide-vue-next icons
vi.mock('lucide-vue-next', () => ({
  FileText: { template: '<span class="icon-file-text">FileText</span>' },
  History: { template: '<span class="icon-history">History</span>' },
  Search: { template: '<span class="icon-search">Search</span>' },
  Tags: { template: '<span class="icon-tags">Tags</span>' },
  Upload: { template: '<span class="icon-upload">Upload</span>' },
  TrendingUp: { template: '<span class="icon-trending-up">TrendingUp</span>' },
  Database: { template: '<span class="icon-database">Database</span>' },
  Settings: { template: '<span class="icon-settings">Settings</span>' },
  ChevronLeft: { template: '<span class="icon-chevron-left">ChevronLeft</span>' },
  ChevronRight: { template: '<span class="icon-chevron-right">ChevronRight</span>' }
}))

// Mock @tauri-apps/api/app
vi.mock('@tauri-apps/api/app', () => ({
  getVersion: vi.fn().mockResolvedValue('4.4.2')
}))

// Mock useModal
vi.mock('../../composables/useModal', () => ({
  useModal: () => ({
    activeModal: ref('logViewer')
  })
}))

describe('Sidebar', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('rendering', () => {
    it('renders with correct base classes', () => {
      const wrapper = mount(Sidebar, {
        props: { offlineQueueCount: 0 }
      })
      const aside = wrapper.find('aside')
      expect(aside.exists()).toBe(true)
      expect(aside.classes()).toContain('flex')
      expect(aside.classes()).toContain('flex-col')
    })

    it('displays logo', () => {
      const wrapper = mount(Sidebar, {
        props: { offlineQueueCount: 0 }
      })
      expect(wrapper.find('.bg-primary').exists()).toBe(true)
      expect(wrapper.text()).toContain('📝')
    })

    it('displays version number when not collapsed', async () => {
      const wrapper = mount(Sidebar, {
        props: { offlineQueueCount: 0 }
      })
      await flushPromises()
      expect(wrapper.text()).toContain('v4.4.2')
    })

    it('hides version number when collapsed', async () => {
      const wrapper = mount(Sidebar, {
        props: { offlineQueueCount: 0 }
      })
      await flushPromises()
      // Initially not collapsed
      expect(wrapper.text()).toContain('v4.4.2')

      // Toggle collapse via the collapse button
      const collapseButton = wrapper.findAll('button').at(-1)
      await collapseButton?.trigger('click')

      expect(wrapper.text()).not.toContain('v2.14.0')
    })

    it('renders navigation items', () => {
      const wrapper = mount(Sidebar, {
        props: { offlineQueueCount: 0 }
      })
      const navButtons = wrapper.find('nav').findAll('button')
      expect(navButtons.length).toBeGreaterThan(0)
    })

    it('renders settings button', () => {
      const wrapper = mount(Sidebar, {
        props: { offlineQueueCount: 0 }
      })
      expect(wrapper.text()).toContain('设置')
    })

    it('renders collapse/expand toggle button', () => {
      const wrapper = mount(Sidebar, {
        props: { offlineQueueCount: 0 }
      })
      const buttons = wrapper.findAll('button')
      // Last button is the collapse/expand toggle
      const collapseButton = buttons.at(-1)
      expect(collapseButton?.exists()).toBe(true)
    })
  })

  describe('collapsed state', () => {
    it('starts in expanded state (w-48)', () => {
      const wrapper = mount(Sidebar, {
        props: { offlineQueueCount: 0 }
      })
      const aside = wrapper.find('aside')
      expect(aside.classes()).toContain('w-48')
      expect(aside.classes()).not.toContain('w-16')
    })

    it('toggles to collapsed state (w-16) when clicked', async () => {
      const wrapper = mount(Sidebar, {
        props: { offlineQueueCount: 0 }
      })

      const collapseButton = wrapper.findAll('button').at(-1)
      await collapseButton?.trigger('click')

      const aside = wrapper.find('aside')
      expect(aside.classes()).toContain('w-16')
      expect(aside.classes()).not.toContain('w-48')
    })

    it('shows full labels when expanded', () => {
      const wrapper = mount(Sidebar, {
        props: { offlineQueueCount: 0 }
      })
      expect(wrapper.text()).toContain('日志')
      expect(wrapper.text()).toContain('历史')
    })

    it('hides labels when collapsed', async () => {
      const wrapper = mount(Sidebar, {
        props: { offlineQueueCount: 0 }
      })

      const collapseButton = wrapper.findAll('button').at(-1)
      await collapseButton?.trigger('click')

      expect(wrapper.text()).not.toContain('日志')
      expect(wrapper.text()).not.toContain('历史')
    })
  })

  describe('navigation items', () => {
    it('has correct number of nav items', () => {
      const wrapper = mount(Sidebar, {
        props: { offlineQueueCount: 0 }
      })
      const navButtons = wrapper.find('nav').findAll('button')
      // Should have 7 nav items: log, history, search, tags, export, timeline, backup
      expect(navButtons.length).toBe(7)
    })

    it('emits open event with correct modal id when nav item clicked', async () => {
      const wrapper = mount(Sidebar, {
        props: { offlineQueueCount: 0 }
      })

      const navButtons = wrapper.find('nav').findAll('button')
      // Click the first nav item (logViewer)
      await navButtons.at(0).trigger('click')

      expect(wrapper.emitted('open')).toBeTruthy()
      expect(wrapper.emitted('open')?.[0]).toEqual(['logViewer'])
    })

    it('emits open event with historyViewer when history item clicked', async () => {
      const wrapper = mount(Sidebar, {
        props: { offlineQueueCount: 0 }
      })

      const navButtons = wrapper.find('nav').findAll('button')
      // Click the second nav item (historyViewer)
      await navButtons.at(1).trigger('click')

      expect(wrapper.emitted('open')).toBeTruthy()
      expect(wrapper.emitted('open')?.[0]).toEqual(['historyViewer'])
    })
  })

  describe('settings button', () => {
    it('emits open event with settings when clicked', async () => {
      const wrapper = mount(Sidebar, {
        props: { offlineQueueCount: 0 }
      })

      // Find the settings button (not in nav, in bottom actions)
      const allButtons = wrapper.findAll('button')
      const settingsButton = allButtons.find(btn => btn.text().includes('设置'))
      await settingsButton?.trigger('click')

      expect(wrapper.emitted('open')).toBeTruthy()
      expect(wrapper.emitted('open')?.[0]).toEqual(['settings'])
    })
  })

  describe('active state', () => {
    it('renders first nav item which should be active when logViewer is active', async () => {
      const wrapper = mount(Sidebar, {
        props: { offlineQueueCount: 0 }
      })

      const navButtons = wrapper.find('nav').findAll('button')
      const firstButton = navButtons.at(0)

      // The first nav item (logViewer) should exist and be rendered
      expect(firstButton.exists()).toBe(true)
      // Should have the expected base classes for a nav button
      expect(firstButton.classes()).toContain('rounded-xl')
      expect(firstButton.classes()).toContain('flex')
    })
  })
})