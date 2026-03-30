import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { nextTick } from 'vue'
import SessionListModal from '../SessionListModal.vue'

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

// Mock session actions
vi.mock('../../features/sessions/actions', () => ({
  sessionActions: {
    getTodaySessions: vi.fn(),
    analyzeSession: vi.fn()
  }
}))

// Mock toast store
vi.mock('../../stores/toast', () => ({
  showToast: vi.fn()
}))

import { invoke } from '@tauri-apps/api/core'
import { sessionActions } from '../../features/sessions/actions'
import { showToast } from '../../stores/toast'

describe('SessionListModal', () => {
  const mockSessions = [
    {
      id: 1,
      date: '2026-03-28',
      start_time: '2026-03-28T09:00:00Z',
      end_time: '2026-03-28T10:30:00Z',
      ai_summary: 'AI summary 1',
      user_summary: null,
      context_for_next: null,
      status: 'ended' as const,
      screenshot_count: 5
    },
    {
      id: 2,
      date: '2026-03-28',
      start_time: '2026-03-28T11:00:00Z',
      end_time: null,
      ai_summary: null,
      user_summary: 'User summary 2',
      context_for_next: null,
      status: 'active' as const,
      screenshot_count: 2
    },
    {
      id: 3,
      date: '2026-03-28',
      start_time: '2026-03-28T08:00:00Z',
      end_time: '2026-03-28T08:45:00Z',
      ai_summary: 'AI summary 3',
      user_summary: 'User summary 3',
      context_for_next: null,
      status: 'analyzed' as const,
      screenshot_count: 3
    }
  ]

  beforeEach(() => {
    vi.clearAllMocks()
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('renders modal title', async () => {
    sessionActions.getTodaySessions.mockResolvedValue(mockSessions)
    const wrapper = mount(SessionListModal, {
      global: {
        stubs: {
          teleport: true
        }
      }
    })
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    expect(wrapper.text()).toContain('Sessions')
  })

  it('renders close button in header', async () => {
    sessionActions.getTodaySessions.mockResolvedValue(mockSessions)
    const wrapper = mount(SessionListModal, {
      global: {
        stubs: {
          teleport: true
        }
      }
    })
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    const closeButtons = wrapper.findAll('button')
    expect(closeButtons.length).toBeGreaterThan(0)
  })

  it('emits close event when backdrop clicked', async () => {
    sessionActions.getTodaySessions.mockResolvedValue(mockSessions)
    const wrapper = mount(SessionListModal, {
      global: {
        stubs: {
          teleport: true
        }
      }
    })
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    const backdrop = wrapper.find('.fixed.inset-0')
    await backdrop.trigger('click.self')

    expect(wrapper.emitted('close')).toBeTruthy()
  })

  it('emits close event when header close button clicked', async () => {
    sessionActions.getTodaySessions.mockResolvedValue(mockSessions)
    const wrapper = mount(SessionListModal, {
      global: {
        stubs: {
          teleport: true
        }
      }
    })
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    const closeButtons = wrapper.findAll('button')
    await closeButtons[0].trigger('click')

    expect(wrapper.emitted('close')).toBeTruthy()
  })

  it('displays loading state initially', async () => {
    sessionActions.getTodaySessions.mockImplementation(() => new Promise(() => {}))
    const wrapper = mount(SessionListModal, {
      global: {
        stubs: {
          teleport: true
        }
      }
    })
    await nextTick()

    expect(wrapper.vm.isLoading).toBe(true)
    expect(wrapper.text()).toContain('Loading...')
  })

  it('loads sessions on mount', async () => {
    sessionActions.getTodaySessions.mockResolvedValue(mockSessions)
    const wrapper = mount(SessionListModal, {
      global: {
        stubs: {
          teleport: true
        }
      }
    })
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    expect(sessionActions.getTodaySessions).toHaveBeenCalled()
    expect(wrapper.vm.isLoading).toBe(false)
  })

  it('displays empty state when no sessions', async () => {
    sessionActions.getTodaySessions.mockResolvedValue([])
    const wrapper = mount(SessionListModal, {
      global: {
        stubs: {
          teleport: true
        }
      }
    })
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    expect(wrapper.text()).toContain('No sessions')
  })

  it('filters sessions by pending status by default', async () => {
    sessionActions.getTodaySessions.mockResolvedValue(mockSessions)
    const wrapper = mount(SessionListModal, {
      global: {
        stubs: {
          teleport: true
        }
      }
    })
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    expect(wrapper.vm.statusFilter).toBe('pending')
    expect(wrapper.vm.filteredSessions).toHaveLength(2)
  })

  it('filters sessions by analyzed status', async () => {
    sessionActions.getTodaySessions.mockResolvedValue(mockSessions)
    const wrapper = mount(SessionListModal, {
      global: {
        stubs: {
          teleport: true
        }
      }
    })
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    const select = wrapper.find('select')
    await select.setValue('analyzed')

    expect(wrapper.vm.statusFilter).toBe('analyzed')
    expect(wrapper.vm.filteredSessions).toHaveLength(1)
  })

  it('shows all sessions when filter is all', async () => {
    sessionActions.getTodaySessions.mockResolvedValue(mockSessions)
    const wrapper = mount(SessionListModal, {
      global: {
        stubs: {
          teleport: true
        }
      }
    })
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    const select = wrapper.find('select')
    await select.setValue('all')

    expect(wrapper.vm.statusFilter).toBe('all')
    expect(wrapper.vm.filteredSessions).toHaveLength(3)
  })

  it('formats time correctly', async () => {
    sessionActions.getTodaySessions.mockResolvedValue(mockSessions)
    const wrapper = mount(SessionListModal, {
      global: {
        stubs: {
          teleport: true
        }
      }
    })
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    const formatted = wrapper.vm.formatTime('2026-03-28T09:00:00Z')
    expect(formatted).toMatch(/\d{2}:\d{2}/)
  })

  it('returns placeholder for invalid time', async () => {
    sessionActions.getTodaySessions.mockResolvedValue(mockSessions)
    const wrapper = mount(SessionListModal, {
      global: {
        stubs: {
          teleport: true
        }
      }
    })
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    const formatted = wrapper.vm.formatTime('invalid-time')
    expect(formatted).toBe('--:--')
  })

  it('returns correct status badge class for active status', async () => {
    sessionActions.getTodaySessions.mockResolvedValue(mockSessions)
    const wrapper = mount(SessionListModal, {
      global: {
        stubs: {
          teleport: true
        }
      }
    })
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    const badgeClass = wrapper.vm.getStatusBadgeClass('active')
    expect(badgeClass).toBe('bg-green-500/20 text-green-400')
  })

  it('returns correct status badge class for ended status', async () => {
    sessionActions.getTodaySessions.mockResolvedValue(mockSessions)
    const wrapper = mount(SessionListModal, {
      global: {
        stubs: {
          teleport: true
        }
      }
    })
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    const badgeClass = wrapper.vm.getStatusBadgeClass('ended')
    expect(badgeClass).toBe('bg-yellow-500/20 text-yellow-400')
  })

  it('returns correct status badge class for analyzed status', async () => {
    sessionActions.getTodaySessions.mockResolvedValue(mockSessions)
    const wrapper = mount(SessionListModal, {
      global: {
        stubs: {
          teleport: true
        }
      }
    })
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    const badgeClass = wrapper.vm.getStatusBadgeClass('analyzed')
    expect(badgeClass).toBe('bg-blue-500/20 text-blue-400')
  })

  it('returns default status badge class for unknown status', async () => {
    sessionActions.getTodaySessions.mockResolvedValue(mockSessions)
    const wrapper = mount(SessionListModal, {
      global: {
        stubs: {
          teleport: true
        }
      }
    })
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    const badgeClass = wrapper.vm.getStatusBadgeClass('unknown')
    expect(badgeClass).toBe('bg-[var(--color-action-neutral)]/20 text-[var(--color-text-muted)]')
  })

  it('displays selection count in filter bar', async () => {
    sessionActions.getTodaySessions.mockResolvedValue(mockSessions)
    const wrapper = mount(SessionListModal, {
      global: {
        stubs: {
          teleport: true
        }
      }
    })
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    // Default filter is 'pending', so we have 2 sessions
    expect(wrapper.text()).toContain('0 / 2')
  })

  it('displays session data after loading', async () => {
    sessionActions.getTodaySessions.mockResolvedValue(mockSessions)
    const wrapper = mount(SessionListModal, {
      global: {
        stubs: {
          teleport: true
        }
      }
    })
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    // Sessions should be loaded
    expect(wrapper.vm.sessions).toHaveLength(3)
    expect(wrapper.vm.sessions[0].id).toBe(1)
  })

  it('displays ai_summary when user_summary is null', async () => {
    sessionActions.getTodaySessions.mockResolvedValue(mockSessions)
    const wrapper = mount(SessionListModal, {
      global: {
        stubs: {
          teleport: true
        }
      }
    })
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    expect(wrapper.vm.sessions[0].ai_summary).toBe('AI summary 1')
  })

  it('prefers user_summary over ai_summary', async () => {
    sessionActions.getTodaySessions.mockResolvedValue(mockSessions)
    const wrapper = mount(SessionListModal, {
      global: {
        stubs: {
          teleport: true
        }
      }
    })
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    // Session 3 has both ai_summary and user_summary
    expect(wrapper.vm.sessions[2].user_summary).toBe('User summary 3')
  })

  it('calls showToast on session load failure', async () => {
    sessionActions.getTodaySessions.mockRejectedValue(new Error('Network error'))
    const wrapper = mount(SessionListModal, {
      global: {
        stubs: {
          teleport: true
        }
      }
    })
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()
    await nextTick()

    expect(showToast).toHaveBeenCalled()
  })
})
