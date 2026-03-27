/**
 * Report Feature Actions
 * Unified Tauri IPC calls for report generation
 */

import { invoke } from '../../shared/api/tauri/client'
import { REPORT_COMMANDS } from '../../shared/api/tauri/commands'

export interface ReanalyzeResult {
  total: number
  success: number
  failed: number
  errors: string[]
}

export interface ReportActions {
  generateDailySummary(): Promise<string>
  generateWeeklyReport(): Promise<string>
  generateMonthlyReport(): Promise<string>
  generateMultilingualDailySummary(targetLang: string): Promise<string>
  reanalyzeTodayRecords(): Promise<ReanalyzeResult>
}

export const reportActions: ReportActions = {
  async generateDailySummary(): Promise<string> {
    return invoke<string>(REPORT_COMMANDS.GENERATE_DAILY_SUMMARY)
  },

  async generateWeeklyReport(): Promise<string> {
    return invoke<string>(REPORT_COMMANDS.GENERATE_WEEKLY_REPORT)
  },

  async generateMonthlyReport(): Promise<string> {
    return invoke<string>(REPORT_COMMANDS.GENERATE_MONTHLY_REPORT)
  },

  async generateMultilingualDailySummary(targetLang: string): Promise<string> {
    return invoke<string>(REPORT_COMMANDS.GENERATE_MULTILINGUAL_DAILY_SUMMARY, { targetLang })
  },

  async reanalyzeTodayRecords(): Promise<ReanalyzeResult> {
    return invoke<ReanalyzeResult>(REPORT_COMMANDS.REANALYZE_TODAY_RECORDS)
  },
}