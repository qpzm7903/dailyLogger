import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { nextTick } from 'vue'
import ScreenshotModal from '../ScreenshotModal.vue'

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

// Mock toast
vi.mock('../../stores/toast', () => ({
  showToast: vi.fn()
}))

// Mock captureActions
vi.mock('../../features/capture/actions', () => ({
  captureActions: {
    getScreenshot: vi.fn(),
    reanalyzeRecord: vi.fn()
  }
}))

// Mock recordsActions
vi.mock('../../features/records/actions', () => ({
  recordsActions: {
    updateRecordUserNotes: vi.fn()
  }
}))

import { invoke } from '@tauri-apps/api/core'
import { showToast } from '../../stores/toast'
import { captureActions } from '../../features/capture/actions'
import { recordsActions } from '../../features/records/actions'

describe('ScreenshotModal - Window Info Display (SMART-001 Task 6)', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    invoke.mockResolvedValue('data:image/png;base64,mockBase64Data')
    captureActions.getScreenshot.mockResolvedValue('data:image/png;base64,mockBase64Data')
  })

  describe('Window info in record details', () => {
    it('displays window title when active_window exists', async () => {
      const record = {
        id: 1,
        timestamp: '2026-03-15T10:00:00Z',
        source_type: 'auto',
        screenshot_path: '/path/screenshot1.png',
        content: JSON.stringify({
          current_focus: 'Working on code',
          active_software: 'VS Code',
          context_keywords: ['code', 'development'],
          active_window: {
            title: 'main.rs - DailyLogger - VS Code',
            process_name: 'Code'
          }
        })
      }

      const wrapper = mount(ScreenshotModal, {
        props: { record },
        global: {
          stubs: {}
        }
      })

      await nextTick()
      await nextTick()

      const html = wrapper.html()
      expect(html).toContain('main.rs - DailyLogger - VS Code')
    })

    it('displays process name when active_window exists', async () => {
      const record = {
        id: 1,
        timestamp: '2026-03-15T10:00:00Z',
        source_type: 'auto',
        screenshot_path: '/path/screenshot1.png',
        content: JSON.stringify({
          current_focus: 'Working on project',
          active_software: 'IntelliJ IDEA',
          context_keywords: ['java'],
          active_window: {
            title: 'MyProject - IntelliJ IDEA',
            process_name: 'idea64'
          }
        })
      }

      const wrapper = mount(ScreenshotModal, {
        props: { record },
        global: {
          stubs: {}
        }
      })

      await nextTick()
      await nextTick()

      const html = wrapper.html()
      expect(html).toContain('idea64')
    })

    it('shows window info section label when window info exists', async () => {
      const record = {
        id: 1,
        timestamp: '2026-03-15T10:00:00Z',
        source_type: 'auto',
        screenshot_path: '/path/screenshot1.png',
        content: JSON.stringify({
          current_focus: 'Working on code',
          active_software: 'VS Code',
          context_keywords: ['code'],
          active_window: {
            title: 'VS Code Window',
            process_name: 'Code'
          }
        })
      }

      const wrapper = mount(ScreenshotModal, {
        props: { record },
        global: {
          stubs: {}
        }
      })

      await nextTick()
      await nextTick()

      const html = wrapper.html()
      expect(html).toContain('Window')
    })

    it('does not show window info section when active_window is missing', async () => {
      const record = {
        id: 1,
        timestamp: '2026-03-15T10:00:00Z',
        source_type: 'auto',
        screenshot_path: '/path/screenshot1.png',
        content: JSON.stringify({
          current_focus: 'Working on code',
          active_software: 'VS Code',
          context_keywords: ['code']
        })
      }

      const wrapper = mount(ScreenshotModal, {
        props: { record },
        global: {
          stubs: {}
        }
      })

      await nextTick()
      await nextTick()

      const windowInfoSection = wrapper.find('.window-info-section')
      expect(windowInfoSection.exists()).toBe(false)
    })

    it('handles empty active_window gracefully', async () => {
      const record = {
        id: 1,
        timestamp: '2026-03-15T10:00:00Z',
        source_type: 'auto',
        screenshot_path: '/path/screenshot1.png',
        content: JSON.stringify({
          current_focus: 'Working on code',
          active_software: 'VS Code',
          context_keywords: ['code'],
          active_window: {
            title: '',
            process_name: ''
          }
        })
      }

      const wrapper = mount(ScreenshotModal, {
        props: { record },
        global: {
          stubs: {}
        }
      })

      await nextTick()
      await nextTick()

      expect(wrapper.html()).toContain('Screenshot Details')
    })

    it('displays window icon based on process name', async () => {
      const record = {
        id: 1,
        timestamp: '2026-03-15T10:00:00Z',
        source_type: 'auto',
        screenshot_path: '/path/screenshot1.png',
        content: JSON.stringify({
          current_focus: 'Browsing',
          active_software: 'Chrome',
          context_keywords: ['web'],
          active_window: {
            title: 'Google - Chrome',
            process_name: 'chrome'
          }
        })
      }

      const wrapper = mount(ScreenshotModal, {
        props: { record },
        global: {
          stubs: {}
        }
      })

      await nextTick()
      await nextTick()

      const html = wrapper.html()
      expect(html).toContain('Google - Chrome')
      expect(html).toContain('chrome')
    })
  })

  describe('parseContent', () => {
    it('parses content JSON correctly with active_window', async () => {
      const record = {
        id: 1,
        timestamp: '2026-03-15T10:00:00Z',
        source_type: 'auto',
        screenshot_path: '/path/screenshot1.png',
        content: JSON.stringify({
          current_focus: 'Writing documentation',
          active_software: 'Typora',
          context_keywords: ['docs', 'markdown'],
          active_window: {
            title: 'README.md - Typora',
            process_name: 'Typora'
          }
        })
      }

      const wrapper = mount(ScreenshotModal, {
        props: { record },
        global: {
          stubs: {}
        }
      })

      await nextTick()
      await nextTick()

      const html = wrapper.html()
      expect(html).toContain('Writing documentation')
      expect(html).toContain('Typora')
      expect(html).toContain('README.md - Typora')
    })

    it('handles malformed content JSON gracefully', async () => {
      const record = {
        id: 1,
        timestamp: '2026-03-15T10:00:00Z',
        source_type: 'auto',
        screenshot_path: '/path/screenshot1.png',
        content: 'This is not valid JSON'
      }

      const wrapper = mount(ScreenshotModal, {
        props: { record },
        global: {
          stubs: {}
        }
      })

      await nextTick()
      await nextTick()

      expect(wrapper.html()).toContain('This is not valid JSON')
    })
  })
})

