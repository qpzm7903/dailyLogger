# DailyLogger 项目规划

> 最后更新: 2026-03-19
> 当前版本: v1.21.0 ✅ 已发布
> 下一版本: v1.22.0（待规划）

---

## 已完成版本

### v1.0.0 ~ v1.9.0（Sprint 1 完成）

Sprint 1 完成了 5 大 Epic（87 故事点，24 个 Story），覆盖核心功能、智能捕获、AI 能力、数据管理和报告功能。

**已交付功能汇总**:

| Epic | 故事数 | 状态 |
|------|--------|------|
| Epic 1: 核心功能完善 (CORE-001~008) | 8 | 全部完成 |
| Epic 2: 智能捕获优化 (SMART-001~004) | 4 | 全部完成 |
| Epic 3: AI 能力提升 (AI-001~005) | 5 | 全部完成 |
| Epic 4: 数据管理与检索 (DATA-001~005) | 5 | 全部完成 |
| Epic 5: 周报月报功能 (REPORT-001~003) | 3 | 全部完成 |

**关键技术成果**:
- Tauri v2 跨平台桌面应用 (macOS/Windows/Linux)
- 自动截屏 + OpenAI Vision AI 分析
- AES-256-GCM API Key 加密存储
- 离线模式 + 任务队列 + 指数退避重试
- Ollama 本地模型集成
- 智能静默时段 + 工作时间自动识别
- 全文搜索、标签系统、数据导出、备份恢复
- 周报/月报/自定义周期报告
- 日志系统 (tracing + 文件输出)
- 397 个 Rust 测试 + 16 个前端测试
- 对比分析报告（REPORT-004）、多 Obsidian Vault 支持（DATA-006）

### v1.10.0（CI/CD 完善与基础设施改进）✅ 已发布

**目标**: 完善 CI/CD 发布流水线，补齐 Linux 构建，规范发布产物命名，改进日志系统。

| ID | 需求 | 状态 |
|----|------|------|
| INFRA-001 | 发布流水线补齐 Linux x64 构建 | ✅ 完成 |
| INFRA-002 | 规范发布产物命名格式 | ✅ 完成 |
| INFRA-003 | Release 自动发布（不再仅 draft） | ✅ 完成 |
| INFRA-004 | 日志文件按日轮转（保留 7 天） | ✅ 完成 |
| 额外 | CI 测试矩阵补齐 Linux runner | ✅ 完成 |
| 额外 | 修复 silent_tracker 全局测试隔离 | ✅ 完成 |
| 额外 | 修复 Linux 构建：升级 ubuntu-24.04 + 补齐 libgbm/libegl 依赖 | ✅ 完成 |

---

## 当前迭代: v1.11.0（功能扩展与 Bug 修复）✅ 已发布

**目标**: 新增报告对比分析、多 Obsidian Vault 输出支持，修复月报路径覆盖 Bug。

| ID | 需求 | 故事点 | 状态 |
|----|------|--------|------|
| REPORT-004 | 报告对比分析 | 5pts | ✅ 完成 |
| DATA-006 | 多 Obsidian Vault 支持 | 3pts | ✅ 完成 |
| FIX-001 | 月报路径覆盖日报路径 Bug | 1pt | ✅ 完成 |

### REPORT-004: 报告对比分析

**功能**: 选择两个时间段，AI 生成对比分析报告（工作量变化、重点转移、效率趋势等）。

**后端变更** (`src-tauri/src/`):
- `synthesis/mod.rs`: 新增 `compare_reports()` Tauri 命令，查询两个时段的记录并发送给 AI 对比分析
- `memory_storage/mod.rs`: Settings 新增 `comparison_report_prompt` 字段（可自定义对比 prompt）
- `main.rs`: 注册新命令到 `generate_handler![]`

**前端变更** (`src/`):
- 新建 `components/ReportComparisonModal.vue`: 双时段选择器 + 生成按钮
- `App.vue`: 新增对比报告入口按钮与状态管理
- 对比结果复用 `DailySummaryViewer.vue` 展示

**验收条件**:
- 用户可选择两个任意时段进行对比
- AI 返回结构化对比报告（Markdown 格式）
- 报告保存到 Obsidian 输出目录

### DATA-006: 多 Obsidian Vault 支持

**功能**: 支持配置多个 Obsidian Vault 输出路径，可设置默认 Vault。

