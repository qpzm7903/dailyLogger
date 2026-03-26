// UX-5: useFocusTrap composable
// Focus trap implementation for modal accessibility

import { ref, onMounted, onBeforeUnmount, type Ref } from 'vue'

export interface UseFocusTrapOptions {
  /** Callback when focus trap is activated */
  onActivate?: () => void
  /** Callback when focus trap is deactivated */
  onDeactivate?: () => void
}

/**
 * useFocusTrap composable
 *
 * Provides focus trap functionality for modals:
 * - When activated, focuses the first focusable element inside the container
 * - Tab key cycles focus within the container (focus cannot escape)
 * - When deactivated, restores focus to the previously focused element
 *
 * @example
 * ```ts
 * const containerRef = ref<HTMLElement | null>(null)
 * const { activate, deactivate } = useFocusTrap(containerRef)
 * ```
 */
export function useFocusTrap(
  containerRef: Ref<HTMLElement | null>,
  options: UseFocusTrapOptions = {}
): {
  activate: () => void
  deactivate: () => void
  isActive: Ref<boolean>
} {
  const isActive = ref(false)
  let previousActiveElement: HTMLElement | null = null

  // Get all focusable elements within the container
  const getFocusableElements = (): HTMLElement[] => {
    if (!containerRef.value) return []

    const selector = [
      'button:not([disabled])',
      'input:not([disabled])',
      'select:not([disabled])',
      'textarea:not([disabled])',
      'a[href]',
      '[tabindex]:not([tabindex="-1"])',
      '[contenteditable="true"]',
    ].join(', ')

    return Array.from(containerRef.value.querySelectorAll<HTMLElement>(selector))
  }

  // Find the first focusable element
  const getFirstFocusable = (): HTMLElement | null => {
    const focusable = getFocusableElements()
    return focusable.length > 0 ? focusable[0] : null
  }

  // Find the last focusable element
  const getLastFocusable = (): HTMLElement | null => {
    const focusable = getFocusableElements()
    return focusable.length > 0 ? focusable[focusable.length - 1] : null
  }

  // Handle keydown event for Tab trapping
  const handleKeydown = (event: KeyboardEvent) => {
    if (!isActive.value || event.key !== 'Tab') return

    const focusable = getFocusableElements()
    if (focusable.length === 0) return

    const first = focusable[0]
    const last = focusable[focusable.length - 1]

    // Shift + Tab: if focus is on first element, move to last
    if (event.shiftKey) {
      if (document.activeElement === first) {
        event.preventDefault()
        last.focus()
      }
    } else {
      // Tab: if focus is on last element, move to first
      if (document.activeElement === last) {
        event.preventDefault()
        first.focus()
      }
    }
  }

  const activate = () => {
    if (isActive.value) return

    // Remember the currently focused element
    previousActiveElement = document.activeElement as HTMLElement

    // Set isActive flag
    isActive.value = true

    // Add keydown listener
    document.addEventListener('keydown', handleKeydown)

    // Focus the first focusable element after a brief delay to ensure DOM is ready
    requestAnimationFrame(() => {
      const firstFocusable = getFirstFocusable()
      if (firstFocusable) {
        firstFocusable.focus()
      } else if (containerRef.value) {
        // If no focusable elements, focus the container itself
        containerRef.value.focus()
      }
    })

    options.onActivate?.()
  }

  const deactivate = () => {
    if (!isActive.value) return

    // Remove keydown listener
    document.removeEventListener('keydown', handleKeydown)

    // Restore focus to the previously focused element
    if (previousActiveElement && previousActiveElement.focus) {
      previousActiveElement.focus()
    }

    isActive.value = false
    options.onDeactivate?.()
  }

  // Cleanup on unmount
  onBeforeUnmount(() => {
    if (isActive.value) {
      document.removeEventListener('keydown', handleKeydown)
    }
  })

  return {
    activate,
    deactivate,
    isActive,
  }
}
