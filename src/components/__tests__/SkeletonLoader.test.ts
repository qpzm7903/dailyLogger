import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import SkeletonLoader from '../SkeletonLoader.vue'

describe('SkeletonLoader', () => {
  describe('rendering', () => {
    it('renders with default count of 3', () => {
      const wrapper = mount(SkeletonLoader)
      const skeletons = wrapper.findAll('.h-16')
      expect(skeletons.length).toBe(3)
    })

    it('renders with custom count', () => {
      const wrapper = mount(SkeletonLoader, {
        props: { count: 5 }
      })
      const skeletons = wrapper.findAll('.h-16')
      expect(skeletons.length).toBe(5)
    })

    it('renders with count of 1', () => {
      const wrapper = mount(SkeletonLoader, {
        props: { count: 1 }
      })
      const skeletons = wrapper.findAll('.h-16')
      expect(skeletons.length).toBe(1)
    })

    it('renders with count of 0', () => {
      const wrapper = mount(SkeletonLoader, {
        props: { count: 0 }
      })
      const skeletons = wrapper.findAll('.h-16')
      expect(skeletons.length).toBe(0)
    })
  })

  describe('styling', () => {
    it('has correct container classes', () => {
      const wrapper = mount(SkeletonLoader)
      const container = wrapper.find('.space-y-3')
      expect(container.exists()).toBe(true)
    })

    it('each skeleton has correct height class', () => {
      const wrapper = mount(SkeletonLoader)
      const skeletons = wrapper.findAll('.h-16')
      skeletons.forEach(skeleton => {
        expect(skeleton.classes()).toContain('h-16')
      })
    })

    it('each skeleton has rounded corners', () => {
      const wrapper = mount(SkeletonLoader)
      const skeletons = wrapper.findAll('.rounded-lg')
      expect(skeletons.length).toBe(3)
    })

    it('each skeleton has shimmer effect', () => {
      const wrapper = mount(SkeletonLoader)
      const skeletons = wrapper.findAll('.shimmer')
      expect(skeletons.length).toBe(3)
    })

    it('each skeleton takes full width', () => {
      const wrapper = mount(SkeletonLoader)
      const skeletons = wrapper.findAll('.w-full')
      expect(skeletons.length).toBe(3)
    })
  })

  describe('v-for rendering', () => {
    it('renders correct number of skeletons based on count prop', () => {
      const testCases = [1, 2, 3, 5, 10]
      testCases.forEach(count => {
        const wrapper = mount(SkeletonLoader, { props: { count } })
        expect(wrapper.findAll('div[class*="h-16"]').length).toBe(count)
      })
    })
  })
})
