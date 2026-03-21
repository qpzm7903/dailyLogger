import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import AISettings from '../AISettings.vue'

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

// Mock toast store
vi.mock('../../../stores/toast', () => ({
  showSuccess: vi.fn(),
  showError: vi.fn()
}))

// Mock usePlatform composable
vi.mock('../../../composables/usePlatform', () => ({
  usePlatform: () => ({
    isDesktop: true
  })
}))

import { invoke } from '@tauri-apps/api/core'
import { showSuccess, showError } from '../../../stores/toast'

// Mock vue-i18n
const mockT = vi.fn((key: string, params?: Record<string, unknown>) => {
  const translations: Record<string, string> = {
    'settings.screenshotAnalysis': 'Screenshot Analysis',
    'settings.analysisModel': 'Analysis Model',
    'settings.analysisPrompt': 'Analysis Prompt',
    'settings.analysisPromptPlaceholder': 'Enter analysis prompt...',
    'settings.dailyReport': 'Daily Report',
    'settings.reportTitleFormat': 'Report Title Format',
    'settings.reportTitlePlaceholder': 'Daily Report - {date}',
    'settings.reportTitleHint': 'Use {date} for the date',
    'settings.reportModel': 'Report Model',
    'settings.reportModelPlaceholder': 'gpt-4o',
    'settings.reportPrompt': 'Report Prompt',
    'settings.reportPromptPlaceholder': 'Enter report prompt...',
    'settings.tagCategories': 'Tag Categories',
    'settings.customTagCategories': 'Custom Tag Categories',
    'settings.tagCategoriesPlaceholder': 'Enter tag categories...',
    'settings.tagCategoriesHint': 'One category per line',
    'settings.includeQuickNotes': 'Include Quick Notes',
    'settings.includeQuickNotesHint': '(recommended)',
    'settings.modelNameRequired': 'Model name is required',
    'settings.modelContextWindow': '{model} context window: {size}k tokens',
    'settings.modelInfoUnavailable': 'Model info unavailable',
    'settings.contextWindow': 'Context Window: {size}k tokens',
    'settings.visionRequired': 'Vision capable model required',
    'settings.textModelHint': 'Any text model works',
    'settings.viewDefaultTags': 'View Default Tags',
    'common.viewDefault': 'View Default',
    'common.resetDefault': 'Reset to Default',
    'common.templateLibrary': 'Template Library',
    'common.exportTemplate': 'Export',
    'common.importTemplate': 'Import'
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
    t: mockT
  })
}))

