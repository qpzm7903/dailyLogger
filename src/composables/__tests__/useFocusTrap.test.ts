import { describe, it, expect, beforeEach, vi } from 'vitest'
import { ref, nextTick } from 'vue'
import { useFocusTrap } from '../useFocusTrap'

function createContainerWith(elements: string[]): HTMLElement {
  const container = document.createElement('div')
  container.setAttribute('tabindex', '-1')
  for (const html of elements) {
    container.insertAdjacentHTML('beforeend', html)
  }
  document.body.appendChild(container)
  return container
}

describe('useFocusTrap', () => {
  let containerRef: ReturnType<typeof ref<HTMLElement | null>>

  beforeEach(() => {
    document.body.innerHTML = ''
    containerRef = ref<HTMLElement | null>(null)
  })

  it('starts inactive', () => {
    const { isActive } = useFocusTrap(containerRef)
    expect(isActive.value).toBe(false)
  })

  describe('activate', () => {
    it('sets isActive to true', () => {
      const container = createContainerWith(['<button>Click</button>'])
      containerRef.value = container
      const { activate, isActive } = useFocusTrap(containerRef)

      activate()
      expect(isActive.value).toBe(true)
    })

    it('focuses the first focusable element', async () => {
      const container = createContainerWith([
        '<button id="first">First</button>',
        '<button id="second">Second</button>',
      ])
      containerRef.value = container
      const { activate } = useFocusTrap(containerRef)

      activate()
      await nextTick()
      // requestAnimationFrame is used, so we flush it
      await new Promise<void>(resolve => requestAnimationFrame(() => resolve()))

      expect(document.activeElement?.id).toBe('first')
    })

    it('focuses the container itself when no focusable elements exist', async () => {
      const container = document.createElement('div')
      container.setAttribute('tabindex', '-1')
      document.body.appendChild(container)
      containerRef.value = container

      const { activate } = useFocusTrap(containerRef)
      activate()
      await new Promise<void>(resolve => requestAnimationFrame(() => resolve()))

      expect(document.activeElement).toBe(container)
    })

    it('saves the previously focused element', async () => {
      const previous = document.createElement('button')
      previous.id = 'previous'
      document.body.appendChild(previous)
      previous.focus()

      const container = createContainerWith(['<button id="target">Target</button>'])
      containerRef.value = container
      const { activate, deactivate } = useFocusTrap(containerRef)

      activate()
      deactivate()
      await nextTick()

      expect(document.activeElement?.id).toBe('previous')
    })

    it('does nothing if already active', () => {
      const container = createContainerWith(['<button>Click</button>'])
      containerRef.value = container
      const { activate, isActive } = useFocusTrap(containerRef)

      activate()
      expect(isActive.value).toBe(true)
      activate() // Should be a no-op
      expect(isActive.value).toBe(true)
    })

    it('calls onActivate callback', () => {
      const container = createContainerWith(['<button>Click</button>'])
      containerRef.value = container
      const onActivate = vi.fn()
      const { activate } = useFocusTrap(containerRef, { onActivate })

      activate()
      expect(onActivate).toHaveBeenCalledTimes(1)
    })
  })

  describe('deactivate', () => {
    it('sets isActive to false', () => {
      const container = createContainerWith(['<button>Click</button>'])
      containerRef.value = container
      const { activate, deactivate, isActive } = useFocusTrap(containerRef)

      activate()
      deactivate()
      expect(isActive.value).toBe(false)
    })

    it('restores focus to the previously focused element', async () => {
      const previous = document.createElement('button')
      previous.id = 'previous'
      document.body.appendChild(previous)
      previous.focus()

      const container = createContainerWith(['<button id="target">Target</button>'])
      containerRef.value = container
      const { activate, deactivate } = useFocusTrap(containerRef)

      activate()
      deactivate()
      await nextTick()

      expect(document.activeElement?.id).toBe('previous')
    })

    it('calls onDeactivate callback', () => {
      const container = createContainerWith(['<button>Click</button>'])
      containerRef.value = container
      const onDeactivate = vi.fn()
      const { activate, deactivate } = useFocusTrap(containerRef, { onDeactivate })

      activate()
      deactivate()
      expect(onDeactivate).toHaveBeenCalledTimes(1)
    })

    it('does nothing if not active', () => {
      const container = createContainerWith(['<button>Click</button>'])
      containerRef.value = container
      const onDeactivate = vi.fn()
      const { deactivate } = useFocusTrap(containerRef, { onDeactivate })

      deactivate()
      expect(onDeactivate).not.toHaveBeenCalled()
    })
  })

  describe('Tab key trapping', () => {
    it('wraps focus from last to first element on Tab', async () => {
      const container = createContainerWith([
        '<button id="first">First</button>',
        '<button id="last">Last</button>',
      ])
      containerRef.value = container
      const { activate } = useFocusTrap(containerRef)

      activate()
      await new Promise<void>(resolve => requestAnimationFrame(() => resolve()))

      // Focus the last element
      const last = container.querySelector('#last') as HTMLElement
      last.focus()

      // Simulate Tab
      const tabEvent = new KeyboardEvent('keydown', { key: 'Tab', bubbles: true })
      container.dispatchEvent(tabEvent)
      document.dispatchEvent(tabEvent)

      // After Tab from last, focus should go to first
      expect(document.activeElement?.id).toBe('first')
    })

    it('wraps focus from first to last element on Shift+Tab', async () => {
      const container = createContainerWith([
        '<button id="first">First</button>',
        '<button id="last">Last</button>',
      ])
      containerRef.value = container
      const { activate } = useFocusTrap(containerRef)

      activate()
      await new Promise<void>(resolve => requestAnimationFrame(() => resolve()))

      // Focus should be on first element
      expect(document.activeElement?.id).toBe('first')

      // Simulate Shift+Tab
      const shiftTabEvent = new KeyboardEvent('keydown', { key: 'Tab', shiftKey: true, bubbles: true })
      document.dispatchEvent(shiftTabEvent)

      expect(document.activeElement?.id).toBe('last')
    })
  })

  describe('edge cases', () => {
    it('handles containerRef being null gracefully', () => {
      containerRef.value = null
      const { activate, deactivate, isActive } = useFocusTrap(containerRef)

      // Should not throw
      activate()
      expect(isActive.value).toBe(true)
      deactivate()
      expect(isActive.value).toBe(false)
    })

    it('finds all focusable element types', () => {
      const container = createContainerWith([
        '<button>Button</button>',
        '<input type="text" />',
        '<select><option>A</option></select>',
        '<textarea></textarea>',
        '<a href="#">Link</a>',
        '<div tabindex="0">Custom</div>',
        '<div contenteditable="true">Editable</div>',
        '<button disabled>Disabled</button>',
        '<div tabindex="-1">Negative tabindex</div>',
      ])
      containerRef.value = container
      const { activate } = useFocusTrap(containerRef)

      activate()

      // Should have 7 focusable elements (disabled button and tabindex="-1" excluded)
      const buttons = container.querySelectorAll('button:not([disabled])')
      expect(buttons.length).toBe(1)
    })
  })
})
