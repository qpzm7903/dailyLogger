import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount } from '@vue/test-utils'
import CaptureSettings from '../CaptureSettings.vue'

// Mock usePlatform composable
vi.mock('../../../composables/usePlatform', () => ({
  usePlatform: () => ({
    isDesktop: true
  })
}))

// Mock vue-i18n
const mockT = vi.fn((key: string) => {
  const translations: Record<string, string> = {
    'settings.timeStrategy': 'Time Strategy',
    'settings.screenshotInterval': 'Screenshot Interval (minutes)',
    'settings.summaryTime': 'Summary Time',
    'settings.smartDedup': 'Smart Deduplication',
    'settings.changeThreshold': 'Change Threshold',
    'settings.changeThresholdHint': 'Skip capture if change is below threshold',
    'settings.maxSilentTime': 'Max Silent Time',
    'settings.maxSilentTimeHint': 'Max minutes without activity before pausing',
    'settings.silentThresholdAdjust': 'Silent Threshold Adjustment',
    'settings.autoAdjustSilent': 'Auto Adjust Silent Threshold',
    'settings.autoAdjustHint': 'Automatically adjust threshold based on activity',
    'settings.workTimeDetection': 'Work Time Detection',
    'settings.autoDetectWorkTime': 'Auto Detect Work Time',
    'settings.autoDetectHint': 'Only capture during work hours',
    'settings.useCustomWorkTime': 'Use Custom Work Time',
    'settings.startTime': 'Start Time',
    'settings.endTime': 'End Time',
    'settings.windowFilter': 'Window Filter',
    'settings.windowWhitelist': 'Whitelist',
    'settings.windowBlacklist': 'Blacklist',
    'settings.whitelistPlaceholder': 'Add app name...',
    'settings.blacklistPlaceholder': 'Add app name...',
    'settings.whitelistOnly': 'Whitelist Only',
    'settings.displaySettings': 'Display Settings',
    'settings.captureMode': 'Capture Mode',
    'settings.primaryMonitor': 'Primary Monitor',
    'settings.secondaryMonitor': 'Secondary Monitor',
    'settings.allMonitors': 'All Monitors',
    'settings.connectedDisplays': 'Connected Displays',
    'settings.primary': 'Primary'
  }
  return translations[key] || key
})

vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: mockT
  })
}))

