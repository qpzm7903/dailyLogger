# DailyLogger 项目规划

> 最后更新: 2026-03-27
> 当前版本: v3.4.0（架构收口一期）
> 下一版本: v3.5.0（架构收口二期）
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

### v3.5.0（架构收口二期）🚧 进行中

**目标**: 抽出 service 边界，降低命令层和具体模块实现的耦合。

**版本类型**: MINOR（内部架构重构，保持行为兼容）

| ID | 需求 | 故事点 | 优先级 | 状态 | Spec |
|----|------|--------|--------|------|------|
| ARCH-004 | 建立 `commands -> services` 调用边界，避免命令入口继续承载业务实现 | 3pts | P0 | 🚧 进行中 | `specs/ARCH-001-architecture-refactor.md` |
| ARCH-005 | 为本轮重构补齐回归基线：cargo test、cargo clippy、前端 typecheck 和 test | 2pts | P1 | ⏳ 规划中 | `specs/ARCH-001-architecture-refactor.md` |

**验收重点**:

- 命令层不再直接承载复杂业务编排
- `src-tauri/src/commands/` 包含所有 `#[tauri::command]` 入口函数
- `src-tauri/src/services/` 包含各领域业务逻辑
- 新增功能时，修改范围可控制在单一 service 内

### 未来 Milestone 概要

| 版本 | 方向 | 说明 |
|------|------|------|
| v3.5.0 | 架构收口二期 | 抽取 Settings/Session/Report/Capture service，按功能整理前端目录，降低模块耦合 |
| v3.6.0 | 架构收口三期 | 统一前后端契约、错误模型和状态边界，减少重复字段和散落单例 |
| v3.7.0 | 多维度输出与标签管理 | 在架构稳定后恢复 Vault、标签、导出等功能增强版本 |
| v4.0.0 | 保留 | 仅当后续确实涉及不兼容的数据模型或分析管线变更时再考虑启用 |

---

## 最近 10 个已完成版本摘要

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