**后端变更** (`src-tauri/src/`):
- `memory_storage/mod.rs`: Settings 新增 `obsidian_vaults` 字段（JSON 数组: `[{name, path, is_default}]`），保留 `obsidian_path` 作为回退兼容
- `synthesis/mod.rs`: 4 个报告生成函数（daily/weekly/monthly/custom）支持指定 Vault 或使用默认 Vault
- `manual_entry/mod.rs`: `open_obsidian_folder` 打开默认 Vault

**前端变更** (`src/`):
- `components/SettingsModal.vue`: 替换单一路径输入为 Vault 列表管理（添加/删除/设默认）
- `App.vue`: 报告生成时支持选择目标 Vault（或使用默认）

**验收条件**:
- 支持添加、删除、编辑多个 Vault
- 可设置一个默认 Vault
- 所有报告类型均支持多 Vault
- 旧版单路径配置自动迁移

### FIX-001: 月报路径覆盖日报路径 Bug

**问题**: `generate_monthly_report()` 将输出路径保存到 `last_summary_path`（与日报共享），导致生成月报后日报路径引用被覆盖。

**修复**:
- `memory_storage/mod.rs`: 新增 `last_monthly_report_path` 字段
- `synthesis/mod.rs`: 月报输出路径保存到 `last_monthly_report_path`
- `App.vue`: 加载设置时读取 `last_monthly_report_path`
- 回归测试：生成月报后验证日报路径未被修改

---

## 当前迭代: v1.12.0（代码质量与 Bug 修复）✅ 已发布

**目标**: 消除 synthesis 模块大量重复代码，修复前端 Bug（内存泄漏、标签选择失效、N+1 查询），提升代码可维护性。

**版本类型**: MINOR（包含用户可感知的行为改进）

| ID | 需求 | 故事点 | 优先级 | 状态 |
|----|------|--------|--------|------|
| REFACTOR-001 | Synthesis 模块 LLM API 调用去重 | 5pts | HIGH | ✅ 完成 |
| FIX-002 | App.vue 自动刷新定时器内存泄漏 | 1pt | HIGH | ✅ 完成 |
| FIX-003 | TagCloud 标签选择无法传递到 HistoryViewer | 2pts | MEDIUM | ✅ 完成 |
| PERF-001 | HistoryViewer 标签加载 N+1 查询优化 | 3pts | MEDIUM | ✅ 完成 |

### REFACTOR-001: Synthesis 模块 LLM API 调用去重

**问题**: `synthesis/mod.rs`（2,014 行）中 5 个报告生成函数（`generate_daily_summary`、`generate_weekly_report`、`generate_monthly_report`、`generate_custom_report`、`compare_reports`）共享几乎相同的流程：加载设置 → 提取 API 配置 → 构建 reqwest 请求 → 发送 → 解析响应 → 写入文件。`reqwest::Client::new()` 被调用 5 次，HTTP 请求/响应/错误处理代码复制粘贴 5 次，约 800 行重复代码。

**方案**:
- 提取 `call_llm_api(api_url, api_key, model, prompt, caller_name)` 通用函数
- 提取 `load_api_config(settings)` 配置提取函数
- 提取 `resolve_obsidian_path(settings, vault_name)` 路径解析函数
- 每个报告函数只保留自身特有逻辑（查询记录、格式化 prompt、写入文件）
- 预计减少 ~600 行重复代码

**验收条件**:
- 所有 50 个 synthesis 测试通过
- 5 个报告生成函数行为不变
- 无新的 clippy 警告

### FIX-002: App.vue 自动刷新定时器内存泄漏

**问题**: `App.vue:668` 中 `setInterval(loadTodayRecords, 30000)` 的返回值未存储到变量，导致 `onUnmounted` 中无法清除该定时器。对比同文件中 `timeInterval`（665 行）和 `networkCheckInterval`（691 行）均正确存储并在 `onUnmounted` 中清理。

**修复**:
- 将 `setInterval` 返回值存入 `recordsRefreshInterval` 变量
- 在 `onUnmounted` 中调用 `clearInterval(recordsRefreshInterval)`

**验收条件**:
- 定时器在组件卸载时被正确清理
- 现有前端测试全部通过

### FIX-003: TagCloud 标签选择无法传递到 HistoryViewer

**问题**: `App.vue:567-573` 中 `handleTagSelected(tag)` 接收 tag 参数后仅打开 HistoryViewer，未将选中的 tag 传递过去。代码注释承认此功能未完成（"the user can select it in the HistoryViewer's TagFilter"）。

**修复**:
- 在 App.vue 中添加 `initialFilterTag` ref
- `handleTagSelected` 设置 `initialFilterTag` 后打开 HistoryViewer
- HistoryViewer 接收 `initialTag` prop，挂载时自动应用为过滤条件
- 过滤完成后清除 `initialFilterTag`

