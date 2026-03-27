/**
 * Records Feature Actions
 * Unified Tauri IPC calls for record management
 */

import { invoke } from '../../shared/api/tauri/client'
import { RECORD_COMMANDS, TAG_COMMANDS } from '../../shared/api/tauri/commands'
import type { LogRecord } from '../../types/tauri'

export interface RecordsActions {
  getTodayRecords(): Promise<LogRecord[]>
  deleteRecord(id: number): Promise<void>
  updateRecordUserNotes(id: number, notes: string | null): Promise<void>
}

export const recordsActions: RecordsActions = {
  async getTodayRecords(): Promise<LogRecord[]> {
    return invoke<LogRecord[]>(RECORD_COMMANDS.GET_TODAY_RECORDS)
  },

  async deleteRecord(id: number): Promise<void> {
    await invoke(RECORD_COMMANDS.DELETE_RECORD, { id })
  },

  async updateRecordUserNotes(id: number, notes: string | null): Promise<void> {
    await invoke(RECORD_COMMANDS.UPDATE_RECORD_USER_NOTES, { id, userNotes: notes })
  },
}

// Tag actions
export interface TagActions {
  addTagToRecord(recordId: number, tagId: number): Promise<void>
  removeTagFromRecord(recordId: number, tagId: number): Promise<void>
  deleteManualTag(id: number): Promise<void>
}

export const tagActions: TagActions = {
  async addTagToRecord(recordId: number, tagId: number): Promise<void> {
    await invoke(TAG_COMMANDS.ADD_TAG_TO_RECORD, { recordId, tagId })
  },

  async removeTagFromRecord(recordId: number, tagId: number): Promise<void> {
    await invoke(TAG_COMMANDS.REMOVE_TAG_FROM_RECORD, { recordId, tagId })
  },

  async deleteManualTag(id: number): Promise<void> {
    await invoke(TAG_COMMANDS.DELETE_MANUAL_TAG, { id })
  },
}