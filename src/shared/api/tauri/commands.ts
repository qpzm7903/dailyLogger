/**
 * Tauri IPC Command Names
 * Centralized command name constants to avoid hardcoded strings
 */

// Capture commands
export const CAPTURE_COMMANDS = {
  TAKE_SCREENSHOT: 'take_screenshot',
  TRIGGER_CAPTURE: 'trigger_capture',
  START_AUTO_CAPTURE: 'start_auto_capture',
  STOP_AUTO_CAPTURE: 'stop_auto_capture',
  GET_SCREENSHOT: 'get_screenshot',
  REANALYZE_RECORD: 'reanalyze_record',
} as const

// Record commands
export const RECORD_COMMANDS = {
  GET_TODAY_RECORDS: 'get_today_records',
  DELETE_RECORD: 'delete_record',
  UPDATE_RECORD_USER_NOTES: 'update_record_user_notes',
  ADD_QUICK_NOTE: 'add_quick_note',
} as const

// Tag commands
export const TAG_COMMANDS = {
  ADD_TAG_TO_RECORD: 'add_tag_to_record',
  REMOVE_TAG_FROM_RECORD: 'remove_tag_from_record',
  DELETE_MANUAL_TAG: 'delete_manual_tag',
} as const

// Session commands
export const SESSION_COMMANDS = {
  GET_TODAY_SESSIONS: 'get_today_sessions',
  ANALYZE_SESSION: 'analyze_session',
  UPDATE_SESSION_USER_SUMMARY: 'update_session_user_summary',
} as const

// Report commands
export const REPORT_COMMANDS = {
  GENERATE_DAILY_SUMMARY: 'generate_daily_summary',
  GENERATE_WEEKLY_REPORT: 'generate_weekly_report',
  GENERATE_MONTHLY_REPORT: 'generate_monthly_report',
  GENERATE_MULTILINGUAL_DAILY_SUMMARY: 'generate_multilingual_daily_summary',
  REANALYZE_TODAY_RECORDS: 'reanalyze_today_records',
} as const

// Settings commands
export const SETTINGS_COMMANDS = {
  GET_SETTINGS: 'get_settings',
  SAVE_SETTINGS: 'save_settings',
  GET_DEFAULT_ANALYSIS_PROMPT: 'get_default_analysis_prompt',
  GET_DEFAULT_SUMMARY_PROMPT: 'get_default_summary_prompt',
  GET_DEFAULT_TAG_CATEGORIES: 'get_default_tag_categories',
} as const

// System commands
export const SYSTEM_COMMANDS = {
  CHECK_NETWORK_STATUS: 'check_network_status',
  GET_OFFLINE_QUEUE_STATUS: 'get_offline_queue_status',
  EXPORT_LOGS: 'export_logs',
  OPEN_EXPORT_DIR: 'open_export_dir',
  DELETE_BACKUP: 'delete_backup',
  TRIGGER_AUTO_BACKUP: 'trigger_auto_backup',
  LOG_FRONTEND_ERROR: 'log_frontend_error',
  TRAY_QUICK_NOTE: 'tray_quick_note',
} as const

// All commands combined
export const COMMANDS = {
  ...CAPTURE_COMMANDS,
  ...RECORD_COMMANDS,
  ...TAG_COMMANDS,
  ...SESSION_COMMANDS,
  ...REPORT_COMMANDS,
  ...SETTINGS_COMMANDS,
  ...SYSTEM_COMMANDS,
} as const

export type CommandName = typeof COMMANDS[keyof typeof COMMANDS]