describe('ScreenshotModal - FEAT-001 Reanalyze', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    captureActions.getScreenshot.mockResolvedValue('data:image/png;base64,mockBase64Data')
  })

  const mockRecord = {
    id: 1,
    timestamp: '2026-03-28T10:00:00Z',
    source_type: 'auto' as const,
    content: JSON.stringify({
      current_focus: 'Working on Rust code',
      active_software: 'VSCode',
      context_keywords: ['rust', 'testing']
    }),
    screenshot_path: '/path/to/screenshot.png',
    user_notes: null
  }

  it('shows reanalyze button', async () => {
    const wrapper = mount(ScreenshotModal, {
      props: { record: mockRecord },
      global: { stubs: {} }
    })
    await nextTick()

    const buttons = wrapper.findAll('button')
    const reanalyzeButton = buttons.find(b => b.text().includes('Reanalyze'))
    expect(reanalyzeButton).toBeTruthy()
  })

  it('calls reanalyzeRecord when button clicked', async () => {
    captureActions.reanalyzeRecord.mockResolvedValue({
      current_focus: 'Reanalyzed focus',
      active_software: 'NewSoftware',
      context_keywords: ['new', 'keywords']
    })

    const wrapper = mount(ScreenshotModal, {
      props: { record: mockRecord },
      global: { stubs: {} }
    })
    await nextTick()

    const buttons = wrapper.findAll('button')
    const reanalyzeButton = buttons.find(b => b.text().includes('Reanalyze'))
    await reanalyzeButton.trigger('click')
    await flushPromises()

    expect(captureActions.reanalyzeRecord).toHaveBeenCalledWith(1)
  })

  it('shows success toast after successful reanalyze', async () => {
    captureActions.reanalyzeRecord.mockResolvedValue({
      current_focus: 'Reanalyzed focus',
      active_software: 'NewSoftware',
      context_keywords: ['new', 'keywords']
    })

    const wrapper = mount(ScreenshotModal, {
      props: { record: mockRecord },
      global: { stubs: {} }
    })
    await nextTick()

    const buttons = wrapper.findAll('button')
    const reanalyzeButton = buttons.find(b => b.text().includes('Reanalyze'))
    await reanalyzeButton.trigger('click')
    await flushPromises()

    expect(showToast).toHaveBeenCalledWith('Reanalysis complete', { type: 'success' })
  })

  it('emits updated event after successful reanalyze', async () => {
    const newAnalysis = {
      current_focus: 'Reanalyzed focus',
      active_software: 'NewSoftware',
      context_keywords: ['new', 'keywords']
    }
    captureActions.reanalyzeRecord.mockResolvedValue(newAnalysis)

    const wrapper = mount(ScreenshotModal, {
      props: { record: mockRecord },
      global: { stubs: {} }
    })
    await nextTick()

    const buttons = wrapper.findAll('button')
    const reanalyzeButton = buttons.find(b => b.text().includes('Reanalyze'))
    await reanalyzeButton.trigger('click')
    await flushPromises()

    expect(wrapper.emitted('updated')).toBeTruthy()
    const updatedEvent = wrapper.emitted('updated')[0][0]
    expect(updatedEvent.content).toBe(JSON.stringify(newAnalysis))
    expect(updatedEvent.analysis_status).toBe('analyzed')
  })

  it('shows error toast when reanalyze fails', async () => {
    captureActions.reanalyzeRecord.mockRejectedValue(new Error('API Error'))

    const wrapper = mount(ScreenshotModal, {
      props: { record: mockRecord },
      global: { stubs: {} }
    })
    await nextTick()

    const buttons = wrapper.findAll('button')
    const reanalyzeButton = buttons.find(b => b.text().includes('Reanalyze'))
    await reanalyzeButton.trigger('click')
    await flushPromises()

    expect(showToast).toHaveBeenCalled()
    const toastCall = showToast.mock.calls[0]
    expect(toastCall[1].type).toBe('error')
  })

  it('disables reanalyze button while reanalyzing', async () => {
    captureActions.reanalyzeRecord.mockReturnValue(new Promise(() => {}))

    const wrapper = mount(ScreenshotModal, {
      props: { record: mockRecord },
      global: { stubs: {} }
    })
    await nextTick()

    const buttons = wrapper.findAll('button')
    const reanalyzeButton = buttons.find(b => b.text().includes('Reanalyze'))
    await reanalyzeButton.trigger('click')
    await nextTick()

    const disabledButton = wrapper.find('button:disabled')
    expect(disabledButton.exists()).toBe(true)
  })

  it('hides reanalyze button for preview-only screenshots', async () => {
    const previewRecord = {
      ...mockRecord,
      id: 0,
      content: ''
    }

    const wrapper = mount(ScreenshotModal, {
      props: { record: previewRecord },
      global: { stubs: {} }
    })
    await nextTick()

    expect(wrapper.text()).toContain('This screenshot is only a preview')
    expect(wrapper.text()).not.toContain('Reanalyze')
  })
})

