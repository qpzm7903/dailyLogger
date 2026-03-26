// UX-010: useModal composable
// Centralized modal management - replaces 21 individual showXxx ref variables

import { ref, readonly, onMounted, onUnmounted, type Ref, type DeepReadonly } from 'vue'
import { useFocusTrap } from './useFocusTrap'

// Modal ID type - all possible modal identifiers
export type ModalId =
  | 'settings'
  | 'backup'
  | 'quickNote'
  | 'screenshot'
  | 'screenshotGallery'
  | 'summaryViewer'
  | 'weeklyReportViewer'
  | 'monthlyReportViewer'
  | 'customReport'
  | 'customReportViewer'
  | 'comparisonReport'
  | 'comparisonReportViewer'
  | 'logViewer'
  | 'historyViewer'
  | 'search'
  | 'tagCloud'
  | 'export'
  | 'timeline'
  | 'pluginPanel'
  | 'reportHistory'
  | 'offlineQueue'
  | 'reanalyzeByDate'
  | 'sessionList'

// Module-level state (singleton pattern)
const activeModal: Ref<ModalId | null> = ref(null)
const previousActiveElement: Ref<HTMLElement | null> = ref(null)

export interface UseModalReturn {
  /** Currently active modal ID (readonly) */
  activeModal: DeepReadonly<Ref<ModalId | null>>
  /** Check if a specific modal is open */
  isOpen: (id: ModalId) => boolean
  /** Open a modal (closes any other open modal first) */
  open: (id: ModalId) => void
  /** Close a specific modal, or close current modal if no id provided */
  close: (id?: ModalId) => void
  /** Toggle a modal's open/close state */
  toggle: (id: ModalId) => void
}

/**
 * useModal composable
 *
 * Provides centralized modal state management.
 * Only one modal can be open at a time.
 *
 * Features (UX-5):
 * - ESC key closes the active modal
 * - Focus trap: Tab cycles within modal, focus restored on close
 *
 * @example
 * ```ts
 * const { isOpen, open, close, toggle } = useModal()
 *
 * // Check if modal is open
 * if (isOpen('settings')) { ... }
 *
 * // Open a modal
 * open('settings')
 *
 * // Close current modal
 * close()
 *
 * // Toggle modal
 * toggle('quickNote')
 * ```
 */
export function useModal(): UseModalReturn {
  const containerRef = ref<HTMLElement | null>(null)
  const { activate: activateFocusTrap, deactivate: deactivateFocusTrap, isActive: isFocusTrapActive } = useFocusTrap(containerRef)

  // Handle ESC key press
  const handleKeydown = (event: KeyboardEvent) => {
    if (event.key === 'Escape' && activeModal.value !== null) {
      event.preventDefault()
      close()
    }
  }

  // Register global ESC listener on mount
  onMounted(() => {
    document.addEventListener('keydown', handleKeydown)
  })

  onUnmounted(() => {
    document.removeEventListener('keydown', handleKeydown)
  })

  const isOpen = (id: ModalId): boolean => {
    return activeModal.value === id
  }

  const open = (id: ModalId): void => {
    // If same modal is already open, do nothing
    if (activeModal.value === id) return

    // Remember the currently focused element before opening modal
    previousActiveElement.value = document.activeElement as HTMLElement

    // Set new active modal (automatically closes any previous one)
    activeModal.value = id
  }

  const close = (id?: ModalId): void => {
    if (id === undefined) {
      // Close current modal
      deactivateFocusTrap()
      activeModal.value = null

      // Restore focus to the previously focused element
      if (previousActiveElement.value && previousActiveElement.value.focus) {
        previousActiveElement.value.focus()
      }
    } else {
      // Close specific modal only if it's currently open
      if (activeModal.value === id) {
        deactivateFocusTrap()
        activeModal.value = null

        // Restore focus to the previously focused element
        if (previousActiveElement.value && previousActiveElement.value.focus) {
          previousActiveElement.value.focus()
        }
      }
    }
  }

  const toggle = (id: ModalId): void => {
    if (activeModal.value === id) {
      close(id)
    } else {
      open(id)
    }
  }

  return {
    activeModal: readonly(activeModal),
    isOpen,
    open,
    close,
    toggle,
  }
}