# DailyLogger 项目规划

> 最后更新: 2026-03-28
> 当前版本: v3.10.0（已完成）
> 下一版本: v4.0.0（规划中）
> 当前 Milestone: v4.0.0 规划中

---

## 当前 Milestone：v4.0.0 规划中

**目标**: 完善数据库迁移系统集成，清理技术债务。

**版本策略**:

| 版本 | 类型 | 目标 |
|------|------|------|
| v4.0.0 | MAJOR | 架构重大变更或不兼容更新 |

**待办事项**:

| ID | 需求 | 故事点 | 优先级 | 状态 |
|----|------|--------|--------|------|
| DEBT-006 | 数据库迁移系统完整集成：将 `run_migrations()` 集成到 `init_database()` 替代分散的 ALTER TABLE 语句 | 5pts | P0 | ✅ 已完成 |
| DEBT-007 | 清理 schema.rs 中冗余的 ALTER TABLE 语句 | 2pts | P1 | ✅ 已完成 |
| DEBT-008 | 确保现有数据库可以正常迁移到新迁移系统 | 3pts | P1 | ✅ 已完成 |

**DEBT-006 修复内容**:
- `init_database()` 应调用 `run_migrations()` 作为主要迁移机制
- 当前 `run_migrations()` 仅在测试中被调用，生产环境未使用
- 迁移 SQL 应包含所有字段添加和表创建操作
- 移除分散在 `init_database()` 中的 ALTER TABLE 语句（schema.rs 第 76-400+ 行）

**DEBT-007 清理内容**:
- schema.rs 中有大量独立 ALTER TABLE 语句
- 这些应在迁移系统中以结构化方式管理
- 清理后 schema.rs 应只保留表创建和初始化逻辑

**DEBT-007 修复内容**:
- 移除了 `init_database()` 中约 300 行冗余的 ALTER TABLE 和 CREATE TABLE 语句
- 修复了遗留数据库（无迁移记录但表已存在）的处理逻辑：现调用 `run_migrations()` 而不是仅记录迁移
- 所有 schema 变更现在统一由迁移系统 `migration.rs` 管理
- 保留了 `init_test_database()` 中的完整 schema 设置（测试专用）
- 508 Rust 测试 + 964 前端测试全部通过

**DEBT-008 验证内容**:
- 已有数据库应能正确迁移到新系统
- 迁移必须是幂等的
- 迁移前后数据应保持一致

**重构原则**:
- 不修改现有数据库结构和数据
- 迁移过程必须幂等（可安全重复执行）
- 保持向后兼容（已有数据库正常升级）

| 版本 | 类型 | 目标 |
|------|------|------|
| v3.4.0 | MINOR | 架构收口一期：拆前端应用壳、统一 IPC 调用边界、精简 Tauri 启动入口 ✅ |
| v3.5.0 | MINOR | 架构收口二期：抽离后端 service 边界、按功能整理前端模块、补齐回归测试基线 ✅ |
| v3.6.0 | MINOR | 架构收口三期：统一 Settings/Session/Report 契约，收敛全局状态和错误模型 ✅ |
| v3.7.1 | MINOR | 标签系统后端化：标签颜色可配置、自动分配优化、旧数据兼容 ✅ |
| v3.8.0 | MINOR | 多维度输出增强：自定义导出模板、输出格式扩展 ✅ |
| v3.9.0 | MINOR | 多 Vault 自动选择：基于窗口标题自动选择输出 Vault ✅ |

**重构原则**:

- 不做一次性大重写，按”前端壳层 -> Tauri 命令层 -> 数据契约层”分阶段推进
- 不在一期引入新框架，优先用现有 Vue 3 + Tauri + Rust 收口边界
- 每个版本都必须保持行为兼容、可测试、可回滚，而不是长期悬空分支
- 多 Vault、标签、导出等功能扩展顺延到架构稳定之后再继续推进

