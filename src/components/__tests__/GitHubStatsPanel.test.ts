import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import GitHubStatsPanel from '../GitHubStatsPanel.vue'

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

import { invoke } from '@tauri-apps/api/core'

const mockInvoke = vi.mocked(invoke)

describe('GitHubStatsPanel', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('loading state', () => {
    it('shows loading state initially', () => {
      mockInvoke.mockImplementation(() => new Promise(() => {})) // Never resolves

      const wrapper = mount(GitHubStatsPanel, {
        global: {
          stubs: {}
        }
      })

      expect(wrapper.text()).toContain('加载中')
    })
  })

  describe('not configured state', () => {
    it('shows not configured message when GitHub is not configured', async () => {
      mockInvoke.mockResolvedValue({
        configured: false,
        stats: null
      })

      const wrapper = mount(GitHubStatsPanel, {
        global: {
          stubs: {}
        }
      })

      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('GitHub 未配置')
      })
    })

    it('emits openSettings when clicking settings link', async () => {
      mockInvoke.mockResolvedValue({
        configured: false,
        stats: null
      })

      const wrapper = mount(GitHubStatsPanel, {
        global: {
          stubs: {}
        }
      })

      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('GitHub 未配置')
      })

      // Find the "前往设置配置 GitHub Token" button
      const buttons = wrapper.findAll('button')
      const settingsButton = buttons.find((b) => b.text().includes('前往设置'))
      await settingsButton?.trigger('click')

      expect(wrapper.emitted('openSettings')).toBeTruthy()
    })
  })

  describe('stats display', () => {
    it('displays stats when configured and data available', async () => {
      mockInvoke.mockResolvedValue({
        configured: true,
        stats: {
          commit_count: 5,
          pr_count: 2,
          estimated_hours: 3.5,
          active_repos: ['owner/repo1', 'owner/repo2'],
          commits_by_hour: { 10: ['commit 1', 'commit 2'], 14: ['commit 3'] },
          pull_requests: ['#42: Add feature']
        }
      })

      const wrapper = mount(GitHubStatsPanel, {
        global: {
          stubs: {}
        }
      })

      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('5')
        expect(wrapper.text()).toContain('2')
        expect(wrapper.text()).toContain('3.5')
      })

      expect(wrapper.text()).toContain('owner/repo1')
      expect(wrapper.text()).toContain('owner/repo2')
      expect(wrapper.text()).toContain('#42: Add feature')
    })

    it('displays commit time distribution', async () => {
      mockInvoke.mockResolvedValue({
        configured: true,
        stats: {
          commit_count: 3,
          pr_count: 0,
          estimated_hours: 1.5,
          active_repos: ['owner/repo'],
          commits_by_hour: { 10: ['commit 1', 'commit 2'], 14: ['commit 3'] },
          pull_requests: []
        }
      })

      const wrapper = mount(GitHubStatsPanel, {
        global: {
          stubs: {}
        }
      })

      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('10:00')
        expect(wrapper.text()).toContain('14:00')
      })
    })

    it('shows no activity message when no commits or PRs', async () => {
      mockInvoke.mockResolvedValue({
        configured: true,
        stats: {
          commit_count: 0,
          pr_count: 0,
          estimated_hours: 0,
          active_repos: [],
          commits_by_hour: {},
          pull_requests: []
        }
      })

      const wrapper = mount(GitHubStatsPanel, {
        global: {
          stubs: {}
        }
      })

      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('今日暂无 GitHub 活动')
      })
    })
  })

  describe('error handling', () => {
    it('shows error message on API failure', async () => {
      mockInvoke.mockRejectedValue('API error')

      const wrapper = mount(GitHubStatsPanel, {
        global: {
          stubs: {}
        }
      })

      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('API error')
      })
    })

    it('shows retry button on error', async () => {
      mockInvoke.mockRejectedValue('API error')

      const wrapper = mount(GitHubStatsPanel, {
        global: {
          stubs: {}
        }
      })

      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('重试')
      })
    })
  })

  describe('refresh functionality', () => {
    it('has refresh button that can be clicked', async () => {
      mockInvoke.mockResolvedValue({
        configured: true,
        stats: {
          commit_count: 1,
          pr_count: 0,
          estimated_hours: 0.5,
          active_repos: [],
          commits_by_hour: {},
          pull_requests: []
        }
      })

      const wrapper = mount(GitHubStatsPanel, {
        global: {
          stubs: {}
        }
      })

      // Wait for initial fetch to complete
      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('1')
      })

      // Verify refresh button exists
      const refreshButton = wrapper.findAll('button')[0]
      expect(refreshButton.text()).toContain('刷新')
      expect(refreshButton.attributes('disabled')).toBeUndefined()
    })

    it('disables refresh button while loading', async () => {
      mockInvoke.mockResolvedValue({
        configured: true,
        stats: {
          commit_count: 1,
          pr_count: 0,
          estimated_hours: 0.5,
          active_repos: [],
          commits_by_hour: {},
          pull_requests: []
        }
      })

      const wrapper = mount(GitHubStatsPanel, {
        global: {
          stubs: {}
        }
      })

      await vi.waitFor(() => {
        expect(wrapper.text()).not.toContain('加载中')
      })

      // Trigger a new loading state
      mockInvoke.mockImplementation(() => new Promise(() => {}))
      const buttons = wrapper.findAll('button')
      const refreshButton = buttons.find((b) => b.text().includes('刷新'))
      await refreshButton?.trigger('click')

      expect(wrapper.text()).toContain('刷新中')
    })
  })

  describe('UI elements', () => {
    it('displays panel title', async () => {
      mockInvoke.mockResolvedValue({
        configured: true,
        stats: null
      })

      const wrapper = mount(GitHubStatsPanel, {
        global: {
          stubs: {}
        }
      })

      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('GitHub 今日活动')
      })
    })

    it('displays GitHub icon', async () => {
      mockInvoke.mockResolvedValue({
        configured: true,
        stats: null
      })

      const wrapper = mount(GitHubStatsPanel, {
        global: {
          stubs: {}
        }
      })

      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('🐙')
      })
    })

    it('has correct stat labels', async () => {
      mockInvoke.mockResolvedValue({
        configured: true,
        stats: {
          commit_count: 5,
          pr_count: 2,
          estimated_hours: 3.5,
          active_repos: [],
          commits_by_hour: {},
          pull_requests: []
        }
      })

      const wrapper = mount(GitHubStatsPanel, {
        global: {
          stubs: {}
        }
      })

      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('提交')
        expect(wrapper.text()).toContain('PR')
        expect(wrapper.text()).toContain('预估工时')
      })
    })
  })
})