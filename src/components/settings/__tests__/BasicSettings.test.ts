import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { ref } from 'vue'
import BasicSettings from '../BasicSettings.vue'

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

// Mock toast store
vi.mock('../../../stores/toast', () => ({
  showSuccess: vi.fn(),
  showError: vi.fn()
}))

// Mock i18n
vi.mock('../../../i18n', () => ({
  setLocale: vi.fn()
}))

// Mock usePlatform composable
vi.mock('../../../composables/usePlatform', () => ({
  usePlatform: () => ({
    isDesktop: true
  })
}))

import { invoke } from '@tauri-apps/api/core'
import { showSuccess, showError } from '../../../stores/toast'
import { setLocale } from '../../../i18n'

// Create a reactive locale ref for testing
const mockLocale = ref('en')

// Mock vue-i18n with actual translations
const mockT = vi.fn((key: string, params?: Record<string, unknown>) => {
  const translations: Record<string, string> = {
    'settings.apiConfig': 'API Configuration',
    'settings.apiKey': 'API Key',
    'settings.apiKeyOllamaHint': '(Optional for Ollama)',
    'settings.testConnection': 'Test Connection',
    'settings.testing': 'Testing...',
    'settings.fetchModels': 'Fetch Model List',
    'settings.fetching': 'Fetching...',
    'settings.connectionSuccess': 'Connection successful ({latency}ms)',
    'settings.apiBaseUrlRequired': 'API Base URL is required',
    'settings.apiKeyRequired': 'API Key is required',
    'settings.selectModel': 'Select Model',
    'settings.refreshModels': 'Refresh',
    'settings.pullModelPlaceholder': 'Model name to pull',
    'settings.defaultQuantization': 'Default',
    'settings.smallest': 'smallest',
    'settings.largest': 'largest',
    'settings.noCompression': 'no compression',
    'settings.quantizationTooltip': 'Quantization level',
    'settings.pullModel': 'Pull',
    'settings.pulling': 'Pulling...',
    'settings.modelNameRequired': 'Model name is required',
    'settings.copyModel': 'Copy Model',
    'settings.deleteModel': 'Delete Model',
    'settings.confirmDeleteModel': 'Delete model {model}?',
    'settings.runningModels': 'Running Models',
    'settings.vramUsage': 'VRAM: {size}',
    'settings.noRunningModels': 'No running models',
    'settings.noModelsFound': 'No models found',
    'settings.ollamaModelsNotFound': 'No models found on Ollama server',
    'settings.ollamaModelsFound': 'Found {count} models',
    'settings.createCustomModel': 'Create Custom Model',
    'settings.fineTuning': 'Fine-tuning',
    'settings.language': 'Language',
    'settings.languageEn': 'English',
    'settings.languageZhCN': '简体中文',
    'settings.languageHint': 'Select interface language',
    'settings.shortcuts': 'Shortcuts',
    'settings.quickNoteShortcut': 'Quick Note: Alt + Space',
    'settings.baseUrlOllamaHint': 'Ollama users: http://localhost:11434/v1',
    'common.hide': 'Hide',
    'common.show': 'Show'
  }
  let result = translations[key] || key
  if (params) {
    Object.entries(params).forEach(([k, v]) => {
      result = result.replace(`{${k}}`, String(v))
    })
  }
  return result
})

vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: mockT,
    locale: mockLocale
  })
}))

