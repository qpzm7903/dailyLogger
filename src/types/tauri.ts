/**
 * Tauri IPC command types
 * Centralized type definitions for all Tauri backend communication
 */

// ============================================
// Settings Types
// ============================================

export interface Settings {
  api_base_url: string
  api_key: string
  model_name: string
  screenshot_interval: number
  summary_time: string
  obsidian_path: string
  auto_capture_enabled: boolean
  last_summary_path: string | null
  silence_detection_enabled?: boolean
  silence_threshold?: number
  window_filter_enabled?: boolean
  window_filter_mode?: 'whitelist' | 'blacklist'
  window_filter_list?: string[]
  multi_monitor_mode?: 'primary' | 'all'
  custom_prompt?: string
  default_obsidian_vault?: string
  obsidian_vaults?: ObsidianVault[]
}

export interface ObsidianVault {
  name: string
  path: string
  is_default?: boolean
}

export interface UpdateSettingsArgs {
  settings: Partial<Settings>
}

// ============================================
// Record Types
// ============================================

export interface LogRecord {
  id: number
  timestamp: string // RFC3339 format
  source_type: 'auto' | 'manual'
  content: string
  screenshot_path: string | null
  monitor_info?: string | null // JSON: MonitorInfo serialized
  tags?: string | null // JSON: Vec<String> serialized
  user_notes?: string | null // FEAT-005: User manual notes (#66)
}

// Alias for backward compatibility - prefer LogRecord to avoid conflict with TS built-in Record
export type Record = LogRecord

export interface CreateRecordArgs {
  timestamp: string
  source_type: 'auto' | 'manual'
  content: string
  screenshot_path?: string | null
}

export interface UpdateRecordArgs {
  id: number
  content: string
}

export interface DeleteRecordArgs {
  id: number
}

export interface GetRecordsArgs {
  start_date?: string
  end_date?: string
  source_type?: 'auto' | 'manual'
  limit?: number
  offset?: number
}

// ============================================
// Tag Types
// ============================================

export interface Tag {
  id: number
  name: string
  color: string
  category_id: number | null
}

export interface TagCategory {
  id: number
  name: string
  color: string
}

export interface CreateTagArgs {
  name: string
  color?: string
  category_id?: number | null
}

export interface UpdateTagArgs {
  id: number
  name?: string
  color?: string
  category_id?: number | null
}

export interface DeleteTagArgs {
  id: number
}

export interface AddTagToRecordArgs {
  record_id: number
  tag_id: number
}

export interface RemoveTagFromRecordArgs {
  record_id: number
  tag_id: number
}

// ============================================
// Summary Types
// ============================================

export interface GenerateSummaryArgs {
  start_date: string
  end_date: string
  summary_type: 'daily' | 'weekly' | 'monthly' | 'custom'
  output_path?: string
}

export interface SummaryResult {
  content: string
  output_path: string
}

// ============================================
// Screenshot Types
// ============================================

export interface CaptureScreenshotResult {
  screenshot_path: string
  timestamp: string
}

// ============================================
// Search Types
// ============================================

export interface SearchResult {
  records: Record[]
  total_count: number
}

export interface SearchRecordsArgs {
  query: string
  start_date?: string
  end_date?: string
  limit?: number
  offset?: number
}

// ============================================
// Backup Types
// ============================================

export interface ExportDataArgs {
  format: 'json' | 'csv'
  start_date?: string
  end_date?: string
}

export interface BackupResult {
  path: string
  filename: string
}

// ============================================
// GitHub Integration Types
// ============================================

export interface GitHubStats {
  total_commits: number
  total_prs: number
  total_issues: number
  work_hours: number
  repositories: RepositoryStats[]
}

export interface RepositoryStats {
  name: string
  commits: number
  prs: number
  issues: number
}

export interface GetGitHubStatsArgs {
  start_date: string
  end_date: string
}

// ============================================
// Plugin Types
// ============================================

export interface PluginInfo {
  name: string
  version: string
  description: string
  enabled: boolean
}

// ============================================
// Notion Integration Types
// ============================================

export interface NotionDatabase {
  id: string
  title: string
}

export interface NotionPage {
  id: string
  title: string
  created_at: string
}

// ============================================
// Slack Integration Types
// ============================================

export interface SlackChannel {
  id: string
  name: string
  is_private: boolean
}

export interface SendToSlackArgs {
  channel_id: string
  content: string
}

// ============================================
// Ollama Types
// ============================================

export interface OllamaModel {
  name: string
  size: string
  modified_at: string
  details?: {
    format: string
    family: string
    parameter_size: string
    quantization_level: string
  }
}

export interface PullModelArgs {
  name: string
  quantization?: string
}

// ============================================
// Timeline Types
// ============================================

export interface TimelineEntry {
  timestamp: string
  content: string
  source_type: 'auto' | 'manual'
  screenshot_path: string | null
  tags: Tag[]
}

export interface GetTimelineArgs {
  date: string
}

// ============================================
// Report Comparison Types
// ============================================

export interface CompareReportsArgs {
  period1_start: string
  period1_end: string
  period2_start: string
  period2_end: string
}

export interface ComparisonResult {
  period1_summary: string
  period2_summary: string
  comparison: string
}