describe('AISettings', () => {
  const defaultProps = {
    settings: {
      model_name: 'gpt-4o',
      analysis_prompt: 'Analyze this screenshot',
      summary_model_name: 'gpt-4o',
      summary_prompt: 'Generate a daily report',
      summary_title_format: 'Daily Report - {date}',
      include_manual_records: true,
      api_base_url: 'https://api.openai.com/v1',
      api_key: 'sk-test'
    },
    tagCategoriesText: 'work\npersonal'
  }

  beforeEach(() => {
    vi.clearAllMocks()
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  // === Rendering ===
  describe('rendering', () => {
    it('renders screenshot analysis section', () => {
      const wrapper = mount(AISettings, { props: defaultProps })
      expect(wrapper.text()).toContain('Screenshot Analysis')
    })

    it('renders daily report section', () => {
      const wrapper = mount(AISettings, { props: defaultProps })
      expect(wrapper.text()).toContain('Daily Report')
    })

    it('renders tag categories section', () => {
      const wrapper = mount(AISettings, { props: defaultProps })
      expect(wrapper.text()).toContain('Tag Categories')
    })

    it('renders analysis model input with correct value', () => {
      const wrapper = mount(AISettings, { props: defaultProps })
      const inputs = wrapper.findAll('input')
      const analysisModelInput = inputs.find(i => i.attributes('placeholder') === 'gpt-4o')
      expect(analysisModelInput?.element.value).toBe('gpt-4o')
    })

    it('renders analysis prompt textarea', () => {
      const wrapper = mount(AISettings, { props: defaultProps })
      const textareas = wrapper.findAll('textarea')
      expect(textareas.length).toBeGreaterThanOrEqual(2)
    })

    it('renders report title format input', () => {
      const wrapper = mount(AISettings, { props: defaultProps })
      const inputs = wrapper.findAll('input')
      const titleInput = inputs.find(i => i.element.value.includes('{date}'))
      expect(titleInput?.exists()).toBe(true)
    })

    it('renders include manual records checkbox', () => {
      const wrapper = mount(AISettings, { props: defaultProps })
      const checkbox = wrapper.find('input[type="checkbox"]')
      expect(checkbox.exists()).toBe(true)
      expect((checkbox.element as HTMLInputElement).checked).toBe(true)
    })

    it('renders tag categories textarea', () => {
      const wrapper = mount(AISettings, { props: defaultProps })
      const textareas = wrapper.findAll('textarea')
      const tagTextarea = textareas.find(t => t.attributes('placeholder')?.includes('tag per line'))
      expect(tagTextarea?.exists()).toBe(true)
    })
  })

  // === Model Info ===
  describe('model info', () => {
    it('fetches analysis model info when info button clicked', async () => {
      vi.mocked(invoke).mockResolvedValueOnce({
        context_window: 128000,
        max_tokens: 4096
      })

      const wrapper = mount(AISettings, { props: defaultProps })
      const infoButtons = wrapper.findAll('button').filter(b => b.text().includes('ℹ️'))
      const analysisInfoButton = infoButtons[0]
      await analysisInfoButton?.trigger('click')
      await flushPromises()

      expect(invoke).toHaveBeenCalledWith('get_model_info', {
        apiBaseUrl: 'https://api.openai.com/v1',
        apiKey: 'sk-test',
        modelName: 'gpt-4o'
      })
    })

    it('fetches summary model info when info button clicked', async () => {
      vi.mocked(invoke).mockResolvedValueOnce({
        context_window: 128000,
        max_tokens: 4096
      })

      const wrapper = mount(AISettings, { props: defaultProps })
      const infoButtons = wrapper.findAll('button').filter(b => b.text().includes('ℹ️'))
      const summaryInfoButton = infoButtons[1]
      await summaryInfoButton?.trigger('click')
      await flushPromises()

      expect(invoke).toHaveBeenCalledWith('get_model_info', {
        apiBaseUrl: 'https://api.openai.com/v1',
        apiKey: 'sk-test',
        modelName: 'gpt-4o'
      })
    })

    it('shows error when model name is empty', async () => {
      const wrapper = mount(AISettings, {
        props: {
          ...defaultProps,
          settings: {
            ...defaultProps.settings,
            model_name: ''
          }
        }
      })

      const infoButtons = wrapper.findAll('button').filter(b => b.text().includes('ℹ️'))
      // Button should be disabled when model name is empty
      expect(infoButtons[0]?.element.disabled).toBe(true)
    })

    it('disables info button when model name is empty', () => {
      const wrapper = mount(AISettings, {
        props: {
          ...defaultProps,
          settings: {
            ...defaultProps.settings,
            model_name: ''
          }
        }
      })

      const infoButtons = wrapper.findAll('button').filter(b => b.text().includes('ℹ️'))
      expect(infoButtons[0]?.element.disabled).toBe(true)
    })
  })

  // === Props and Events ===
  describe('props and events', () => {
    it('syncs local settings with props', async () => {
      const wrapper = mount(AISettings, { props: defaultProps })
      const inputs = wrapper.findAll('input')
      const analysisModelInput = inputs.find(i => i.attributes('placeholder') === 'gpt-4o')
      await analysisModelInput?.setValue('gpt-4-turbo')

      const emitted = wrapper.emitted('update:settings')
      expect(emitted).toBeTruthy()
      expect(emitted?.[0][0].model_name).toBe('gpt-4-turbo')
    })

    it('syncs tag categories text with props', async () => {
      const wrapper = mount(AISettings, { props: defaultProps })
      const textareas = wrapper.findAll('textarea')
      const tagTextarea = textareas.find(t => t.attributes('placeholder')?.includes('tag per line'))
      await tagTextarea?.setValue('new-category')
      await flushPromises()

      const emitted = wrapper.emitted('update:tagCategoriesText')
      expect(emitted).toBeTruthy()
    })

    it('updates local settings when props change', async () => {
      const wrapper = mount(AISettings, { props: defaultProps })
      await wrapper.setProps({
        settings: {
          ...defaultProps.settings,
          model_name: 'gpt-4-turbo'
        },
        tagCategoriesText: defaultProps.tagCategoriesText
      })

      expect(wrapper.vm.localSettings.model_name).toBe('gpt-4-turbo')
    })

    it('emits show-default-prompt-modal when view default clicked', async () => {
      const wrapper = mount(AISettings, { props: defaultProps })
      const buttons = wrapper.findAll('button')
      const viewDefaultButton = buttons.find(b => b.text() === 'View Default')
      await viewDefaultButton?.trigger('click')

      expect(wrapper.emitted('show-default-prompt-modal')).toBeTruthy()
    })

    it('resets analysis prompt when reset clicked', async () => {
      const mockDefaultPrompt = 'Default analysis prompt content'
      vi.mocked(invoke).mockResolvedValueOnce(mockDefaultPrompt)

      const wrapper = mount(AISettings, { props: defaultProps })
      const buttons = wrapper.findAll('button')
      const resetButton = buttons.find(b => b.text() === 'Reset to Default')
      await resetButton?.trigger('click')
      await flushPromises()

      expect(invoke).toHaveBeenCalledWith('get_default_analysis_prompt')
      expect(wrapper.vm.localSettings.analysis_prompt).toBe(mockDefaultPrompt)
    })

    it('emits show-default-summary-prompt-modal when view default summary clicked', async () => {
      const wrapper = mount(AISettings, { props: defaultProps })
      // Find the second "View Default" button (for summary prompt)
      const buttons = wrapper.findAll('button')
      const viewDefaultButtons = buttons.filter(b => b.text() === 'View Default')
      await viewDefaultButtons[1]?.trigger('click')

      expect(wrapper.emitted('show-default-summary-prompt-modal')).toBeTruthy()
    })

    it('emits show-template-library-modal when template library clicked', async () => {
      const wrapper = mount(AISettings, { props: defaultProps })
      const buttons = wrapper.findAll('button')
      const templateButton = buttons.find(b => b.text() === 'Template Library')
      await templateButton?.trigger('click')

      expect(wrapper.emitted('show-template-library-modal')).toBeTruthy()
    })

    it('emits show-default-tag-categories-modal when view default tags clicked', async () => {
      const wrapper = mount(AISettings, { props: defaultProps })
      const buttons = wrapper.findAll('button')
      const viewDefaultTagsButton = buttons.find(b => b.text() === 'View Default Tags')
      await viewDefaultTagsButton?.trigger('click')

      expect(wrapper.emitted('show-default-tag-categories-modal')).toBeTruthy()
    })

    it('resets tag categories when reset clicked', async () => {
      const wrapper = mount(AISettings, { props: defaultProps })
      const buttons = wrapper.findAll('button')
      const resetButton = buttons.filter(b => b.text() === 'Reset to Default')
      // The last reset button should be for tag categories
      await resetButton[resetButton.length - 1]?.trigger('click')

      expect(wrapper.vm.localTagCategoriesText).toBe('')
    })
  })

  // === Checkbox ===
  describe('include manual records checkbox', () => {
    it('toggles include manual records', async () => {
      const wrapper = mount(AISettings, { props: defaultProps })
      const checkbox = wrapper.find('input[type="checkbox"]')
      await checkbox.setValue(false)

      expect(wrapper.vm.localSettings.include_manual_records).toBe(false)
    })

    it('emits update:settings when checkbox toggled', async () => {
      const wrapper = mount(AISettings, { props: defaultProps })
      const checkbox = wrapper.find('input[type="checkbox"]')
      await checkbox.setValue(false)

      const emitted = wrapper.emitted('update:settings')
      expect(emitted).toBeTruthy()
      expect(emitted?.[0][0].include_manual_records).toBe(false)
    })
  })
})