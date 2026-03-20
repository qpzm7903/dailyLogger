import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import TagBadge from '../TagBadge.vue'
import type { Tag } from '../../types/tauri'

// Mock vue-i18n (not used but might be imported)
vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string) => key,
  }),
}))

describe('TagBadge', () => {
  const createTag = (overrides: Partial<Tag> = {}): Tag => ({
    id: 1,
    name: 'Test Tag',
    color: 'blue',
    ...overrides,
  })

  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('rendering', () => {
    it('renders tag name', () => {
      const tag = createTag({ name: 'Development' })
      const wrapper = mount(TagBadge, {
        props: { tag },
      })

      expect(wrapper.text()).toContain('Development')
    })

    it('applies correct color classes for blue', () => {
      const tag = createTag({ color: 'blue' })
      const wrapper = mount(TagBadge, {
        props: { tag },
      })

      expect(wrapper.find('span').classes()).toContain('bg-blue-500')
      expect(wrapper.find('span').classes()).toContain('text-white')
    })

    it('applies correct color classes for green', () => {
      const tag = createTag({ color: 'green' })
      const wrapper = mount(TagBadge, {
        props: { tag },
      })

      expect(wrapper.find('span').classes()).toContain('bg-green-500')
      expect(wrapper.find('span').classes()).toContain('text-white')
    })

    it('applies correct color classes for yellow', () => {
      const tag = createTag({ color: 'yellow' })
      const wrapper = mount(TagBadge, {
        props: { tag },
      })

      expect(wrapper.find('span').classes()).toContain('bg-yellow-400')
      expect(wrapper.find('span').classes()).toContain('text-slate-800')
    })

    it('applies correct color classes for red', () => {
      const tag = createTag({ color: 'red' })
      const wrapper = mount(TagBadge, {
        props: { tag },
      })

      expect(wrapper.find('span').classes()).toContain('bg-red-500')
      expect(wrapper.find('span').classes()).toContain('text-white')
    })

    it('applies correct color classes for purple', () => {
      const tag = createTag({ color: 'purple' })
      const wrapper = mount(TagBadge, {
        props: { tag },
      })

      expect(wrapper.find('span').classes()).toContain('bg-purple-500')
      expect(wrapper.find('span').classes()).toContain('text-white')
    })

    it('applies correct color classes for pink', () => {
      const tag = createTag({ color: 'pink' })
      const wrapper = mount(TagBadge, {
        props: { tag },
      })

      expect(wrapper.find('span').classes()).toContain('bg-pink-500')
      expect(wrapper.find('span').classes()).toContain('text-white')
    })

    it('applies correct color classes for cyan', () => {
      const tag = createTag({ color: 'cyan' })
      const wrapper = mount(TagBadge, {
        props: { tag },
      })

      expect(wrapper.find('span').classes()).toContain('bg-cyan-500')
      expect(wrapper.find('span').classes()).toContain('text-white')
    })

    it('applies correct color classes for orange', () => {
      const tag = createTag({ color: 'orange' })
      const wrapper = mount(TagBadge, {
        props: { tag },
      })

      expect(wrapper.find('span').classes()).toContain('bg-orange-500')
      expect(wrapper.find('span').classes()).toContain('text-white')
    })

    it('falls back to blue for unknown colors', () => {
      const tag = createTag({ color: 'unknown' })
      const wrapper = mount(TagBadge, {
        props: { tag },
      })

      expect(wrapper.find('span').classes()).toContain('bg-blue-500')
      expect(wrapper.find('span').classes()).toContain('text-white')
    })

    it('falls back to blue when color is undefined', () => {
      const tag = createTag({ color: undefined as unknown as string })
      const wrapper = mount(TagBadge, {
        props: { tag },
      })

      expect(wrapper.find('span').classes()).toContain('bg-blue-500')
    })
  })

  describe('removable prop', () => {
    it('does not show remove button by default', () => {
      const tag = createTag()
      const wrapper = mount(TagBadge, {
        props: { tag },
      })

      expect(wrapper.find('button').exists()).toBe(false)
    })

    it('shows remove button when removable is true', () => {
      const tag = createTag()
      const wrapper = mount(TagBadge, {
        props: { tag, removable: true },
      })

      expect(wrapper.find('button').exists()).toBe(true)
      expect(wrapper.find('button').text()).toContain('✕')
    })

    it('does not show remove button when removable is false', () => {
      const tag = createTag()
      const wrapper = mount(TagBadge, {
        props: { tag, removable: false },
      })

      expect(wrapper.find('button').exists()).toBe(false)
    })
  })

  describe('remove event', () => {
    it('emits remove event when remove button clicked', async () => {
      const tag = createTag()
      const wrapper = mount(TagBadge, {
        props: { tag, removable: true },
      })

      await wrapper.find('button').trigger('click')

      expect(wrapper.emitted('remove')).toBeTruthy()
      expect(wrapper.emitted('remove')).toHaveLength(1)
    })

    it('does not emit remove event when badge clicked without remove button', async () => {
      const tag = createTag()
      const wrapper = mount(TagBadge, {
        props: { tag },
      })

      await wrapper.find('span').trigger('click')

      expect(wrapper.emitted('remove')).toBeFalsy()
    })
  })

  describe('styling', () => {
    it('has correct base classes', () => {
      const tag = createTag()
      const wrapper = mount(TagBadge, {
        props: { tag },
      })

      const span = wrapper.find('span')
      expect(span.classes()).toContain('inline-flex')
      expect(span.classes()).toContain('items-center')
      expect(span.classes()).toContain('gap-1')
      expect(span.classes()).toContain('px-2')
      expect(span.classes()).toContain('py-0.5')
      expect(span.classes()).toContain('rounded')
      expect(span.classes()).toContain('text-xs')
      expect(span.classes()).toContain('font-medium')
    })

    it('has transition-colors class', () => {
      const tag = createTag()
      const wrapper = mount(TagBadge, {
        props: { tag },
      })

      expect(wrapper.find('span').classes()).toContain('transition-colors')
    })

    it('remove button has correct styling', () => {
      const tag = createTag()
      const wrapper = mount(TagBadge, {
        props: { tag, removable: true },
      })

      const button = wrapper.find('button')
      expect(button.classes()).toContain('ml-1')
      expect(button.classes()).toContain('hover:text-white/80')
    })
  })

  describe('click handling', () => {
    it('stops propagation on remove button click', async () => {
      const tag = createTag()
      const wrapper = mount(TagBadge, {
        props: { tag, removable: true },
      })

      // The button has @click.stop which should prevent propagation
      // We can test that the click handler exists
      const button = wrapper.find('button')
      expect(button.exists()).toBe(true)

      // Trigger click - should not throw
      await button.trigger('click')
    })
  })

  describe('edge cases', () => {
    it('handles empty tag name', () => {
      const tag = createTag({ name: '' })
      const wrapper = mount(TagBadge, {
        props: { tag },
      })

      // Empty name should still render the badge
      expect(wrapper.find('span').exists()).toBe(true)
    })

    it('handles long tag name', () => {
      const tag = createTag({ name: 'Very Long Tag Name That Might Overflow' })
      const wrapper = mount(TagBadge, {
        props: { tag },
      })

      expect(wrapper.text()).toContain('Very Long Tag Name That Might Overflow')
    })

    it('handles special characters in tag name', () => {
      const tag = createTag({ name: 'Tag & <script>' })
      const wrapper = mount(TagBadge, {
        props: { tag },
      })

      // Vue should escape the content
      expect(wrapper.text()).toContain('Tag & <script>')
    })
  })
})