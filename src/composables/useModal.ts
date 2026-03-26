// UX-010: useModal composable
// Centralized modal management - replaces 21 individual showXxx ref variables

import { ref, readonly, type Ref, type DeepReadonly } from 'vue'

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
  const isOpen = (id: ModalId): boolean => {
    return activeModal.value === id
  }

  const open = (id: ModalId): void => {
    // If same modal is already open, do nothing
    if (activeModal.value === id) return
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
  }

  const toggle = (id: ModalId): void => {
    if (activeModal.value === id) {
      activeModal.value = null
    } else {
      activeModal.value = id
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