**验收条件**:
- 在 TagCloud 中点击标签后，HistoryViewer 自动展示该标签的记录
- 手动打开 HistoryViewer 时不受影响
- 前端测试覆盖此交互流程

### PERF-001: HistoryViewer 标签加载 N+1 查询优化

**问题**: `HistoryViewer.vue:255-265` 中 `loadRecordTags()` 对每条记录逐个调用 `invoke('get_tags_for_record')`，产生 N+1 查询问题。页面加载 20 条记录时会发出 20 次 IPC 调用。

**修复**:
- **后端**: 新增 `get_tags_for_records(record_ids: Vec<i64>)` 批量查询命令，一次 SQL 查询返回所有记录的标签
- **前端**: `loadRecordTags()` 改用批量接口，单次调用获取所有标签
- **注册命令**: 在 `main.rs` 的 `generate_handler![]` 中注册新命令

**验收条件**:
- 加载记录列表时只发出 1 次标签查询（替代 N 次）
- 现有标签相关测试全部通过
- 新增后端单元测试验证批量查询正确性

---

## 当前迭代: v1.13.0（技术债务清理与代码架构改善）✅ 已发布

**目标**: 消除高风险技术债务（Settings 位置索引脆弱性）、清理死代码和重复代码、改善模块结构、增强 CI 覆盖。

**版本类型**: PATCH（内部质量改善，无用户可见功能变化）

| ID | 需求 | 故事点 | 优先级 | 状态 |
|----|------|--------|--------|------|
| CLEAN-001 | 清理死代码和未使用依赖 | 2pts | HIGH | ✅ 完成 |
| DEBT-001a | Settings 读写改用命名列访问替代位置索引 | 5pts | HIGH | ✅ 完成 |
| CLEAN-002 | 合并 test_api_connection 与 Ollama 版本（修复连接测试 Bug） | 2pts | MEDIUM | ✅ 完成 |
| REFACTOR-002 | 拆分 memory_storage/mod.rs 为子模块 | 5pts | MEDIUM | ✅ 完成 |
| CI-001 | CI 工作流改进（Rust 缓存、一致性） | 1pt | LOW | ✅ 完成 |

### CLEAN-001: 清理死代码和未使用依赖

**问题**:
- `lib.rs` 中的 `add()` 函数是遗留占位符，无外部调用
- `Cargo.toml` 中 `thiserror = "2"` 依赖未被使用
- `get_app_data_dir()` 在 4 个文件中重复定义（memory_storage/mod.rs, main.rs, backup/mod.rs, crypto/mod.rs）
- `setup_test_db()` 在 memory_storage/mod.rs 中定义了 3 次

**修复**:
- 移除 `add()` 函数及其测试
- 移除 `thiserror` 依赖
- 将 `get_app_data_dir()` 提取到 `lib.rs` 作为共享函数
- 统一测试数据库初始化函数

### DEBT-001a: Settings 读写改用命名列访问

**问题**: `get_settings_sync()` 使用 `row.get(0)?` ~ `row.get(37)?` 的位置索引读取 38 个字段；`save_settings_sync()` 使用 `?1` ~ `?38` 的位置参数写入。列顺序变化会导致静默数据错乱。

**修复**:
- `get_settings_sync()` 改用 `row.get::<_, T>("column_name")` 命名访问
- `save_settings_sync()` 改用 named parameters（`:column_name`）
- 添加 `schema_version` 到 settings 表用于跟踪迁移版本

### CLEAN-002: 合并 API 连接测试

**问题**: `memory_storage::test_api_connection` 和 `ollama::test_api_connection_with_ollama` 功能重叠，后者支持 Ollama 但未注册到命令中。`ConnectionTestResult` 在两个模块中重复定义。

**修复**:
- 统一 `ConnectionTestResult` 到公共模块
- 将 Ollama 检测逻辑合并到已注册的 `test_api_connection` 命令中
- 移除 `ollama.rs` 中的死代码

### REFACTOR-002: 拆分 memory_storage 模块

**问题**: `memory_storage/mod.rs` 达 4,396 行，包含 7 个不同功能域。测试占 55%（2,410 行）。

**修复**:
- 拆分为 `schema.rs`、`records.rs`、`settings.rs`、`tags.rs`、`api_test.rs`
- `mod.rs` 保留为薄 re-export 层
- 测试随对应模块迁移

### CI-001: CI 工作流改进

