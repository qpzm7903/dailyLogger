# DailyLogger 项目规划

> 最后更新: 2026-03-17
> 当前版本: v1.12.0 ✅ 已发布
> 下一版本: v1.13.0（待规划）

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

## 长期规划: v2.0.0+（集成与扩展）

**目标**: 与第三方工具集成，扩展应用场景。

| ID | 需求 | 故事点 | 状态 |
|----|------|--------|------|
| INT-001 | Notion 导出支持 | 5pts | Backlog |
| INT-002 | Logseq 导出支持 | 3pts | Backlog |
| INT-003 | GitHub 工时统计 | 8pts | Backlog |
| INT-004 | Slack/钉钉通知 | 5pts | Backlog |

**远期功能**:
- 时间线可视化（图形化展示一天工作流）
- 多语言支持 (i18n)
- 插件系统架构
- 移动端适配（Tauri Mobile）
- 团队协作模式
- 本地 AI 模型深度集成（模型管理、微调）

---

## 技术债务追踪

| ID | 描述 | 来源 | 优先级 | 状态 |
|----|------|------|--------|------|
| DEBT-001 | 数据库 Schema 版本化迁移（settings 表已 38 字段，ALTER TABLE 链已 33 条，get/save_settings 使用脆弱的位置索引） | CORE/DATA/AI 回顾 | HIGH | 待开发 |
| DEBT-002 | 离线队列 ScreenshotAnalysis 重试为空操作 | 代码审查 | MEDIUM | 待开发 |
| DEBT-003 | 统一测试数据库 Schema 初始化 | AI 回顾 | MEDIUM | 待开发 |
| DEBT-004 | 前端组件测试覆盖率 | 多 Epic 回顾 | MEDIUM | 待开发 |
| DEBT-005 | 学习数据持久化（SilentPatternTracker / WorkTimePatternLearner） | SMART 回顾 | MEDIUM | 待开发 |
| DEBT-006 | 硬件抽象层（窗口/显示器/截图 API） | SMART 回顾 | LOW | 待开发 |

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
