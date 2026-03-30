/**
 * Capture Feature Actions
 * Unified Tauri IPC calls for capture-related functionality
 */

import { invoke } from '../../shared/api/tauri/client'
import { CAPTURE_COMMANDS, RECORD_COMMANDS } from '../../shared/api/tauri/commands'
import type { LogRecord } from '../../types/tauri'

export interface ScreenAnalysis {
  current_focus?: string
  active_software?: string
  context_keywords?: string[]
  active_window?: { title?: string; process_name?: string }
  tags?: string[]
}

export interface CaptureActions {
  takeScreenshot(): Promise<string>
  triggerCapture(): Promise<void>
  startAutoCapture(): Promise<void>
  stopAutoCapture(): Promise<void>
  getScreenshot(path: string): Promise<string>
  reanalyzeRecord(recordId: number): Promise<ScreenAnalysis>
}

export const captureActions: CaptureActions = {
  async takeScreenshot(): Promise<string> {
    return invoke<string>(CAPTURE_COMMANDS.TAKE_SCREENSHOT)
  },

  async triggerCapture(): Promise<void> {
    await invoke(CAPTURE_COMMANDS.TRIGGER_CAPTURE)
  },

  async startAutoCapture(): Promise<void> {
    await invoke(CAPTURE_COMMANDS.START_AUTO_CAPTURE)
  },

  async stopAutoCapture(): Promise<void> {
    await invoke(CAPTURE_COMMANDS.STOP_AUTO_CAPTURE)
  },

  async getScreenshot(path: string): Promise<string> {
    return invoke<string>(CAPTURE_COMMANDS.GET_SCREENSHOT, { path })
  },

  async reanalyzeRecord(recordId: number): Promise<ScreenAnalysis> {
    return invoke<ScreenAnalysis>(CAPTURE_COMMANDS.REANALYZE_RECORD, { recordId })
  },
}

// Quick note action (related to capture)
export async function addQuickNote(content: string): Promise<void> {
  await invoke(RECORD_COMMANDS.ADD_QUICK_NOTE, { content })
}