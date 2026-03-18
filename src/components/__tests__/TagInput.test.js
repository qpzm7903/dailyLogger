import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import TagInput from '../TagInput.vue'
import TagBadge from '../TagBadge.vue'

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

// Mock toast store
vi.mock('../../stores/toast', () => ({
  showSuccess: vi.fn(),
  showError: vi.fn()
}))

import { invoke } from '@tauri-apps/api/core'
import { showSuccess, showError } from '../../stores/toast'

describe('TagInput', () => {
  // Factory function to create fresh mock data for each test
  const createMockTags = () => [
    { id: 1, name: '工作', color: 'blue', usage_count: 10 },
    { id: 2, name: '学习', color: 'green', usage_count: 5 },
    { id: 3, name: '项目A', color: 'red', usage_count: 3 }
  ]

  const defaultProps = {
    modelValue: [],
    recordId: 123,
    placeholder: '输入标签名...'
  }

  beforeEach(() => {
    vi.clearAllMocks()
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  // === Rendering ===
  describe('rendering', () => {
    it('renders input field with placeholder', async () => {
      invoke.mockResolvedValue(createMockTags())
      const wrapper = mount(TagInput, { props: defaultProps })
      await flushPromises()

      const input = wrapper.find('input')
      expect(input.exists()).toBe(true)
      expect(input.attributes('placeholder')).toBe('输入标签名...')
    })

    it('renders with custom placeholder', async () => {
      invoke.mockResolvedValue(createMockTags())
      const wrapper = mount(TagInput, {
        props: { ...defaultProps, placeholder: '添加标签...' }
      })
      await flushPromises()

      const input = wrapper.find('input')
      expect(input.attributes('placeholder')).toBe('添加标签...')
    })

    it('renders color selector buttons', async () => {
      invoke.mockResolvedValue(createMockTags())
      const wrapper = mount(TagInput, { props: defaultProps })
      await flushPromises()

      const colorButtons = wrapper.findAll('.flex.items-center.gap-1 button')
      expect(colorButtons.length).toBe(8) // 8 preset colors
    })

    it('renders add button', async () => {
      invoke.mockResolvedValue(createMockTags())
      const wrapper = mount(TagInput, { props: defaultProps })
      await flushPromises()

      const addButton = wrapper.findAll('button').find(b => b.text() === '添加')
      expect(addButton?.exists()).toBe(true)
    })

    it('does not render existing tags section when empty', async () => {
      invoke.mockResolvedValue(createMockTags())
      const wrapper = mount(TagInput, { props: defaultProps })
      await flushPromises()

      const tagsContainer = wrapper.find('.flex.flex-wrap.gap-1\\.5')
      expect(tagsContainer.exists()).toBe(false)
    })

    it('renders existing tags when modelValue has items', async () => {
      const tags = createMockTags()
      invoke.mockResolvedValue(tags)
      const wrapper = mount(TagInput, {
        props: { ...defaultProps, modelValue: [tags[0], tags[1]] }
      })
      await flushPromises()

      const badges = wrapper.findAllComponents(TagBadge)
      expect(badges.length).toBe(2)
    })

    it('shows tag limit warning when at 10 tags', async () => {
      invoke.mockResolvedValue(createMockTags())
      const tenTags = Array(10).fill(null).map((_, i) => ({
        id: i + 1, name: `Tag ${i + 1}`, color: 'blue'
      }))

      const wrapper = mount(TagInput, {
        props: { ...defaultProps, modelValue: tenTags }
      })
      await flushPromises()

      const warning = wrapper.find('p.text-yellow-500')
      expect(warning.exists()).toBe(true)
      expect(warning.text()).toContain('已达标签上限')
    })
  })

  // === Color Selection ===
  describe('color selection', () => {
    it('defaults to blue color', async () => {
      invoke.mockResolvedValue(createMockTags())
      const wrapper = mount(TagInput, { props: defaultProps })
      await flushPromises()

      expect(wrapper.vm.selectedColor).toBe('blue')
    })

    it('changes color on button click', async () => {
      invoke.mockResolvedValue(createMockTags())
      const wrapper = mount(TagInput, { props: defaultProps })
      await flushPromises()

      const colorButtons = wrapper.findAll('.flex.items-center.gap-1 button')
      await colorButtons[2].trigger('click')

      expect(wrapper.vm.selectedColor).toBe('yellow')
    })

    it('highlights selected color with ring', async () => {
      invoke.mockResolvedValue(createMockTags())
      const wrapper = mount(TagInput, { props: defaultProps })
      await flushPromises()

      const colorButtons = wrapper.findAll('.flex.items-center.gap-1 button')
      const blueButton = colorButtons[0]

      expect(blueButton.classes()).toContain('ring-2')
      expect(blueButton.classes()).toContain('scale-110')
    })
  })

  // === Dropdown Behavior ===
  describe('dropdown behavior', () => {
    it('shows dropdown on input focus', async () => {
      invoke.mockResolvedValue(createMockTags())
      const wrapper = mount(TagInput, { props: defaultProps })
      await flushPromises()

      const input = wrapper.find('input')
      await input.trigger('focus')

      expect(wrapper.vm.showDropdown).toBe(true)
    })

    it('hides dropdown on escape key', async () => {
      invoke.mockResolvedValue(createMockTags())
      const wrapper = mount(TagInput, { props: defaultProps })
      await flushPromises()

      wrapper.vm.showDropdown = true
      const input = wrapper.find('input')
      await input.trigger('keydown.escape')

      expect(wrapper.vm.showDropdown).toBe(false)
    })

    it('shows filtered tags in dropdown', async () => {
      invoke.mockResolvedValue(createMockTags())
      const wrapper = mount(TagInput, { props: defaultProps })
      await flushPromises()

      wrapper.vm.searchQuery = '工作'
      wrapper.vm.showDropdown = true
      await wrapper.vm.$nextTick()

      const dropdown = wrapper.find('.absolute.top-full')
      expect(dropdown.exists()).toBe(true)

      const tagButtons = dropdown.findAll('button')
      expect(tagButtons.length).toBe(1)
      expect(tagButtons[0].text()).toContain('工作')
    })

    it('filters out already added tags from dropdown', async () => {
      const tags = createMockTags()
      invoke.mockResolvedValue(tags)
      const wrapper = mount(TagInput, {
        props: { ...defaultProps, modelValue: [tags[0]] }
      })
      await flushPromises()

      wrapper.vm.searchQuery = ''
      wrapper.vm.showDropdown = true
      await wrapper.vm.$nextTick()

      const dropdownItems = wrapper.findAll('.absolute.top-full button')
      const texts = dropdownItems.map(b => b.text())

      expect(texts.some(t => t.includes('工作'))).toBe(false)
      expect(texts.some(t => t.includes('学习'))).toBe(true)
    })
  })

  // === Tag Selection ===
  describe('tag selection', () => {
    it('selects existing tag from dropdown', async () => {
      const tags = createMockTags()
      invoke.mockResolvedValueOnce(tags)
      invoke.mockResolvedValueOnce(undefined) // add_tag_to_record
      const wrapper = mount(TagInput, { props: defaultProps })
      await flushPromises()

      wrapper.vm.showDropdown = true
      await wrapper.vm.$nextTick()

      const tagButton = wrapper.findAll('.absolute.top-full button')[0]
      await tagButton.trigger('click')
      await flushPromises()

      expect(invoke).toHaveBeenCalledWith('add_tag_to_record', {
        recordId: 123,
        tagId: 1
      })
    })

    it('emits update:modelValue when tag selected', async () => {
      const tags = createMockTags()
      invoke.mockResolvedValueOnce(tags)
      invoke.mockResolvedValueOnce(undefined)
      const wrapper = mount(TagInput, { props: defaultProps })
      await flushPromises()

      await wrapper.vm.selectTag(tags[0])
      await flushPromises()

      const emitted = wrapper.emitted('update:modelValue')
      expect(emitted).toBeTruthy()
      expect(emitted[0][0]).toContainEqual(tags[0])
    })

    it('shows error when selecting tag at limit (10)', async () => {
      invoke.mockResolvedValue(createMockTags())
      const tenTags = Array(10).fill(null).map((_, i) => ({
        id: i + 1, name: `Tag ${i + 1}`, color: 'blue'
      }))
      const tags = createMockTags()

      const wrapper = mount(TagInput, {
        props: { ...defaultProps, modelValue: tenTags }
      })
      await flushPromises()

      await wrapper.vm.selectTag(tags[0])

      expect(showError).toHaveBeenCalledWith('每条记录最多只能添加 10 个标签')
    })
  })

  // === Tag Creation ===
  describe('tag creation', () => {
    it('creates new tag on Enter key', async () => {
      invoke.mockResolvedValueOnce(createMockTags())
      invoke.mockResolvedValueOnce({ id: 10, name: '新标签', color: 'blue' })
      invoke.mockResolvedValueOnce(undefined)

      const wrapper = mount(TagInput, { props: defaultProps })
      await flushPromises()

      const input = wrapper.find('input')
      await input.setValue('新标签')
      await input.trigger('keydown.enter')
      await flushPromises()

      expect(invoke).toHaveBeenCalledWith('create_manual_tag', {
        name: '新标签',
        color: 'blue'
      })
    })

    it('creates tag with selected color', async () => {
      invoke.mockResolvedValueOnce(createMockTags())
      invoke.mockResolvedValueOnce({ id: 10, name: '新标签', color: 'red' })
      invoke.mockResolvedValueOnce(undefined)

      const wrapper = mount(TagInput, { props: defaultProps })
      await flushPromises()

      wrapper.vm.selectedColor = 'red'
      const input = wrapper.find('input')
      await input.setValue('新标签')
      await input.trigger('keydown.enter')
      await flushPromises()

      expect(invoke).toHaveBeenCalledWith('create_manual_tag', {
        name: '新标签',
        color: 'red'
      })
    })

    it('shows success toast after creating tag', async () => {
      invoke.mockResolvedValueOnce(createMockTags())
      invoke.mockResolvedValueOnce({ id: 10, name: '新标签', color: 'blue' })
      invoke.mockResolvedValueOnce(undefined)

      const wrapper = mount(TagInput, { props: defaultProps })
      await flushPromises()

      const input = wrapper.find('input')
      await input.setValue('新标签')
      await input.trigger('keydown.enter')
      await flushPromises()

      expect(showSuccess).toHaveBeenCalledWith('标签已创建')
    })

    it('selects existing tag if name matches', async () => {
      const tags = createMockTags()
      invoke.mockResolvedValueOnce(tags)
      invoke.mockResolvedValueOnce(undefined)

      const wrapper = mount(TagInput, { props: defaultProps })
      await flushPromises()

      const input = wrapper.find('input')
      await input.setValue('工作')
      await input.trigger('keydown.enter')
      await flushPromises()

      expect(invoke).toHaveBeenCalledWith('add_tag_to_record', {
        recordId: 123,
        tagId: 1
      })
    })

    it('clears input if tag already added to record', async () => {
      const tags = createMockTags()
      invoke.mockResolvedValueOnce(tags)

      const wrapper = mount(TagInput, {
        props: { ...defaultProps, modelValue: [tags[0]] }
      })
      await flushPromises()

      const input = wrapper.find('input')
      await input.setValue('工作')
      await input.trigger('keydown.enter')
      await flushPromises()

      expect(wrapper.vm.searchQuery).toBe('')
      expect(wrapper.vm.showDropdown).toBe(false)
    })

    it('shows error when creating tag at limit (10)', async () => {
      invoke.mockResolvedValueOnce(createMockTags())
      const tenTags = Array(10).fill(null).map((_, i) => ({
        id: i + 1, name: `Tag ${i + 1}`, color: 'blue'
      }))

      const wrapper = mount(TagInput, {
        props: { ...defaultProps, modelValue: tenTags }
      })
      await flushPromises()

      const input = wrapper.find('input')
      await input.setValue('新标签')
      await input.trigger('keydown.enter')
      await flushPromises()

      expect(showError).toHaveBeenCalledWith('每条记录最多只能添加 10 个标签')
    })

    it('shows loading state while creating', async () => {
      let resolveCreate
      invoke.mockResolvedValueOnce(createMockTags())
      invoke.mockImplementationOnce(() => new Promise(resolve => {
        resolveCreate = resolve
      }))

      const wrapper = mount(TagInput, { props: defaultProps })
      await flushPromises()

      const input = wrapper.find('input')
      await input.setValue('新标签')
      await input.trigger('keydown.enter')
      await wrapper.vm.$nextTick()

      // Button should show "..."
      const addButton = wrapper.findAll('button').find(b => b.text() === '...')
      expect(addButton?.exists()).toBe(true)

      // Resolve
      resolveCreate({ id: 10, name: '新标签', color: 'blue' })
      invoke.mockResolvedValueOnce(undefined)
      await flushPromises()

      expect(wrapper.vm.isCreating).toBe(false)
    })

    it('disables add button when input is empty', async () => {
      invoke.mockResolvedValue(createMockTags())
      const wrapper = mount(TagInput, { props: defaultProps })
      await flushPromises()

      const addButton = wrapper.findAll('button').find(b => b.text() === '添加')
      expect(addButton?.element.disabled).toBe(true)
    })

    it('enables add button when input has text', async () => {
      invoke.mockResolvedValue(createMockTags())
      const wrapper = mount(TagInput, { props: defaultProps })
      await flushPromises()

      const input = wrapper.find('input')
      await input.setValue('新标签')
      await wrapper.vm.$nextTick()

      const addButton = wrapper.findAll('button').find(b => b.text() === '添加')
      expect(addButton?.element.disabled).toBe(false)
    })
  })

  // === Tag Removal ===
  describe('tag removal', () => {
    it('removes tag when TagBadge emits remove', async () => {
      const tags = createMockTags()
      invoke.mockResolvedValueOnce(tags)
      invoke.mockResolvedValueOnce(undefined)

      const wrapper = mount(TagInput, {
        props: { ...defaultProps, modelValue: [tags[0], tags[1]] }
      })
      await flushPromises()

      const badge = wrapper.findComponent(TagBadge)
      await badge.vm.$emit('remove', 1)
      await flushPromises()

      expect(invoke).toHaveBeenCalledWith('remove_tag_from_record', {
        recordId: 123,
        tagId: 1
      })
    })

    it('emits update:modelValue after removal', async () => {
      const tags = createMockTags()
      invoke.mockResolvedValueOnce(tags)
      invoke.mockResolvedValueOnce(undefined)

      const wrapper = mount(TagInput, {
        props: { ...defaultProps, modelValue: [tags[0], tags[1]] }
      })
      await flushPromises()

      await wrapper.vm.removeTag(1)
      await flushPromises()

      const emitted = wrapper.emitted('update:modelValue')
      expect(emitted).toBeTruthy()
      expect(emitted[0][0].length).toBe(1)
      expect(emitted[0][0][0].id).toBe(2)
    })

    it('shows error if removal fails', async () => {
      const tags = createMockTags()
      invoke.mockResolvedValueOnce(tags)
      invoke.mockRejectedValueOnce(new Error('Removal failed'))

      const wrapper = mount(TagInput, {
        props: { ...defaultProps, modelValue: [tags[0]] }
      })
      await flushPromises()

      await wrapper.vm.removeTag(1)
      await flushPromises()

      expect(showError).toHaveBeenCalled()
    })
  })

  // === Error Handling ===
  describe('error handling', () => {
    it('shows error when tag creation fails', async () => {
      invoke.mockResolvedValueOnce(createMockTags())
      invoke.mockRejectedValueOnce(new Error('Creation failed'))

      const wrapper = mount(TagInput, { props: defaultProps })
      await flushPromises()

      const input = wrapper.find('input')
      await input.setValue('新标签')
      await input.trigger('keydown.enter')
      await flushPromises()

      expect(showError).toHaveBeenCalled()
    })

    it('shows error when tag selection fails', async () => {
      invoke.mockResolvedValueOnce(createMockTags())
      invoke.mockRejectedValueOnce(new Error('Selection failed'))

      const wrapper = mount(TagInput, { props: defaultProps })
      await flushPromises()

      const tags = createMockTags()
      await wrapper.vm.selectTag(tags[0])
      await flushPromises()

      expect(showError).toHaveBeenCalled()
    })

    it('handles loadAllTags failure gracefully', async () => {
      invoke.mockRejectedValueOnce(new Error('Load failed'))
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})

      const wrapper = mount(TagInput, { props: defaultProps })
      await flushPromises()

      expect(consoleSpy).toHaveBeenCalled()
      consoleSpy.mockRestore()
    })
  })

  // === Helper Functions ===
  describe('helper functions', () => {
    it('getDotClass returns correct class for valid color', async () => {
      invoke.mockResolvedValue(createMockTags())
      const wrapper = mount(TagInput, { props: defaultProps })
      await flushPromises()

      const result = wrapper.vm.getDotClass('green')
      expect(result).toContain('bg-green-500')
    })

    it('getDotClass returns default for invalid color', async () => {
      invoke.mockResolvedValue(createMockTags())
      const wrapper = mount(TagInput, { props: defaultProps })
      await flushPromises()

      const result = wrapper.vm.getDotClass('invalid')
      expect(result).toContain('bg-blue-500')
    })
  })

  // === Click Outside ===
  describe('click outside behavior', () => {
    it('closes dropdown when clicking outside', async () => {
      invoke.mockResolvedValue(createMockTags())
      const wrapper = mount(TagInput, { props: defaultProps })
      await flushPromises()

      wrapper.vm.showDropdown = true
      const mockEvent = { target: document.createElement('div') }
      wrapper.vm.handleClickOutside(mockEvent)

      expect(wrapper.vm.showDropdown).toBe(false)
    })
  })

  // === Lifecycle ===
  describe('lifecycle', () => {
    it('loads all tags on mount', async () => {
      const tags = createMockTags()
      invoke.mockResolvedValueOnce(tags)

      const wrapper = mount(TagInput, { props: defaultProps })
      await flushPromises()

      expect(invoke).toHaveBeenCalledWith('get_all_manual_tags')
      expect(wrapper.vm.allTags).toEqual(tags)
    })

    it('adds click listener on mount', async () => {
      const addSpy = vi.spyOn(document, 'addEventListener')
      invoke.mockResolvedValue(createMockTags())

      const wrapper = mount(TagInput, { props: defaultProps })
      await flushPromises()

      expect(addSpy).toHaveBeenCalledWith('click', expect.any(Function))
      addSpy.mockRestore()
    })

    it('removes click listener on unmount', async () => {
      const removeSpy = vi.spyOn(document, 'removeEventListener')
      invoke.mockResolvedValue(createMockTags())

      const wrapper = mount(TagInput, { props: defaultProps })
      await flushPromises()
      wrapper.unmount()

      expect(removeSpy).toHaveBeenCalledWith('click', expect.any(Function))
      removeSpy.mockRestore()
    })
  })
})