**修复**:
- build.yml 添加 Rust 缓存（`Swatinem/rust-cache@v2`）
- 统一 Ubuntu runner 版本

---

## v1.13.1（Windows 免安装版本）✅ 已发布

**目标**: 提供 Windows 免安装（portable）版本，用户无需安装即可直接运行。

| ID | 需求 | 状态 |
|----|------|------|
| ISSUE-017 | Windows 免安装版本（portable exe） | ✅ 完成 |

**变更**:
- Release 构建新增 `*-portable.exe`（无需安装，直接运行）
- 保留原有 `*-setup.exe` 安装版（推荐，自动安装 WebView2）
- Release 说明更新为区分安装版和免安装版

---

## v1.13.2（Windows 启动崩溃修复）✅ 已发布

**目标**: 修复 Windows 版本启动时 FTS5 tokenize 解析错误导致的崩溃问题。

| ID | 需求 | 状态 |
|----|------|------|
| ISSUE-018 | Windows 版本启动崩溃（FTS5 parse error） | ✅ 完成 |

**变更**:
- 移除 FTS5 tokenize 指令中的 `tokenchars "-_"` 参数（Windows SQLite 无法解析）
- 简化为 `tokenize='unicode61'` 以确保跨平台兼容性
- 所有 349 个测试通过

---

## v1.14.0（Logseq 导出支持）✅ 已发布

**目标**: 新增 Logseq 导出支持，报告可同时输出到 Obsidian 和 Logseq。

| ID | 需求 | 故事点 | 状态 |
|----|------|--------|------|
| INT-002 | Logseq 导出支持 | 3pts | ✅ 完成 |

### INT-002: Logseq 导出支持

**功能**: 支持将报告输出到 Logseq 图谱的 `pages` 文件夹，可配置多个图谱并设置默认图谱。

**后端变更** (`src-tauri/src/`):
- `memory_storage/mod.rs`: Settings 新增 `logseq_graphs` 字段，新增 `LogseqGraph` 结构体
- `memory_storage/settings.rs`: 新增 `get_logseq_output_path()` 方法
- `synthesis/mod.rs`: 新增 `write_report_to_logseq()` 函数，集成到所有 5 个报告生成函数

**前端变更** (`src/`):
- `components/SettingsModal.vue`: 新增 Logseq 图谱管理 UI（添加/删除/设默认）

**验收条件**:
- 用户可配置多个 Logseq 图谱
- 报告自动写入到图谱的 `pages` 文件夹
- 同时支持 Obsidian 和 Logseq 双输出

---

## v1.15.0（Notion 导出支持）✅ 已发布

**目标**: 新增 Notion 导出支持，报告可通过 API 写入 Notion 数据库。

| ID | 需求 | 故事点 | 状态 |
|----|------|--------|------|
| INT-001 | Notion 导出支持 | 5pts | ✅ 完成 |

### INT-001: Notion 导出支持

**功能**: 支持将报告通过 Notion API 写入指定的数据库，可配置多个 Notion 数据库并设置默认。

**后端变更** (`src-tauri/src/`):
- `memory_storage/mod.rs`: Settings 新增 `notion_databases` 字段，新增 `NotionDatabase` 结构体
- `memory_storage/settings.rs`: 新增 `get_notion_output_config()` 方法，API Key 加密存储
- `notion.rs`: 新增 Notion API 集成模块，包含 `write_report_to_notion()` 和 `test_notion_connection()` 函数
- `synthesis/mod.rs`: 集成 Notion 输出到所有 5 个报告生成函数

**前端变更** (`src/`):
- `components/SettingsModal.vue`: 新增 Notion 数据库管理 UI（添加/删除/设默认、连接测试）

**验收条件**:
- 用户可配置多个 Notion 数据库（API Key、Database ID）
- 报告自动通过 API 写入 Notion 数据库
- 支持连接测试验证配置正确性
- 同时支持 Obsidian、Logseq 和 Notion 三重输出

---

## v1.16.0（GitHub 工时统计）✅ 已完成（待发布）

**目标**: 新增 GitHub 工时统计功能，自动分析 GitHub 提交和 PR 活动计算工作时长。

| ID | 需求 | 故事点 | 状态 |
|----|------|--------|------|
| INT-003 | GitHub 工时统计 | 8pts | ✅ 完成 |

### INT-003: GitHub 工时统计

**功能**: 通过 GitHub API 获取用户的提交和 PR 数据，计算预估工作时长，并集成到日报中。

