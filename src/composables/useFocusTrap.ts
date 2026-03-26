// UX-5: Focus trap composable for modal accessibility
// Implements focus trap pattern: focus stays within modal, Tab cycles within modal

import { ref, onMounted, onBeforeUnmount, type Ref } from 'vue'

export interface UseFocusTrapOptions {
  /** Whether to auto-activate when container is mounted */
  autoActivate?: boolean
  /** Callback when trap is activated */
  onActivate?: () => void
  /** Callback when trap is deactivated */
  onDeactivate?: () => void
}

export interface UseFocusTrapReturn {
  /** Container ref to attach to the modal element */
  containerRef: Ref<HTMLElement | null>
  /** Whether the trap is currently active */
  isActive: Ref<boolean>
  /** Trigger element that opened the modal (for focus restoration) */
  triggerElement: Ref<HTMLElement | null>
  /** Activate the focus trap */
  activate: (trigger?: HTMLElement) => void
  /** Deactivate the focus trap and restore focus */
  deactivate: () => void
}

const FOCUSABLE_SELECTORS = [
  'a[href]',
  'button:not([disabled])',
  'input:not([disabled])',
  'select:not([disabled])',
  'textarea:not([disabled])',
  '[tabindex]:not([tabindex="-1"])',
  'details > summary',
].join(', ')

export function useFocusTrap(options: UseFocusTrapOptions = {}): UseFocusTrapReturn {
  const containerRef = ref<HTMLElement | null>(null)
  const isActive = ref(false)
  const triggerElement = ref<HTMLElement | null>(null)

  const getFocusableElements = (): HTMLElement[] => {
    if (!containerRef.value) return []
    return Array.from(containerRef.value.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTORS))
  }

  const handleKeyDown = (event: KeyboardEvent) => {
    if (!isActive.value || event.key !== 'Tab') return

    const focusable = getFocusableElements()
    if (focusable.length === 0) return

    const first = focusable[0]
    const last = focusable[focusable.length - 1]

    if (event.shiftKey) {
      // Shift+Tab: if on first element, wrap to last
      if (document.activeElement === first) {
        event.preventDefault()
        last.focus()
      }
    } else {
      // Tab: if on last element, wrap to first
      if (document.activeElement === last) {
        event.preventDefault()
        first.focus()
      }
    }
  }

  const activate = (trigger?: HTMLElement) => {
    if (isActive.value) return

    // Remember trigger element for restoration later
    triggerElement.value = trigger || document.activeElement as HTMLElement

    isActive.value = true
    document.addEventListener('keydown', handleKeyDown)

    // Move focus to first focusable element in modal
    requestAnimationFrame(() => {
      const focusable = getFocusableElements()
      if (focusable.length > 0) {
        focusable[0].focus()
      } else if (containerRef.value) {
        // If no focusable elements, focus the container itself
        containerRef.value.focus()
      }
    })

    options.onActivate?.()
  }

  const deactivate = () => {
    if (!isActive.value) return

    isActive.value = false
    document.removeEventListener('keydown', handleKeyDown)

    // Restore focus to trigger element
    if (triggerElement.value && triggerElement.value.isConnected) {
      triggerElement.value.focus()
    }

    options.onDeactivate?.()
  }

  onBeforeUnmount(() => {
    if (isActive.value) {
      deactivate()
    }
  })

  return {
    containerRef,
    isActive,
    triggerElement,
    activate,
    deactivate,
  }
}