describe('BasicSettings', () => {
  const defaultProps = {
    settings: {
      api_base_url: 'https://api.openai.com/v1',
      api_key: 'sk-test-key',
      model_name: 'gpt-4o'
    }
  }

  beforeEach(() => {
    vi.clearAllMocks()
    mockLocale.value = 'en'
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  // === Rendering ===
  describe('rendering', () => {
    it('renders API configuration section', () => {
      const wrapper = mount(BasicSettings, { props: defaultProps })
      expect(wrapper.find('h3').text()).toBe('API Configuration')
    })

    it('renders base URL input with correct value', () => {
      const wrapper = mount(BasicSettings, { props: defaultProps })
      const baseUrlInput = wrapper.findAll('input')[0]
      expect(baseUrlInput.element.value).toBe('https://api.openai.com/v1')
    })

    it('renders API key input with password type by default', () => {
      const wrapper = mount(BasicSettings, { props: defaultProps })
      const apiKeyInput = wrapper.findAll('input')[1]
      expect(apiKeyInput.attributes('type')).toBe('password')
    })

    it('shows API key in plain text when show button clicked', async () => {
      const wrapper = mount(BasicSettings, { props: defaultProps })
      const showButton = wrapper.findAll('button')[0]
      await showButton.trigger('click')
      const apiKeyInput = wrapper.findAll('input')[1]
      expect(apiKeyInput.attributes('type')).toBe('text')
    })

    it('renders test connection button', () => {
      const wrapper = mount(BasicSettings, { props: defaultProps })
      const buttons = wrapper.findAll('button')
      const testButton = buttons.find(b => b.text().includes('Test Connection'))
      expect(testButton?.exists()).toBe(true)
    })

    it('renders language selection buttons', () => {
      const wrapper = mount(BasicSettings, { props: defaultProps })
      const buttons = wrapper.findAll('button')
      const enButton = buttons.find(b => b.text() === 'English')
      const zhButton = buttons.find(b => b.text() === '简体中文')
      expect(enButton?.exists()).toBe(true)
      expect(zhButton?.exists()).toBe(true)
    })

    it('renders shortcuts section on desktop', () => {
      const wrapper = mount(BasicSettings, { props: defaultProps })
      const shortcutText = wrapper.text()
      expect(shortcutText).toContain('Shortcuts')
      expect(shortcutText).toContain('Alt + Space')
    })

    it('shows Ollama indicator when using Ollama endpoint', () => {
      const wrapper = mount(BasicSettings, {
        props: {
          settings: {
            api_base_url: 'http://localhost:11434/v1',
            api_key: '',
            model_name: 'llama3.2'
          }
        }
      })
      expect(wrapper.text()).toContain('Ollama')
    })
  })

  // === Ollama Features ===
  describe('ollama features', () => {
    const ollamaProps = {
      settings: {
        api_base_url: 'http://localhost:11434/v1',
        api_key: '',
        model_name: 'llama3.2'
      }
    }

    it('shows fetch models button for Ollama endpoint', () => {
      const wrapper = mount(BasicSettings, { props: ollamaProps })
      const buttons = wrapper.findAll('button')
      const fetchButton = buttons.find(b => b.text().includes('Fetch Model List'))
      expect(fetchButton?.exists()).toBe(true)
    })

    it('shows pull model section for Ollama endpoint', () => {
      const wrapper = mount(BasicSettings, { props: ollamaProps })
      const inputs = wrapper.findAll('input')
      const pullInput = inputs.find(i => i.attributes('placeholder')?.includes('Model name'))
      expect(pullInput?.exists()).toBe(true)
    })

    it('fetches Ollama models when fetch button clicked', async () => {
      const mockModels = [
        { name: 'llama3.2', size: '2GB' },
        { name: 'mistral', size: '4GB' }
      ]
      vi.mocked(invoke).mockResolvedValueOnce({ success: true, models: mockModels })

      const wrapper = mount(BasicSettings, { props: ollamaProps })
      const buttons = wrapper.findAll('button')
      const fetchButton = buttons.find(b => b.text().includes('Fetch Model List'))
      await fetchButton?.trigger('click')
      await flushPromises()

      expect(invoke).toHaveBeenCalledWith('get_ollama_models', {
        baseUrl: 'http://localhost:11434/v1'
      })
    })

    it('selects Ollama model when clicked', async () => {
      const mockModels = [{ name: 'llama3.2', size: '2GB' }]
      vi.mocked(invoke).mockResolvedValueOnce({ success: true, models: mockModels })

      const wrapper = mount(BasicSettings, { props: ollamaProps })
      const buttons = wrapper.findAll('button')
      const fetchButton = buttons.find(b => b.text().includes('Fetch Model List'))
      await fetchButton?.trigger('click')
      await flushPromises()

      const modelButtons = wrapper.findAll('button').filter(b => b.text().includes('llama3.2'))
      if (modelButtons.length > 0) {
        await modelButtons[0].trigger('click')
        expect(wrapper.vm.localSettings.model_name).toBe('llama3.2')
      }
    })

    it('has quantization select dropdown', () => {
      const wrapper = mount(BasicSettings, { props: ollamaProps })
      const select = wrapper.find('select')
      expect(select.exists()).toBe(true)

      const options = select.findAll('option')
      expect(options.length).toBeGreaterThan(0)
    })
  })

  // === Connection Test ===
  describe('connection test', () => {
    it('tests connection successfully', async () => {
      vi.mocked(invoke).mockResolvedValueOnce({
        success: true,
        message: 'Connection OK',
        latency_ms: 150
      })

      const wrapper = mount(BasicSettings, { props: defaultProps })
      const buttons = wrapper.findAll('button')
      const testButton = buttons.find(b => b.text().includes('Test Connection'))
      await testButton?.trigger('click')
      await flushPromises()

      expect(invoke).toHaveBeenCalledWith('test_api_connection_with_ollama', {
        apiBaseUrl: 'https://api.openai.com/v1',
        apiKey: 'sk-test-key',
        modelName: 'gpt-4o',
        proxyEnabled: false,
        proxyHost: null,
        proxyPassword: null,
        proxyPort: null,
        proxyUsername: null
      })
    })

    it('disables test button when missing required fields', () => {
      const wrapper = mount(BasicSettings, {
        props: {
          settings: {
            api_base_url: '',
            api_key: '',
            model_name: ''
          }
        }
      })
      const buttons = wrapper.findAll('button')
      const testButton = buttons.find(b => b.text().includes('Test Connection'))
      expect(testButton?.element.disabled).toBe(true)
    })

    it('requires base URL for test connection', () => {
      const wrapper = mount(BasicSettings, {
        props: {
          settings: {
            api_base_url: '',
            api_key: 'key',
            model_name: 'model'
          }
        }
      })
      const buttons = wrapper.findAll('button')
      const testButton = buttons.find(b => b.text().includes('Test Connection'))
      expect(testButton?.element.disabled).toBe(true)
    })

    it('requires model name for test connection', () => {
      const wrapper = mount(BasicSettings, {
        props: {
          settings: {
            api_base_url: 'https://api.openai.com/v1',
            api_key: 'key',
            model_name: ''
          }
        }
      })
      const buttons = wrapper.findAll('button')
      const testButton = buttons.find(b => b.text().includes('Test Connection'))
      expect(testButton?.element.disabled).toBe(true)
    })

    it('requires API key for non-Ollama endpoints', () => {
      const wrapper = mount(BasicSettings, {
        props: {
          settings: {
            api_base_url: 'https://api.openai.com/v1',
            api_key: '',
            model_name: 'gpt-4o'
          }
        }
      })
      const buttons = wrapper.findAll('button')
      const testButton = buttons.find(b => b.text().includes('Test Connection'))
      expect(testButton?.element.disabled).toBe(true)
    })

    it('does not require API key for Ollama endpoints', () => {
      const wrapper = mount(BasicSettings, {
        props: {
          settings: {
            api_base_url: 'http://localhost:11434/v1',
            api_key: '',
            model_name: 'llama3.2'
          }
        }
      })
      const buttons = wrapper.findAll('button')
      const testButton = buttons.find(b => b.text().includes('Test Connection'))
      expect(testButton?.element.disabled).toBe(false)
    })
  })

  // === Language Selection ===
  describe('language selection', () => {
    it('changes language to English', async () => {
      const wrapper = mount(BasicSettings, { props: defaultProps })
      const buttons = wrapper.findAll('button')
      const enButton = buttons.find(b => b.text() === 'English')
      await enButton?.trigger('click')
      expect(setLocale).toHaveBeenCalledWith('en')
    })

    it('changes language to Chinese', async () => {
      const wrapper = mount(BasicSettings, { props: defaultProps })
      const buttons = wrapper.findAll('button')
      const zhButton = buttons.find(b => b.text() === '简体中文')
      await zhButton?.trigger('click')
      expect(setLocale).toHaveBeenCalledWith('zh-CN')
    })

    it('highlights selected language', () => {
      const wrapper = mount(BasicSettings, { props: defaultProps })
      const buttons = wrapper.findAll('button')
      const enButton = buttons.find(b => b.text() === 'English')
      // English should be highlighted (default locale is 'en')
      expect(enButton?.classes()).toContain('bg-primary')
    })
  })

  // === Props and Events ===
  describe('props and events', () => {
    it('syncs local settings with props', async () => {
      const wrapper = mount(BasicSettings, { props: defaultProps })
      const inputs = wrapper.findAll('input')
      await inputs[0].setValue('https://new-url.com/v1')

      const emitted = wrapper.emitted('update:settings')
      expect(emitted).toBeTruthy()
      expect(emitted?.[0][0]).toEqual({
        api_base_url: 'https://new-url.com/v1',
        api_key: 'sk-test-key',
        model_name: 'gpt-4o'
      })
    })

    it('updates local settings when props change', async () => {
      const wrapper = mount(BasicSettings, { props: defaultProps })
      await wrapper.setProps({
        settings: {
          api_base_url: 'https://new-url.com/v1',
          api_key: 'new-key',
          model_name: 'gpt-4'
        }
      })

      expect(wrapper.vm.localSettings.api_base_url).toBe('https://new-url.com/v1')
      expect(wrapper.vm.localSettings.api_key).toBe('new-key')
      expect(wrapper.vm.localSettings.model_name).toBe('gpt-4')
    })
  })
})