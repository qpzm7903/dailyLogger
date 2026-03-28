import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick, ref } from 'vue'
import SessionDetailView from '../SessionDetailView.vue'

// Mock sessionActions
vi.mock('../../features/sessions/actions', () => ({
  sessionActions: {
    updateSessionUserSummary: vi.fn()
  }
}))

// Mock showToast
vi.mock('../stores/toast', () => ({
  showToast: vi.fn()
}))

// Mock i18n
vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string, params?: Record<string, unknown>) => {
      const translations: Record<string, string> = {
        'sessionDetailView.title': 'Session Details',
        'sessionDetailView.sessionTime': 'Session Time',
        'sessionDetailView.ongoing': 'Ongoing',
        'sessionDetailView.userSummary': 'User Summary',
        'sessionDetailView.aiSummary': 'AI Summary',
        'sessionDetailView.userEdited': 'User Edited',
        'sessionDetailView.editSummary': 'Edit Summary',
        'sessionDetailView.userSummaryPlaceholder': 'Enter your summary...',
        'sessionDetailView.cancel': 'Cancel',
        'sessionDetailView.save': 'Save',
        'sessionDetailView.saving': 'Saving...',
        'sessionDetailView.noSummary': 'No summary available',
        'sessionDetailView.summarySaved': 'Summary saved',
        'sessionDetailView.summarySaveFailed': 'Failed to save summary'
      }
      let result = translations[key] || key
      if (params?.error) {
        result = `${result}: ${params.error}`
      }
      return result
    },
    locale: {
      value: 'en'
    }
  })
}))