### v3.5.0（架构收口二期）✅ 完成

**目标**: 抽出 service 边界，降低命令层和具体模块实现的耦合。

**版本类型**: MINOR（内部架构重构，保持行为兼容）

| ID | 需求 | 故事点 | 优先级 | 状态 | Spec |
|----|------|--------|--------|------|------|
| ARCH-004 | 建立 `commands -> services` 调用边界，避免命令入口继续承载业务实现 | 3pts | P0 | ✅ 已完成 | `specs/ARCH-001-architecture-refactor.md` |

**ARCH-004 进展记录**:
- ✅ `get_model_info` 已迁移到 `commands/model_commands.rs` + `services/model_service.rs`
- ✅ `get_settings` / `save_settings` 已迁移到 `commands/settings_commands.rs` + `services/settings_service.rs`
- ✅ 模式已验证：命令为薄包装，调用 service 函数
- ✅ Session 命令已迁移：`get_today_sessions` / `analyze_session` / `get_session_screenshots` / `update_session_user_summary` → `commands/session_commands.rs` + `services/session_service.rs`
- ✅ Report 命令已迁移：`generate_daily_summary` / `generate_multilingual_daily_summary` / `generate_weekly_report` / `generate_monthly_report` / `generate_custom_report` / `compare_reports` → `commands/report_commands.rs` + `services/report_service.rs`
- ✅ Capture commands 已集成：`commands/capture_commands.rs` 薄命令包装器已注册到 Tauri 命令层，`auto_perception/mod.rs` 中重复命令已移除
- ✅ 所有 486 Rust 测试和 964 前端测试通过，质量基线稳固

| ARCH-005 | 为本轮重构补齐回归基线：cargo test、cargo clippy、前端 typecheck 和 test | 2pts | P1 | ✅ 已完成 | `specs/ARCH-001-architecture-refactor.md` |

**验收重点**:

- 命令层不再直接承载复杂业务编排
- `src-tauri/src/commands/` 包含所有 `#[tauri::command]` 入口函数
- `src-tauri/src/services/` 包含各领域业务逻辑
- 新增功能时，修改范围可控制在单一 service 内

### v3.5.1（启动崩溃补丁）✅ 完成

**目标**: 修复 v3.5.0 Windows 启动崩溃问题。

**版本类型**: PATCH（缺陷修复）

**修复内容**:
- 将 `start_scheduler()` 和 `check_and_run_startup_backup()` 从 `init_app()` 移至 Tauri setup 阶段
- setup 回调执行时 Tokio runtime 已经就绪，避免 "there is no reactor running" 错误
- 提交: `dc3c07b` + `18cbc00`（Cargo.lock 同步）

### v3.6.0（架构收口三期）✅ 完成

**目标**: 统一契约、错误模型和全局状态边界。

**版本类型**: MINOR（内部架构重构，保持行为兼容）

**修复内容**:

| ID | 需求 | 故事点 | 优先级 | 状态 |
|----|------|--------|--------|------|
| ARCH-006 | 统一 Settings 契约：更新前端类型定义，移除死字段，补充缺失字段 | 3pts | P0 | ✅ 已完成 |
| ARCH-007 | 统一 Record/Session 契约：补齐缺失字段，确保类型对齐 | 2pts | P0 | ✅ 已完成 |
| ARCH-008 | 建立结构化错误模型：后端返回 `AppError` 枚举，前端统一错误处理 | 3pts | P1 | ✅ 已完成 |
| ARCH-009 | 收敛全局状态：建立 `AppState` 容器，减少 `Lazy<Mutex<...>>` 扩散 | 2pts | P1 | ✅ 已完成 |
| ARCH-010 | 建立架构约束文档：明确命令层 vs service 层边界规则 | 1pt | P2 | ✅ 已完成 |

