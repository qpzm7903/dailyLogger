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
    'settings.logseqGraphs': 'Logseq Graphs',
    'settings.noGraphConfigured': 'No graph configured',
    'settings.notionIntegration': 'Notion Integration',
    'settings.notionApiKey': 'Notion API Key',
    'settings.notionApiKeyPlaceholder': 'secret_...',
    'settings.notionDatabaseId': 'Database ID',
    'settings.notionDatabaseIdPlaceholder': 'xxx...',
    'settings.notionHint': 'Configure Notion to export reports',
    'settings.githubWorkTime': 'GitHub Work Time Statistics',
    'settings.githubToken': 'GitHub Token',
    'settings.githubTokenPlaceholder': 'ghp_...',
    'settings.githubRepos': 'Repositories',
    'settings.githubHint': 'Track GitHub activity',
    'settings.slackNotification': 'Slack Notification',
    'settings.slackWebhookUrl': 'Slack Webhook URL',
    'settings.slackWebhookPlaceholder': 'https://hooks.slack.com/...',
    'settings.slackHint': 'Send reports to Slack',
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
      notion_api_key: null,
      notion_database_id: null,
      github_token: null,
      github_repositories: '',
      slack_webhook_url: null
    },
    vaults: [
      { name: 'Personal', path: '/Users/test/Documents/Personal', is_default: true }
    ],
    graphs: []
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

    it('renders Logseq graphs section', () => {
      const wrapper = mount(OutputSettings, { props: defaultProps })
      expect(wrapper.text()).toContain('Logseq Graphs')
    })

    it('renders Notion integration section', () => {
      const wrapper = mount(OutputSettings, { props: defaultProps })
      expect(wrapper.text()).toContain('Notion Integration')
    })

    it('renders GitHub work time section', () => {
      const wrapper = mount(OutputSettings, { props: defaultProps })
      expect(wrapper.text()).toContain('GitHub Work Time')
    })

    it('renders Slack notification section', () => {
      const wrapper = mount(OutputSettings, { props: defaultProps })
      expect(wrapper.text()).toContain('Slack Notification')
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

    it('renders no graph configured message when empty', () => {
      const wrapper = mount(OutputSettings, { props: defaultProps })
      expect(wrapper.text()).toContain('No graph configured')
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

  // === Graph Management ===
  describe('graph management', () => {
    it('adds new graph', async () => {
      const wrapper = mount(OutputSettings, { props: defaultProps })

      const inputs = wrapper.findAll('input[type="text"]')
      // Find graph inputs (second set of name/path inputs)
      const nameInputs = inputs.filter(i => i.attributes('placeholder') === 'Name')
      const pathInputs = inputs.filter(i => i.attributes('placeholder') === 'Path')

      if (nameInputs.length > 1 && pathInputs.length > 1) {
        await nameInputs[1].setValue('MyGraph')
        await pathInputs[1].setValue('/Users/test/Logseq/MyGraph')

        const addButtons = wrapper.findAll('button').filter(b => b.text() === 'Add')
        await addButtons[1]?.trigger('click')
      }

      const emitted = wrapper.emitted('update:graphs')
      expect(emitted).toBeTruthy()
    })

    it('removes graph when clicked', async () => {
      const wrapper = mount(OutputSettings, {
        props: {
          ...defaultProps,
          graphs: [{ name: 'MyGraph', path: '/path' }]
        }
      })

      const removeButtons = wrapper.findAll('button').filter(b => b.text() === '✕')
      // The last remove button should be for the graph
      await removeButtons[removeButtons.length - 1]?.trigger('click')

      const emitted = wrapper.emitted('update:graphs')
      expect(emitted).toBeTruthy()
    })
  })

  // === Notion Integration ===
  describe('notion integration', () => {
    it('updates notion api key', async () => {
      const wrapper = mount(OutputSettings, { props: defaultProps })
      const inputs = wrapper.findAll('input[type="password"]')
      const notionKeyInput = inputs.find(i => i.attributes('placeholder')?.includes('secret'))
      if (notionKeyInput) {
        await notionKeyInput.setValue('secret_test_key')
      }

      const emitted = wrapper.emitted('update:settings')
      expect(emitted).toBeTruthy()
    })

    it('tests notion connection', async () => {
      vi.mocked(invoke).mockResolvedValueOnce({ success: true, message: 'OK' })

      const wrapper = mount(OutputSettings, {
        props: {
          ...defaultProps,
          settings: {
            ...defaultProps.settings,
            notion_api_key: 'secret_test',
            notion_database_id: 'db123'
          }
        }
      })

      const testButtons = wrapper.findAll('button').filter(b => b.text().includes('Test Connection'))
      await testButtons[0]?.trigger('click')
      await flushPromises()

      expect(invoke).toHaveBeenCalledWith('test_notion_connection', expect.objectContaining({
        apiKey: 'secret_test',
        databaseId: 'db123'
      }))
    })
  })

  // === GitHub Integration ===
  describe('github integration', () => {
    it('updates github token', async () => {
      const wrapper = mount(OutputSettings, { props: defaultProps })
      const inputs = wrapper.findAll('input[type="password"]')
      const githubTokenInput = inputs.find(i => i.attributes('placeholder')?.includes('ghp'))
      if (githubTokenInput) {
        await githubTokenInput.setValue('ghp_test_token')
      }

      const emitted = wrapper.emitted('update:settings')
      expect(emitted).toBeTruthy()
    })

    it('tests github connection', async () => {
      vi.mocked(invoke).mockResolvedValueOnce({ success: true, message: 'OK' })

      const wrapper = mount(OutputSettings, {
        props: {
          ...defaultProps,
          settings: {
            ...defaultProps.settings,
            github_token: 'ghp_test'
          }
        }
      })

      const testButtons = wrapper.findAll('button').filter(b => b.text().includes('Test Connection'))
      await testButtons[1]?.trigger('click')
      await flushPromises()

      expect(invoke).toHaveBeenCalledWith('test_github_connection', expect.objectContaining({
        token: 'ghp_test'
      }))
    })
  })

  // === Slack Integration ===
  describe('slack integration', () => {
    it('updates slack webhook url', async () => {
      const wrapper = mount(OutputSettings, { props: defaultProps })
      const inputs = wrapper.findAll('input[type="password"]')
      const slackInput = inputs.find(i => i.attributes('placeholder')?.includes('hooks.slack.com'))
      if (slackInput) {
        await slackInput.setValue('https://hooks.slack.com/test')
      }

      const emitted = wrapper.emitted('update:settings')
      expect(emitted).toBeTruthy()
    })

    it('tests slack connection', async () => {
      vi.mocked(invoke).mockResolvedValueOnce({ success: true, message: 'OK' })

      const wrapper = mount(OutputSettings, {
        props: {
          ...defaultProps,
          settings: {
            ...defaultProps.settings,
            slack_webhook_url: 'https://hooks.slack.com/test'
          }
        }
      })

      const testButtons = wrapper.findAll('button').filter(b => b.text().includes('Test Connection'))
      await testButtons[2]?.trigger('click')
      await flushPromises()

      expect(invoke).toHaveBeenCalledWith('test_slack_webhook', expect.objectContaining({
        webhookUrl: 'https://hooks.slack.com/test'
      }))
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
          notion_api_key: 'new_key'
        }
      })

      expect(wrapper.vm.localSettings.notion_api_key).toBe('new_key')
    })

    it('updates local vaults when props change', async () => {
      const wrapper = mount(OutputSettings, { props: defaultProps })
      await wrapper.setProps({
        ...defaultProps,
        vaults: [
          { name: 'NewVault', path: '/new/path', is_default: true }
        ]
      })

      expect(wrapper.vm.localVaults.length).toBe(1)
      expect(wrapper.vm.localVaults[0].name).toBe('NewVault')
    })

    it('updates local graphs when props change', async () => {
      const wrapper = mount(OutputSettings, { props: defaultProps })
      await wrapper.setProps({
        ...defaultProps,
        graphs: [
          { name: 'NewGraph', path: '/new/graph' }
        ]
      })

      expect(wrapper.vm.localGraphs.length).toBe(1)
      expect(wrapper.vm.localGraphs[0].name).toBe('NewGraph')
    })
  })
})