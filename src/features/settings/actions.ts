/**
 * Settings Feature Actions
 * Unified Tauri IPC calls for settings management
 */

import { invoke } from '../../shared/api/tauri/client'
import { SETTINGS_COMMANDS } from '../../shared/api/tauri/commands'
import type { Settings } from '../../types/tauri'

export interface SettingsActions {
  getSettings(): Promise<Settings>
  saveSettings(settings: Partial<Settings>): Promise<void>
  getDefaultAnalysisPrompt(): Promise<string>
  getDefaultSummaryPrompt(): Promise<string>
  getDefaultTagCategories(): Promise<string>
}

export const settingsActions: SettingsActions = {
  async getSettings(): Promise<Settings> {
    return invoke<Settings>(SETTINGS_COMMANDS.GET_SETTINGS)
  },

  async saveSettings(settings: Partial<Settings>): Promise<void> {
    await invoke(SETTINGS_COMMANDS.SAVE_SETTINGS, { settings })
  },

  async getDefaultAnalysisPrompt(): Promise<string> {
    return invoke<string>(SETTINGS_COMMANDS.GET_DEFAULT_ANALYSIS_PROMPT)
  },

  async getDefaultSummaryPrompt(): Promise<string> {
    return invoke<string>(SETTINGS_COMMANDS.GET_DEFAULT_SUMMARY_PROMPT)
  },

  async getDefaultTagCategories(): Promise<string> {
    return invoke<string>(SETTINGS_COMMANDS.GET_DEFAULT_TAG_CATEGORIES)
  },
}