**后端变更** (`src-tauri/src/`):
- `memory_storage/mod.rs`: Settings 新增 `github_token` 和 `github_repositories` 字段
- `memory_storage/settings.rs`: github_token 加密存储
- `github.rs`: 新增 GitHub API 集成模块，包含 `fetch_commits()`、`fetch_pull_requests()`、`calculate_work_time_stats()`、`format_github_activity_for_report()` 函数
- `synthesis/mod.rs`: 集成 GitHub 活动统计到日报生成函数

**前端变更** (`src/`):
- `components/SettingsModal.vue`: 新增 GitHub 配置 UI（Token 输入、仓库列表管理、测试连接按钮）

**验收条件**:
- 用户可配置 GitHub Token 和监控的仓库列表
- 自动获取当日提交和 PR 数据
- 基于提交时间聚类算法预估工作时长
- 日报中包含 GitHub 活动统计章节（提交数、PR数、预估工时、活跃仓库、提交时间分布）

---

## v1.17.0（Slack 通知集成）✅ 已发布

**目标**: 新增 Slack 通知集成，支持将报告发送到 Slack 频道。

| ID | 需求 | 故事点 | 状态 |
|----|------|--------|------|
| INT-004 | Slack 通知集成 | 5pts | ✅ 完成 |

### INT-004: Slack 通知集成

**功能**: 支持将报告通过 Slack Webhook 发送到指定频道，可配置多个 Webhook 并设置默认频道。

**后端变更** (`src-tauri/src/`):
- `memory_storage/mod.rs`: Settings 新增 `slack_webhooks` 字段，新增 `SlackWebhook` 结构体
- `memory_storage/settings.rs`: 新增 `get_slack_webhooks()` 方法
- `slack.rs`: 新增 Slack Webhook 集成模块，包含 `send_to_slack()` 和 `test_slack_webhook()` 函数
- `synthesis/mod.rs`: 集成 Slack 输出到所有 5 个报告生成函数
- `main.rs`: 注册 `test_slack_webhook` 命令

**前端变更** (`src/`):
- `components/SettingsModal.vue`: 新增 Slack Webhook 管理 UI（添加/删除/设默认、测试连接）

**验收条件**:
- 用户可配置多个 Slack Webhook URL
- 报告自动发送到默认 Slack 频道
- 支持连接测试验证配置正确性
- 同时支持 Obsidian、Logseq、Notion 和 Slack 四重输出

---

## v1.18.0（插件系统架构）✅ 已发布

**目标**: 新增插件系统架构，支持用户扩展应用功能。

| ID | 需求 | 故事点 | 状态 |
|----|------|--------|------|
| PLUGIN-001 | 插件系统核心架构 | 5pts | ✅ 完成 |
| PLUGIN-002 | 插件加载与发现机制 | 3pts | ✅ 完成 |
| PLUGIN-003 | 插件配置管理 UI | 3pts | ✅ 完成 |

### PLUGIN-001: 插件系统核心架构

**功能**: 定义插件接口、生命周期管理和钩子系统。

**后端变更** (`src-tauri/src/`):
- `plugin.rs`: 新增插件模块
  - `Plugin` trait: 定义插件接口（metadata, on_load, on_unload）
  - `PluginMetadata`: 插件元数据（ID、名称、版本、描述、作者等）
  - `PluginManager`: 插件管理器（注册/注销插件、钩子管理）
  - `HookPoint` enum: 扩展点（报告生成、记录保存、截图捕获等）
  - `HookData` enum: 钩子数据传递
  - 8 个单元测试

**验收条件**:
- 插件可注册到 PluginManager
- 插件生命周期钩子正常调用
- 钩子系统可触发回调
- 插件元数据可序列化

### PLUGIN-002: 插件加载与发现机制

**功能**: 从插件目录自动发现并加载插件。

**后端变更** (`src-tauri/src/`):
- `plugin.rs`: 新增插件发现功能
  - `PluginManifest`: 插件清单结构（plugin.json 格式）
  - `PluginStatus`: 插件状态（Ready/Disabled/Error）
  - `DiscoveredPlugin`: 发现的插件信息
  - `discover_plugins()`: 扫描插件目录
  - `load_manifest()`/`save_manifest()`: 清单读写
  - `get_plugins_directory()`/`ensure_plugins_directory()`: 目录管理
  - 9 个新单元测试

**验收条件**:
- 可从插件目录扫描发现插件
- 可解析 plugin.json 清单文件
- 可验证插件元数据有效性
- 支持启用/禁用插件状态

### PLUGIN-003: 插件配置管理 UI