**ARCH-006/007 修复内容**:
- 前端 `Settings` 类型移除死字段: `silence_detection_enabled`, `silence_threshold`, `window_filter_*`, `multi_monitor_mode`, `custom_prompt`, `default_obsidian_vault`
- 前端 `Settings` 类型补充缺失字段: `summary_model_name`, `analysis_prompt`, `window_whitelist/blacklist`, `auto_adjust_silent`, `capture_mode`, `proxy_*`, `quality_filter_*`, `auto_backup_*` 等
- 前端 `LogRecord` 补齐 `session_id` 和 `analysis_status` 字段
- 前端 `ErrorType` 统一命名（snake_case）并添加 `internal` 类型

**ARCH-008 修复内容**:
- 新增 `src-tauri/src/errors.rs` 模块
- 定义 `AppError` 结构体和 `ErrorCode` 枚举（10 种错误类型）
- 实现 `From<String>`, `From<rusqlite::Error>`, `From<reqwest::Error>` 等转换
- 前端 `createErrorInfo()` 支持解析结构化 `AppError`
- 添加 5 个 error module 测试

**ARCH-009 修复内容**:
- 新增 `src-tauri/src/infrastructure/mod.rs`
- 新增 `src-tauri/src/infrastructure/state.rs` 文档
- 记录模块级状态 vs 应用级状态的区分原则
- 建立新增全局状态的检查流程

**ARCH-010 修复内容**:
- 新增 `specs/ARCH-010-architecture-constraints.md`
- 定义 5 层架构：前端组件 → feature actions → IPC → commands → services
- 明确命令层、service 层、前端 IPC、全局状态、错误处理的硬性约束
- 提供代码示例和检查清单

### 未来 Milestone 概要

| 版本 | 方向 | 说明 |
|------|------|------|
| v4.0.0 | MAJOR | 架构重大变更或不兼容更新（暂定） |

---

### v3.9.0（多 Vault 自动选择）✅ 完成

**目标**: 完成 VAULT-001 剩余任务，实现基于窗口标题的自动 Vault 选择。

**版本类型**: MINOR（新功能增强）

**完成内容**:

| ID | 任务 | 优先级 | 状态 |
|----|------|--------|------|
| VAULT-001 Task 5 | 前端 - OutputSettings 添加项目检测 UI | P1 | ✅ 已完成 |
| VAULT-001 Task 6 | 单元测试覆盖 | P1 | ✅ 已完成 |

**Task 5 修复内容**:
- OutputSettings.vue 添加"根据窗口标题自动选择 Vault"开关
- 每个 Vault 编辑区域添加"窗口标题匹配模式"输入框（多个用逗号分隔）
- `getVaultPatternsString()` / `updateVaultPatterns()` 函数处理模式字符串与数组转换

**Task 6 修复内容**:
- 新增 12 个 vault 相关单元测试（tests_vault_001 模块）
- 测试覆盖：`get_vault_by_name`、`get_vault_by_window_title`、`get_effective_vault`
- 覆盖场景：精确匹配、部分匹配、大小写不敏感、多模式匹配、空模式、无匹配

**已完成**:

- ✅ VAULT-001 Task 1: ObsidianVault 数据结构扩展（window_patterns 字段）
- ✅ VAULT-001 Task 2: Rust 后端 - generate_daily_summary 支持 vault 参数
- ✅ VAULT-001 Task 3: Rust 后端 - 基于窗口标题的自动 Vault 选择
- ✅ VAULT-001 Task 4: 前端 - ReportDropdown 添加 Vault 选择器

---

### v3.10.0（技术债务清偿 - DEBT-002）✅ 完成

**目标**: 建立带版本追踪的数据库迁移机制，确保 schema 变更在不同升级场景下可靠应用。

**版本类型**: PATCH（技术债务修复）

**完成内容**:

| ID | 任务 | 优先级 | 状态 |
|----|------|--------|------|
| DEBT-002 | 数据库版本迁移机制 | P1 | ✅ 已完成 |

