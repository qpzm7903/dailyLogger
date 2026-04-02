/**
 * 测试数据工厂
 * 生成结构一致的测试数据，与 src/types/tauri.ts 类型定义保持同步
 */

import type {
  Settings,
  LogRecord,
  Tag,
  TagCategory,
  SearchResult,
  BackupResult,
  OllamaModel,
  TimelineEntry,
} from '../../../src/types/tauri';

// ============================================
// Settings Factory
// ============================================

export function createSettings(overrides: Partial<Settings> = {}): Settings {
  return {
    api_base_url: 'https://api.openai.com/v1',
    api_key: '',
    model_name: 'gpt-4o',
    screenshot_interval: 5,
    summary_time: '18:00',
    obsidian_path: '',
    auto_capture_enabled: false,
    last_summary_path: null,
    window_whitelist: '[]',
    window_blacklist: '[]',
    use_whitelist_only: false,
    capture_mode: 'primary',
    obsidian_vaults: '[]',
    custom_headers: '[]',
    proxy_enabled: false,
    onboarding_completed: true,
    ...overrides,
  };
}

// ============================================
// Record Factory
// ============================================

let recordIdCounter = 1;

export function createRecord(overrides: Partial<LogRecord> = {}): LogRecord {
  const id = overrides.id ?? recordIdCounter++;
  return {
    id,
    timestamp: new Date().toISOString(),
    source_type: 'manual',
    content: `Test record ${id}`,
    screenshot_path: null,
    ...overrides,
  };
}

export function createRecords(count: number, overrides: Partial<LogRecord> = {}): LogRecord[] {
  return Array.from({ length: count }, () => createRecord(overrides));
}

// ============================================
// Tag Factory
// ============================================

let tagIdCounter = 1;

export function createTag(overrides: Partial<Tag> = {}): Tag {
  const id = overrides.id ?? tagIdCounter++;
  return {
    id,
    name: `Tag ${id}`,
    color: '#3B82F6',
    category_id: null,
    ...overrides,
  };
}

export function createTags(names: string[]): Tag[] {
  return names.map((name, index) => createTag({ name, id: index + 1 }));
}

// ============================================
// Tag Category Factory
// ============================================

let categoryIdCounter = 1;

export function createTagCategory(overrides: Partial<TagCategory> = {}): TagCategory {
  const id = overrides.id ?? categoryIdCounter++;
  return {
    id,
    name: `Category ${id}`,
    color: '#10B981',
    ...overrides,
  };
}

// ============================================
// Search Result Factory
// ============================================

export function createSearchResult(records: LogRecord[] = [], totalCount?: number): SearchResult {
  return {
    records,
    total_count: totalCount ?? records.length,
  };
}

// ============================================
// Backup Factory
// ============================================

export function createBackupResult(overrides: Partial<BackupResult> = {}): BackupResult {
  return {
    path: '/backups/backup-2024-01-01.json',
    filename: 'backup-2024-01-01.json',
    ...overrides,
  };
}

// ============================================
// Ollama Model Factory
// ============================================

export function createOllamaModel(overrides: Partial<OllamaModel> = {}): OllamaModel {
  return {
    name: 'llama3.2-vision',
    size: '4.7 GB',
    modified_at: '2024-01-01T00:00:00Z',
    details: {
      format: 'gguf',
      family: 'llama',
      parameter_size: '3B',
      quantization_level: 'Q4_K_M',
    },
    ...overrides,
  };
}

// ============================================
// Timeline Factory
// ============================================

export function createTimelineEntry(overrides: Partial<TimelineEntry> = {}): TimelineEntry {
  return {
    timestamp: new Date().toISOString(),
    content: 'Timeline entry content',
    source_type: 'manual',
    screenshot_path: null,
    tags: [],
    ...overrides,
  };
}

// ============================================
// Network Status Factory
// ============================================

export function createNetworkStatus(online: boolean = true) {
  return { online };
}

// ============================================
// Platform Info Factory
// ============================================

export function createPlatformInfo() {
  return {
    os: 'macos',
    arch: 'aarch64',
    version: '14.0',
  };
}

// ============================================
// Auto Capture Status Factory
// ============================================

export function createAutoCaptureStatus(running: boolean = false) {
  return { running };
}

// ============================================
// Offline Queue Status Factory
// ============================================

export function createOfflineQueueStatus() {
  return {
    pending_count: 0,
    oldest_pending_age: null,
  };
}

// ============================================
// Monitor Factory
// ============================================

export function createMonitors() {
  return [
    { id: 1, name: 'Primary', width: 1920, height: 1080, is_primary: true },
  ];
}

// Reset all counters (useful for test isolation)
export function resetFactories() {
  recordIdCounter = 1;
  tagIdCounter = 1;
  categoryIdCounter = 1;
}
