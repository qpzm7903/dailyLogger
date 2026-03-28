---
stepsCompleted: ["step-01-validate-prerequisites", "step-02-design-epics", "step-03-create-stories"]
inputDocuments: ["/workspace/_bmad-output/planning-artifacts/PRD.md", "/workspace/_bmad-output/planning-artifacts/architecture.md"]
---

# DailyLogger - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for DailyLogger v3.4.0, decomposing the remaining requirements from the PRD, Architecture requirements, and identified technical debt into implementable stories.

## Requirements Inventory

### Functional Requirements (Remaining from PRD Section 11)

**未实现的 P2 功能:**

FR11: 多 Obsidian Vault 支持 - 支持不同项目输出到不同 Vault
FR12: 标签系统 - 手动给记录打标签便于检索
FR13: 数据导出 - 导出 JSON/Markdown 备份

**已实现 (v3.0.0 ~ v3.3.0):**
- FR1~FR7: 核心功能 (自动感知、闪念胶囊、日报生成、截图回顾、系统托盘、设置管理、工作时段管理)
- FR8: 智能截图质量评分 (EXP-002)
- FR9: 工作时间线可视化 (EXP-001)
- FR10: 今日工作摘要 Widget (EXP-005)
- FR_全文搜索: 全文搜索 (EXP-004)
- FR_记录重分析: 记录重分析 (EXP-003)

### NonFunctional Requirements

**From PRD Section 7 (未完全解决):**

NFR1: 性能 - 应用启动时间 <3秒，截图处理延迟 <2秒，AI 分析延迟 <10秒，日报生成时间 <30秒(100条记录)，内存占用 <200MB
NFR2: 安全 - API Key 本地加密存储 (AES-256)，不上传用户数据到除 AI API 外的任何服务，截图仅本地处理和存储
NFR3: 兼容性 - Windows 10+ / macOS 11+ / Ubuntu 20.04+
NFR4: 可用性 - 离线模式正常，AI 调用失败时保留截图并提示重试，自动重连

**已实现:**
- NFR1 性能基准测试 ✅ (CORE-008)
- NFR2 安全 ✅ (CORE-006 API Key 加密)
- NFR4 可用性 ✅ (CORE-004 错误处理)
- NFR5~NFR7 ✅ (v3.0.0 ~ v3.3.0)

### Additional Requirements

**From Architecture:**

AR1: Tauri v2 框架 - 必须使用 Tauri v2 的插件系统
AR2: Rust 后端 - 所有核心逻辑在 Rust 端实现
AR3: SQLite 数据库 - 单文件数据库，便于备份和迁移
AR4: Vue 3 前端 - 使用 Composition API 和 `<script setup>`
AR5: TailwindCSS - 唯一样式方案，无独立 CSS 文件
AR6: 日志系统 - 日志文件保存在用户目录项目命名的文件夹下
AR7: 构建相关操作 - 必须放在 GitHub Actions 上执行

**From Project Retrospective (Technical Debt):**

TD1: 测试数据库 schema 统一 - 创建 `setup_test_db_with_schema()` 机制 (High)
TD2: 数据库版本迁移 - 引入 schema_version 表或 Tauri 迁移机制 (High)
TD3: 前端组件测试覆盖 - 至少测试组件挂载、props 渲染、事件触发 (Medium)
TD4: 371 处硬编码颜色迁移 - PERF-006 组件颜色迁移到 CSS 变量 (Medium)
TD5: 游标分页前端集成 - PERF-004 API 准备就绪但前端未使用 (Medium)

### UX Design Requirements

（无独立 UX Design 文档 - UX 需求从 Epic 9/10 的实现中继承）

### FR Coverage Map

| FR | 描述 | Epic | Story | 状态 |
|----|------|------|-------|------|
| FR1~FR7 | 核心功能 | Epic 1~8 | CORE/SESSION | ✅ 已实现 |
| FR8 | 智能截图质量评分 | Epic 7 | EXP-002 | ✅ 已实现 |
| FR9 | 工作时间线可视化 | Epic 7 | EXP-001 | ✅ 已实现 |
| FR10 | 今日工作摘要 Widget | Epic 7 | EXP-005 | ✅ 已实现 |
| FR_搜索 | 全文搜索 | Epic 7 | EXP-004 | ✅ 已实现 |
| FR_重分析 | 记录重分析 | Epic 7 | EXP-003 | ✅ 已实现 |
| FR11 | 多 Obsidian Vault 支持 | Epic 12 | VAULT-001 | ⏳ 待实现 |
| FR12 | 标签系统 | Epic 12 | TAG-001 | ⏳ 待实现 |
| FR13 | 数据导出 | Epic 13 | EXPORT-001 | ⏳ 待实现 |

## Epic List

### Epic 12: 多维度输出与标签管理 (OUTPUT) ✅ 完成 (v3.7.1-v3.9.0)

**Goal:** 扩展输出能力，支持多 Vault、标签分类和数据导出，让用户更好地组织和复用工作记录

**Priority:** P2

**Stories:**

- [x] VAULT-001: 多 Obsidian Vault 支持 ✅ (v3.9.0)
- [x] TAG-001: 统一标签系统 ✅ (v3.7.1)
- [x] EXPORT-001: 数据导出功能 (JSON/Markdown) ✅ (v3.8.0)

---

### Epic 13: 技术债务清偿 (DEBT) ✅ 完成 (v4.0.0)

**Goal:** 清偿关键技术债务，提升代码质量、测试覆盖和长期可维护性

**Priority:** P1

**Stories:**

- [x] DEBT-001: 测试数据库 schema 统一 ✅ (v4.0.0)
- [x] DEBT-002: 数据库版本迁移机制 ✅ (v4.0.0)
- [x] DEBT-003: 组件颜色迁移到 CSS 变量 ✅ (v4.0.0)

---

## v4.1.0 已完成 ✅ (2026-03-28)

**已完成:**
- 前端组件测试覆盖率提升 (40+ 组件测试，1165 tests)
- 62 个测试文件全部通过

---

## v4.2.0 规划中

**候选方向:**
- 性能监控与自动化基准测试
- 新功能开发（待定）
- 用户体验优化

