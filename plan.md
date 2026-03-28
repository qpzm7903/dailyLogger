# DailyLogger 项目规划

> 最后更新: 2026-03-28
> 当前版本: v3.5.1（启动崩溃补丁）
> 下一版本: v3.6.0（架构收口三期）
> 当前 Milestone: 架构收口与可维护性重构（v3.4.0 ~ v3.6.0）

---

## 当前 Milestone：架构收口与可维护性重构

**目标**: 在不改变核心功能和用户工作流的前提下，优先解决入口文件臃肿、前端 IPC 调用分散、Tauri 命令层与业务层耦合、前后端契约重复维护的问题，恢复后续功能迭代速度。

**版本策略**:

| 版本 | 类型 | 目标 |
|------|------|------|
| v3.4.0 | MINOR | 架构收口一期：拆前端应用壳、统一 IPC 调用边界、精简 Tauri 启动入口 ✅ |
| v3.5.0 | MINOR | 架构收口二期：抽离后端 service 边界、按功能整理前端模块、补齐回归测试基线 |
| v3.6.0 | MINOR | 架构收口三期：统一 Settings/Session/Report 契约，收敛全局状态和错误模型 |

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

### v3.6.0（架构收口三期）进行中

**目标**: 统一契约、错误模型和全局状态边界。

**版本类型**: MINOR（内部架构重构，保持行为兼容）

**发现的问题**:
- 前端 `Settings` 类型定义严重过时：缺少后端大量已有字段，定义了一些后端不存在的字段
- 前端 `LogRecord` 缺少 `session_id` 和 `analysis_status` 字段
- 前端 `Session.status` 使用字符串字面量，后端使用 `SessionStatus` 枚举（但 serde rename 保证序列化兼容）

| ID | 需求 | 故事点 | 优先级 | 状态 |
|----|------|--------|--------|------|
| ARCH-006 | 统一 Settings 契约：更新前端类型定义，移除死字段，补充缺失字段 | 3pts | P0 | 进行中 |
| ARCH-007 | 统一 Record/Session 契约：补齐缺失字段，确保类型对齐 | 2pts | P0 | 待开始 |
| ARCH-008 | 建立结构化错误模型：后端返回 `AppError` 枚举，前端统一错误处理 | 3pts | P1 | 待开始 |
| ARCH-009 | 收敛全局状态：建立 `AppState` 容器，减少 `Lazy<Mutex<...>>` 扩散 | 2pts | P1 | 待开始 |
| ARCH-010 | 建立架构约束文档：明确命令层 vs service 层边界规则 | 1pt | P2 | 待开始 |

**ARCH-006 进展记录**:
- 🔄 前端 `Settings` 类型修复中
- 移除死字段: `silence_detection_enabled`, `silence_threshold`, `window_filter_enabled`, `window_filter_mode`, `window_filter_list`, `multi_monitor_mode`, `custom_prompt`, `default_obsidian_vault`
- 补充缺失字段: `summary_model_name`, `analysis_prompt`, `summary_prompt`, `window_whitelist`, `window_blacklist`, `use_whitelist_only`, `auto_adjust_silent`, `capture_mode`, `proxy_*`, `quality_filter_*`, `auto_backup_*` 等

### 未来 Milestone 概要

| 版本 | 方向 | 说明 |
|------|------|------|
| v3.5.0 | 架构收口二期 | 抽取 Settings/Session/Report/Capture service，按功能整理前端目录，降低模块耦合 |
| v3.6.0 | 架构收口三期 | 统一前后端契约、错误模型和状态边界，减少重复字段和散落单例 |
| v3.7.0 | 多维度输出与标签管理 | 在架构稳定后恢复 Vault、标签、导出等功能增强版本 |
| v4.0.0 | 保留 | 仅当后续确实涉及不兼容的数据模型或分析管线变更时再考虑启用 |

---

## 最近 10 个已完成版本摘要

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

### v2.9.0 — 截图重新分析 ✅
- Gallery 卡片新增重新分析按钮
- 支持单张截图一键触发 AI 重分析

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
