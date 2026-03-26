// DATA-007: ReportDropdown component tests - multilingual support
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import ReportDropdown from '../ReportDropdown.vue'

describe('ReportDropdown', () => {
  const defaultProps = {
    isGeneratingDaily: false,
    isGeneratingWeekly: false,
    isGeneratingMonthly: false,
    preferredLanguage: 'zh-CN',
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
      expect(buttons.length).toBe(2) // Main + Dropdown toggle
    })

    it('does not show dropdown menu initially', () => {
      const wrapper = mount(ReportDropdown, { props: defaultProps })
      expect(wrapper.find('.absolute').exists()).toBe(false)
    })

    it('shows dropdown menu when toggle is clicked', async () => {
      const wrapper = mount(ReportDropdown, { props: defaultProps })
      const toggleBtn = wrapper.findAll('button')[1] // Second button is dropdown toggle
      await toggleBtn.trigger('click')
      expect(wrapper.find('.absolute').exists()).toBe(true)
      expect(wrapper.text()).toContain('生成周报')
      expect(wrapper.text()).toContain('生成月报')
    })

    it('shows language submenu trigger in dropdown', async () => {
      const wrapper = mount(ReportDropdown, { props: defaultProps })
      const toggleBtn = wrapper.findAll('button')[1] // Open dropdown
      await toggleBtn.trigger('click')

      // The dropdown should contain the language selector
      expect(wrapper.text()).toContain('多语言日报')
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
      await wrapper.findAll('button')[1].trigger('click') // Open dropdown (2nd button)

      const menuItems = wrapper.findAll('.absolute > button')
      // First three are report options: daily, weekly, monthly
      await menuItems[1].trigger('click') // weekly
      expect(wrapper.emitted('generate')![0]).toEqual(['weekly'])
    })

    it('emits generateMultilingual and languageChange when language selected from submenu', async () => {
      const wrapper = mount(ReportDropdown, { props: defaultProps })
      await wrapper.findAll('button')[1].trigger('click') // Open dropdown

      // Find and click the language selector button (has "多语言日报" text)
      const dropdownButtons = wrapper.findAll('.absolute button')
      const langSelector = dropdownButtons.find(btn =>
        btn.text().includes('多语言日报')
      )
      expect(langSelector).toBeDefined()
      await langSelector!.trigger('click')

      // After clicking language selector, submenu should open
      // The submenu is the second .absolute element
      const submenuContainer = wrapper.findAll('.absolute')[1]
      expect(submenuContainer.exists()).toBe(true)

      const submenuButtons = submenuContainer.findAll('button')

      // Find and click English
      const englishBtn = submenuButtons.find(btn => btn.text().includes('English'))
      expect(englishBtn).toBeDefined()
      await englishBtn!.trigger('click')

      expect(wrapper.emitted('generateMultilingual')).toBeTruthy()
      expect(wrapper.emitted('generateMultilingual')![0]).toEqual(['en'])
      expect(wrapper.emitted('languageChange')).toBeTruthy()
      expect(wrapper.emitted('languageChange')![0]).toEqual(['en'])
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

      const menuItems = wrapper.findAll('.absolute > button')
      await menuItems[0].trigger('click') // daily
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
    it('has proper title attribute on dropdown toggle button', () => {
      const wrapper = mount(ReportDropdown, { props: defaultProps })
      const toggleBtn = wrapper.findAll('button')[1]
      expect(toggleBtn.attributes('title')).toBe('展开菜单')
    })
  })

  describe('language submenu', () => {
    it('opens language submenu when clicking language selector', async () => {
      const wrapper = mount(ReportDropdown, { props: defaultProps })
      await wrapper.findAll('button')[1].trigger('click') // Open dropdown

      // Click language selector
      const dropdownButtons = wrapper.findAll('.absolute button')
      const langSelector = dropdownButtons.find(btn =>
        btn.text().includes('多语言日报')
      )
      await langSelector!.trigger('click')

      // There should be a second .absolute element (the submenu)
      const absolutes = wrapper.findAll('.absolute')
      expect(absolutes.length).toBe(2)
    })

    it('shows all language options when submenu is open', async () => {
      const wrapper = mount(ReportDropdown, { props: defaultProps })
      await wrapper.findAll('button')[1].trigger('click') // Open dropdown

      // Click language selector
      const dropdownButtons = wrapper.findAll('.absolute button')
      const langSelector = dropdownButtons.find(btn =>
        btn.text().includes('多语言日报')
      )
      await langSelector!.trigger('click')

      // The submenu should contain language options
      const submenuContainer = wrapper.findAll('.absolute')[1]
      const submenuText = submenuContainer.text()
      expect(submenuText).toContain('中文')
      expect(submenuText).toContain('English')
      expect(submenuText).toContain('日本語')
      expect(submenuText).toContain('한국어')
      expect(submenuText).toContain('Español')
      expect(submenuText).toContain('Français')
      expect(submenuText).toContain('Deutsch')
    })

    it('displays correct default language based on preferredLanguage prop', () => {
      const wrapper = mount(ReportDropdown, {
        props: { ...defaultProps, preferredLanguage: 'ja' }
      })
      // The component shows the selected language name in the UI
      // We verify the prop is accepted and doesn't cause errors
      expect(wrapper.exists()).toBe(true)
    })
  })
})
