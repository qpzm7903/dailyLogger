import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import Dashboard from '../components/layout/Dashboard.vue'

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

// Mock vue-i18n with a factory function
vi.mock('vue-i18n', () => {
  return {
    useI18n: () => {
      return {
        t: (key: string, params?: Record<string, unknown>) => {
          const translations: Record<string, string> = {
            'autoCapture.title': 'Auto Capture',
            'autoCapture.description': 'Automatic screen capture',
            'autoCapture.running': 'Running',
            'autoCapture.stopped': 'Stopped',
            'autoCapture.screenshot': 'Screenshot',
            'autoCapture.screenshotting': 'Screenshotting...',
            'autoCapture.analyze': 'Analyze',
            'autoCapture.start': 'Start',
            'autoCapture.stop': 'Stop',
            'quickNote.title': 'Quick Note',
            'quickNote.shortcut': 'Alt+Space',
            'quickNote.todayRecords': 'Today: {count}',
            'quickNote.record': 'Record',
            'reportHistory.title': 'Report History'
          }
          let result = translations[key] || key
          if (params) {
            Object.entries(params).forEach(([k, v]) => {
              result = result.replace(`{${k}}`, String(v))
            })
          }
          return result
        }
      }
    }
  }
})

// Helper to wait for async updates
const waitFor = async (condition: () => boolean, timeout = 1000) => {
  const start = Date.now()
  while (!condition() && Date.now() - start < timeout) {
    await new Promise(resolve => setTimeout(resolve, 50))
  }
}

// Mock data factory
const createMockRecord = (overrides: Partial<{
  id: number
  timestamp: string
  source_type: string
  content: string
  screenshot_path: string | null
  tags: string
}> = {}) => ({
  id: Math.random(),
  timestamp: new Date().toISOString(),
  source_type: 'auto',
  content: JSON.stringify({ current_focus: 'Work' }),
  screenshot_path: null,
  ...overrides
})

describe('Dashboard.vue - Window Info Display', () => {
  const defaultProps = {
    isDesktop: true,
    autoCaptureEnabled: false,
    isCapturing: false,
    quickNotesCount: 0,
    todayRecords: [] as Array<{
      id: number
      timestamp: string
      source_type: string
      content: string
      screenshot_path: string | null
      tags?: string
    }>,
    isGenerating: false,
    isGeneratingWeekly: false,
    isGeneratingMonthly: false,
    screenshotCount: 0,
    summaryPath: '',
    weeklyReportPath: '',
    monthlyReportPath: '',
    customReportPath: '',
    comparisonReportPath: ''
  }

  it('shows window title when record has active_window info', async () => {
    const recordsWithWindow = [
      createMockRecord({
        source_type: 'auto',
        screenshot_path: '/path/screenshot1.png',
        content: JSON.stringify({
          current_focus: 'Working on code',
          active_software: 'VS Code',
          active_window: {
            title: 'main.rs - DailyLogger - VS Code',
            process_name: 'Code'
          }
        })
      })
    ]

    const wrapper = mount(Dashboard, {
      props: { ...defaultProps, todayRecords: recordsWithWindow }
    })

    await nextTick()
    const html = wrapper.html()
    expect(html).toContain('main.rs - DailyLogger - VS Code')
  })

  it('does not show window info section when active_window is missing', async () => {
    const recordsWithoutWindow = [
      createMockRecord({
        source_type: 'auto',
        screenshot_path: '/path/screenshot1.png',
        content: JSON.stringify({
          current_focus: 'Working on code',
          active_software: 'VS Code'
        })
      })
    ]

    const wrapper = mount(Dashboard, {
      props: { ...defaultProps, todayRecords: recordsWithoutWindow }
    })

    await nextTick()
    const windowInfoSection = wrapper.find('.window-info')
    expect(windowInfoSection.exists()).toBe(false)
  })

  it('handles manual records without window info gracefully', async () => {
    const manualRecord = [
      createMockRecord({
        source_type: 'manual',
        screenshot_path: null,
        content: 'Quick note about my task'
      })
    ]

    const wrapper = mount(Dashboard, {
      props: { ...defaultProps, todayRecords: manualRecord }
    })

    await nextTick()
    expect(wrapper.text()).toContain('Quick note about my task')
  })
})

