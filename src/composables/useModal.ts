// UX-5: useModal composable
// Centralized modal management - replaces 21 individual showXxx ref variables
// UX-5: Extended with ESC key listener

import { ref, readonly, type Ref, type DeepReadonly } from 'vue'
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
  | 'reportHistory'
  | 'offlineQueue'
  | 'reanalyzeByDate'
  | 'sessionList'
  | 'statistics'

// Module-level state (singleton pattern)
const activeModal: Ref<ModalId | null> = ref(null)
const previousActiveElement = ref<HTMLElement | null>(null)
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
  focusTrap: {
    activate: () => void
    deactivate: () => void
    isActive: Ref<boolean>
    containerRef: Ref<HTMLElement | null>
  }
}

/**
 * useModal composable
 *
 * Provides centralized modal state management.
 * Only one modal can be open at a time.
 *
 * UX-5: Automatically handles ESC key to close modals.
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
  const focusTrapContainerRef = ref<HTMLElement | null>(null)
  const { activate: activateFocusTrap, deactivate: deactivateFocusTrap, isActive: focusTrapActive } = useFocusTrap(focusTrapContainerRef)

  const handleKeydown = (event: KeyboardEvent) => {
    if (event.key === 'Escape' && activeModal.value !== null) {
      event.preventDefault()
      activeModal.value = null
    }
  }

  const ensureEscListener = () => {
    if (!escListenerRegistered) {
      document.addEventListener('keydown', handleKeydown)
      escListenerRegistered = true
    }
  }

  const isOpen = (id: ModalId): boolean => {
    return activeModal.value === id
  }

  const open = (id: ModalId): void => {
    if (activeModal.value === id) return
    previousActiveElement.value = document.activeElement as HTMLElement
    ensureEscListener()
    activeModal.value = id
  }

  const close = (id?: ModalId): void => {
    if (id === undefined) {
      activeModal.value = null
      deactivateFocusTrap()
      if (previousActiveElement.value && previousActiveElement.value.focus) {
        previousActiveElement.value.focus()
      }
    } else {
      if (activeModal.value === id) {
        activeModal.value = null
        deactivateFocusTrap()
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
    focusTrap: {
      activate: activateFocusTrap,
      deactivate: deactivateFocusTrap,
      isActive: focusTrapActive,
      containerRef: focusTrapContainerRef,
    },
  }
}
