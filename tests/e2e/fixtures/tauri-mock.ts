/**
 * Tauri IPC Mock 框架
 * 在浏览器环境中模拟 Tauri 的 invoke() 和 listen() API
 */

import type {
  Settings,
  LogRecord,
  Tag,
  SearchResult,
  BackupResult,
  TimelineEntry,
} from '../../../src/types/tauri';
import * as Factory from './test-data';

// ============================================
// 类型定义
// ============================================

export type MockHandler = (args?: unknown) => unknown;
export type MockOverrides = Record<string, MockHandler>;

interface TauriInternals {
  invoke: (cmd: string, args?: unknown) => Promise<unknown>;
  listen: (event: string, callback: (event: unknown) => void) => Promise<() => void>;
}

// ============================================
// Mock 存储
// ============================================

// 存储测试级 mock 覆盖
let mockOverrides: MockOverrides = {};

// 存储 mock 数据状态（用于模拟 CRUD 操作）
interface MockState {
  settings: Settings;
  records: LogRecord[];
  tags: Tag[];
  backups: BackupResult[];
}

let mockState: MockState = {
  settings: Factory.createSettings(),
  records: [],
  tags: [],
  backups: [],
};

// ============================================
// 默认 Mock 处理器
// ============================================

const DEFAULT_MOCKS: Record<string, (args?: unknown) => unknown> = {
  // Settings
  get_settings: () => mockState.settings,
  save_settings: (args) => {
    const { settings } = args as { settings: Partial<Settings> };
    mockState.settings = { ...mockState.settings, ...settings };
    return 'ok';
  },

  // Records
  get_today_records: () => mockState.records.filter(r => {
    const today = new Date().toDateString();
    return new Date(r.timestamp).toDateString() === today;
  }),
  get_records_by_date_range: (args) => {
    const { start_date, end_date } = args as { start_date?: string; end_date?: string };
    let filtered = [...mockState.records];
    if (start_date) {
      filtered = filtered.filter(r => r.timestamp >= start_date);
    }
    if (end_date) {
      filtered = filtered.filter(r => r.timestamp <= end_date);
    }
    return filtered;
  },
  get_history_records: () => mockState.records,
  search_records: (args) => {
    const { query } = args as { query: string };
    const results = mockState.records.filter(r =>
      r.content.toLowerCase().includes(query.toLowerCase())
    );
    return { records: results, total_count: results.length } as SearchResult;
  },
  delete_record: (args) => {
    const { id } = args as { id: number };
    mockState.records = mockState.records.filter(r => r.id !== id);
    return 'ok';
  },

  // Tags
  get_all_tags: () => [],
  get_all_manual_tags: () => mockState.tags,
  create_manual_tag: (args) => {
    const { name, color } = args as { name: string; color?: string };
    const tag = Factory.createTag({ name, color: color || '#3B82F6' });
    mockState.tags.push(tag);
    return tag;
  },
  update_manual_tag: (args) => {
    const { id, name, color } = args as { id: number; name?: string; color?: string };
    const tag = mockState.tags.find(t => t.id === id);
    if (tag) {
      if (name) tag.name = name;
      if (color) tag.color = color;
    }
    return 'ok';
  },
  delete_manual_tag: (args) => {
    const { id } = args as { id: number };
    mockState.tags = mockState.tags.filter(t => t.id !== id);
    return 'ok';
  },
  get_tags_for_record: () => [],
  get_tags_for_records: () => ({}),
  get_records_by_manual_tags: () => [],
  add_tag_to_record: () => 'ok',
  remove_tag_from_record: () => 'ok',
  get_records_by_tag: () => [],
  get_default_tag_categories: () => [],

  // Manual Entry
  add_quick_note: (args) => {
    const { content, timestamp } = args as { content: string; timestamp?: string };
    const record = Factory.createRecord({
      content,
      source_type: 'manual',
      timestamp: timestamp || new Date().toISOString(),
    });
    mockState.records.push(record);
    return record;
  },
  get_screenshot: () => null,
  read_file: () => '',
  get_recent_logs: () => [],
  get_logs_for_export: () => [],
  get_log_file_path: () => '/path/to/log',
  open_obsidian_folder: () => 'ok',
  list_report_files: () => [],
  tray_quick_note: () => 'ok',

  // Summary
  generate_daily_summary: () => ({
    content: '# Daily Summary\n\nGenerated summary content.',
    output_path: '/path/to/summary.md',
  }),
  get_default_summary_prompt: () => 'Default summary prompt',
  generate_weekly_report: () => ({
    content: '# Weekly Report\n\nGenerated report content.',
    output_path: '/path/to/report.md',
  }),
  generate_monthly_report: () => ({
    content: '# Monthly Report\n\nGenerated report content.',
    output_path: '/path/to/report.md',
  }),
  generate_custom_report: () => ({
    content: '# Custom Report\n\nGenerated report content.',
    output_path: '/path/to/report.md',
  }),
  compare_reports: () => ({
    period1_summary: 'Period 1 summary',
    period2_summary: 'Period 2 summary',
    comparison: 'Comparison analysis',
  }),

  // Export
  export_records: () => ({
    path: '/exports/export.json',
    filename: 'export.json',
  }),
  open_export_dir: () => 'ok',

  // Backup
  create_backup: () => {
    const backup = Factory.createBackupResult();
    mockState.backups.push(backup);
    return backup;
  },
  get_backup_info: () => ({
    total_backups: mockState.backups.length,
    total_size_mb: 1.5,
  }),
  list_backups: () => mockState.backups,
  delete_backup: (args) => {
    const { filename } = args as { filename: string };
    mockState.backups = mockState.backups.filter(b => b.filename !== filename);
    return 'ok';
  },
  restore_backup: () => 'ok',

  // Ollama
  get_ollama_models: () => [],
  pull_ollama_model: () => 'ok',
  delete_ollama_model: () => 'ok',
  get_running_models: () => [],
  create_ollama_model: () => 'ok',
  copy_ollama_model: () => 'ok',
  show_ollama_model: () => ({ license: '', modelfile: '', parameters: '', template: '' }),
  test_api_connection_with_ollama: () => true,

  // Network
  get_network_status: () => Factory.createNetworkStatus(true),
  check_network_status: () => Factory.createNetworkStatus(true),

  // Performance
  get_platform_info: () => Factory.createPlatformInfo(),
  get_memory_usage_mb: () => 100,
  benchmark_database_query: () => ({ duration_ms: 5 }),
  run_performance_benchmark: () => ({ score: 100 }),

  // Auto Capture (screenshot feature - not available in browser)
  start_auto_capture: () => 'ok',
  stop_auto_capture: () => 'ok',
  trigger_capture: () => null,
  take_screenshot: () => null,
  reanalyze_record: () => 'ok',
  get_default_analysis_prompt: () => 'Default analysis prompt',
  get_auto_capture_status: () => Factory.createAutoCaptureStatus(false),
  get_work_time_status: () => ({ work_minutes: 0, is_active: false }),
  get_monitors: () => Factory.createMonitors(),

  // Timeline
  get_timeline_today: () => [] as TimelineEntry[],
  get_timeline_for_date: () => [] as TimelineEntry[],
  get_timeline_for_range: () => [] as TimelineEntry[],

  // Plugins
  list_discovered_plugins: () => [],
  enable_plugin: () => 'ok',
  disable_plugin: () => 'ok',
  open_plugins_directory: () => 'ok',

  // Integrations
  test_notion_connection: () => true,
  test_github_connection: () => true,
  test_slack_connection: () => true,

  // Model Info
  get_model_info: () => ({
    analysis_model: 'gpt-4o',
    report_model: 'gpt-4o',
  }),

  // Offline Queue
  get_offline_queue_status: () => Factory.createOfflineQueueStatus(),
  process_offline_queue: () => 'ok',

  // Fine Tuning
  prepare_training_data: () => '/path/to/training.jsonl',
  start_fine_tuning: () => 'job-123',
  get_default_fine_tuning_config: () => ({
    model: 'gpt-3.5-turbo',
    n_epochs: 3,
    batch_size: 4,
  }),
};

