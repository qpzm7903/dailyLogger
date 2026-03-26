// UX-5: useModal composable
// Centralized modal management - replaces 21 individual showXxx ref variables
// UX-5: Extended with ESC key listener

import { ref, readonly, onMounted, onUnmounted, type Ref, type DeepReadonly } from 'vue'

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
  const focusTrapActive = ref(false)
  let focusTrapPreviousElement: HTMLElement | null = null

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

  const getFocusableElements = (): HTMLElement[] => {
    if (!focusTrapContainerRef.value) return []
    const selector = [
      'button:not([disabled])',
      'input:not([disabled])',
      'select:not([disabled])',
      'textarea:not([disabled])',
      'a[href]',
      '[tabindex]:not([tabindex="-1"])',
      '[contenteditable="true"]',
    ].join(', ')
    return Array.from(focusTrapContainerRef.value.querySelectorAll<HTMLElement>(selector))
  }

  const handleTabKey = (event: KeyboardEvent) => {
    if (!focusTrapActive.value || event.key !== 'Tab') return
    const focusable = getFocusableElements()
    if (focusable.length === 0) return
    const first = focusable[0]
    const last = focusable[focusable.length - 1]
    if (event.shiftKey) {
      if (document.activeElement === first) {
        event.preventDefault()
        last.focus()
      }
    } else {
      if (document.activeElement === last) {
        event.preventDefault()
        first.focus()
      }
    }
  }

  const activateFocusTrap = () => {
    if (focusTrapActive.value) return
    focusTrapPreviousElement = document.activeElement as HTMLElement
    focusTrapActive.value = true
    document.addEventListener('keydown', handleTabKey)
    requestAnimationFrame(() => {
      const focusable = getFocusableElements()
      if (focusable.length > 0) {
        focusable[0].focus()
      } else if (focusTrapContainerRef.value) {
        focusTrapContainerRef.value.focus()
      }
    })
  }

  const deactivateFocusTrap = () => {
    if (!focusTrapActive.value) return
    document.removeEventListener('keydown', handleTabKey)
    if (focusTrapPreviousElement && focusTrapPreviousElement.focus) {
      focusTrapPreviousElement.focus()
    }
    focusTrapActive.value = false
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
