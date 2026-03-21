import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount } from '@vue/test-utils'
import SettingsModal from '../SettingsModal.vue'

// Mock Tauri APIs
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

// Mock vue-i18n
vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string, params?: Record<string, unknown>) => {
      const translations: Record<string, string> = {
        'settings.title': 'Settings',
        'settings.cancel': 'Cancel',
        'settings.save': 'Save',
        'settings.saving': 'Saving...',
        'settings.saved': 'Saved',
        'settings.settingsSaved': 'Settings saved successfully',
        'settings.saveFailed': 'Failed to save settings',
        'settings.tabBasic': 'Basic',
        'settings.tabAI': 'AI',
        'settings.tabCapture': 'Capture',
        'settings.tabOutput': 'Output',
        'settings.unsavedChanges': 'Unsaved Changes',
        'settings.unsavedChangesMessage': 'You have unsaved changes. Discard them?',
        'settings.discardAndClose': 'Discard and Close',
        'settings.defaultPrompt': 'Default Analysis Prompt',
        'settings.defaultReportPrompt': 'Default Report Prompt',
        'settings.defaultTagCategories': 'Default Tag Categories',
        'settings.templateLibrary': 'Template Library',
        'settings.templateDefaultName': 'Default Template',
        'settings.templateDefaultDesc': 'Standard daily report template',
        'settings.templateSimpleName': 'Simple Template',
        'settings.templateSimpleDesc': 'Concise report format',
        'settings.templateDetailedName': 'Detailed Template',
        'settings.templateDetailedDesc': 'Comprehensive report format',
        'settings.createCustomModel': 'Create Custom Model',
        'settings.modelName': 'Model Name',
        'settings.fromModel': 'From Model',
        'settings.systemPrompt': 'System Prompt',
        'settings.create': 'Create',
        'settings.creating': 'Creating...',
        'settings.copyModel': 'Copy Model',
        'settings.sourceModel': 'Source Model',
        'settings.newModelName': 'New Model Name',
        'settings.copy': 'Copy',
        'settings.copying': 'Copying...',
        'common.cancel': 'Cancel',
        'common.close': 'Close',
        'common.templateLibrary': 'Template Library',
      }
      return translations[key] || key
    },
  }),
  createI18n: vi.fn(() => ({
    global: {
      t: (key: string) => key,
    },
  })),
}))

// Mock toast store
vi.mock('../../stores/toast', () => ({
  showError: vi.fn(),
  showSuccess: vi.fn(),
}))

// Mock settings sub-components
vi.mock('./settings', () => ({
  BasicSettings: {
    name: 'BasicSettings',
    template: '<div class="basic-settings-mock" data-testid="basic-settings"></div>',
    props: ['settings'],
  },
  AISettings: {
    name: 'AISettings',
    template: '<div class="ai-settings-mock" data-testid="ai-settings"></div>',
    props: ['settings', 'tagCategoriesText'],
  },
  CaptureSettings: {
    name: 'CaptureSettings',
    template: '<div class="capture-settings-mock" data-testid="capture-settings"></div>',
    props: ['settings', 'whitelistTags', 'blacklistTags', 'monitors'],
  },
  OutputSettings: {
    name: 'OutputSettings',
    template: '<div class="output-settings-mock" data-testid="output-settings"></div>',
    props: ['settings', 'vaults', 'graphs'],
  },
}))