describe('Dashboard.vue - Tag Functionality', () => {
  const defaultProps = {
    isDesktop: true,
    autoCaptureEnabled: false,
    isCapturing: false,
    quickNotesCount: 0,
    todayRecords: [] as Array<{
      id: number
      timestamp: string
      source_type: string
      content: string
      screenshot_path: string | null
      tags?: string
    }>,
    isGenerating: false,
    isGeneratingWeekly: false,
    isGeneratingMonthly: false,
    screenshotCount: 0,
    summaryPath: '',
    weeklyReportPath: '',
    monthlyReportPath: '',
    customReportPath: '',
    comparisonReportPath: ''
  }

  describe('Tag badges display', () => {
    it('shows tag badges when record has tags field', async () => {
      const recordsWithTags = [
        createMockRecord({
          source_type: 'auto',
          screenshot_path: '/path/screenshot1.png',
          content: JSON.stringify({ current_focus: 'Working on code' }),
          tags: JSON.stringify(['开发', '测试'])
        })
      ]

      const wrapper = mount(Dashboard, {
        props: { ...defaultProps, todayRecords: recordsWithTags }
      })

      await nextTick()
      const html = wrapper.html()
      expect(html).toContain('开发')
      expect(html).toContain('测试')
    })

    it('shows tags from content field for auto records without tags field', async () => {
      const recordsWithContentTags = [
        createMockRecord({
          source_type: 'auto',
          screenshot_path: '/path/screenshot1.png',
          content: JSON.stringify({
            current_focus: 'In a meeting',
            active_software: 'Zoom',
            tags: ['会议', '沟通']
          }),
          tags: undefined
        })
      ]

      const wrapper = mount(Dashboard, {
        props: { ...defaultProps, todayRecords: recordsWithContentTags }
      })

      await nextTick()
      const html = wrapper.html()
      expect(html).toContain('会议')
      expect(html).toContain('沟通')
    })

    it('limits display to 3 tags per record', async () => {
      const recordsWithManyTags = [
        createMockRecord({
          source_type: 'auto',
          screenshot_path: '/path/screenshot1.png',
          content: JSON.stringify({ current_focus: 'Work' }),
          tags: JSON.stringify(['开发', '测试', '文档', '会议', '设计'])
        })
      ]

      const wrapper = mount(Dashboard, {
        props: { ...defaultProps, todayRecords: recordsWithManyTags }
      })

      await nextTick()
      const tags = wrapper.vm.getRecordTags(wrapper.vm.todayRecords[0])
      expect(tags.length).toBeLessThanOrEqual(3)
    })

    it('does not show tag section when record has no tags', async () => {
      const recordsWithoutTags = [
        createMockRecord({
          source_type: 'manual',
          screenshot_path: null,
          content: 'Quick note',
          tags: undefined
        })
      ]

      const wrapper = mount(Dashboard, {
        props: { ...defaultProps, todayRecords: recordsWithoutTags }
      })

      await nextTick()
      const tags = wrapper.vm.getRecordTags(wrapper.vm.todayRecords[0])
      expect(tags).toEqual([])
    })
  })

  describe('Tag filtering functionality', () => {
    it('shows tag filter when records have tags', async () => {
      const recordsWithTags = [
        createMockRecord({
          content: JSON.stringify({ current_focus: 'Work' }),
          tags: JSON.stringify(['开发'])
        }),
        createMockRecord({
          content: JSON.stringify({ current_focus: 'Meeting' }),
          tags: JSON.stringify(['会议'])
        })
      ]

      const wrapper = mount(Dashboard, {
        props: { ...defaultProps, todayRecords: recordsWithTags }
      })

      await nextTick()
      const html = wrapper.html()
      expect(html).toContain('全部')
    })

    it('calculates correct tag counts', async () => {
      const recordsWithTags = [
        createMockRecord({
          content: JSON.stringify({ current_focus: 'Dev work' }),
          tags: JSON.stringify(['开发'])
        }),
        createMockRecord({
          content: JSON.stringify({ current_focus: 'Dev work 2' }),
          tags: JSON.stringify(['开发', '测试'])
        }),
        createMockRecord({
          content: JSON.stringify({ current_focus: 'Meeting' }),
          tags: JSON.stringify(['会议'])
        })
      ]

      const wrapper = mount(Dashboard, {
        props: { ...defaultProps, todayRecords: recordsWithTags }
      })

      await nextTick()
      const counts = wrapper.vm.tagCounts
      expect(counts['开发']).toBe(2)
      expect(counts['测试']).toBe(1)
      expect(counts['会议']).toBe(1)
    })

    it('filters records by selected tag', async () => {
      const recordsWithTags = [
        createMockRecord({
          id: 1,
          content: JSON.stringify({ current_focus: 'Dev work' }),
          tags: JSON.stringify(['开发'])
        }),
        createMockRecord({
          id: 2,
          content: JSON.stringify({ current_focus: 'Meeting' }),
          tags: JSON.stringify(['会议'])
        })
      ]

      const wrapper = mount(Dashboard, {
        props: { ...defaultProps, todayRecords: recordsWithTags }
      })

      await nextTick()

      expect(wrapper.vm.filteredRecords.length).toBe(2)

      wrapper.vm.selectedTagFilter = '开发'
      await nextTick()

      expect(wrapper.vm.filteredRecords.length).toBe(1)
      expect(wrapper.vm.filteredRecords[0].id).toBe(1)
    })

    it('clears filter when selecting "全部"', async () => {
      const recordsWithTags = [
        createMockRecord({
          id: 1,
          content: JSON.stringify({ current_focus: 'Dev work' }),
          tags: JSON.stringify(['开发'])
        }),
        createMockRecord({
          id: 2,
          content: JSON.stringify({ current_focus: 'Meeting' }),
          tags: JSON.stringify(['会议'])
        })
      ]

      const wrapper = mount(Dashboard, {
        props: { ...defaultProps, todayRecords: recordsWithTags }
      })

      await nextTick()

      wrapper.vm.selectedTagFilter = '开发'
      await nextTick()
      expect(wrapper.vm.filteredRecords.length).toBe(1)

      wrapper.vm.selectedTagFilter = ''
      await nextTick()
      expect(wrapper.vm.filteredRecords.length).toBe(2)
    })
  })
})

