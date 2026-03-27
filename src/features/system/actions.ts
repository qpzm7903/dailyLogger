/**
 * System Feature Actions
 * Unified Tauri IPC calls for system-level operations
 */

import { invoke } from '../../shared/api/tauri/client'
import { SYSTEM_COMMANDS } from '../../shared/api/tauri/commands'

export interface NetworkStatus {
  pending_count: number
}

export interface SystemActions {
  checkNetworkStatus(): Promise<boolean>
  getOfflineQueueStatus(): Promise<NetworkStatus>
  exportLogs(): Promise<void>
  openExportDir(path: string): Promise<void>
  deleteBackup(backupPath: string): Promise<void>
  triggerAutoBackup(): Promise<void>
  logFrontendError(message: string, stack?: string): Promise<void>
  trayQuickNote(content: string): Promise<void>
}

export const systemActions: SystemActions = {
  async checkNetworkStatus(): Promise<boolean> {
    return invoke<boolean>(SYSTEM_COMMANDS.CHECK_NETWORK_STATUS)
  },

  async getOfflineQueueStatus(): Promise<NetworkStatus> {
    return invoke<NetworkStatus>(SYSTEM_COMMANDS.GET_OFFLINE_QUEUE_STATUS)
  },

  async exportLogs(): Promise<void> {
    await invoke(SYSTEM_COMMANDS.EXPORT_LOGS)
  },

  async openExportDir(path: string): Promise<void> {
    await invoke(SYSTEM_COMMANDS.OPEN_EXPORT_DIR, { path })
  },

  async deleteBackup(backupPath: string): Promise<void> {
    await invoke(SYSTEM_COMMANDS.DELETE_BACKUP, { backupPath })
  },

  async triggerAutoBackup(): Promise<void> {
    await invoke(SYSTEM_COMMANDS.TRIGGER_AUTO_BACKUP)
  },

  async logFrontendError(message: string, stack?: string): Promise<void> {
    await invoke(SYSTEM_COMMANDS.LOG_FRONTEND_ERROR, { message, stack })
  },

  async trayQuickNote(content: string): Promise<void> {
    await invoke(SYSTEM_COMMANDS.TRAY_QUICK_NOTE, { content })
  },
}