**功能**: 在设置界面中添加插件管理面板，支持查看、启用/禁用插件。

**后端变更** (`src-tauri/src/`):
- `plugin.rs`: 新增 Tauri 命令
  - `list_discovered_plugins()`: 返回所有发现的插件列表
  - `enable_plugin(plugin_id)`: 启用指定插件
  - `disable_plugin(plugin_id)`: 禁用指定插件
  - `open_plugins_directory()`: 打开插件目录
  - `PluginInfo`: 前端显示用的插件信息结构体

**前端变更** (`src/`):
- 新建 `components/PluginPanel.vue`: 插件管理面板组件
  - 插件列表展示（名称、版本、作者、状态）
  - 启用/禁用按钮
  - 打开插件目录按钮
- `SettingsModal.vue`: 集成 PluginPanel 组件
- `locales/en.json` / `locales/zh-CN.json`: 添加插件相关翻译

**验收条件**:
- 用户可在设置中查看所有已发现的插件
- 可一键启用/禁用插件
- 可快速打开插件目录添加新插件

---

## v1.19.0（Ollama 模型管理增强）✅ 已发布

**目标**: 增强 Ollama 本地模型管理功能，支持运行状态监控、自定义模型创建、模型复制。

| ID | 需求 | 故事点 | 状态 |
|----|------|--------|------|
| OLLAMA-001 | 运行模型状态监控 | 2pts | ✅ 完成 |
| OLLAMA-002 | 自定义模型创建（Modelfile） | 3pts | ✅ 完成 |
| OLLAMA-003 | 模型拉取量化参数支持 | 1pt | ✅ 完成 |
| OLLAMA-004 | 模型复制功能 | 2pts | ✅ 完成 |

### OLLAMA-001: 运行模型状态监控

**功能**: 显示当前 Ollama 加载的模型列表及 VRAM 占用。

**后端变更** (`src-tauri/src/`):
- `ollama.rs`: 新增 `get_running_models()` Tauri 命令，调用 Ollama `/api/ps` 端点
- `RunningModelInfo`/`RunningModelsResult` 结构体

**前端变更** (`src/`):
- `SettingsModal.vue`: 新增"运行中的模型"区块，显示模型名称和 VRAM 占用
- 刷新按钮、绿色脉冲指示器动画

### OLLAMA-002: 自定义模型创建

**功能**: 基于 Modelfile 创建自定义 Ollama 模型。

**后端变更** (`src-tauri/src/`):
- `ollama.rs`: 新增 `create_ollama_model()` Tauri 命令
- `CreateModelParams`/`CreateModelResult` 结构体

**前端变更** (`src/`):
- `SettingsModal.vue`: 新增"创建自定义模型"弹窗
- 支持设置模型名称、基础模型、系统提示词、温度、上下文长度

### OLLAMA-003: 模型拉取量化参数支持

**功能**: 拉取模型时支持指定量化级别。

**后端变更** (`src-tauri/src/`):
- `ollama.rs`: `pull_ollama_model()` 新增可选 `quantization` 参数
- 支持 q4_0, q4_1, q5_0, q5_1, q8_0, f16

**前端变更** (`src/`):
- `SettingsModal.vue`: 模型拉取界面新增量化下拉选择框

### OLLAMA-004: 模型复制功能

**功能**: 复制现有模型创建变体，用于后续参数修改。

**后端变更** (`src-tauri/src/`):
- `ollama.rs`: 新增 `copy_ollama_model()` Tauri 命令
- `CopyModelResult` 结构体

**前端变更** (`src/`):
- `SettingsModal.vue`: 模型列表中每个模型旁新增复制按钮
- 复制对话框输入新模型名称

---

## v1.19.1（Windows 启动诊断）✅ 已发布

**目标**: 添加启动诊断日志功能，帮助定位 Windows 版本无法显示窗口的问题。

| ID | 需求 | 状态 |
|----|------|------|
| FIX-WIN-001 | 添加启动诊断日志 | ✅ 完成 |

### FIX-WIN-001: 启动诊断日志

**问题**: Windows 版本启动后进程存在但窗口不显示，且无日志输出。

**诊断方案**:
- 在应用启动时立即写入诊断文件 `dailylogger-startup.log`
- 尝试多个写入位置：exe 目录 → AppData → 用户主目录
- 在关键启动阶段记录进度：panic hook 安装 → 日志初始化 → WebView2 检查 → 应用初始化 → Tauri 构建 → 窗口创建