describe('Dashboard.vue - Auto Capture Card', () => {
  const defaultProps = {
    isDesktop: true,
    autoCaptureEnabled: false,
    isCapturing: false,
    quickNotesCount: 0,
    todayRecords: [],
    isGenerating: false,
    isGeneratingWeekly: false,
    isGeneratingMonthly: false,
    screenshotCount: 0,
    summaryPath: '',
    weeklyReportPath: '',
    monthlyReportPath: '',
    customReportPath: '',
    comparisonReportPath: ''
  }

  it('shows auto capture card on desktop', async () => {
    const wrapper = mount(Dashboard, {
      props: { ...defaultProps, isDesktop: true }
    })
    await nextTick()

    const html = wrapper.html()
    expect(html).toContain('Auto Capture')
  })

  it('hides auto capture card on mobile', async () => {
    const wrapper = mount(Dashboard, {
      props: { ...defaultProps, isDesktop: false }
    })
    await nextTick()

    const autoCaptureSection = wrapper.find('.grid-cols-2')
    expect(autoCaptureSection.exists()).toBe(false)
  })

  it('shows running status when autoCaptureEnabled is true', async () => {
    const wrapper = mount(Dashboard, {
      props: { ...defaultProps, autoCaptureEnabled: true }
    })
    await nextTick()

    const html = wrapper.html()
    expect(html).toContain('Running')
  })

  it('shows stopped status when autoCaptureEnabled is false', async () => {
    const wrapper = mount(Dashboard, {
      props: { ...defaultProps, autoCaptureEnabled: false }
    })
    await nextTick()

    const html = wrapper.html()
    expect(html).toContain('Stopped')
  })
})

describe('Dashboard.vue - Output Files Card', () => {
  const defaultProps = {
    isDesktop: true,
    autoCaptureEnabled: false,
    isCapturing: false,
    quickNotesCount: 0,
    todayRecords: [],
    isGenerating: false,
    isGeneratingWeekly: false,
    isGeneratingMonthly: false,
    screenshotCount: 0,
    summaryPath: '',
    weeklyReportPath: '',
    monthlyReportPath: '',
    customReportPath: '',
    comparisonReportPath: ''
  }

  it('shows daily report path when summaryPath is provided', async () => {
    const wrapper = mount(Dashboard, {
      props: { ...defaultProps, summaryPath: '/path/to/daily-report.md' }
    })
    await nextTick()

    const html = wrapper.html()
    expect(html).toContain('/path/to/daily-report.md')
    expect(html).toContain('日报')
  })

  it('shows weekly report path when weeklyReportPath is provided', async () => {
    const wrapper = mount(Dashboard, {
      props: { ...defaultProps, weeklyReportPath: '/path/to/weekly-report.md' }
    })
    await nextTick()

    const html = wrapper.html()
    expect(html).toContain('/path/to/weekly-report.md')
    expect(html).toContain('周报')
  })

  it('shows monthly report path when monthlyReportPath is provided', async () => {
    const wrapper = mount(Dashboard, {
      props: { ...defaultProps, monthlyReportPath: '/path/to/monthly-report.md' }
    })
    await nextTick()

    const html = wrapper.html()
    expect(html).toContain('/path/to/monthly-report.md')
    expect(html).toContain('月报')
  })

  it('shows placeholder when no daily report generated', async () => {
    const wrapper = mount(Dashboard, {
      props: { ...defaultProps, summaryPath: '' }
    })
    await nextTick()

    const html = wrapper.html()
    expect(html).toContain('尚未生成日报')
  })
})