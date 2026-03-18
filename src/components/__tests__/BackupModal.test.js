import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import BackupModal from '../BackupModal.vue'

// Mock Tauri APIs
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn()
}))

import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'

// Helper to setup invoke mock
function setupInvokeMock(responses = {}) {
  invoke.mockImplementation((cmd, args) => {
    if (cmd === 'list_backups') {
      return Promise.resolve(responses.list_backups || [])
    }
    if (cmd === 'create_backup') {
      return Promise.resolve(responses.create_backup || { path: '/backups/test.zip', size_bytes: 1024, record_count: 10, screenshot_count: 5 })
    }
    if (cmd === 'get_backup_info') {
      return Promise.resolve(responses.get_backup_info || { path: '/backups/test.zip', created_at: '2026-03-18T10:00:00Z', size_bytes: 2048, record_count: 25, screenshot_count: 10 })
    }
    if (cmd === 'restore_backup') {
      return Promise.resolve(responses.restore_backup || { record_count: 50, screenshot_count: 20, auto_backup_created: true })
    }
    if (cmd === 'delete_backup') {
      return Promise.resolve(responses.delete_backup || { success: true })
    }
    return Promise.resolve(null)
  })
}

describe('BackupModal', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    setupInvokeMock()
  })

  describe('Rendering', () => {
    it('renders the modal with header', async () => {
      const wrapper = mount(BackupModal)

      await nextTick()

      expect(wrapper.html()).toContain('数据备份与恢复')
      expect(wrapper.find('h2').text()).toBe('数据备份与恢复')
    })

    it('renders close button in header', async () => {
      const wrapper = mount(BackupModal)

      await nextTick()

      const closeButton = wrapper.find('button')
      expect(closeButton.text()).toContain('✕')
    })

    it('emits close event when close button is clicked', async () => {
      const wrapper = mount(BackupModal)

      await nextTick()

      const closeButton = wrapper.find('h2 + button')
      await closeButton.trigger('click')

      expect(wrapper.emitted('close')).toBeTruthy()
    })

    it('emits close event when clicking backdrop', async () => {
      const wrapper = mount(BackupModal)

      await nextTick()

      const backdrop = wrapper.find('.fixed.inset-0')
      await backdrop.trigger('click')

      expect(wrapper.emitted('close')).toBeTruthy()
    })
  })

  describe('Tab Navigation', () => {
    it('renders three tabs', async () => {
      const wrapper = mount(BackupModal)

      await nextTick()

      // Select tabs container specifically
      const tabsContainer = wrapper.findAll('.flex.border-b')[1] // Second .flex.border-b is the tabs
      const tabs = tabsContainer.findAll('button')
      expect(tabs).toHaveLength(3)
      expect(tabs[0].text()).toContain('创建备份')
      expect(tabs[1].text()).toContain('恢复数据')
      expect(tabs[2].text()).toContain('备份历史')
    })

    it('shows backup tab by default', async () => {
      const wrapper = mount(BackupModal)

      await nextTick()

      expect(wrapper.html()).toContain('备份说明')
      expect(wrapper.html()).toContain('创建备份')
    })

    it('switches to restore tab when clicked', async () => {
      const wrapper = mount(BackupModal)

      await nextTick()

      const tabsContainer = wrapper.findAll('.flex.border-b')[1]
      const tabs = tabsContainer.findAll('button')
      await tabs[1].trigger('click')

      await nextTick()

      expect(wrapper.html()).toContain('恢复说明')
      expect(wrapper.html()).toContain('选择备份文件')
    })

    it('switches to history tab when clicked', async () => {
      const wrapper = mount(BackupModal)

      await nextTick()

      const tabsContainer = wrapper.findAll('.flex.border-b')[1]
      const tabs = tabsContainer.findAll('button')
      await tabs[2].trigger('click')

      await nextTick()

      expect(wrapper.html()).toContain('暂无备份记录')
    })

    it('highlights active tab', async () => {
      const wrapper = mount(BackupModal)

      await nextTick()

      const tabsContainer = wrapper.findAll('.flex.border-b')[1]
      const tabs = tabsContainer.findAll('button')

      // First tab should be active
      expect(tabs[0].classes()).toContain('text-primary')
      expect(tabs[0].classes()).toContain('border-b-2')

      // Switch to second tab
      await tabs[1].trigger('click')
      await nextTick()

      // Second tab should now be active
      expect(tabs[1].classes()).toContain('text-primary')
      expect(tabs[1].classes()).toContain('border-b-2')
    })
  })

  describe('Backup Tab', () => {
    it('shows backup information section', async () => {
      const wrapper = mount(BackupModal)

      await nextTick()

      expect(wrapper.html()).toContain('备份将包含数据库和所有截图文件')
      expect(wrapper.html()).toContain('备份文件保存为 ZIP 格式')
    })

    it('has backup directory input', async () => {
      const wrapper = mount(BackupModal)

      await nextTick()

      const input = wrapper.find('input[type="text"]')
      expect(input.exists()).toBe(true)
      expect(input.attributes('placeholder')).toContain('默认')
    })

    it('has create backup button', async () => {
      const wrapper = mount(BackupModal)

      await nextTick()

      const buttons = wrapper.findAll('button')
      const createButton = buttons.find(b => b.text().includes('创建备份'))
      expect(createButton).toBeDefined()
    })

    it('disables create button while backing up', async () => {
      const wrapper = mount(BackupModal)
      const vm = wrapper.vm

      await nextTick()

      // Set backing up state
      vm.isBackingUp = true
      await nextTick()

      const buttons = wrapper.findAll('button')
      const createButton = buttons.find(b => b.text().includes('备份中'))
      expect(createButton).toBeDefined()
      expect(createButton.attributes('disabled')).toBeDefined()
    })

    it('shows backup result when available', async () => {
      const wrapper = mount(BackupModal)
      const vm = wrapper.vm

      await nextTick()

      vm.backupResult = {
        path: '/backups/backup-2026-03-18.zip',
        size_bytes: 1024000,
        record_count: 50,
        screenshot_count: 20
      }
      await nextTick()

      expect(wrapper.html()).toContain('备份成功')
      expect(wrapper.html()).toContain('/backups/backup-2026-03-18.zip')
    })

    it('calls selectBackupDir when select button is clicked', async () => {
      open.mockResolvedValue('/selected/backup/dir')

      const wrapper = mount(BackupModal)

      await nextTick()

      const selectButton = wrapper.findAll('button').find(b => b.text().includes('选择'))
      await selectButton.trigger('click')

      await nextTick()

      expect(open).toHaveBeenCalledWith({
        directory: true,
        multiple: false,
        title: '选择备份目录'
      })
    })

    it('creates backup when create button is clicked', async () => {
      setupInvokeMock({
        create_backup: {
          path: '/backups/test.zip',
          size_bytes: 1024,
          record_count: 10,
          screenshot_count: 5
        }
      })

      const wrapper = mount(BackupModal)

      await nextTick()

      // Find the create backup button (has w-full class, not a tab)
      const createButton = wrapper.findAll('button').find(b =>
        b.text().includes('创建备份') && b.classes().includes('w-full')
      )
      await createButton.trigger('click')

      await nextTick()

      expect(invoke).toHaveBeenCalledWith('create_backup', { backupDir: null })
    })
  })

  describe('Restore Tab', () => {
    it('shows restore information section', async () => {
      const wrapper = mount(BackupModal)

      await nextTick()

      const tabsContainer = wrapper.findAll('.flex.border-b')[1]
      const tabs = tabsContainer.findAll('button')
      await tabs[1].trigger('click')
      await nextTick()

      expect(wrapper.html()).toContain('选择要恢复的备份文件')
      expect(wrapper.html()).toContain('恢复前会自动备份当前数据')
    })

    it('has select backup file button', async () => {
      const wrapper = mount(BackupModal)

      await nextTick()

      const tabsContainer = wrapper.findAll('.flex.border-b')[1]
      const tabs = tabsContainer.findAll('button')
      await tabs[1].trigger('click')
      await nextTick()

      const selectFileButton = wrapper.findAll('button').find(b => b.text().includes('选择备份文件'))
      expect(selectFileButton).toBeDefined()
    })

    it('shows selected backup info when available', async () => {
      const wrapper = mount(BackupModal)
      const vm = wrapper.vm

      await nextTick()

      vm.activeTab = 'restore'
      vm.selectedBackup = {
        path: '/backups/test.zip',
        created_at: '2026-03-18T10:00:00Z',
        size_bytes: 2048,
        record_count: 25,
        screenshot_count: 10
      }
      await nextTick()

      expect(wrapper.html()).toContain('选择的备份')
      expect(wrapper.html()).toContain('25')
      expect(wrapper.html()).toContain('10')
    })

    it('shows confirm button when backup is selected', async () => {
      const wrapper = mount(BackupModal)
      const vm = wrapper.vm

      await nextTick()

      vm.activeTab = 'restore'
      vm.selectedBackup = {
        path: '/backups/test.zip',
        created_at: '2026-03-18T10:00:00Z',
        size_bytes: 2048,
        record_count: 25,
        screenshot_count: 10
      }
      await nextTick()

      const confirmButton = wrapper.findAll('button').find(b => b.text().includes('确认恢复'))
      expect(confirmButton).toBeDefined()
    })

    it('shows confirmation dialog when confirm button is clicked', async () => {
      const wrapper = mount(BackupModal)
      const vm = wrapper.vm

      await nextTick()

      vm.activeTab = 'restore'
      vm.selectedBackup = {
        path: '/backups/test.zip',
        created_at: '2026-03-18T10:00:00Z',
        size_bytes: 2048,
        record_count: 25,
        screenshot_count: 10
      }
      await nextTick()

      const confirmButton = wrapper.findAll('button').find(b => b.text().includes('确认恢复'))
      await confirmButton.trigger('click')
      await nextTick()

      expect(wrapper.html()).toContain('确认恢复')
      expect(wrapper.html()).toContain('恢复操作将用备份数据替换当前数据')
    })

    it('shows restore result when available', async () => {
      const wrapper = mount(BackupModal)
      const vm = wrapper.vm

      await nextTick()

      vm.activeTab = 'restore'
      vm.restoreResult = {
        record_count: 50,
        screenshot_count: 20,
        auto_backup_created: true
      }
      await nextTick()

      expect(wrapper.html()).toContain('恢复成功')
      expect(wrapper.html()).toContain('已创建数据回滚备份')
    })
  })

  describe('History Tab', () => {
    it('shows empty state when no backups', async () => {
      setupInvokeMock({ list_backups: [] })
      const wrapper = mount(BackupModal)

      await nextTick()

      const tabsContainer = wrapper.findAll('.flex.border-b')[1]
      const tabs = tabsContainer.findAll('button')
      await tabs[2].trigger('click')
      await nextTick()

      expect(wrapper.html()).toContain('暂无备份记录')
    })

    it('shows loading state while loading backups', async () => {
      invoke.mockImplementation(() => new Promise(resolve => setTimeout(() => resolve([]), 100)))
      const wrapper = mount(BackupModal)

      await nextTick()

      const vm = wrapper.vm
      vm.activeTab = 'history'
      vm.isLoadingBackups = true
      await nextTick()

      expect(wrapper.html()).toContain('加载中')
    })

    it('displays backup list', async () => {
      setupInvokeMock({
        list_backups: [
          {
            path: '/backups/backup1.zip',
            created_at: '2026-03-18T10:00:00Z',
            size_bytes: 1024,
            record_count: 10,
            screenshot_count: 5
          },
          {
            path: '/backups/backup2.zip',
            created_at: '2026-03-17T10:00:00Z',
            size_bytes: 2048,
            record_count: 20,
            screenshot_count: 8
          }
        ]
      })

      const wrapper = mount(BackupModal)

      await nextTick()
      await nextTick()

      const tabsContainer = wrapper.findAll('.flex.border-b')[1]
      const tabs = tabsContainer.findAll('button')
      await tabs[2].trigger('click')
      await nextTick()

      expect(wrapper.html()).toContain('10 条记录')
      expect(wrapper.html()).toContain('20 条记录')
    })

    it('has restore button for each backup', async () => {
      setupInvokeMock({
        list_backups: [
          {
            path: '/backups/backup1.zip',
            created_at: '2026-03-18T10:00:00Z',
            size_bytes: 1024,
            record_count: 10,
            screenshot_count: 5
          }
        ]
      })

      const wrapper = mount(BackupModal)

      await nextTick()
      await nextTick()

      const tabsContainer = wrapper.findAll('.flex.border-b')[1]
      const tabs = tabsContainer.findAll('button')
      await tabs[2].trigger('click')
      await nextTick()

      const restoreButtons = wrapper.findAll('button').filter(b => b.text().includes('恢复'))
      expect(restoreButtons.length).toBeGreaterThan(0)
    })

    it('has delete button for each backup', async () => {
      setupInvokeMock({
        list_backups: [
          {
            path: '/backups/backup1.zip',
            created_at: '2026-03-18T10:00:00Z',
            size_bytes: 1024,
            record_count: 10,
            screenshot_count: 5
          }
        ]
      })

      const wrapper = mount(BackupModal)

      await nextTick()
      await nextTick()

      const tabsContainer = wrapper.findAll('.flex.border-b')[1]
      const tabs = tabsContainer.findAll('button')
      await tabs[2].trigger('click')
      await nextTick()

      const deleteButtons = wrapper.findAll('button').filter(b => b.text().includes('删除'))
      expect(deleteButtons.length).toBeGreaterThan(0)
    })

    it('has refresh button', async () => {
      setupInvokeMock({ list_backups: [] })
      const wrapper = mount(BackupModal)

      await nextTick()

      const tabsContainer = wrapper.findAll('.flex.border-b')[1]
      const tabs = tabsContainer.findAll('button')
      await tabs[2].trigger('click')
      await nextTick()

      const refreshButton = wrapper.findAll('button').find(b => b.text().includes('刷新'))
      expect(refreshButton).toBeDefined()
    })

    it('loads backups on mount', async () => {
      setupInvokeMock({ list_backups: [] })

      mount(BackupModal)

      await nextTick()

      expect(invoke).toHaveBeenCalledWith('list_backups')
    })
  })

  describe('Helper Functions', () => {
    it('formats small sizes correctly', async () => {
      const wrapper = mount(BackupModal)
      const vm = wrapper.vm

      await nextTick()

      expect(vm.formatSize(500)).toBe('500 B')
      expect(vm.formatSize(1023)).toBe('1023 B')
    })

    it('formats KB sizes correctly', async () => {
      const wrapper = mount(BackupModal)
      const vm = wrapper.vm

      await nextTick()

      expect(vm.formatSize(1024)).toBe('1.0 KB')
      expect(vm.formatSize(1536)).toBe('1.5 KB')
      expect(vm.formatSize(1024 * 1023)).toBe('1023.0 KB')
    })

    it('formats MB sizes correctly', async () => {
      const wrapper = mount(BackupModal)
      const vm = wrapper.vm

      await nextTick()

      expect(vm.formatSize(1024 * 1024)).toBe('1.0 MB')
      expect(vm.formatSize(1024 * 1024 * 512)).toBe('512.0 MB')
    })

    it('formats GB sizes correctly', async () => {
      const wrapper = mount(BackupModal)
      const vm = wrapper.vm

      await nextTick()

      expect(vm.formatSize(1024 * 1024 * 1024)).toBe('1.0 GB')
      expect(vm.formatSize(1024 * 1024 * 1024 * 2)).toBe('2.0 GB')
    })

    it('formats valid ISO date strings', async () => {
      const wrapper = mount(BackupModal)
      const vm = wrapper.vm

      await nextTick()

      const result = vm.formatDate('2026-03-18T10:30:00Z')
      expect(result).toBeTruthy()
      // Should be a localized string
      expect(typeof result).toBe('string')
    })

    it('returns "Invalid Date" for invalid date strings (toLocaleString behavior)', async () => {
      const wrapper = mount(BackupModal)
      const vm = wrapper.vm

      await nextTick()

      // new Date('invalid-date') doesn't throw, it creates an Invalid Date
      const result = vm.formatDate('invalid-date')
      expect(result).toBe('Invalid Date')
    })
  })

  describe('Error Handling', () => {
    it('handles create backup error gracefully', async () => {
      const mockAlert = vi.spyOn(window, 'alert').mockImplementation(() => {})
      invoke.mockImplementation((cmd) => {
        if (cmd === 'list_backups') return Promise.resolve([])
        return Promise.reject(new Error('Backup failed'))
      })

      const wrapper = mount(BackupModal)

      await nextTick()

      const vm = wrapper.vm
      await vm.createBackup()

      expect(mockAlert).toHaveBeenCalledWith(expect.stringContaining('备份失败'))
      expect(vm.isBackingUp).toBe(false)

      mockAlert.mockRestore()
    })

    it('handles load backups error gracefully', async () => {
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
      invoke.mockImplementation((cmd) => {
        if (cmd === 'list_backups') return Promise.reject(new Error('Load failed'))
        return Promise.resolve(null)
      })

      const wrapper = mount(BackupModal)
      const vm = wrapper.vm

      await nextTick()

      await vm.loadBackups()

      expect(consoleSpy).toHaveBeenCalled()
      expect(vm.isLoadingBackups).toBe(false)

      consoleSpy.mockRestore()
    })

    it('handles restore error gracefully', async () => {
      const mockAlert = vi.spyOn(window, 'alert').mockImplementation(() => {})
      invoke.mockImplementation((cmd) => {
        if (cmd === 'list_backups') return Promise.resolve([])
        return Promise.reject(new Error('Restore failed'))
      })

      const wrapper = mount(BackupModal)
      const vm = wrapper.vm

      await nextTick()

      vm.selectedBackup = { path: '/test/backup.zip' }

      await vm.confirmRestore()

      expect(mockAlert).toHaveBeenCalledWith(expect.stringContaining('恢复失败'))
      expect(vm.isRestoring).toBe(false)

      mockAlert.mockRestore()
    })
  })
})