---
stepsCompleted: ["step-01-validate-prerequisites"]
inputDocuments: ["/workspace/_bmad-output/planning-artifacts/PRD.md", "/workspace/_bmad-output/planning-artifacts/architecture.md"]
---

# DailyLogger - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for DailyLogger, decomposing the requirements from the PRD, UX Design if it exists, and Architecture requirements into implementable stories.

## Requirements Inventory

### Functional Requirements

**From PRD Section 6 (Implemented Features):**

FR1: 自动感知 - 定时截取屏幕并按工作时段组织，通过批量上下文分析提供准确的工作记录
FR2: 闪念胶囊 - 全局快捷键快速记录想法，不打断工作流
FR3: AI 日报生成 - 汇总全天记录，生成结构化 Markdown 日报
FR4: 截图回顾 - 网格展示当日截图缩略图，点击查看大图
FR5: 系统托盘 - 最小化到托盘，后台静默运行
FR6: 设置管理 - 配置应用参数（API、截图间隔、Obsidian路径等）
FR7: 工作时段管理 - 将连续截图按工作时段自动分组，以时段为单位进行上下文感知的 AI 分析

**From PRD Section 11 (Future Planning - 未实现):**

FR8: 智能截图质量评分 - 自动识别低质量截图并过滤，减少记录噪音 (P1)
FR9: 工作时间线可视化 - 图形化展示一天工作流，让用户直观回顾 (P1)
FR10: 今日工作摘要 Widget - 实时展示当天已记录内容，随时感知进度 (P1)

**已实现 (Epic 7 - EXP-001~EXP-005):**
- EXP-001: 工作时间线视图 ✅
- EXP-002: 截图质量过滤 ✅
- EXP-003: 记录重分析 ✅
- EXP-004: 全文搜索 ✅ (注意：PRD FR10 全文搜索与 EXP-004 重复)
- EXP-005: 今日工作摘要 Widget ✅

### NonFunctional Requirements

**From PRD Section 7:**

NFR1: 性能 - 应用启动时间 <3秒，截图处理延迟 <2秒，AI 分析延迟 <10秒，日报生成时间 <30秒(100条记录)，内存占用 <200MB
NFR2: 安全 - API Key 本地加密存储 (AES-256)，不上传用户数据到除 AI API 外的任何服务，截图仅本地处理和存储
NFR3: 兼容性 - Windows 10+ / macOS 11+ / Ubuntu 20.04+，截图支持 Graphics Capture API (Windows) / xcap (macOS/Linux)
NFR4: 可用性 - 离线模式正常，AI 调用失败时保留截图并提示重试，自动重连

**From Architecture Section 10:**

NFR5: 截图去重优化 - 指纹对比 + 阈值，减少 70% AI 调用
NFR6: 数据库索引 - timestamp DESC 查询 <10ms
NFR7: 前端轮询优化 - 30秒间隔降低 IPC 调用频率

**已实现:**
- NFR1 性能基准测试 ✅ (CORE-008)
- NFR2 安全 ✅ (CORE-006 API Key 加密)
- NFR4 可用性 ✅ (CORE-004 错误处理)
- NFR5 截图去重 ✅ (v3.0.0)
- NFR6 数据库索引 ✅ (PERF-004)
- NFR7 前端轮询优化 ✅ (PERF-003 虚拟滚动)

### Additional Requirements

**From Architecture:**

AR1: Tauri v2 框架 - 必须使用 Tauri v2 的插件系统
AR2: Rust 后端 - 所有核心逻辑在 Rust 端实现
AR3: SQLite 数据库 - 单文件数据库，便于备份和迁移
AR4: Vue 3 前端 - 使用 Composition API 和 `<script setup>`
AR5: TailwindCSS - 唯一样式方案，无独立 CSS 文件
AR6: 跨平台截图 - Windows: Windows Graphics Capture API, macOS/Linux: xcap
AR7: 日志系统 - 日志文件保存在用户目录项目命名的文件夹下
AR8: 构建相关操作 - 必须放在 GitHub Actions 上执行（本地缺少多环境构建环境）

**From PRD Section 10:**

AR9: AI API 成本控制 - 支持本地模型，可配置调用频率
AR10: 截图隐私 - 支持白名单模式

### UX Design Requirements

（无 UX Design 文档 - planning-artifacts 中不存在 UX 相关文件）

### FR Coverage Map

| FR | 描述 | Epic | Story | 状态 |
|----|------|------|-------|------|
| FR1 | 自动感知 | Epic 1 | CORE-001~CORE-008 | ✅ 已实现 |
| FR2 | 闪念胶囊 | Epic 1 | CORE-001~CORE-008 | ✅ 已实现 |
| FR3 | AI 日报生成 | Epic 1, Epic 5 | CORE-003, REPORT-001~004 | ✅ 已实现 |
| FR4 | 截图回顾 | Epic 1, Epic 4 | CORE-002, DATA-001 | ✅ 已实现 |
| FR5 | 系统托盘 | Epic 1 | CORE-005 | ✅ 已实现 |
| FR6 | 设置管理 | Epic 1 | CORE-001 | ✅ 已实现 |
| FR7 | 工作时段管理 | Epic 8 | SESSION-001~005 | ✅ 已实现 |
| FR8 | 智能截图质量评分 | Epic 7 | EXP-002 | ✅ 已实现 (v3.2.0) |
| FR9 | 工作时间线可视化 | Epic 7 | EXP-001 | ✅ 已实现 (v3.2.0) |
| FR10 | 今日工作摘要 Widget | Epic 7 | EXP-005 | ✅ 已实现 (v3.2.0) |

## Epic List

### Epic 11: 数据增强与稳定性 (Data Enhancement & Stability)

**Goal:** 增强数据管理能力、提升系统稳定性、为未来扩展打好基础

**Priority:** P1

**Stories:**

- [ ] DATA-007: 多语言日报导出
- [ ] DATA-008: 数据统计面板
- [ ] STAB-001: 错误边界与优雅降级
- [ ] STAB-002: 自动备份与恢复

