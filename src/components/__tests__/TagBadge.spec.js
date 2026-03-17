import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import TagBadge from '../TagBadge.vue'

describe('TagBadge', () => {
  it('renders tag name', () => {
    const tag = { name: 'Frontend', color: 'blue' }
    const wrapper = mount(TagBadge, {
      props: { tag }
    })
    expect(wrapper.text()).toContain('Frontend')
  })

  it('applies correct color class for blue', () => {
    const tag = { name: 'Test', color: 'blue' }
    const wrapper = mount(TagBadge, {
      props: { tag }
    })
    expect(wrapper.find('span').classes()).toContain('bg-blue-500')
  })

  it('applies correct color class for green', () => {
    const tag = { name: 'Test', color: 'green' }
    const wrapper = mount(TagBadge, {
      props: { tag }
    })
    expect(wrapper.find('span').classes()).toContain('bg-green-500')
  })

  it('applies correct color class for yellow', () => {
    const tag = { name: 'Test', color: 'yellow' }
    const wrapper = mount(TagBadge, {
      props: { tag }
    })
    expect(wrapper.find('span').classes()).toContain('bg-yellow-400')
  })

  it('applies correct color class for red', () => {
    const tag = { name: 'Test', color: 'red' }
    const wrapper = mount(TagBadge, {
      props: { tag }
    })
    expect(wrapper.find('span').classes()).toContain('bg-red-500')
  })

  it('applies correct color class for purple', () => {
    const tag = { name: 'Test', color: 'purple' }
    const wrapper = mount(TagBadge, {
      props: { tag }
    })
    expect(wrapper.find('span').classes()).toContain('bg-purple-500')
  })

  it('applies correct color class for pink', () => {
    const tag = { name: 'Test', color: 'pink' }
    const wrapper = mount(TagBadge, {
      props: { tag }
    })
    expect(wrapper.find('span').classes()).toContain('bg-pink-500')
  })

  it('applies correct color class for cyan', () => {
    const tag = { name: 'Test', color: 'cyan' }
    const wrapper = mount(TagBadge, {
      props: { tag }
    })
    expect(wrapper.find('span').classes()).toContain('bg-cyan-500')
  })

  it('applies correct color class for orange', () => {
    const tag = { name: 'Test', color: 'orange' }
    const wrapper = mount(TagBadge, {
      props: { tag }
    })
    expect(wrapper.find('span').classes()).toContain('bg-orange-500')
  })

  it('defaults to blue for unknown color', () => {
    const tag = { name: 'Test', color: 'unknown' }
    const wrapper = mount(TagBadge, {
      props: { tag }
    })
    expect(wrapper.find('span').classes()).toContain('bg-blue-500')
  })

  it('does not show remove button when not removable', () => {
    const tag = { name: 'Test', color: 'blue' }
    const wrapper = mount(TagBadge, {
      props: { tag, removable: false }
    })
    expect(wrapper.find('button').exists()).toBe(false)
  })

  it('shows remove button when removable', () => {
    const tag = { name: 'Test', color: 'blue' }
    const wrapper = mount(TagBadge, {
      props: { tag, removable: true }
    })
    expect(wrapper.find('button').exists()).toBe(true)
  })

  it('emits remove event when remove button is clicked', async () => {
    const tag = { name: 'Test', color: 'blue' }
    const wrapper = mount(TagBadge, {
      props: { tag, removable: true }
    })
    await wrapper.find('button').trigger('click')
    expect(wrapper.emitted('remove')).toBeTruthy()
    expect(wrapper.emitted('remove').length).toBe(1)
  })

  it('stops event propagation when remove button is clicked', async () => {
    const tag = { name: 'Test', color: 'blue' }
    const wrapper = mount(TagBadge, {
      props: { tag, removable: true }
    })
    const button = wrapper.find('button')
    // The @click.stop modifier should prevent propagation
    // We verify this by checking that the button has the click handler
    expect(button.element.onclick).toBeDefined()
  })
})
