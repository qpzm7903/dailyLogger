/**
 * Tauri IPC command types
 * Centralized type definitions for all Tauri backend communication
 */

// ============================================
// Settings Types
// ============================================

export interface Settings {
  // Basic API settings
  api_base_url: string
  api_key: string
  model_name: string
  screenshot_interval: number
  summary_time: string
  obsidian_path: string
  auto_capture_enabled: boolean
  last_summary_path: string | null

  // Model and prompts (AI-004, AI-005)
  summary_model_name?: string
  analysis_prompt?: string
  summary_prompt?: string
  is_ollama?: boolean
  custom_headers?: string // JSON: CustomHeader[]

  // Capture behavior (SMART-001, SMART-002, SMART-004)
  change_threshold?: number
  max_silent_minutes?: number
  window_whitelist?: string // JSON: Vec<String>
  window_blacklist?: string // JSON: Vec<String>
  use_whitelist_only?: boolean
  auto_adjust_silent?: boolean
  silent_adjustment_paused_until?: string
  capture_mode?: string // "primary" | "secondary" | "all"
  selected_monitor_index?: number

  // Work time detection (SMART-003)
  auto_detect_work_time?: boolean
  use_custom_work_time?: boolean
  custom_work_time_start?: string // "HH:MM"
  custom_work_time_end?: string // "HH:MM"
  learned_work_time?: string // JSON: {"periods": [...]}

  // Session management (SESSION-001)
  session_gap_minutes?: number

  // Tags (AI-004)
  tag_categories?: string // JSON: Vec<String>

  // Report generation (REPORT-001, REPORT-002, REPORT-003, REPORT-004)
  weekly_report_prompt?: string
  weekly_report_day?: number // 0=周一, 6=周日
  last_weekly_report_path?: string
  monthly_report_prompt?: string
  last_monthly_report_path?: string
  custom_report_prompt?: string
  last_custom_report_path?: string
  comparison_report_prompt?: string

  // Obsidian integration (DATA-006, VAULT-001)
  obsidian_vaults?: string // JSON: ObsidianVault[]
  auto_detect_vault_by_window?: boolean

  // Feature flags
  capture_only_mode?: boolean // FEAT-006
  quality_filter_enabled?: boolean // EXP-002
  quality_filter_threshold?: number
  summary_title_format?: string
  include_manual_records?: boolean

  // Proxy (PERF-001)
  proxy_enabled?: boolean
  proxy_host?: string
  proxy_port?: number
  proxy_username?: string
  proxy_password?: string
  test_model_name?: string

  // Backup (STAB-002)
  auto_backup_enabled?: boolean
  auto_backup_interval?: string // "daily" | "weekly" | "monthly"
  auto_backup_retention?: number
  last_auto_backup_at?: string

  // Export template (FEAT-008: v3.8.0)
  custom_export_template?: string

  // User preferences (PERF-002, PERF-005)
  onboarding_completed?: boolean
  language?: string
  preferred_language?: string
  supported_languages?: string
}

export interface ObsidianVault {
  name: string
  path: string
  is_default?: boolean
  window_patterns?: string[]
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
  // SESSION-001: Session association and analysis status
  session_id?: number | null
  analysis_status?: 'pending' | 'analyzed' | 'user_edited' | null
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

// ============================================
// Statistics Types (DATA-008)
// ============================================

export interface DateRange {
  start: string  // RFC3339
  end: string    // RFC3339
  label: string  // "今日" / "本周" / "本月" / "自定义"
}

export interface DailyStatistic {
  date: string           // YYYY-MM-DD
  screenshot_count: number
  session_count: number
  record_count: number
}

export interface Statistics {
  date_range: DateRange
  screenshot_count: number
  session_count: number
  record_count: number
  analysis_success_rate: number  // Percentage 0-100
  daily_breakdown: DailyStatistic[]
}

export type TimeRangeType = 'today' | 'week' | 'month' | 'custom'

export interface GetStatisticsArgs {
  range_type: TimeRangeType
  custom_start?: string  // YYYY-MM-DD, required when range_type is 'custom'
  custom_end?: string    // YYYY-MM-DD, required when range_type is 'custom'
}

// ============================================
// Productivity Trend Types (ANALYTICS-001)
// ============================================

export interface PeriodComparison {
  current_total: number
  previous_total: number
  change_percent: number  // Percentage change, positive = increase
  trend: 'up' | 'down' | 'stable'
}

export interface HourlyDistribution {
  hour: number       // 0-23
  count: number      // Number of records in this hour
  percentage: number // Percentage of total
}

export interface DailyTrendPoint {
  date: string           // YYYY-MM-DD
  screenshot_count: number
  record_count: number
}

export interface ProductivityTrend {
  comparison_type: 'week' | 'month'
  current_period: DateRange
  previous_period: DateRange
  screenshot_comparison: PeriodComparison
  record_comparison: PeriodComparison
  daily_trend: DailyTrendPoint[]      // Daily data for current period
  peak_hours: HourlyDistribution[]    // Top 5 busiest hours
  average_daily_records: number       // Average records per day
}

export interface GetProductivityTrendArgs {
  comparison_type: 'week' | 'month'
}
