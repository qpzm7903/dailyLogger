import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import OutputSettings from '../OutputSettings.vue'

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

// Mock toast store
vi.mock('../../../stores/toast', () => ({
  showSuccess: vi.fn(),
  showError: vi.fn()
}))

import { invoke } from '@tauri-apps/api/core'
import { showSuccess, showError } from '../../../stores/toast'

// Mock vue-i18n
const mockT = vi.fn((key: string) => {
  const translations: Record<string, string> = {
    'settings.outputConfig': 'Output Configuration',
    'settings.obsidianVaults': 'Obsidian Vaults',
    'settings.noVaultConfigured': 'No vault configured',
    'settings.debugTools': 'Debug Tools',
    'settings.exportLogs': 'Export Logs',
    'settings.exporting': 'Exporting...',
    'common.name': 'Name',
    'common.path': 'Path',
    'common.add': 'Add',
    'common.testConnection': 'Test Connection',
    'common.testing': 'Testing...',
    'common.connected': 'Connected',
    'common.failed': 'Failed'
  }
  return translations[key] || key
})

vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: mockT
  })
}))

describe('OutputSettings', () => {
  const defaultProps = {
    settings: {
      auto_detect_vault_by_window: false
    },
    vaults: [
      { name: 'Personal', path: '/Users/test/Documents/Personal', is_default: true }
    ]
  }

  beforeEach(() => {
    vi.clearAllMocks()
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  // === Rendering ===
  describe('rendering', () => {
    it('renders output configuration section', () => {
      const wrapper = mount(OutputSettings, { props: defaultProps })
      expect(wrapper.text()).toContain('Output Configuration')
    })

    it('renders Obsidian vaults section', () => {
      const wrapper = mount(OutputSettings, { props: defaultProps })
      expect(wrapper.text()).toContain('Obsidian Vaults')
    })

    it('renders debug tools section', () => {
      const wrapper = mount(OutputSettings, { props: defaultProps })
      expect(wrapper.text()).toContain('Debug Tools')
    })

    it('renders existing vault', () => {
      const wrapper = mount(OutputSettings, { props: defaultProps })
      expect(wrapper.text()).toContain('Personal')
      expect(wrapper.text()).toContain('/Users/test/Documents/Personal')
    })

    it('renders add vault form', () => {
      const wrapper = mount(OutputSettings, { props: defaultProps })
      const inputs = wrapper.findAll('input')
      const addButtons = wrapper.findAll('button').filter(b => b.text() === 'Add')
      expect(addButtons.length).toBeGreaterThan(0)
    })
  })

  // === Vault Management ===
  describe('vault management', () => {
    it('adds new vault', async () => {
      const wrapper = mount(OutputSettings, { props: defaultProps })

      const inputs = wrapper.findAll('input[type="text"]')
      const nameInput = inputs.find(i => i.attributes('placeholder') === 'Name')
      const pathInput = inputs.find(i => i.attributes('placeholder') === 'Path')

      if (nameInput && pathInput) {
        await nameInput.setValue('Work')
        await pathInput.setValue('/Users/test/Documents/Work')

        const addButtons = wrapper.findAll('button').filter(b => b.text() === 'Add')
        await addButtons[0]?.trigger('click')
      }

      const emitted = wrapper.emitted('update:vaults')
      expect(emitted).toBeTruthy()
      expect(emitted?.[0][0].length).toBe(2)
    })

    it('removes vault when clicked', async () => {
      const wrapper = mount(OutputSettings, { props: defaultProps })
      const removeButtons = wrapper.findAll('button').filter(b => b.text() === '✕')
      await removeButtons[0]?.trigger('click')

      const emitted = wrapper.emitted('update:vaults')
      expect(emitted).toBeTruthy()
      expect(emitted?.[0][0].length).toBe(0)
    })

    it('sets default vault when star clicked', async () => {
      const wrapper = mount(OutputSettings, {
        props: {
          ...defaultProps,
          vaults: [
            { name: 'Personal', path: '/path1', is_default: true },
            { name: 'Work', path: '/path2', is_default: false }
          ]
        }
      })

      const starButtons = wrapper.findAll('button').filter(b => b.text().includes('☆'))
      await starButtons[0]?.trigger('click')

      const emitted = wrapper.emitted('update:vaults')
      expect(emitted).toBeTruthy()
    })

    it('disables add button when inputs are empty', () => {
      const wrapper = mount(OutputSettings, { props: defaultProps })
      const addButtons = wrapper.findAll('button').filter(b => b.text() === 'Add')
      expect(addButtons[0]?.element.disabled).toBe(true)
    })
  })

  // === Export Logs ===
  describe('export logs', () => {
    it('exports logs when button clicked', async () => {
      vi.mocked(invoke).mockResolvedValueOnce(undefined)

      const wrapper = mount(OutputSettings, { props: defaultProps })
      const exportButton = wrapper.findAll('button').find(b => b.text().includes('Export Logs'))
      await exportButton?.trigger('click')
      await flushPromises()

      expect(invoke).toHaveBeenCalledWith('export_logs')
    })

    it('shows error when export fails', async () => {
      vi.mocked(invoke).mockRejectedValueOnce(new Error('Export failed'))

      const wrapper = mount(OutputSettings, { props: defaultProps })
      const exportButton = wrapper.findAll('button').find(b => b.text().includes('Export Logs'))
      await exportButton?.trigger('click')
      await flushPromises()

      expect(showError).toHaveBeenCalled()
    })
  })

  // === Props and Events ===
  describe('props and events', () => {
    it('updates local settings when props change', async () => {
      const wrapper = mount(OutputSettings, { props: defaultProps })
      await wrapper.setProps({
        ...defaultProps,
        settings: {
          ...defaultProps.settings,
          auto_detect_vault_by_window: true
        }
      })

      expect((wrapper.vm as any).localSettings.auto_detect_vault_by_window).toBe(true)
    })

    it('updates local vaults when props change', async () => {
      const wrapper = mount(OutputSettings, { props: defaultProps })
      await wrapper.setProps({
        ...defaultProps,
        vaults: [
          { name: 'NewVault', path: '/new/path', is_default: true }
        ]
      })

      expect((wrapper.vm as any).localVaults.length).toBe(1)
      expect((wrapper.vm as any).localVaults[0].name).toBe('NewVault')
    })

  })
})