// ============================================
// Mock 实现
// ============================================

function createMockInvoke(): (cmd: string, args?: unknown) => Promise<unknown> {
  return async (cmd: string, args?: unknown): Promise<unknown> => {
    // 优先使用测试级覆盖
    if (mockOverrides[cmd]) {
      return mockOverrides[cmd](args);
    }
    // 使用默认 mock
    if (DEFAULT_MOCKS[cmd]) {
      return DEFAULT_MOCKS[cmd](args);
    }
    // 未知命令返回 null
    console.warn(`[Tauri Mock] Unknown command: ${cmd}`);
    return null;
  };
}

function createMockListen(): (event: string, callback: (event: unknown) => void) => Promise<() => void> {
  return async (_event: string, _callback: (event: unknown) => void) => {
    // 返回 unlisten 函数
    return () => {};
  };
}

// ============================================
// Mock 控制 API
// ============================================

/**
 * 设置测试级 mock 覆盖
 */
export function setMockOverrides(overrides: MockOverrides) {
  mockOverrides = { ...overrides };
}

/**
 * 清除测试级 mock 覆盖
 */
export function clearMockOverrides() {
  mockOverrides = {};
}

/**
 * 重置 mock 状态（每个测试前调用）
 */
export function resetMockState(initialState?: Partial<MockState>) {
  mockState = {
    settings: initialState?.settings ?? Factory.createSettings(),
    records: initialState?.records ?? [],
    tags: initialState?.tags ?? [],
    backups: initialState?.backups ?? [],
  };
  mockOverrides = {};
  Factory.resetFactories();
}

