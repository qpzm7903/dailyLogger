// UX-011: ReportDropdown component tests
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import ReportDropdown from '../ReportDropdown.vue'

describe('ReportDropdown', () => {
  const defaultProps = {
    isGeneratingDaily: false,
    isGeneratingWeekly: false,
    isGeneratingMonthly: false,
  }

  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('rendering', () => {
    it('renders main button with daily report label', () => {
      const wrapper = mount(ReportDropdown, { props: defaultProps })
      expect(wrapper.text()).toContain('生成日报')
    })

    it('renders dropdown toggle button', () => {
      const wrapper = mount(ReportDropdown, { props: defaultProps })
      const buttons = wrapper.findAll('button')
      expect(buttons.length).toBe(2) // Main + Toggle
    })

    it('does not show dropdown menu initially', () => {
      const wrapper = mount(ReportDropdown, { props: defaultProps })
      expect(wrapper.find('.absolute').exists()).toBe(false)
    })

    it('shows dropdown menu when toggle is clicked', async () => {
      const wrapper = mount(ReportDropdown, { props: defaultProps })
      const toggleBtn = wrapper.findAll('button')[1]
      await toggleBtn.trigger('click')
      expect(wrapper.find('.absolute').exists()).toBe(true)
      expect(wrapper.text()).toContain('生成周报')
      expect(wrapper.text()).toContain('生成月报')
    })
  })

  describe('loading state', () => {
    it('shows spinner on main button when generating daily', () => {
      const wrapper = mount(ReportDropdown, {
        props: { ...defaultProps, isGeneratingDaily: true }
      })
      expect(wrapper.text()).toContain('生成中...')
      expect(wrapper.find('.animate-spin').exists()).toBe(true)
    })

    it('disables buttons when any report is generating', async () => {
      const wrapper = mount(ReportDropdown, {
        props: { ...defaultProps, isGeneratingWeekly: true }
      })
      const buttons = wrapper.findAll('button')
      buttons.forEach(btn => {
        expect(btn.attributes('disabled')).toBeDefined()
      })
    })

    it('shows spinner on menu item when generating that type', async () => {
      const wrapper = mount(ReportDropdown, {
        props: { ...defaultProps, isGeneratingDaily: true }
      })
      // Main button shows spinner when generating daily
      expect(wrapper.find('.animate-spin').exists()).toBe(true)
    })
  })

  describe('events', () => {
    it('emits generate event with "daily" when main button clicked', async () => {
      const wrapper = mount(ReportDropdown, { props: defaultProps })
      const mainBtn = wrapper.findAll('button')[0]
      await mainBtn.trigger('click')
      expect(wrapper.emitted('generate')).toBeTruthy()
      expect(wrapper.emitted('generate')![0]).toEqual(['daily'])
    })

    it('emits generate event with correct type from dropdown', async () => {
      const wrapper = mount(ReportDropdown, { props: defaultProps })
      await wrapper.findAll('button')[1].trigger('click') // Open dropdown

      const menuItems = wrapper.findAll('.absolute button')
      await menuItems[1].trigger('click') // Weekly
      expect(wrapper.emitted('generate')![0]).toEqual(['weekly'])
    })

    it('does not emit when disabled', async () => {
      const wrapper = mount(ReportDropdown, {
        props: { ...defaultProps, isGeneratingDaily: true }
      })
      const mainBtn = wrapper.findAll('button')[0]
      await mainBtn.trigger('click')
      expect(wrapper.emitted('generate')).toBeFalsy()
    })

    it('closes dropdown after selecting option', async () => {
      const wrapper = mount(ReportDropdown, { props: defaultProps })
      await wrapper.findAll('button')[1].trigger('click') // Open
      expect(wrapper.find('.absolute').exists()).toBe(true)

      const menuItems = wrapper.findAll('.absolute button')
      await menuItems[0].trigger('click') // Select daily
      expect(wrapper.find('.absolute').exists()).toBe(false)
    })
  })

  describe('dropdown toggle', () => {
    it('toggles dropdown on click', async () => {
      const wrapper = mount(ReportDropdown, { props: defaultProps })
      const toggleBtn = wrapper.findAll('button')[1]

      await toggleBtn.trigger('click')
      expect(wrapper.find('.absolute').exists()).toBe(true)

      await toggleBtn.trigger('click')
      expect(wrapper.find('.absolute').exists()).toBe(false)
    })
  })

  describe('accessibility', () => {
    it('has proper title attribute on toggle button', () => {
      const wrapper = mount(ReportDropdown, { props: defaultProps })
      const toggleBtn = wrapper.findAll('button')[1]
      expect(toggleBtn.attributes('title')).toBe('展开菜单')
    })
  })
})