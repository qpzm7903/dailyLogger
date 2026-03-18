import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { ref } from 'vue'
import TagFilter from '../TagFilter.vue'

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

// Mock toast store
vi.mock('../../stores/toast', () => ({
  showError: vi.fn()
}))

import { invoke } from '@tauri-apps/api/core'
import { showError } from '../../stores/toast'

const mockTags = [
  { id: 1, name: 'work', color: 'blue', usage_count: 10 },
  { id: 2, name: 'personal', color: 'green', usage_count: 5 },
  { id: 3, name: 'urgent', color: 'red', usage_count: 3 },
  { id: 4, name: 'meeting', color: 'yellow', usage_count: 8 }
]

describe('TagFilter', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    invoke.mockResolvedValue(mockTags)
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  // Helper to mount component with props
  const mountTagFilter = (props = {}) => {
    return mount(TagFilter, {
      props: {
        modelValue: [],
        ...props
      },
      global: {
        stubs: {
          TagBadge: {
            template: `
              <div class="tag-badge">
                <span>{{ tag.name }}</span>
                <button v-if="removable" class="remove-btn" @click="$emit('remove')">×</button>
              </div>
            `,
            props: ['tag', 'removable']
          }
        }
      }
    })
  }

  describe('Rendering', () => {
    it('renders label text', async () => {
      const wrapper = mountTagFilter()
      await wrapper.vm.$nextTick()
      await new Promise(resolve => setTimeout(resolve, 0))

      expect(wrapper.text()).toContain('标签筛选:')
    })

    it('renders dropdown button with placeholder when no tags selected', async () => {
      const wrapper = mountTagFilter()
      await wrapper.vm.$nextTick()
      await new Promise(resolve => setTimeout(resolve, 0))

      const button = wrapper.find('button')
      expect(button.text()).toContain('选择标签筛选...')
    })

    it('renders dropdown button with different text when tags selected', async () => {
      const wrapper = mountTagFilter({
        modelValue: [mockTags[0]]
      })
      await wrapper.vm.$nextTick()
      await new Promise(resolve => setTimeout(resolve, 0))

      // Find the dropdown toggle button by its class
      const buttons = wrapper.findAll('button')
      const dropdownButton = buttons.find(btn => btn.classes().includes('w-full'))
      expect(dropdownButton.text()).toContain('添加更多标签...')
    })

    it('does not show clear all button when no tags selected', async () => {
      const wrapper = mountTagFilter()
      await wrapper.vm.$nextTick()
      await new Promise(resolve => setTimeout(resolve, 0))

      expect(wrapper.text()).not.toContain('清除全部')
    })

    it('shows clear all button when tags are selected', async () => {
      const wrapper = mountTagFilter({
        modelValue: [mockTags[0]]
      })
      await wrapper.vm.$nextTick()
      await new Promise(resolve => setTimeout(resolve, 0))

      expect(wrapper.text()).toContain('清除全部')
    })

    it('shows logic hint when multiple tags selected', async () => {
      const wrapper = mountTagFilter({
        modelValue: [mockTags[0], mockTags[1]]
      })
      await wrapper.vm.$nextTick()
      await new Promise(resolve => setTimeout(resolve, 0))

      expect(wrapper.text()).toContain('显示同时包含所有选中标签的记录')
    })

    it('does not show logic hint when single tag selected', async () => {
      const wrapper = mountTagFilter({
        modelValue: [mockTags[0]]
      })
      await wrapper.vm.$nextTick()
      await new Promise(resolve => setTimeout(resolve, 0))

      expect(wrapper.text()).not.toContain('显示同时包含所有选中标签的记录')
    })
  })

  describe('Tag Loading', () => {
    it('calls get_all_manual_tags on mount', async () => {
      mountTagFilter()
      await new Promise(resolve => setTimeout(resolve, 0))

      expect(invoke).toHaveBeenCalledWith('get_all_manual_tags')
    })

    it('loads and stores tags on successful API call', async () => {
      const wrapper = mountTagFilter()
      await new Promise(resolve => setTimeout(resolve, 0))
      await wrapper.vm.$nextTick()

      expect(wrapper.vm.allTags).toEqual(mockTags)
    })

    it('shows error when tag loading fails', async () => {
      invoke.mockRejectedValue(new Error('Network error'))

      const wrapper = mountTagFilter()
      await new Promise(resolve => setTimeout(resolve, 0))
      await wrapper.vm.$nextTick()

      expect(showError).toHaveBeenCalledWith('加载标签失败')
    })

    it('exposes loadTags method for parent components', async () => {
      const wrapper = mountTagFilter()
      await new Promise(resolve => setTimeout(resolve, 0))

      expect(typeof wrapper.vm.loadTags).toBe('function')
    })

    it('can reload tags via exposed loadTags method', async () => {
      const wrapper = mountTagFilter()
      await new Promise(resolve => setTimeout(resolve, 0))

      invoke.mockClear()
      invoke.mockResolvedValue([{ id: 5, name: 'new', color: 'purple' }])

      await wrapper.vm.loadTags()
      expect(invoke).toHaveBeenCalledWith('get_all_manual_tags')
    })
  })

  describe('Dropdown Behavior', () => {
    it('dropdown is hidden by default', async () => {
      const wrapper = mountTagFilter()
      await wrapper.vm.$nextTick()
      await new Promise(resolve => setTimeout(resolve, 0))

      const dropdown = wrapper.find('.absolute.top-full')
      expect(dropdown.exists()).toBe(false)
    })

    it('shows dropdown when button clicked', async () => {
      const wrapper = mountTagFilter()
      await wrapper.vm.$nextTick()
      await new Promise(resolve => setTimeout(resolve, 0))

      const button = wrapper.find('button')
      await button.trigger('click')

      expect(wrapper.vm.showDropdown).toBe(true)
    })

    it('hides dropdown when button clicked again', async () => {
      const wrapper = mountTagFilter()
      await wrapper.vm.$nextTick()
      await new Promise(resolve => setTimeout(resolve, 0))

      const button = wrapper.find('button')
      await button.trigger('click')
      expect(wrapper.vm.showDropdown).toBe(true)

      await button.trigger('click')
      expect(wrapper.vm.showDropdown).toBe(false)
    })

    it('dropdown shows available tags (not selected)', async () => {
      const wrapper = mountTagFilter({
        modelValue: [mockTags[0]] // work tag selected
      })
      await wrapper.vm.$nextTick()
      await new Promise(resolve => setTimeout(resolve, 0))

      // Find the dropdown toggle button (the one with placeholder text)
      const buttons = wrapper.findAll('button')
      const dropdownButton = buttons.find(btn => btn.text().includes('添加更多标签'))
      await dropdownButton.trigger('click')
      await wrapper.vm.$nextTick()

      const dropdown = wrapper.find('.absolute.top-full')
      expect(dropdown.exists()).toBe(true)

      // Should show 3 tags (not the selected 'work' tag)
      const tagButtons = dropdown.findAll('button')
      expect(tagButtons.length).toBe(3)
    })

    it('dropdown is hidden when no available tags', async () => {
      const wrapper = mountTagFilter({
        modelValue: mockTags // all tags selected
      })
      await wrapper.vm.$nextTick()
      await new Promise(resolve => setTimeout(resolve, 0))

      const button = wrapper.find('button')
      await button.trigger('click')

      const dropdown = wrapper.find('.absolute.top-full')
      expect(dropdown.exists()).toBe(false)
    })
  })

  describe('Tag Selection', () => {
    it('emits update:modelValue when adding tag', async () => {
      const wrapper = mountTagFilter()
      await wrapper.vm.$nextTick()
      await new Promise(resolve => setTimeout(resolve, 0))

      // Open dropdown
      const button = wrapper.find('button')
      await button.trigger('click')

      // Click first available tag
      const dropdown = wrapper.find('.absolute.top-full')
      const tagButton = dropdown.find('button')
      await tagButton.trigger('click')

      expect(wrapper.emitted('update:modelValue')).toBeTruthy()
      expect(wrapper.emitted('update:modelValue')[0][0]).toEqual([mockTags[0]])
    })

    it('closes dropdown after adding tag', async () => {
      const wrapper = mountTagFilter()
      await wrapper.vm.$nextTick()
      await new Promise(resolve => setTimeout(resolve, 0))

      const button = wrapper.find('button')
      await button.trigger('click')
      expect(wrapper.vm.showDropdown).toBe(true)

      const dropdown = wrapper.find('.absolute.top-full')
      const tagButton = dropdown.find('button')
      await tagButton.trigger('click')

      expect(wrapper.vm.showDropdown).toBe(false)
    })

    it('emits update:modelValue when removing tag', async () => {
      const wrapper = mountTagFilter({
        modelValue: [mockTags[0], mockTags[1]]
      })
      await wrapper.vm.$nextTick()
      await new Promise(resolve => setTimeout(resolve, 0))

      // Call removeTag directly (simulating TagBadge @remove event)
      wrapper.vm.removeTag(1)

      expect(wrapper.emitted('update:modelValue')).toBeTruthy()
      const emittedValue = wrapper.emitted('update:modelValue')[0][0]
      expect(emittedValue.length).toBe(1)
      expect(emittedValue[0].id).toBe(2) // second tag remains
    })

    it('clears all tags when clear button clicked', async () => {
      const wrapper = mountTagFilter({
        modelValue: [mockTags[0], mockTags[1]]
      })
      await wrapper.vm.$nextTick()
      await new Promise(resolve => setTimeout(resolve, 0))

      const clearButton = wrapper.findAll('button').find(btn => btn.text() === '清除全部')
      await clearButton.trigger('click')

      expect(wrapper.emitted('update:modelValue')).toBeTruthy()
      expect(wrapper.emitted('update:modelValue')[0][0]).toEqual([])
    })
  })

  describe('Click Outside Behavior', () => {
    it('closes dropdown when clicking outside', async () => {
      const wrapper = mountTagFilter()
      await wrapper.vm.$nextTick()
      await new Promise(resolve => setTimeout(resolve, 0))

      // Open dropdown
      const button = wrapper.find('button')
      await button.trigger('click')
      expect(wrapper.vm.showDropdown).toBe(true)

      // Simulate click outside
      const event = new MouseEvent('click', { bubbles: true })
      Object.defineProperty(event, 'target', {
        value: document.body,
        writable: false
      })
      document.dispatchEvent(event)

      await wrapper.vm.$nextTick()
      expect(wrapper.vm.showDropdown).toBe(false)
    })

    it('adds click listener on mount', async () => {
      const addSpy = vi.spyOn(document, 'addEventListener')

      mountTagFilter()
      await new Promise(resolve => setTimeout(resolve, 0))

      expect(addSpy).toHaveBeenCalledWith('click', expect.any(Function))
      addSpy.mockRestore()
    })

    it('removes click listener on unmount', async () => {
      const removeSpy = vi.spyOn(document, 'removeEventListener')

      const wrapper = mountTagFilter()
      await new Promise(resolve => setTimeout(resolve, 0))

      wrapper.unmount()

      expect(removeSpy).toHaveBeenCalledWith('click', expect.any(Function))
      removeSpy.mockRestore()
    })
  })

  describe('Color Dot Classes', () => {
    it('renders correct color dot for each tag in dropdown', async () => {
      const wrapper = mountTagFilter()
      await wrapper.vm.$nextTick()
      await new Promise(resolve => setTimeout(resolve, 0))

      const button = wrapper.find('button')
      await button.trigger('click')

      const dropdown = wrapper.find('.absolute.top-full')
      const dots = dropdown.findAll('span.w-2')

      // Check that dots exist
      expect(dots.length).toBeGreaterThan(0)
    })
  })

  describe('Usage Count Display', () => {
    it('shows usage count for each tag in dropdown', async () => {
      const wrapper = mountTagFilter()
      await wrapper.vm.$nextTick()
      await new Promise(resolve => setTimeout(resolve, 0))

      const button = wrapper.find('button')
      await button.trigger('click')

      const dropdown = wrapper.find('.absolute.top-full')
      const text = dropdown.text()

      // Check that usage counts are shown
      expect(text).toContain('10次')
      expect(text).toContain('5次')
      expect(text).toContain('3次')
    })
  })

  describe('Edge Cases', () => {
    it('handles empty modelValue prop', async () => {
      const wrapper = mountTagFilter({ modelValue: [] })
      await wrapper.vm.$nextTick()
      await new Promise(resolve => setTimeout(resolve, 0))

      expect(wrapper.vm.selectedTags).toEqual([])
      expect(wrapper.vm.availableTags.length).toBe(4)
    })

    it('handles undefined modelValue by using default', async () => {
      const wrapper = mount(TagFilter)
      await wrapper.vm.$nextTick()
      await new Promise(resolve => setTimeout(resolve, 0))

      expect(wrapper.vm.selectedTags).toEqual([])
    })

    it('handles empty tags list from API', async () => {
      invoke.mockResolvedValue([])

      const wrapper = mountTagFilter()
      await wrapper.vm.$nextTick()
      await new Promise(resolve => setTimeout(resolve, 0))

      expect(wrapper.vm.allTags).toEqual([])
      expect(wrapper.vm.availableTags).toEqual([])
    })

    it('handles tags without usage_count', async () => {
      invoke.mockResolvedValue([
        { id: 1, name: 'test', color: 'blue' }
      ])

      const wrapper = mountTagFilter()
      await wrapper.vm.$nextTick()
      await new Promise(resolve => setTimeout(resolve, 0))

      const button = wrapper.find('button')
      await button.trigger('click')

      // Should show 0 for missing usage_count
      const dropdown = wrapper.find('.absolute.top-full')
      expect(dropdown.text()).toContain('0次')
    })
  })

  describe('Computed Properties', () => {
    it('selectedTags returns modelValue prop', async () => {
      const wrapper = mountTagFilter({
        modelValue: [mockTags[0], mockTags[2]]
      })
      await wrapper.vm.$nextTick()
      await new Promise(resolve => setTimeout(resolve, 0))

      expect(wrapper.vm.selectedTags).toEqual([mockTags[0], mockTags[2]])
    })

    it('availableTags excludes selected tags', async () => {
      const wrapper = mountTagFilter({
        modelValue: [mockTags[0]]
      })
      await wrapper.vm.$nextTick()
      await new Promise(resolve => setTimeout(resolve, 0))

      const available = wrapper.vm.availableTags
      expect(available.length).toBe(3)
      expect(available.find(t => t.id === 1)).toBeUndefined()
    })
  })
})