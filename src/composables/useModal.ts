// UX-010: useModal composable
// Centralized modal management - replaces 21 individual showXxx ref variables
// UX-5: Extended with ESC key listener and focus trap integration

import { ref, readonly, onMounted, onUnmounted, type Ref, type DeepReadonly } from 'vue'
import { useFocusTrap, type UseFocusTrapReturn } from './useFocusTrap'

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
const previousActiveElement = ref<HTMLElement | null>(null)

// Focus trap instance (singleton)
let focusTrap: UseFocusTrapReturn | null = null
let escListenerRegistered = false

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
  /** Focus trap control for modal components */
  focusTrap: UseFocusTrapReturn
}

/**
 * useModal composable
 *
 * Provides centralized modal state management.
 * Only one modal can be open at a time.
 *
 * UX-5: Automatically handles ESC key to close modals and manages focus trap.
 *
 * @example
 * ```ts
 * const { isOpen, open, close, toggle, focusTrap } = useModal()
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
 *
 * // In modal component template:
 * // <div :ref="focusTrap.containerRef">...</div>
 * // In modal onMounted: focusTrap.activate(event.currentTarget as HTMLElement)
 * // In modal onBeforeUnmount: focusTrap.deactivate()
 * ```
 */
export function useModal(): UseModalReturn {
  // Lazily create focus trap instance
  if (!focusTrap) {
    focusTrap = useFocusTrap({
      onDeactivate: () => {
        // When focus trap deactivates, close the modal
        activeModal.value = null
      },
    })
  }

  const handleEscKey = (event: KeyboardEvent) => {
    if (event.key === 'Escape' && activeModal.value !== null) {
      event.preventDefault()
      activeModal.value = null
    }
  }

  const ensureEscListener = () => {
    if (!escListenerRegistered) {
      document.addEventListener('keydown', handleEscKey)
      escListenerRegistered = true
    }
  }

  const isOpen = (id: ModalId): boolean => {
    return activeModal.value === id
  }

  const open = (id: ModalId): void => {
    // If same modal is already open, do nothing
    if (activeModal.value === id) return

    // Remember the currently focused element before opening modal
    previousActiveElement.value = document.activeElement as HTMLElement

    // Ensure ESC listener is registered
    ensureEscListener()

    // Set new active modal (automatically closes any previous one)
    activeModal.value = id
  }

  const close = (id?: ModalId): void => {
    if (id === undefined) {
      // Close current modal
      activeModal.value = null
    } else {
      // Close specific modal only if it's currently open
      if (activeModal.value === id) {
        activeModal.value = null
      }
    }

    // Deactivate focus trap when modal closes
    if (focusTrap && activeModal.value === null) {
      focusTrap.deactivate()
    }
  }

  const toggle = (id: ModalId): void => {
    if (activeModal.value === id) {
      activeModal.value = null
      if (focusTrap) {
        focusTrap.deactivate()
      }
    } else {
      previousActiveElement.value = document.activeElement as HTMLElement
      ensureEscListener()
      activeModal.value = id
    }
  }

  return {
    activeModal: readonly(activeModal),
    isOpen,
    open,
    close,
    toggle,
    focusTrap: focusTrap!,
  }
}