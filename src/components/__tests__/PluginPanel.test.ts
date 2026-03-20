import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import PluginPanel from '../PluginPanel.vue'
import { createI18n } from 'vue-i18n'
import en from '../../locales/en.json'

// Create i18n instance for testing
const createTestI18n = () => {
  return createI18n({
    legacy: false,
    locale: 'en',
    messages: {
      en
    }
  })
}

// Mock Tauri APIs
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

interface Plugin {
  id: string
  name: string
  description: string
  version: string
  author: string
  enabled: boolean
  status: 'ready' | 'disabled' | 'error'
}

const mockPlugins: Plugin[] = [
  {
    id: 'test-plugin-1',
    name: 'Test Plugin 1',
    description: 'A test plugin for testing',
    version: '1.0.0',
    author: 'Test Author',
    enabled: true,
    status: 'ready'
  },
  {
    id: 'test-plugin-2',
    name: 'Test Plugin 2',
    description: 'Another test plugin',
    version: '2.0.0',
    author: 'Another Author',
    enabled: false,
    status: 'disabled'
  },
  {
    id: 'test-plugin-3',
    name: 'Error Plugin',
    description: 'A plugin with errors',
    version: '0.1.0',
    author: 'Error Author',
    enabled: false,
    status: 'error'
  }
]