/**
 * 获取当前 mock 状态（用于测试断言）
 */
export function getMockState(): Readonly<MockState> {
  return mockState;
}

/**
 * 直接修改 mock 状态
 */
export function setMockState(state: Partial<MockState>) {
  mockState = { ...mockState, ...state };
}

// ============================================
// 注入脚本生成
// ============================================

/**
 * 生成注入到页面的 mock 脚本
 * 通过 page.addInitScript() 使用
 */
export function getMockInjectionScript(overrides: MockOverrides = {}): string {
  return `
    (function() {
      // 存储状态
      // 确保 overrides 是对象（处理 undefined 的情况）
      const mockOverrides = ${JSON.stringify(overrides || {})};
      const mockState = {
        settings: ${JSON.stringify(Factory.createSettings())},
        records: [],
        tags: [],
        backups: [],
        eventListeners: {},
      };

      // 回调计数器
      let callbackId = 0;

      // 创建 mock __TAURI_INTERNALS__
      window.__TAURI_INTERNALS__ = {
        // invoke 用于调用后端命令
        invoke: async (cmd, args) => {
          console.log('[Tauri Mock]', cmd, args);

          // 默认 mock 实现
          const defaults = {
            get_settings: () => {
              // 确保 onboarding_completed 为 true，避免显示 onboarding 弹窗
              return { ...mockState.settings, onboarding_completed: true };
            },
            save_settings: (args) => {
              Object.assign(mockState.settings, args.settings || {});
              return 'ok';
            },
            get_today_records: () => mockState.records,
            get_history_records: () => mockState.records,
            search_records: () => ({ records: [], total_count: 0 }),
            get_all_manual_tags: () => mockState.tags,
            create_manual_tag: (args) => {
              const tag = { id: Date.now(), name: args.name, color: args.color || '#3B82F6', category_id: null };
              mockState.tags.push(tag);
              return tag;
            },
            get_network_status: () => ({ online: true }),
            check_network_status: () => ({ online: true }),
            get_platform_info: () => ({ os: 'macos', arch: 'aarch64', version: '14.0' }),
            get_auto_capture_status: () => ({ running: false }),
            get_offline_queue_status: () => ({ pending_count: 0, oldest_pending_age: null }),
            add_quick_note: (args) => {
              const record = {
                id: Date.now(),
                timestamp: new Date().toISOString(),
                source_type: 'manual',
                content: args.content,
                screenshot_path: null
              };
              mockState.records.push(record);
              return record;
            },
            delete_record: (args) => {
              mockState.records = mockState.records.filter(r => r.id !== args.id);
              return 'ok';
            },
            generate_daily_summary: () => ({ content: 'Summary', output_path: '/tmp/summary.md' }),
            get_today_stats: () => ({
              total_count: 0,
              auto_count: 0,
              manual_count: 0,
              first_record_time: null,
              latest_record_time: null,
              busiest_hour: null,
              busiest_hour_count: 0,
            }),
            list_backups: () => mockState.backups,
            create_backup: () => {
              const backup = { path: '/backups/backup.json', filename: 'backup.json' };
              mockState.backups.push(backup);
              return backup;
            },
            get_timeline_today: () => [],
            get_timeline_for_date: () => [],
            list_discovered_plugins: () => [],
            get_model_info: () => ({ analysis_model: 'gpt-4o', report_model: 'gpt-4o' }),
            get_all_tags: () => [],
            get_tag_colors: () => ({}),
            log_frontend_error: () => 'ok',
            reanalyze_record: () => 'ok',
            get_monitors: () => [{ id: 1, name: 'Primary', width: 1920, height: 1080, is_primary: true }],
            get_default_analysis_prompt: () => 'Default analysis prompt',
            get_default_summary_prompt: () => 'Default summary prompt',
            get_tags_for_record: () => [],
            get_tags_for_records: () => ({}),
            get_records_by_manual_tags: () => [],
            get_default_tag_categories: () => [],
            list_discovered_plugins: () => [],
            get_model_info: () => ({ analysis_model: 'gpt-4o', report_model: 'gpt-4o' }),
            'plugin:event|listen': () => (() => { return () => {}; })(),
            'plugin:global-shortcut|register': () => 'ok',
          };

          if (mockOverrides[cmd]) {
            return mockOverrides[cmd](args);
          }
          if (defaults[cmd]) {
            return defaults[cmd](args);
          }
          console.warn('[Tauri Mock] Unknown command:', cmd);
          return null;
        },

        // transformCallback 用于事件监听的回调 ID 生成
        transformCallback: (callback, once = false) => {
          const id = ++callbackId;
          const name = \`_\${id}\`;
          window[name] = (result) => {
            if (once) {
              delete window[name];
            }
            callback(result);
          };
          return id;
        },

        // listen 用于监听事件
        listen: async (eventName, callback) => {
          console.log('[Tauri Mock] listen:', eventName);
          if (!mockState.eventListeners[eventName]) {
            mockState.eventListeners[eventName] = [];
          }
          mockState.eventListeners[eventName].push(callback);
          // 返回 unlisten 函数
          return () => {
            mockState.eventListeners[eventName] = mockState.eventListeners[eventName].filter(cb => cb !== callback);
          };
        },

        // emit 用于触发事件（测试用）
        emit: async (eventName, payload) => {
          const listeners = mockState.eventListeners[eventName] || [];
          for (const cb of listeners) {
            cb({ event: eventName, payload });
          }
        },
      };

      // 添加 platform mock（用于 @tauri-apps/plugin-os）
      window.__TAURI__ = {
        os: {
          platform: () => 'macos',
          arch: () => 'aarch64',
          version: () => '14.0',
          type: () => 'Darwin',
        },
      };

      console.log('[Tauri Mock] Initialized');
    })();
  `;
}