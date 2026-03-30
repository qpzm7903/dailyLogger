import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import EmptyState from '../EmptyState.vue'

describe('EmptyState', () => {
  describe('type variations', () => {
    it('renders screenshots type with correct SVG', () => {
      const wrapper = mount(EmptyState, {
        props: { type: 'screenshots' }
      })
      expect(wrapper.find('svg').exists()).toBe(true)
      // Screenshots SVG has specific elements
      expect(wrapper.html()).toContain('rect')
      expect(wrapper.html()).toContain('circle')
    })

    it('renders dailyReport type with correct SVG', () => {
      const wrapper = mount(EmptyState, {
        props: { type: 'dailyReport' }
      })
      expect(wrapper.find('svg').exists()).toBe(true)
    })

    it('renders searchResults type with correct SVG', () => {
      const wrapper = mount(EmptyState, {
        props: { type: 'searchResults' }
      })
      expect(wrapper.find('svg').exists()).toBe(true)
      // Search results SVG has a circle (magnifying glass)
      expect(wrapper.html()).toContain('circle')
    })

    it('renders generic type as default', () => {
      const wrapper = mount(EmptyState, {
        props: { type: 'generic' }
      })
      expect(wrapper.find('svg').exists()).toBe(true)
    })
  })

  describe('description prop', () => {
    it('displays custom description when provided', () => {
      const wrapper = mount(EmptyState, {
        props: {
          type: 'generic',
          description: 'No items found'
        }
      })
      expect(wrapper.find('p').text()).toBe('No items found')
    })

    it('prefers description prop over slot content', () => {
      const wrapper = mount(EmptyState, {
        props: {
          type: 'generic',
          description: 'Custom description'
        },
        slots: {
          default: 'Slot content'
        }
      })
      expect(wrapper.find('p').text()).toBe('Custom description')
    })
  })

  describe('slot content', () => {
    it('displays slot content when no description is provided', () => {
      const wrapper = mount(EmptyState, {
        props: { type: 'generic' },
        slots: {
          default: 'No screenshots yet'
        }
      })
      expect(wrapper.find('p').text()).toBe('No screenshots yet')
    })
  })

  describe('styling', () => {
    it('has correct container classes', () => {
      const wrapper = mount(EmptyState, {
        props: { type: 'generic' }
      })
      const container = wrapper.find('div')
      expect(container.classes()).toContain('flex')
      expect(container.classes()).toContain('flex-col')
      expect(container.classes()).toContain('items-center')
      expect(container.classes()).toContain('justify-center')
      expect(container.classes()).toContain('py-8')
      expect(container.classes()).toContain('text-[var(--color-text-muted)]')
    })

    it('SVG has opacity styling', () => {
      const wrapper = mount(EmptyState, {
        props: { type: 'generic' }
      })
      const svgContainer = wrapper.find('div.mb-4')
      expect(svgContainer.classes()).toContain('mb-4')
      expect(svgContainer.classes()).toContain('opacity-60')
    })

    it('description text has correct styling', () => {
      const wrapper = mount(EmptyState, {
        props: {
          type: 'generic',
          description: 'Test description'
        }
      })
      const paragraph = wrapper.find('p')
      expect(paragraph.classes()).toContain('text-sm')
      expect(paragraph.classes()).toContain('text-center')
      expect(paragraph.classes()).toContain('max-w-xs')
    })
  })
})