**变更**:
- `main.rs`: 新增 `write_diagnostic_file()` 函数
- 在启动流程各阶段添加诊断点

**用户指引**:
- 检查 `dailylogger-startup.log` 文件内容
- 文件位置取决于可写入的目录
- 如果文件存在且包含 "Tauri setup completed"，说明应用启动成功但窗口未显示
- 如果文件不存在或内容不完整，可定位具体失败阶段

---

## v1.20.0（本地 AI 模型微调）✅ 已发布

**目标**: 新增本地 AI 模型微调功能，支持基于用户数据训练定制化模型。

| ID | 需求 | 故事点 | 状态 |
|----|------|--------|------|
| FUTURE-003 | 本地 AI 模型微调 | 5pts | ✅ 完成 |

### FUTURE-003: 本地 AI 模型微调

**功能**: 支持从用户记录导出训练数据，通过 Ollama API 进行模型微调，创建定制化模型。

**后端变更** (`src-tauri/src/`):
- 新增 `fine_tuning.rs` 模块
  - `FineTuningConfig`: 微调配置结构体（基础模型、输出模型名、训练轮次、学习率等）
  - `TrainingDataEntry`: 训练数据条目结构体
  - `prepare_training_data()`: 导出训练数据到 JSONL 文件
  - `start_fine_tuning()`: 启动 Ollama 微调任务
  - `get_default_fine_tuning_config()`: 获取默认配置
  - 4 个单元测试

**前端变更** (`src/`):
- `SettingsModal.vue`: 新增"模型微调"按钮和微调弹窗
  - 基础模型选择、输出模型名设置
  - 训练数据配置（自动/手动记录、回溯天数）
  - 高级参数（系统提示词、温度、上下文长度、训练轮次）
  - 支持导出训练数据到 JSONL 文件
  - en/zh-CN 翻译支持

**验收条件**:
- 用户可选择基础模型和训练数据范围
- 训练数据导出为 JSONL 格式
- 微调任务通过 Ollama API 启动
- 配置参数可视化设置

---

## v1.21.0（团队协作模式）✅ 已发布

**目标**: 新增团队协作功能，支持用户注册、团队管理、记录共享。

| ID | 需求 | 故事点 | 状态 |
|----|------|--------|------|
| TEAM-001 | 用户认证（注册/登录/会话管理） | 5pts | ✅ 完成 |
| TEAM-002 | 团队 CRUD（创建/加入/邀请/角色管理） | 5pts | ✅ 完成 |
| TEAM-003 | 记录共享（分享记录到团队） | 3pts | ✅ 完成 |

### TEAM-001: 用户认证

**功能**: 支持用户注册、登录、会话持久化。

**后端变更** (`src-tauri/src/`):
- 新增 `auth/mod.rs` 模块
  - `User`: 用户实体
  - `Session`: 会话实体
  - `register_user()`: 用户注册（Argon2 密码哈希）
  - `login_user()`: 用户登录验证
  - `logout()`: 登出
  - `get_current_session()`: 获取当前会话
  - `users` 和 `sessions` 数据库表

**前端变更** (`src/`):
- 新增 `LoginModal.vue`: 登录/注册界面
- `App.vue`: 用户信息显示、登出按钮

### TEAM-002: 团队管理

**功能**: 支持创建团队、邀请成员、角色管理。

**后端变更** (`src-tauri/src/`):
- 新增 `team/mod.rs` 模块
  - `Team`: 团队实体
  - `TeamMember`: 成员实体
  - `TeamRole`: 角色枚举（Admin/Member/Viewer）
  - 12 个 Tauri 命令（create_team, join_team, invite_member 等）
  - `teams` 和 `team_members` 数据库表

**前端变更** (`src/`):
- 新增 `TeamPanel.vue`: 团队管理界面
- 邀请码生成和分享功能

### TEAM-003: 记录共享

**功能**: 支持将记录分享到团队。

**后端变更** (`src-tauri/src/`):
- `team/mod.rs`: 新增共享功能
  - `share_record_to_team()`: 分享记录
  - `get_team_shared_records()`: 获取团队共享记录
  - `shared_records` 数据库表

**前端变更** (`src/`):
- `HistoryViewer.vue`: 新增分享按钮
- 分享弹窗选择目标团队

---

## 长期规划: v2.0.0+（集成与扩展）

**目标**: 与第三方工具集成，扩展应用场景。