describe('ScreenshotModal - FEAT-005 User Notes', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    captureActions.getScreenshot.mockResolvedValue('data:image/png;base64,mockBase64Data')
  })

  const mockRecord = {
    id: 1,
    timestamp: '2026-03-28T10:00:00Z',
    source_type: 'auto' as const,
    content: JSON.stringify({
      current_focus: 'Working on Rust code',
      active_software: 'VSCode',
      context_keywords: ['rust', 'testing']
    }),
    screenshot_path: '/path/to/screenshot.png',
    user_notes: null
  }

  it('displays user notes textarea', async () => {
    const wrapper = mount(ScreenshotModal, {
      props: { record: mockRecord },
      global: { stubs: {} }
    })
    await nextTick()

    const textarea = wrapper.find('textarea')
    expect(textarea.exists()).toBe(true)
  })

  it('saves user notes successfully', async () => {
    recordsActions.updateRecordUserNotes.mockResolvedValue(undefined)

    const wrapper = mount(ScreenshotModal, {
      props: { record: mockRecord },
      global: { stubs: {} }
    })
    await nextTick()

    const textarea = wrapper.find('textarea')
    await textarea.setValue('New note content')
    await wrapper.vm.$nextTick()

    const buttons = wrapper.findAll('button')
    const saveButton = buttons.find(b => b.text().includes('Save'))
    expect(saveButton).toBeTruthy()
    await saveButton.trigger('click')
    await flushPromises()

    expect(recordsActions.updateRecordUserNotes).toHaveBeenCalledWith(1, 'New note content')
    expect(showToast).toHaveBeenCalledWith('Notes saved', { type: 'success' })
  })

  it('emits updated event after saving notes', async () => {
    recordsActions.updateRecordUserNotes.mockResolvedValue(undefined)

    const wrapper = mount(ScreenshotModal, {
      props: { record: mockRecord },
      global: { stubs: {} }
    })
    await nextTick()

    const textarea = wrapper.find('textarea')
    await textarea.setValue('New note content')
    await wrapper.vm.$nextTick()

    const buttons = wrapper.findAll('button')
    const saveButton = buttons.find(b => b.text().includes('Save'))
    await saveButton.trigger('click')
    await flushPromises()

    expect(wrapper.emitted('updated')).toBeTruthy()
  })

  it('shows error toast when saving notes fails', async () => {
    recordsActions.updateRecordUserNotes.mockRejectedValue(new Error('Save failed'))

    const wrapper = mount(ScreenshotModal, {
      props: { record: mockRecord },
      global: { stubs: {} }
    })
    await nextTick()

    const textarea = wrapper.find('textarea')
    await textarea.setValue('New note content')
    await wrapper.vm.$nextTick()

    const buttons = wrapper.findAll('button')
    const saveButton = buttons.find(b => b.text().includes('Save'))
    await saveButton.trigger('click')
    await flushPromises()

    expect(showToast).toHaveBeenCalled()
    const toastCall = showToast.mock.calls[0]
    expect(toastCall[1].type).toBe('error')
  })

  it('disables notes editing for preview-only screenshots', async () => {
    const previewRecord = {
      ...mockRecord,
      id: 0,
      content: ''
    }

    const wrapper = mount(ScreenshotModal, {
      props: { record: previewRecord },
      global: { stubs: {} }
    })
    await nextTick()

    const textarea = wrapper.find('textarea')
    expect(textarea.attributes('disabled')).toBeDefined()
    expect(wrapper.text()).not.toContain('Save')
  })
})