describe('SessionDetailView', () => {
  const mockSession = {
    id: 1,
    date: '2024-01-15',
    start_time: '2024-01-15T08:30:00+08:00',
    end_time: '2024-01-15T10:30:00+08:00',
    ai_summary: 'This is an AI summary',
    user_summary: null,
    context_for_next: null,
    status: 'ended' as const,
    screenshot_count: 5
  }

  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('rendering', () => {
    it('renders with overlay and modal', () => {
      const wrapper = mount(SessionDetailView, {
        props: { session: mockSession }
      })
      expect(wrapper.find('.fixed.inset-0').exists()).toBe(true)
      expect(wrapper.find('.bg-\\[var\\(--color-surface-1\\)\\]').exists()).toBe(true)
    })

    it('displays session title', () => {
      const wrapper = mount(SessionDetailView, {
        props: { session: mockSession }
      })
      expect(wrapper.text()).toContain('Session Details')
    })

    it('displays close button', () => {
      const wrapper = mount(SessionDetailView, {
        props: { session: mockSession }
      })
      const closeButton = wrapper.find('button')
      expect(closeButton.text()).toBe('✕')
    })

    it('displays session time with emoji', () => {
      const wrapper = mount(SessionDetailView, {
        props: { session: mockSession }
      })
      expect(wrapper.text()).toContain('⏱️')
      expect(wrapper.text()).toContain('Session Time')
    })

    it('displays formatted start time', () => {
      const wrapper = mount(SessionDetailView, {
        props: { session: mockSession }
      })
      // Formatted time should be in the component (may vary by timezone)
      const text = wrapper.text()
      expect(text).toMatch(/\d{2}:\d{2}/)
    })

    it('displays formatted end time when session has ended', () => {
      const wrapper = mount(SessionDetailView, {
        props: { session: mockSession }
      })
      // Formatted time should be in the component (may vary by timezone)
      const text = wrapper.text()
      const timeMatches = text.match(/\d{2}:\d{2}/g)
      // Should have at least two times (start and end)
      expect(timeMatches?.length).toBeGreaterThanOrEqual(2)
    })

    it('displays "Ongoing" when end_time is null', () => {
      const ongoingSession = { ...mockSession, end_time: null }
      const wrapper = mount(SessionDetailView, {
        props: { session: ongoingSession }
      })
      expect(wrapper.text()).toContain('Ongoing')
    })
  })

  describe('summary display', () => {
    it('displays AI summary when user has not edited', () => {
      const wrapper = mount(SessionDetailView, {
        props: { session: mockSession }
      })
      expect(wrapper.text()).toContain('This is an AI summary')
    })

    it('displays user summary when user has edited', () => {
      const editedSession = {
        ...mockSession,
        user_summary: 'This is user edited summary'
      }
      const wrapper = mount(SessionDetailView, {
        props: { session: editedSession }
      })
      expect(wrapper.text()).toContain('This is user edited summary')
    })

    it('prioritizes user summary over AI summary in display', () => {
      const editedSession = {
        ...mockSession,
        user_summary: 'User summary'
      }
      const wrapper = mount(SessionDetailView, {
        props: { session: editedSession }
      })
      // The main display should show user summary
      expect(wrapper.text()).toContain('User summary')
    })

    it('displays "No summary" when both are empty', () => {
      const emptySession = {
        ...mockSession,
        ai_summary: null,
        user_summary: null
      }
      const wrapper = mount(SessionDetailView, {
        props: { session: emptySession }
      })
      expect(wrapper.text()).toContain('No summary available')
    })

    it('shows edit button when not editing', () => {
      const wrapper = mount(SessionDetailView, {
        props: { session: mockSession }
      })
      expect(wrapper.text()).toContain('Edit Summary')
    })
  })

  describe('editing mode', () => {
    it('enters edit mode when edit button is clicked', async () => {
      const wrapper = mount(SessionDetailView, {
        props: { session: mockSession }
      })

      const editButton = wrapper.findAll('button').find(btn => btn.text() === 'Edit Summary')
      await editButton?.trigger('click')

      expect(wrapper.vm.isEditing).toBe(true)
    })

    it('prefills textarea with existing summary when entering edit mode', async () => {
      const wrapper = mount(SessionDetailView, {
        props: { session: mockSession }
      })

      const editButton = wrapper.findAll('button').find(btn => btn.text() === 'Edit Summary')
      await editButton?.trigger('click')

      const textarea = wrapper.find('textarea')
      expect(textarea.exists()).toBe(true)
      expect(textarea.element.value).toBe('This is an AI summary')
    })

    it('shows cancel and save buttons when editing', async () => {
      const wrapper = mount(SessionDetailView, {
        props: { session: mockSession }
      })

      const editButton = wrapper.findAll('button').find(btn => btn.text() === 'Edit Summary')
      await editButton?.trigger('click')

      expect(wrapper.text()).toContain('Cancel')
      expect(wrapper.text()).toContain('Save')
    })

    it('exits edit mode when cancel is clicked', async () => {
      const wrapper = mount(SessionDetailView, {
        props: { session: mockSession }
      })

      // Enter edit mode
      const editButton = wrapper.findAll('button').find(btn => btn.text() === 'Edit Summary')
      await editButton?.trigger('click')
      expect(wrapper.vm.isEditing).toBe(true)

      // Click cancel
      const cancelButton = wrapper.findAll('button').find(btn => btn.text() === 'Cancel')
      await cancelButton?.trigger('click')

      expect(wrapper.vm.isEditing).toBe(false)
    })

    it('shows "User Edited" badge when user has summary', () => {
      const editedSession = {
        ...mockSession,
        user_summary: 'User summary'
      }
      const wrapper = mount(SessionDetailView, {
        props: { session: editedSession }
      })
      expect(wrapper.text()).toContain('User Edited')
    })

    it('does not show "User Edited" badge when only AI summary exists', () => {
      const wrapper = mount(SessionDetailView, {
        props: { session: mockSession }
      })
      expect(wrapper.text()).not.toContain('User Edited')
    })
  })

  describe('save functionality', () => {
    it('calls updateSessionUserSummary when save is clicked', async () => {
      const { sessionActions } = await import('../../features/sessions/actions')
      const wrapper = mount(SessionDetailView, {
        props: { session: mockSession }
      })

      // Enter edit mode
      const editButton = wrapper.findAll('button').find(btn => btn.text() === 'Edit Summary')
      await editButton?.trigger('click')

      // Modify the summary
      const textarea = wrapper.find('textarea')
      await textarea.setValue('New summary content')

      // Click save
      const saveButton = wrapper.findAll('button').find(btn => btn.text() === 'Save')
      await saveButton?.trigger('click')

      expect(sessionActions.updateSessionUserSummary).toHaveBeenCalledWith(1, 'New summary content')
    })

    it('emits updated event after successful save', async () => {
      const { sessionActions } = await import('../../features/sessions/actions')
      ;(sessionActions.updateSessionUserSummary as ReturnType<typeof vi.fn>).mockResolvedValue(undefined)

      const wrapper = mount(SessionDetailView, {
        props: { session: mockSession }
      })

      // Enter edit mode and change summary
      const editButton = wrapper.findAll('button').find(btn => btn.text() === 'Edit Summary')
      await editButton?.trigger('click')

      const textarea = wrapper.find('textarea')
      await textarea.setValue('New summary')

      // Click save
      const saveButton = wrapper.findAll('button').find(btn => btn.text() === 'Save')
      await saveButton?.trigger('click')

      await nextTick()

      expect(wrapper.emitted('updated')).toBeTruthy()
    })

    it('disables save button while saving', async () => {
      const { sessionActions } = await import('../../features/sessions/actions')
      ;(sessionActions.updateSessionUserSummary as ReturnType<typeof vi.fn>).mockImplementation(() => new Promise(() => {}))

      const wrapper = mount(SessionDetailView, {
        props: { session: mockSession }
      })

      // Enter edit mode
      const editButton = wrapper.findAll('button').find(btn => btn.text() === 'Edit Summary')
      await editButton?.trigger('click')

      // Click save
      const saveButton = wrapper.findAll('button').find(btn => btn.text() === 'Save')
      await saveButton?.trigger('click')

      expect(wrapper.vm.isSaving).toBe(true)
    })
  })

  describe('close event', () => {
    it('emits close when close button is clicked', async () => {
      const wrapper = mount(SessionDetailView, {
        props: { session: mockSession }
      })

      const closeButton = wrapper.findAll('button').at(0)
      await closeButton?.trigger('click')

      expect(wrapper.emitted('close')).toBeTruthy()
    })

    it('emits close when overlay is clicked', async () => {
      const wrapper = mount(SessionDetailView, {
        props: { session: mockSession }
      })

      const overlay = wrapper.find('.fixed.inset-0')
      await overlay.trigger('click')

      expect(wrapper.emitted('close')).toBeTruthy()
    })
  })
})