describe('PluginPanel.vue', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('initial render', () => {
    it('renders the title correctly', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      invoke.mockResolvedValue([])

      const i18n = createTestI18n()
      const wrapper = mount(PluginPanel, {
        global: {
          plugins: [i18n]
        }
      })

      await nextTick()
      expect(wrapper.find('h3').text()).toBe('Plugins')
    })

    it('renders the open directory button', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      invoke.mockResolvedValue([])

      const i18n = createTestI18n()
      const wrapper = mount(PluginPanel, {
        global: {
          plugins: [i18n]
        }
      })

      await nextTick()
      const button = wrapper.find('button')
      expect(button.text()).toBe('Open Plugins Folder')
    })
  })

  describe('loading state', () => {
    it('shows loading message initially', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      // Don't resolve immediately to test loading state
      invoke.mockImplementation(() => new Promise(() => {}))

      const i18n = createTestI18n()
      const wrapper = mount(PluginPanel, {
        global: {
          plugins: [i18n]
        }
      })

      await nextTick()
      expect(wrapper.text()).toContain('Loading plugins...')
    })
  })

  describe('empty state', () => {
    it('shows no plugins message when list is empty', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      invoke.mockResolvedValue([])

      const i18n = createTestI18n()
      const wrapper = mount(PluginPanel, {
        global: {
          plugins: [i18n]
        }
      })

      // Wait for loading to complete
      await new Promise(resolve => setTimeout(resolve, 10))
      await nextTick()

      expect(wrapper.text()).toContain('No plugins found')
    })
  })

  describe('plugin list', () => {
    it('displays plugins when loaded', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      invoke.mockResolvedValue(mockPlugins)

      const i18n = createTestI18n()
      const wrapper = mount(PluginPanel, {
        global: {
          plugins: [i18n]
        }
      })

      // Wait for loading to complete
      await new Promise(resolve => setTimeout(resolve, 10))
      await nextTick()

      expect(wrapper.text()).toContain('Test Plugin 1')
      expect(wrapper.text()).toContain('Test Plugin 2')
      expect(wrapper.text()).toContain('Error Plugin')
    })

    it('displays plugin details correctly', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      invoke.mockResolvedValue([mockPlugins[0]])

      const i18n = createTestI18n()
      const wrapper = mount(PluginPanel, {
        global: {
          plugins: [i18n]
        }
      })

      await new Promise(resolve => setTimeout(resolve, 10))
      await nextTick()

      expect(wrapper.text()).toContain('Test Plugin 1')
      expect(wrapper.text()).toContain('A test plugin for testing')
      expect(wrapper.text()).toContain('Version: 1.0.0')
      expect(wrapper.text()).toContain('Author: Test Author')
    })
  })

  describe('plugin status display', () => {
    it('shows enabled status for enabled plugin', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      invoke.mockResolvedValue([mockPlugins[0]])

      const i18n = createTestI18n()
      const wrapper = mount(PluginPanel, {
        global: {
          plugins: [i18n]
        }
      })

      await new Promise(resolve => setTimeout(resolve, 10))
      await nextTick()

      expect(wrapper.text()).toContain('Enabled')
    })

    it('shows disabled status for disabled plugin', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      invoke.mockResolvedValue([mockPlugins[1]])

      const i18n = createTestI18n()
      const wrapper = mount(PluginPanel, {
        global: {
          plugins: [i18n]
        }
      })

      await new Promise(resolve => setTimeout(resolve, 10))
      await nextTick()

      expect(wrapper.text()).toContain('Disabled')
    })

    it('shows error status for plugin with error', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      invoke.mockResolvedValue([mockPlugins[2]])

      const i18n = createTestI18n()
      const wrapper = mount(PluginPanel, {
        global: {
          plugins: [i18n]
        }
      })

      await new Promise(resolve => setTimeout(resolve, 10))
      await nextTick()

      expect(wrapper.text()).toContain('Error')
    })
  })

  describe('toggle button', () => {
    it('shows disable button for enabled plugin', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      invoke.mockResolvedValue([mockPlugins[0]])

      const i18n = createTestI18n()
      const wrapper = mount(PluginPanel, {
        global: {
          plugins: [i18n]
        }
      })

      await new Promise(resolve => setTimeout(resolve, 10))
      await nextTick()

      const toggleButtons = wrapper.findAll('button').filter(b =>
        b.text() === 'Disable' || b.text() === 'Enable'
      )
      expect(toggleButtons.length).toBeGreaterThan(0)
      expect(toggleButtons[0].text()).toBe('Disable')
    })

    it('shows enable button for disabled plugin', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      invoke.mockResolvedValue([mockPlugins[1]])

      const i18n = createTestI18n()
      const wrapper = mount(PluginPanel, {
        global: {
          plugins: [i18n]
        }
      })

      await new Promise(resolve => setTimeout(resolve, 10))
      await nextTick()

      const toggleButtons = wrapper.findAll('button').filter(b =>
        b.text() === 'Disable' || b.text() === 'Enable'
      )
      expect(toggleButtons.length).toBeGreaterThan(0)
      expect(toggleButtons[0].text()).toBe('Enable')
    })

    it('disables toggle button while toggling', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      invoke.mockResolvedValue([mockPlugins[0]])

      const i18n = createTestI18n()
      const wrapper = mount(PluginPanel, {
        global: {
          plugins: [i18n]
        }
      })

      await new Promise(resolve => setTimeout(resolve, 10))
      await nextTick()

      // Start toggle (don't await)
      const toggleButton = wrapper.findAll('button').find(b =>
        b.text() === 'Disable' || b.text() === 'Enable'
      )

      // Simulate toggling state
      ;(wrapper.vm as unknown as { toggling: string | null }).toggling = 'test-plugin-1'
      await nextTick()

      expect(toggleButton?.attributes('disabled')).toBeDefined()
    })

    it('shows processing text while toggling', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      invoke.mockResolvedValue([mockPlugins[0]])

      const i18n = createTestI18n()
      const wrapper = mount(PluginPanel, {
        global: {
          plugins: [i18n]
        }
      })

      await new Promise(resolve => setTimeout(resolve, 10))
      await nextTick()

      // Simulate toggling state
      ;(wrapper.vm as unknown as { toggling: string | null }).toggling = 'test-plugin-1'
      await nextTick()

      const toggleButton = wrapper.findAll('button').find(b =>
        b.text() === 'Processing...'
      )
      expect(toggleButton?.exists()).toBe(true)
    })
  })

  describe('toggle functionality', () => {
    it('calls disable_plugin when disabling an enabled plugin', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      invoke.mockResolvedValueOnce([mockPlugins[0]])
      invoke.mockResolvedValueOnce(undefined) // disable_plugin
      invoke.mockResolvedValueOnce([mockPlugins[0]]) // reload

      const i18n = createTestI18n()
      const wrapper = mount(PluginPanel, {
        global: {
          plugins: [i18n]
        }
      })

      await new Promise(resolve => setTimeout(resolve, 10))
      await nextTick()

      const toggleButton = wrapper.findAll('button').find(b => b.text() === 'Disable')
      await toggleButton?.trigger('click')
      await new Promise(resolve => setTimeout(resolve, 10))

      expect(invoke).toHaveBeenCalledWith('disable_plugin', { pluginId: 'test-plugin-1' })
    })

    it('calls enable_plugin when enabling a disabled plugin', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      invoke.mockResolvedValueOnce([mockPlugins[1]])
      invoke.mockResolvedValueOnce(undefined) // enable_plugin
      invoke.mockResolvedValueOnce([mockPlugins[1]]) // reload

      const i18n = createTestI18n()
      const wrapper = mount(PluginPanel, {
        global: {
          plugins: [i18n]
        }
      })

      await new Promise(resolve => setTimeout(resolve, 10))
      await nextTick()

      const toggleButton = wrapper.findAll('button').find(b => b.text() === 'Enable')
      await toggleButton?.trigger('click')
      await new Promise(resolve => setTimeout(resolve, 10))

      expect(invoke).toHaveBeenCalledWith('enable_plugin', { pluginId: 'test-plugin-2' })
    })
  })

  describe('open plugins directory', () => {
    it('calls open_plugins_directory when button clicked', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      invoke.mockResolvedValue([])

      const i18n = createTestI18n()
      const wrapper = mount(PluginPanel, {
        global: {
          plugins: [i18n]
        }
      })

      await new Promise(resolve => setTimeout(resolve, 10))
      await nextTick()

      const openButton = wrapper.findAll('button').find(b => b.text() === 'Open Plugins Folder')
      await openButton?.trigger('click')

      expect(invoke).toHaveBeenCalledWith('open_plugins_directory')
    })
  })

  describe('error handling', () => {
    it('handles load plugins error gracefully', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      invoke.mockRejectedValue(new Error('Failed to load plugins'))

      const i18n = createTestI18n()
      const wrapper = mount(PluginPanel, {
        global: {
          plugins: [i18n]
        }
      })

      await new Promise(resolve => setTimeout(resolve, 10))
      await nextTick()

      // Should show empty state
      expect(wrapper.text()).toContain('No plugins found')
    })

    it('handles toggle plugin error gracefully', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      invoke.mockResolvedValueOnce([mockPlugins[0]])
      invoke.mockRejectedValueOnce(new Error('Failed to toggle'))

      const i18n = createTestI18n()
      const wrapper = mount(PluginPanel, {
        global: {
          plugins: [i18n]
        }
      })

      await new Promise(resolve => setTimeout(resolve, 10))
      await nextTick()

      const toggleButton = wrapper.findAll('button').find(b => b.text() === 'Disable')
      await toggleButton?.trigger('click')
      await new Promise(resolve => setTimeout(resolve, 10))

      // Should not crash and toggling should be cleared
      expect((wrapper.vm as unknown as { toggling: string | null }).toggling).toBeNull()
    })
  })

  describe('styling', () => {
    it('applies correct status badge classes for enabled plugin', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      invoke.mockResolvedValue([mockPlugins[0]])

      const i18n = createTestI18n()
      const wrapper = mount(PluginPanel, {
        global: {
          plugins: [i18n]
        }
      })

      await new Promise(resolve => setTimeout(resolve, 10))
      await nextTick()

      const badge = wrapper.find('.bg-green-900')
      expect(badge.exists()).toBe(true)
    })

    it('applies correct status badge classes for disabled plugin', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      invoke.mockResolvedValue([mockPlugins[1]])

      const i18n = createTestI18n()
      const wrapper = mount(PluginPanel, {
        global: {
          plugins: [i18n]
        }
      })

      await new Promise(resolve => setTimeout(resolve, 10))
      await nextTick()

      const badge = wrapper.find('.bg-yellow-900')
      expect(badge.exists()).toBe(true)
    })

    it('applies correct status badge classes for error plugin', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      invoke.mockResolvedValue([mockPlugins[2]])

      const i18n = createTestI18n()
      const wrapper = mount(PluginPanel, {
        global: {
          plugins: [i18n]
        }
      })

      await new Promise(resolve => setTimeout(resolve, 10))
      await nextTick()

      const badge = wrapper.find('.bg-red-900')
      expect(badge.exists()).toBe(true)
    })

    it('applies correct button classes for enabled plugin', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      invoke.mockResolvedValue([mockPlugins[0]])

      const i18n = createTestI18n()
      const wrapper = mount(PluginPanel, {
        global: {
          plugins: [i18n]
        }
      })

      await new Promise(resolve => setTimeout(resolve, 10))
      await nextTick()

      const toggleButton = wrapper.findAll('button').find(b => b.text() === 'Disable')
      expect(toggleButton?.classes()).toContain('bg-red-600')
    })

    it('applies correct button classes for disabled plugin', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      invoke.mockResolvedValue([mockPlugins[1]])

      const i18n = createTestI18n()
      const wrapper = mount(PluginPanel, {
        global: {
          plugins: [i18n]
        }
      })

      await new Promise(resolve => setTimeout(resolve, 10))
      await nextTick()

      const toggleButton = wrapper.findAll('button').find(b => b.text() === 'Enable')
      expect(toggleButton?.classes()).toContain('bg-green-600')
    })
  })
})