describe('CaptureSettings', () => {
  const defaultProps = {
    settings: {
      screenshot_interval: 5,
      summary_time: '18:00',
      change_threshold: 5,
      max_silent_minutes: 30,
      auto_adjust_silent: false,
      auto_detect_work_time: false,
      use_custom_work_time: false,
      custom_work_time_start: '09:00',
      custom_work_time_end: '18:00',
      use_whitelist_only: false,
      capture_mode: 'primary',
      selected_monitor_index: 0
    },
    whitelistTags: ['VSCode', 'Chrome'],
    blacklistTags: ['Spotify'],
    monitors: undefined
  }

  beforeEach(() => {
    vi.clearAllMocks()
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  // === Rendering ===
  describe('rendering', () => {
    it('renders time strategy section', () => {
      const wrapper = mount(CaptureSettings, { props: defaultProps })
      expect(wrapper.text()).toContain('Time Strategy')
    })

    it('renders smart deduplication section', () => {
      const wrapper = mount(CaptureSettings, { props: defaultProps })
      expect(wrapper.text()).toContain('Smart Deduplication')
    })

    it('renders work time detection section', () => {
      const wrapper = mount(CaptureSettings, { props: defaultProps })
      expect(wrapper.text()).toContain('Work Time')
    })

    it('renders window filter section on desktop', () => {
      const wrapper = mount(CaptureSettings, { props: defaultProps })
      expect(wrapper.text()).toContain('Window Filter')
    })

    it('renders screenshot interval input with correct value', () => {
      const wrapper = mount(CaptureSettings, { props: defaultProps })
      const inputs = wrapper.findAll('input[type="number"]')
      const intervalInput = inputs.find(i => i.attributes('min') === '1' && i.attributes('max') === '60')
      expect(intervalInput?.element.value).toBe('5')
    })

    it('renders summary time input', () => {
      const wrapper = mount(CaptureSettings, { props: defaultProps })
      const inputs = wrapper.findAll('input[type="time"]')
      expect(inputs.length).toBeGreaterThanOrEqual(1)
    })

    it('renders auto adjust silent checkbox', () => {
      const wrapper = mount(CaptureSettings, { props: defaultProps })
      const checkboxes = wrapper.findAll('input[type="checkbox"]')
      expect(checkboxes.length).toBeGreaterThan(0)
    })

    it('renders whitelist tags', () => {
      const wrapper = mount(CaptureSettings, { props: defaultProps })
      expect(wrapper.text()).toContain('VSCode')
      expect(wrapper.text()).toContain('Chrome')
    })

    it('renders blacklist tags', () => {
      const wrapper = mount(CaptureSettings, { props: defaultProps })
      expect(wrapper.text()).toContain('Spotify')
    })
  })

  // === Props and Events ===
  describe('props and events', () => {
    it('syncs local settings with props', async () => {
      const wrapper = mount(CaptureSettings, { props: defaultProps })
      const inputs = wrapper.findAll('input[type="number"]')
      const intervalInput = inputs.find(i => i.attributes('min') === '1' && i.attributes('max') === '60')
      await intervalInput?.setValue(10)

      const emitted = wrapper.emitted('update:settings')
      expect(emitted).toBeTruthy()
      expect(emitted?.[0][0].screenshot_interval).toBe(10)
    })

    it('updates local settings when props change', async () => {
      const wrapper = mount(CaptureSettings, { props: defaultProps })
      await wrapper.setProps({
        settings: {
          ...defaultProps.settings,
          screenshot_interval: 15
        },
        whitelistTags: defaultProps.whitelistTags,
        blacklistTags: defaultProps.blacklistTags
      })

      expect(wrapper.vm.localSettings.screenshot_interval).toBe(15)
    })

    it('toggles auto adjust silent checkbox', async () => {
      const wrapper = mount(CaptureSettings, { props: defaultProps })
      const checkboxes = wrapper.findAll('input[type="checkbox"]')
      const autoAdjustCheckbox = checkboxes[0]
      await autoAdjustCheckbox?.setValue(true)

      expect(wrapper.vm.localSettings.auto_adjust_silent).toBe(true)
    })

    it('toggles auto detect work time checkbox', async () => {
      const wrapper = mount(CaptureSettings, { props: defaultProps })
      const checkboxes = wrapper.findAll('input[type="checkbox"]')
      const workTimeCheckbox = checkboxes[1]
      await workTimeCheckbox?.setValue(true)

      expect(wrapper.vm.localSettings.auto_detect_work_time).toBe(true)
    })
  })

  // === Whitelist Tags ===
  describe('whitelist tags', () => {
    it('adds whitelist tag on enter', async () => {
      const wrapper = mount(CaptureSettings, { props: defaultProps })
      const inputs = wrapper.findAll('input[type="text"]')
      const whitelistInput = inputs.find(i => i.attributes('placeholder')?.includes('app name'))
      if (whitelistInput) {
        await whitelistInput.setValue('Terminal')
        await whitelistInput.trigger('keyup.enter')
      }

      const emitted = wrapper.emitted('update:whitelistTags')
      expect(emitted).toBeTruthy()
      expect(emitted?.[0][0]).toContain('Terminal')
    })

    it('removes whitelist tag when clicked', async () => {
      const wrapper = mount(CaptureSettings, { props: defaultProps })
      // Find the remove button for the first whitelist tag
      const buttons = wrapper.findAll('button')
      const removeButton = buttons.find(b => b.text() === '✕')
      await removeButton?.trigger('click')

      const emitted = wrapper.emitted('update:whitelistTags')
      expect(emitted).toBeTruthy()
      expect(emitted?.[0][0].length).toBe(1) // Removed one tag
    })

    it('does not add empty whitelist tag', async () => {
      const wrapper = mount(CaptureSettings, { props: defaultProps })
      const inputs = wrapper.findAll('input[type="text"]')
      const whitelistInput = inputs.find(i => i.attributes('placeholder')?.includes('app name'))
      if (whitelistInput) {
        await whitelistInput.setValue('   ')
        await whitelistInput.trigger('keyup.enter')
      }

      // Should not emit since tag is empty
      expect(wrapper.emitted('update:whitelistTags')).toBeFalsy()
    })
  })

  // === Blacklist Tags ===
  describe('blacklist tags', () => {
    it('adds blacklist tag on enter', async () => {
      const wrapper = mount(CaptureSettings, { props: defaultProps })
      const inputs = wrapper.findAll('input[type="text"]')
      // Find the blacklist input (second text input)
      const blacklistInput = inputs[inputs.length - 1]
      if (blacklistInput) {
        await blacklistInput.setValue('Discord')
        await blacklistInput.trigger('keyup.enter')
      }

      const emitted = wrapper.emitted('update:blacklistTags')
      expect(emitted).toBeTruthy()
      expect(emitted?.[0][0]).toContain('Discord')
    })

    it('removes blacklist tag when clicked', async () => {
      const wrapper = mount(CaptureSettings, { props: defaultProps })
      // Find the remove button for the blacklist tag
      const buttons = wrapper.findAll('button')
      const removeButtons = buttons.filter(b => b.text() === '✕')
      // Click the blacklist remove button (last one)
      await removeButtons[removeButtons.length - 1]?.trigger('click')

      const emitted = wrapper.emitted('update:blacklistTags')
      expect(emitted).toBeTruthy()
    })
  })

  // === Custom Work Time ===
  describe('custom work time', () => {
    it('shows custom work time inputs when enabled', async () => {
      const wrapper = mount(CaptureSettings, { props: defaultProps })
      const checkboxes = wrapper.findAll('input[type="checkbox"]')

      // Enable auto_detect_work_time
      await checkboxes[1]?.setValue(true)
      await wrapper.vm.$nextTick()

      // Enable use_custom_work_time
      const customWorkTimeCheckbox = wrapper.findAll('input[type="checkbox"]')[2]
      await customWorkTimeCheckbox?.setValue(true)
      await wrapper.vm.$nextTick()

      // Check that custom time inputs are rendered
      const timeInputs = wrapper.findAll('input[type="time"]')
      expect(timeInputs.length).toBeGreaterThanOrEqual(2)
    })

    it('hides custom work time inputs when disabled', () => {
      const wrapper = mount(CaptureSettings, { props: defaultProps })
      // By default, auto_detect_work_time is false, so custom inputs should not show
      const labels = wrapper.findAll('label')
      const customTimeLabel = labels.find(l => l.text().includes('Custom Work Time'))
      expect(customTimeLabel).toBeFalsy()
    })
  })

  // === Multi-monitor ===
  describe('multi-monitor', () => {
    const multiMonitorProps = {
      ...defaultProps,
      monitors: [
        { id: 1, name: 'Monitor 1', width: 1920, height: 1080, is_primary: true, index: 0, resolution: '1920x1080' },
        { id: 2, name: 'Monitor 2', width: 2560, height: 1440, is_primary: false, index: 1, resolution: '2560x1440' }
      ]
    }

    it('shows monitor list when multiple monitors', () => {
      const wrapper = mount(CaptureSettings, { props: multiMonitorProps })
      expect(wrapper.text()).toContain('Connected Displays')
      expect(wrapper.text()).toContain('Monitor 1')
      expect(wrapper.text()).toContain('Monitor 2')
    })

    it('shows capture mode radio buttons when multiple monitors', () => {
      const wrapper = mount(CaptureSettings, { props: multiMonitorProps })
      expect(wrapper.text()).toContain('Capture Mode')
      expect(wrapper.text()).toContain('Primary Monitor')
      expect(wrapper.text()).toContain('All Monitors')
    })

    it('shows primary badge on primary monitor', () => {
      const wrapper = mount(CaptureSettings, { props: multiMonitorProps })
      expect(wrapper.text()).toContain('Primary')
    })
  })
})