**DEBT-002 修复内容**:
- 新增 `src-tauri/src/memory_storage/migration.rs` 模块
- 创建 `schema_version` 表追踪当前数据库版本
- 创建 `schema_migrations` 表记录迁移历史
- 实现 `Migration` 结构体支持结构化迁移
- 实现幂等迁移执行器：`run_migrations()`
- 提供版本查询函数：`get_current_version()`、`get_migration_history()`
- 添加 5 个迁移相关测试，验证版本追踪和幂等性
- 508 Rust 测试 + 964 前端测试全部通过

---

## 最近 10 个已完成版本摘要

### v3.8.0 — 多维度输出增强 ✅
- 新增自定义导出模板功能，支持用户自定义 Markdown 导出格式
- 实现模板占位符：`{{date}}`, `{{time}}`, `{{content}}`, `{{source_type}}`, `{{source_icon}}`, `{{tags}}`
- 新增 `get_default_export_template` / `get_default_record_entry_template` 后端命令
- 更新 ExportModal UI，支持自定义模板编辑器和预览

### v3.7.1 — 标签管理增强 ✅
- 标签颜色后端化：后端存储标签颜色，前端从缓存获取
- 实现 `get_tag_colors()` / `set_tag_color()` 命令
- 三级回退逻辑：缓存 → 默认颜色表 → 哈希分配

### v3.6.0 — 架构收口三期 ✅
- 统一前后端契约：修复 Settings 和 LogRecord 类型定义
- 建立结构化错误模型：AppError 枚举和统一错误处理
- 收敛全局状态：建立 infrastructure/state.rs 文档规范
- 建立架构约束文档：specs/ARCH-010-architecture-constraints.md

### v3.5.0 — 架构收口二期 ✅
- 抽取 Settings/Session/Report/Capture 四个领域 service 边界
- 命令层重构为薄 IPC 适配器，业务逻辑下沉到 services
- 补齐回归基线：486 Rust 测试 + 964 前端测试全部通过

### v3.4.0 — 架构收口一期 ✅
- 提取前端应用壳：AppShell、AppModals、useAppBootstrap
- 建立统一 Tauri IPC Client 和 feature actions，组件不再直接散落 `invoke()`
- 拆分 `main.rs`：提取 bootstrap/logging.rs、bootstrap/tray.rs、bootstrap/commands.rs

### v3.3.0 — 体验极致化续 ✅
- 新用户引导、截图加载优化、数据库查询优化、多语言支持、浅色主题全部落地
- Epic 10 完成，整体体验和性能明显提升

### v3.2.0 — AI 代理配置 ✅
- AI API 请求支持 HTTP 代理和认证
- 补充测试连接模型和前端折叠配置面板

### v3.1.1 — CI 修复 ✅
- 修复 Build and Release workflow 中 release 发布链路异常
- 保证版本发布流程恢复可用

### v3.0.0 — 工作时段感知分析 + GitHub 移除 ✅
- 捕获与分析解耦，新增 session 管理、批量分析、用户编辑和手动触发分析
- 日报生成改为以时段为核心；同时移除 GitHub 集成

### v2.10.0 — 今日摘要 Widget ✅
- Dashboard 新增摘要 Widget 和实时统计
- 搜索防抖与结果导航体验优化

### v2.8.0 — 截图质量过滤 ✅
- 自动跳过低信息截图，降低无效分析开销
- 为后续捕获链路优化打下基础

### v2.6.0 — Slack/钉钉通知 ✅
- 报告生成后可自动发送到 Slack/钉钉
- 通知失败不影响主流程

### v2.5.0 — GitHub 工时统计 ✅
- Dashboard 独立展示 GitHub 工时统计面板
- 补齐对应前后端测试

### v2.4.0 — GitHub API 集成验证 ✅
- 验证 GitHub API 集成实现完整性
- 为后续功能裁剪前的集成能力建立基线
