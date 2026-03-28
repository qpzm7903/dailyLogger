import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { nextTick, ref } from 'vue'
import OnboardingModal from '../OnboardingModal.vue'

// Mock Tauri APIs
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn()
}))

vi.mock('../../features/settings/actions', () => ({
  settingsActions: {
    getSettings: vi.fn(),
    saveSettings: vi.fn()
  }
}))

describe('OnboardingModal', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('rendering', () => {
    it('renders with overlay', async () => {
      const { settingsActions } = await import('../../features/settings/actions')
      ;(settingsActions.getSettings as ReturnType<typeof vi.fn>).mockResolvedValue({
        onboarding_completed: false,
        api_base_url: '',
        api_key: '',
        model_name: '',
        obsidian_path: ''
      })

      const wrapper = mount(OnboardingModal)
      expect(wrapper.find('.fixed.inset-0').exists()).toBe(true)
    })

    it('renders step indicator when not completed', async () => {
      const { settingsActions } = await import('../../features/settings/actions')
      ;(settingsActions.getSettings as ReturnType<typeof vi.fn>).mockResolvedValue({
        onboarding_completed: false,
        api_base_url: '',
        api_key: '',
        model_name: '',
        obsidian_path: ''
      })

      const wrapper = mount(OnboardingModal)
      await flushPromises()
      expect(wrapper.findAll('.w-8.h-8.rounded-full').length).toBe(3)
    })

    it('does not render step indicator when completed', async () => {
      const { settingsActions } = await import('../../features/settings/actions')
      ;(settingsActions.getSettings as ReturnType<typeof vi.fn>).mockResolvedValue({
        onboarding_completed: false,
        api_base_url: '',
        api_key: '',
        model_name: '',
        obsidian_path: ''
      })

      const wrapper = mount(OnboardingModal)
      await flushPromises()
      // Simulate completed state
      wrapper.vm.isCompleted = true
      await flushPromises()
      expect(wrapper.findAll('.w-8.h-8.rounded-full').length).toBe(0)
    })

    it('shows welcome message when completed', async () => {
      const { settingsActions } = await import('../../features/settings/actions')
      ;(settingsActions.getSettings as ReturnType<typeof vi.fn>).mockResolvedValue({
        onboarding_completed: false,
        api_base_url: '',
        api_key: '',
        model_name: '',
        obsidian_path: ''
      })

      const wrapper = mount(OnboardingModal)
      await flushPromises()
      wrapper.vm.isCompleted = true
      await flushPromises()
      expect(wrapper.text()).toContain('设置完成')
    })
  })

  describe('step navigation', () => {
    it('starts at step 1', async () => {
      const { settingsActions } = await import('../../features/settings/actions')
      ;(settingsActions.getSettings as ReturnType<typeof vi.fn>).mockResolvedValue({
        onboarding_completed: false,
        api_base_url: '',
        api_key: '',
        model_name: '',
        obsidian_path: ''
      })

      const wrapper = mount(OnboardingModal)
      await flushPromises()
      expect(wrapper.vm.currentStep).toBe(1)
    })

    it('shows step 1 content by default', async () => {
      const { settingsActions } = await import('../../features/settings/actions')
      ;(settingsActions.getSettings as ReturnType<typeof vi.fn>).mockResolvedValue({
        onboarding_completed: false,
        api_base_url: '',
        api_key: '',
        model_name: '',
        obsidian_path: ''
      })

      const wrapper = mount(OnboardingModal)
      await flushPromises()
      expect(wrapper.text()).toContain('Step 1')
      expect(wrapper.text()).toContain('API')
    })

    it('shows step 2 content when navigated to step 2', async () => {
      const { settingsActions } = await import('../../features/settings/actions')
      ;(settingsActions.getSettings as ReturnType<typeof vi.fn>).mockResolvedValue({
        onboarding_completed: false,
        api_base_url: '',
        api_key: '',
        model_name: '',
        obsidian_path: ''
      })

      const wrapper = mount(OnboardingModal)
      await flushPromises()

      // Manually set step to 2
      wrapper.vm.currentStep = 2
      await flushPromises()

      expect(wrapper.text()).toContain('Step 2')
      expect(wrapper.text()).toContain('Obsidian')
    })

    it('shows step 3 content when navigated to step 3', async () => {
      const { settingsActions } = await import('../../features/settings/actions')
      ;(settingsActions.getSettings as ReturnType<typeof vi.fn>).mockResolvedValue({
        onboarding_completed: false,
        api_base_url: '',
        api_key: '',
        model_name: '',
        obsidian_path: ''
      })

      const wrapper = mount(OnboardingModal)
      await flushPromises()

      // Manually set step to 3
      wrapper.vm.currentStep = 3
      await flushPromises()

      expect(wrapper.text()).toContain('Step 3')
      expect(wrapper.text()).toContain('完成设置')
    })

    it('can navigate back from step 2 to step 1', async () => {
      const { settingsActions } = await import('../../features/settings/actions')
      ;(settingsActions.getSettings as ReturnType<typeof vi.fn>).mockResolvedValue({
        onboarding_completed: false,
        api_base_url: '',
        api_key: '',
        model_name: '',
        obsidian_path: ''
      })

      const wrapper = mount(OnboardingModal)
      await flushPromises()

      // Set to step 2
      wrapper.vm.currentStep = 2
      await flushPromises()

      // Go back
      wrapper.vm.previousStep()
      await flushPromises()

      expect(wrapper.vm.currentStep).toBe(1)
    })
  })

  describe('API configuration (step 1)', () => {
    it('has API inputs', async () => {
      const { settingsActions } = await import('../../features/settings/actions')
      ;(settingsActions.getSettings as ReturnType<typeof vi.fn>).mockResolvedValue({
        onboarding_completed: false,
        api_base_url: '',
        api_key: '',
        model_name: '',
        obsidian_path: ''
      })

      const wrapper = mount(OnboardingModal)
      await flushPromises()

      const inputs = wrapper.findAll('input')
      expect(inputs.length).toBeGreaterThan(0)
    })

    it('has test connection button', async () => {
      const { settingsActions } = await import('../../features/settings/actions')
      ;(settingsActions.getSettings as ReturnType<typeof vi.fn>).mockResolvedValue({
        onboarding_completed: false,
        api_base_url: '',
        api_key: '',
        model_name: '',
        obsidian_path: ''
      })

      const wrapper = mount(OnboardingModal)
      await flushPromises()

      expect(wrapper.text()).toContain('测试连接')
    })
  })

  describe('skip functionality', () => {
    it('shows skip button', async () => {
      const { settingsActions } = await import('../../features/settings/actions')
      ;(settingsActions.getSettings as ReturnType<typeof vi.fn>).mockResolvedValue({
        onboarding_completed: false,
        api_base_url: '',
        api_key: '',
        model_name: '',
        obsidian_path: ''
      })

      const wrapper = mount(OnboardingModal)
      await flushPromises()

      expect(wrapper.text()).toContain('跳过')
    })
  })

  describe('close event', () => {
    it('emits close when close button is clicked in completed state', async () => {
      const { settingsActions } = await import('../../features/settings/actions')
      ;(settingsActions.getSettings as ReturnType<typeof vi.fn>).mockResolvedValue({
        onboarding_completed: false,
        api_base_url: '',
        api_key: '',
        model_name: '',
        obsidian_path: ''
      })

      const wrapper = mount(OnboardingModal)
      await flushPromises()
      wrapper.vm.isCompleted = true
      await flushPromises()

      const closeButton = wrapper.findAll('button').at(0)
      await closeButton?.trigger('click')

      expect(wrapper.emitted('close')).toBeTruthy()
    })
  })
})