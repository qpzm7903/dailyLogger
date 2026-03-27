/**
 * Sessions Feature Actions
 * Unified Tauri IPC calls for session management
 */

import { invoke } from '../../shared/api/tauri/client'
import { SESSION_COMMANDS } from '../../shared/api/tauri/commands'

// Session type (should match the type defined in SessionListModal.vue)
export interface Session {
  id: number
  date: string
  start_time: string
  end_time: string | null
  ai_summary: string | null
  user_summary: string | null
  context_for_next: string | null
  status: 'active' | 'ended' | 'analyzed'
  screenshot_count?: number
}

export interface SessionActions {
  getTodaySessions(): Promise<Session[]>
  analyzeSession(sessionId: number): Promise<void>
  updateSessionUserSummary(sessionId: number, summary: string): Promise<void>
}

export const sessionActions: SessionActions = {
  async getTodaySessions(): Promise<Session[]> {
    return invoke<Session[]>(SESSION_COMMANDS.GET_TODAY_SESSIONS)
  },

  async analyzeSession(sessionId: number): Promise<void> {
    await invoke(SESSION_COMMANDS.ANALYZE_SESSION, { sessionId })
  },

  async updateSessionUserSummary(sessionId: number, summary: string): Promise<void> {
    await invoke(SESSION_COMMANDS.UPDATE_SESSION_USER_SUMMARY, { sessionId, userSummary: summary })
  },
}