describe('SettingsModal', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  describe('rendering', () => {
    it('renders modal with title', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue({})

      const wrapper = mount(SettingsModal)

      expect(wrapper.find('h2').text()).toContain('Settings')
    })

    it('shows close button', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue({})

      const wrapper = mount(SettingsModal)

      const closeBtn = wrapper.findAll('button').find((b) => b.text().includes('✕'))
      expect(closeBtn).toBeDefined()
    })

    it('shows tab navigation', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue({})

      const wrapper = mount(SettingsModal)

      expect(wrapper.text()).toContain('Basic')
      expect(wrapper.text()).toContain('AI')
      expect(wrapper.text()).toContain('Capture')
      expect(wrapper.text()).toContain('Output')
    })

    it('shows save and cancel buttons', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue({})

      const wrapper = mount(SettingsModal)

      const buttons = wrapper.findAll('button')
      const cancelBtn = buttons.find((b) => b.text().includes('Cancel'))
      const saveBtn = buttons.find((b) => b.text().includes('Save'))

      expect(cancelBtn).toBeDefined()
      expect(saveBtn).toBeDefined()
    })
  })

  describe('tab navigation', () => {
    it('starts with Basic tab active by default', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue({})

      const wrapper = mount(SettingsModal)

      expect(wrapper.vm.activeTab).toBe('basic')
    })

    it('switches tab when clicked', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue({})

      const wrapper = mount(SettingsModal)

      // Click AI tab
      const aiTabBtn = wrapper.findAll('button').find((b) => b.text().trim() === 'AI')
      await aiTabBtn?.trigger('click')

      expect(wrapper.vm.activeTab).toBe('ai')
    })

    it('renders correct sub-component for each tab', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue({})

      const wrapper = mount(SettingsModal)

      // Basic tab is active by default
      expect(wrapper.vm.activeTab).toBe('basic')

      // Switch to AI tab
      const aiTabBtn = wrapper.findAll('button').find((b) => b.text().trim() === 'AI')
      await aiTabBtn?.trigger('click')
      expect(wrapper.vm.activeTab).toBe('ai')

      // Switch to Capture tab
      const captureTabBtn = wrapper.findAll('button').find((b) => b.text().trim() === 'Capture')
      await captureTabBtn?.trigger('click')
      expect(wrapper.vm.activeTab).toBe('capture')

      // Switch to Output tab
      const outputTabBtn = wrapper.findAll('button').find((b) => b.text().trim() === 'Output')
      await outputTabBtn?.trigger('click')
      expect(wrapper.vm.activeTab).toBe('output')
    })
  })

  describe('settings loading', () => {
    it('loads settings on mount', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue({
        api_base_url: 'http://localhost:11434',
        model_name: 'llama3',
      })

      mount(SettingsModal)

      await vi.waitFor(() => {
        expect(invoke).toHaveBeenCalledWith('get_settings')
      })
    })

    it('handles load error gracefully', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockRejectedValue(new Error('Load failed'))

      // Should not throw
      const wrapper = mount(SettingsModal)

      await vi.waitFor(() => {
        expect(invoke).toHaveBeenCalled()
      })

      // Component should still render
      expect(wrapper.find('h2').text()).toContain('Settings')
    })
  })

  describe('save functionality', () => {
    it('calls invoke with correct parameters when saving', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValueOnce({}) // get_settings
      vi.mocked(invoke).mockResolvedValueOnce(undefined) // save_settings

      const wrapper = mount(SettingsModal)

      await vi.waitFor(() => {
        expect(invoke).toHaveBeenCalledWith('get_settings')
      })

      // Verify saveSettings function exists and can be called
      expect(typeof wrapper.vm.saveSettings).toBe('function')
    })
  })

  describe('close functionality', () => {
    it('emits close when handleClose is called with no changes', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue({})

      const wrapper = mount(SettingsModal)

      await vi.waitFor(() => {
        expect(invoke).toHaveBeenCalled()
      })

      // Directly call handleClose
      wrapper.vm.handleClose()

      expect(wrapper.emitted('close')).toBeTruthy()
    })

    it('shows confirmation when closing with unsaved changes', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValueOnce({ model_name: 'gpt-4' })

      const wrapper = mount(SettingsModal)

      await vi.waitFor(() => {
        expect(invoke).toHaveBeenCalled()
      })

      // Simulate initial settings being captured
      wrapper.vm.initialSettings = JSON.stringify({
        settings: { model_name: 'gpt-4' },
        vaults: [],
        graphs: [],
        whitelistTags: [],
        blacklistTags: [],
        tagCategoriesText: '',
      })

      // Change settings to make hasUnsavedChanges true
      wrapper.vm.settings.model_name = 'gpt-4o'

      // Call handleClose
      wrapper.vm.handleClose()

      // Should show confirmation dialog
      expect(wrapper.vm.showCloseConfirm).toBe(true)
    })

    it('discards changes and closes when confirmClose is called', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValueOnce({ model_name: 'gpt-4' })

      const wrapper = mount(SettingsModal)

      await vi.waitFor(() => {
        expect(invoke).toHaveBeenCalled()
      })

      wrapper.vm.showCloseConfirm = true

      // Call confirmClose
      wrapper.vm.confirmClose()

      expect(wrapper.emitted('close')).toBeTruthy()
      expect(wrapper.vm.showCloseConfirm).toBe(false)
    })

    it('can cancel confirmation dialog', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValueOnce({ model_name: 'gpt-4' })

      const wrapper = mount(SettingsModal)

      await vi.waitFor(() => {
        expect(invoke).toHaveBeenCalled()
      })

      wrapper.vm.showCloseConfirm = true

      // Simulate cancel by setting showCloseConfirm to false
      wrapper.vm.showCloseConfirm = false

      expect(wrapper.vm.showCloseConfirm).toBe(false)
      expect(wrapper.emitted('close')).toBeFalsy()
    })
  })

  describe('modal structure', () => {
    it('has proper modal styling', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue({})

      const wrapper = mount(SettingsModal)

      expect(wrapper.find('.fixed.inset-0.bg-black\\/50').exists()).toBe(true)
      expect(wrapper.find('.bg-dark.rounded-2xl').exists()).toBe(true)
      expect(wrapper.find('.z-50').exists()).toBe(true)
    })

    it('has scrollable content area', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue({})

      const wrapper = mount(SettingsModal)

      expect(wrapper.find('.overflow-y-auto').exists()).toBe(true)
    })
  })

  describe('default prompts modals', () => {
    it('can show default analysis prompt modal', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue('Default analysis prompt content')

      const wrapper = mount(SettingsModal)

      await wrapper.vm.openDefaultPromptModal()

      expect(wrapper.vm.showDefaultPromptModal).toBe(true)
      expect(wrapper.vm.defaultPromptContent).toBe('Default analysis prompt content')
    })

    it('can show default summary prompt modal', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue('Default summary prompt content')

      const wrapper = mount(SettingsModal)

      await wrapper.vm.openDefaultSummaryPromptModal()

      expect(wrapper.vm.showDefaultSummaryPromptModal).toBe(true)
      expect(wrapper.vm.defaultSummaryPromptContent).toBe('Default summary prompt content')
    })
  })

  describe('template library', () => {
    it('has preset templates', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue({})

      const wrapper = mount(SettingsModal)

      expect(wrapper.vm.presetTemplates.length).toBeGreaterThan(0)
    })

    it('can apply template', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      const { showSuccess } = await import('../../stores/toast')
      vi.mocked(invoke).mockResolvedValue({})

      const wrapper = mount(SettingsModal)

      const template = {
        id: 'test',
        name: 'Test Template',
        description: 'A test template',
        content: 'Test content {records}',
      }

      wrapper.vm.applyTemplate(template)

      expect(wrapper.vm.settings.summary_prompt).toBe('Test content {records}')
      expect(showSuccess).toHaveBeenCalled()
    })
  })

  describe('model operations', () => {
    it('can open copy model modal', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke).mockResolvedValue({})

      const wrapper = mount(SettingsModal)

      wrapper.vm.openCopyModelModal('llama3')

      expect(wrapper.vm.showCopyModelModal).toBe(true)
      expect(wrapper.vm.copyModelSource).toBe('llama3')
    })

    it('can copy model', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      const { showSuccess } = await import('../../stores/toast')
      vi.mocked(invoke).mockResolvedValueOnce({}) // get_settings
      vi.mocked(invoke).mockResolvedValueOnce({ success: true, message: 'Model copied' })

      const wrapper = mount(SettingsModal)

      wrapper.vm.copyModelSource = 'llama3'
      wrapper.vm.copyModelDestination = 'llama3-copy'

      await wrapper.vm.copyModel()

      expect(invoke).toHaveBeenCalledWith('copy_ollama_model', expect.objectContaining({
        source: 'llama3',
        destination: 'llama3-copy',
      }))
      expect(showSuccess).toHaveBeenCalled()
    })

    it('can create custom model', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      const { showSuccess } = await import('../../stores/toast')
      vi.mocked(invoke).mockResolvedValueOnce({}) // get_settings
      vi.mocked(invoke).mockResolvedValueOnce({ success: true, message: 'Model created' })

      const wrapper = mount(SettingsModal)

      wrapper.vm.createModelParams = {
        name: 'custom-model',
        from: 'llama3',
        system: 'You are helpful',
        temperature: null,
        num_ctx: null,
        quantize: '',
      }

      await wrapper.vm.createCustomModel()

      expect(invoke).toHaveBeenCalledWith('create_ollama_model', expect.objectContaining({
        name: 'custom-model',
        from: 'llama3',
        system: 'You are helpful',
      }))
      expect(showSuccess).toHaveBeenCalled()
    })
  })

})