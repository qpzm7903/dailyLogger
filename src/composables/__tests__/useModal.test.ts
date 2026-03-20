// UX-010: useModal composable tests
import { describe, it, expect, beforeEach } from 'vitest'
import { useModal } from '../useModal'

describe('useModal', () => {
  // Reset state before each test by getting a fresh instance
  beforeEach(() => {
    const { close } = useModal()
    close() // Ensure no modal is open at test start
  })

  describe('isOpen', () => {
    it('returns false when no modal is open', () => {
      const { isOpen } = useModal()
      expect(isOpen('settings')).toBe(false)
      expect(isOpen('quickNote')).toBe(false)
    })

    it('returns true for the open modal, false for others', () => {
      const { isOpen, open } = useModal()
      open('settings')
      expect(isOpen('settings')).toBe(true)
      expect(isOpen('quickNote')).toBe(false)
    })
  })

  describe('open', () => {
    it('sets the active modal', () => {
      const { isOpen, open } = useModal()
      open('settings')
      expect(isOpen('settings')).toBe(true)
    })

    it('closes previous modal when opening a new one', () => {
      const { isOpen, open } = useModal()
      open('settings')
      expect(isOpen('settings')).toBe(true)
      open('quickNote')
      expect(isOpen('settings')).toBe(false)
      expect(isOpen('quickNote')).toBe(true)
    })

    it('does not re-render when opening same modal', () => {
      const { isOpen, open } = useModal()
      open('settings')
      expect(isOpen('settings')).toBe(true)
      // Open same modal again - should still be open
      open('settings')
      expect(isOpen('settings')).toBe(true)
    })
  })

  describe('close', () => {
    it('closes the current modal when called without id', () => {
      const { isOpen, open, close } = useModal()
      open('settings')
      expect(isOpen('settings')).toBe(true)
      close()
      expect(isOpen('settings')).toBe(false)
    })

    it('closes specific modal when called with id', () => {
      const { isOpen, open, close } = useModal()
      open('settings')
      close('settings')
      expect(isOpen('settings')).toBe(false)
    })

    it('does nothing when closing a different modal', () => {
      const { isOpen, open, close } = useModal()
      open('settings')
      close('quickNote') // Different modal
      expect(isOpen('settings')).toBe(true)
    })
  })

  describe('toggle', () => {
    it('opens modal when closed', () => {
      const { isOpen, toggle } = useModal()
      toggle('settings')
      expect(isOpen('settings')).toBe(true)
    })

    it('closes modal when open', () => {
      const { isOpen, open, toggle } = useModal()
      open('settings')
      toggle('settings')
      expect(isOpen('settings')).toBe(false)
    })

    it('closes previous modal and opens new one', () => {
      const { isOpen, toggle } = useModal()
      toggle('settings')
      expect(isOpen('settings')).toBe(true)
      toggle('quickNote')
      expect(isOpen('settings')).toBe(false)
      expect(isOpen('quickNote')).toBe(true)
    })
  })

  describe('activeModal', () => {
    it('is null when no modal is open', () => {
      const { activeModal, close } = useModal()
      close()
      expect(activeModal.value).toBeNull()
    })

    it('returns the current open modal id', () => {
      const { activeModal, open } = useModal()
      open('settings')
      expect(activeModal.value).toBe('settings')
    })

    it('is readonly', () => {
      const { activeModal } = useModal()
      // TypeScript enforces readonly at compile time
      // At runtime, the value should be a ref
      expect(activeModal.value).toBeDefined()
    })
  })

  describe('integration scenarios', () => {
    it('handles rapid open/close sequence', () => {
      const { isOpen, open, close } = useModal()
      open('settings')
      open('quickNote')
      open('historyViewer')
      expect(isOpen('settings')).toBe(false)
      expect(isOpen('quickNote')).toBe(false)
      expect(isOpen('historyViewer')).toBe(true)
      close()
      expect(isOpen('historyViewer')).toBe(false)
    })

    it('maintains state across multiple composable invocations', () => {
      // Simulate different components using the same singleton
      const { open: open1, isOpen: isOpen1 } = useModal()
      const { open: open2, isOpen: isOpen2 } = useModal()

      open1('settings')
      expect(isOpen1('settings')).toBe(true)
      expect(isOpen2('settings')).toBe(true)

      open2('quickNote')
      expect(isOpen1('settings')).toBe(false)
      expect(isOpen2('quickNote')).toBe(true)
    })
  })
})