| ID | 需求 | 故事点 | 状态 |
|----|------|--------|------|
| INT-001 | Notion 导出支持 | 5pts | ✅ 完成 (v1.15.0) |
| INT-002 | Logseq 导出支持 | 3pts | ✅ 完成 (v1.14.0) |
| INT-003 | GitHub 工时统计 | 8pts | ✅ 完成 (v1.16.0) |
| INT-004 | Slack 通知集成 | 5pts | ✅ 完成 (v1.17.0) |

**远期功能**:
- ~~时间线可视化（图形化展示一天工作流）~~ ✅ 已完成 (v1.17.0)
- ~~多语言支持 (i18n)~~ ✅ 已完成 (v1.17.0)
- ~~插件系统架构~~ ✅ 已完成 (v1.18.0)
- ~~本地 AI 模型管理（Ollama）~~ ✅ 已完成 (v1.19.0)
- ~~本地 AI 模型微调~~ ✅ 已完成 (v1.20.0)
- ~~团队协作模式~~ ✅ 已完成 (v1.21.0)
- 移动端适配（Tauri Mobile）— 基础完成（硬件抽象层、平台检测、桌面专属 UI 隐藏）

---

## 技术债务追踪

| ID | 描述 | 来源 | 优先级 | 状态 |
|----|------|------|--------|------|
| DEBT-001 | 数据库 Schema 版本化迁移（settings 表已 38 字段，ALTER TABLE 链已 33 条，get/save_settings 使用脆弱的位置索引） | CORE/DATA/AI 回顾 | HIGH | ✅ 部分解决 (v1.13.0 DEBT-001a: 命名列访问) |
| DEBT-002 | 离线队列 ScreenshotAnalysis 重试为空操作 | 代码审查 | MEDIUM | ✅ 完成 (v1.14.0: retry_screenshot_analysis 实现) |
| DEBT-003 | 统一测试数据库 Schema 初始化 | AI 回顾 | MEDIUM | ✅ 完成 (v1.13.2: init_test_database 提取到 schema.rs) |
| DEBT-004 | 前端组件测试覆盖率 | 多 Epic 回顾 | MEDIUM | ✅ 完成 (v1.13.2: 531 个前端组件测试，覆盖所有主要组件) |
| DEBT-005 | 学习数据持久化（SilentPatternTracker / WorkTimePatternLearner） | SMART 回顾 | MEDIUM | ✅ 完成 (v1.14.0: 数据库持久化实现) |
| DEBT-006 | 硬件抽象层（窗口/显示器/截图 API） | SMART 回顾 | LOW | ✅ 完成 (post-v1.17.0: hardware module traits + platform implementations) |

---

## 已知问题

| 问题 | 影响 | 关联版本 | 状态 |
|------|------|----------|------|
| ~~历史 GitHub Release (v1.0.0~v1.9.0) 均为 Draft 未发布~~ | ~~用户无法下载旧版本~~ | v1.0.0 ~ v1.9.0 | ✅ 已修复（v1.4.0~v1.9.0 已发布，重复 Draft 已清理） |
| 月报生成覆盖日报路径（`last_summary_path`） | 生成月报后日报路径丢失 | v1.8.0+ | ✅ 已修复 (v1.11.0 FIX-001) |
| Windows/macOS 构建产物无法打开 (#15) | 用户下载后无法启动应用 | v1.10.0 | ✅ 已修复 (v1.11.0 NSIS 安装程序) |
| App.vue 自动刷新定时器未清理（内存泄漏） | 组件卸载后定时器继续运行 | v1.0.0+ | ✅ 已修复 (v1.12.0 FIX-002) |
| TagCloud 标签选择未传递到 HistoryViewer | 点击标签后需手动重新选择过滤条件 | v1.5.0+ | ✅ 已修复 (v1.12.0 FIX-003) |
| HistoryViewer 标签加载 N+1 查询 | 每条记录单独查询标签，页面加载慢 | v1.5.0+ | ✅ 已修复 (v1.12.0 PERF-001) |

---

## 版本发布检查清单

每次发布新版本时，需确认以下事项：

1. [ ] 所有需求开发完成，测试通过
2. [ ] `cargo fmt && cargo clippy -- -D warnings && cargo test` 全绿
3. [ ] `npm run test` 全绿
4. [ ] 更新版本号：`package.json` / `Cargo.toml` / `tauri.conf.json`
5. [ ] 提交版本升级 commit
6. [ ] 创建并推送 tag: `git tag vX.Y.Z && git push && git push --tags`
7. [ ] GitHub Release 构建成功，产物已发布
8. [ ] 关闭相关 Issues
9. [ ] 更新 README.md（如有必要）
10. [ ] 更新本文件状态