describe('ScreenshotModal - General', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    captureActions.getScreenshot.mockResolvedValue('data:image/png;base64,mockBase64Data')
  })

  const mockRecord = {
    id: 1,
    timestamp: '2026-03-28T10:00:00Z',
    source_type: 'auto' as const,
    content: JSON.stringify({
      current_focus: 'Working on Rust code',
      active_software: 'VSCode',
      context_keywords: ['rust', 'testing'],
      active_window: {
        title: 'test.rs - VSCode',
        process_name: 'Code'
      }
    }),
    screenshot_path: '/path/to/screenshot.png',
    user_notes: null
  }

  it('emits close event when close button clicked', async () => {
    const wrapper = mount(ScreenshotModal, {
      props: { record: mockRecord },
      global: { stubs: {} }
    })
    await nextTick()

    const buttons = wrapper.findAll('button')
    const closeButton = buttons.find(b => b.text().includes('✕'))
    await closeButton.trigger('click')

    expect(wrapper.emitted('close')).toBeTruthy()
  })

  it('loads screenshot on mount', async () => {
    const wrapper = mount(ScreenshotModal, {
      props: { record: mockRecord },
      global: { stubs: {} }
    })
    await nextTick()

    expect(captureActions.getScreenshot).toHaveBeenCalledWith('/path/to/screenshot.png')
  })

  it('displays parsed content from record', async () => {
    const wrapper = mount(ScreenshotModal, {
      props: { record: mockRecord },
      global: { stubs: {} }
    })
    await nextTick()

    expect(wrapper.html()).toContain('Working on Rust code')
    expect(wrapper.html()).toContain('VSCode')
  })

  it('shows pending status for delayed-analysis records', async () => {
    const pendingRecord = {
      ...mockRecord,
      analysis_status: 'pending' as const,
      content: JSON.stringify({
        current_focus: '待分析',
        active_software: 'Code',
        context_keywords: [],
        offline_pending: true
      })
    }

    const wrapper = mount(ScreenshotModal, {
      props: { record: pendingRecord },
      global: { stubs: {} }
    })
    await nextTick()

    expect(wrapper.text()).toContain('Pending')
  })

  it('displays window icon for VSCode', async () => {
    const wrapper = mount(ScreenshotModal, {
      props: { record: mockRecord },
      global: { stubs: {} }
    })
    await nextTick()

    expect(wrapper.text()).toContain('💻')
  })

  it('closes modal when clicking outside', async () => {
    const wrapper = mount(ScreenshotModal, {
      props: { record: mockRecord },
      global: { stubs: {} }
    })
    await nextTick()

    const overlay = wrapper.find('.fixed.inset-0.bg-black\\/80')
    await overlay.trigger('click.self')

    expect(wrapper.emitted('close')).toBeTruthy()
  })

  it('formats timestamp correctly', async () => {
    const wrapper = mount(ScreenshotModal, {
      props: { record: mockRecord },
      global: { stubs: {} }
    })
    await nextTick()

    const formatted = wrapper.vm.formatTime('2026-03-28T10:00:00Z')
    expect(formatted).toMatch(/2026/)
    expect(formatted).toMatch(/28/)
  })

  it('handles non-JSON content gracefully', async () => {
    const plainContentRecord = { ...mockRecord, content: 'Plain text content' }
    const wrapper = mount(ScreenshotModal, {
      props: { record: plainContentRecord },
      global: { stubs: {} }
    })
    await nextTick()

    const parsed = wrapper.vm.parseContent('Plain text content')
    expect(parsed).toBe('Plain text content')
  })
})
