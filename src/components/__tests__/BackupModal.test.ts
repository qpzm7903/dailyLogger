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

      // HTML encodes & as &amp;
      expect(wrapper.html()).toContain('Data Backup')
      expect(wrapper.find('h2').text()).toBe('Data Backup & Restore')
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
      expect(tabs[0].text()).toContain('Create Backup')
      expect(tabs[1].text()).toContain('Restore Data')
      expect(tabs[2].text()).toContain('Backup History')
    })

    it('shows backup tab by default', async () => {
      const wrapper = mount(BackupModal)

      await nextTick()

      expect(wrapper.html()).toContain('Backup Information')
      expect(wrapper.html()).toContain('Create Backup')
    })

    it('switches to restore tab when clicked', async () => {
      const wrapper = mount(BackupModal)

      await nextTick()

      const tabsContainer = wrapper.findAll('.flex.border-b')[1]
      const tabs = tabsContainer.findAll('button')
      await tabs[1].trigger('click')

      await nextTick()

      expect(wrapper.html()).toContain('Restore Information')
      expect(wrapper.html()).toContain('Select Backup File')
    })

    it('switches to history tab when clicked', async () => {
      const wrapper = mount(BackupModal)

      await nextTick()

      const tabsContainer = wrapper.findAll('.flex.border-b')[1]
      const tabs = tabsContainer.findAll('button')
      await tabs[2].trigger('click')

      await nextTick()

      expect(wrapper.html()).toContain('No backup records')
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

      expect(wrapper.html()).toContain('Backup includes database and all screenshots')
      expect(wrapper.html()).toContain('Backup file saved as ZIP format')
    })

    it('has backup directory input', async () => {
      const wrapper = mount(BackupModal)

      await nextTick()

      const input = wrapper.find('input[type="text"]')
      expect(input.exists()).toBe(true)
      expect(input.attributes('placeholder')).toContain('Default')
    })

    it('has create backup button', async () => {
      const wrapper = mount(BackupModal)

      await nextTick()

      const buttons = wrapper.findAll('button')
      const createButton = buttons.find(b => b.text().includes('Create Backup'))
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
      const createButton = buttons.find(b => b.text().includes('Backing up'))
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

      expect(wrapper.html()).toContain('Backup Successful')
      expect(wrapper.html()).toContain('/backups/backup-2026-03-18.zip')
    })

    it('calls selectBackupDir when select button is clicked', async () => {
      open.mockResolvedValue('/selected/backup/dir')

      const wrapper = mount(BackupModal)

      await nextTick()

      const selectButton = wrapper.findAll('button').find(b => b.text().includes('Select'))
      await selectButton.trigger('click')

      await nextTick()

      expect(open).toHaveBeenCalledWith({
        directory: true,
        multiple: false,
        title: 'Select'
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
        b.text().includes('Create Backup') && b.classes().includes('w-full')
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

      expect(wrapper.html()).toContain('Select a backup file to restore')
      expect(wrapper.html()).toContain('Current data will be auto-backed up before restore')
    })

    it('has select backup file button', async () => {
      const wrapper = mount(BackupModal)

      await nextTick()

      const tabsContainer = wrapper.findAll('.flex.border-b')[1]
      const tabs = tabsContainer.findAll('button')
      await tabs[1].trigger('click')
      await nextTick()

      const selectFileButton = wrapper.findAll('button').find(b => b.text().includes('Select Backup File'))
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

      expect(wrapper.html()).toContain('Selected Backup')
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

      const confirmButton = wrapper.findAll('button').find(b => b.text().includes('Confirm Restore'))
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

      const confirmButton = wrapper.findAll('button').find(b => b.text().includes('Confirm Restore'))
      await confirmButton.trigger('click')
      await nextTick()

      expect(wrapper.html()).toContain('Confirm Restore')
      expect(wrapper.html()).toContain('Restore will replace current data')
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

      expect(wrapper.html()).toContain('Restore Successful')
      expect(wrapper.html()).toContain('Rollback backup created')
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

      expect(wrapper.html()).toContain('No backup records')
    })

    it('shows loading state while loading backups', async () => {
      invoke.mockImplementation(() => new Promise(resolve => setTimeout(() => resolve([]), 100)))
      const wrapper = mount(BackupModal)

      await nextTick()

      const vm = wrapper.vm
      vm.activeTab = 'history'
      vm.isLoadingBackups = true
      await nextTick()

      expect(wrapper.html()).toContain('Loading')
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

      expect(wrapper.html()).toContain('10')
      expect(wrapper.html()).toContain('20')
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

      const restoreButtons = wrapper.findAll('button').filter(b => b.text().includes('Restore Data'))
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

      const deleteButtons = wrapper.findAll('button').filter(b => b.text().includes('Delete'))
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

      const refreshButton = wrapper.findAll('button').find(b => b.text().includes('Refresh'))
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

      expect(mockAlert).toHaveBeenCalledWith(expect.stringContaining('Backup failed'))
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

      expect(mockAlert).toHaveBeenCalledWith(expect.stringContaining('Restore failed'))
      expect(vm.isRestoring).toBe(false)

      mockAlert.mockRestore()
    })
  })
})