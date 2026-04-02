# Sprint 变更提案 v2

**日期**: 2026-03-22
**提案类型**: 架构重构 — 分析管线重设计 + 冗余功能清理
**变更范围**: Major（重大）
**状态**: ✅ 已批准

---

## 第1节：问题陈述

### 触发背景

产品负责人 Weiyicheng 在日常使用 DailyLogger 过程中发现两个结构性问题：

### 问题 A：外部统计集成与实际使用场景不符

用户明确声明不使用这类外部统计功能。相关实现共计 **1,360+ 行代码**（866 行 Rust + 181 行 Vue + 313 行测试）在项目中没有任何实际价值，增加维护负担和包体积，在 Dashboard 占据 UI 空间。

### 问题 B：单张截图分析的根本性设计缺陷

当前 `capture_and_store()` 在每次截图后立即调用 `analyze_screen()`，对单张截图做独立分析。这违反了一个基本事实：**工作是连续的**。

| 问题 | 表现 |
|------|------|
| 上下文缺失 | AI 无法区分"刚打开 VS Code"和"编码 2 小时"，因为它只看到一张静态截图 |
| 分析粒度错误 | 5 分钟一张截图，每张独立分析，产生大量重复且碎片化的结果 |
| 用户无法纠正 | AI 分析结果不准确时，用户没有便捷的编辑/自写途径 |
| API 成本浪费 | 每张截图一次 Vision API 调用，成本高且大量分析结果质量不佳 |

用户的核心洞察：
> "分析是需要有上下文的，工作是有连续的，所以单靠一张图来进行分析可能不够"

---

## 第2节：影响分析

### Epic 影响

| Epic | 当前状态 | 变更 |
|------|---------|------|
| Epic 1-5 | ✅ 已完成 | 代码保留，无直接改动 |
| Epic 6 (集成扩展) | ✅ 已完成 | **先移除外部统计集成代码**；其余第三方集成当时暂保留，后续版本已全部清理 |
| Epic 7 (核心体验) | ✅ 已完成 | EXP-003（记录重分析）概念被新架构吸收并扩展 |
| **Epic 8（新增）** | 📋 新建 | **工作时段感知分析** — 核心分析管线重设计 |

### Artifact 影响

| 文档 | 变更内容 | 影响程度 |
|------|---------|---------|
| PRD | Section 6.1 重写（分离捕获与分析）；新增 Section 6.7（工作时段管理） | 重大 |
| Architecture | Section 3.1 流程重写；Section 5 新增 sessions 表；Section 2.2 新增 session_manager 模块 | 重大 |
| Epics | 新增 Epic 8（5 个 Stories）；Epic 6 标记外部统计集成已移除 | 中等 |
| 代码 | auto_perception 重构；新增 session_manager 模块；移除外部统计模块 | 重大 |
| UI | 移除外部统计面板；新增 SessionView；ScreenshotGallery 按时段分组 | 中等 |

### 技术影响

**移除清单（外部统计集成）**：

| 文件 | 行数 | 操作 |
|------|------|------|
| 外部统计后端模块 | 866 | 删除 |
| 外部统计前端面板 | 181 | 删除 |
| 外部统计前端测试 | 313 | 删除 |
| 命令注册 | 2 行 | 移除相关 command |
| 模块导出 | 1 行 | 移除相关 `pub mod` |
| `src-tauri/src/synthesis/mod.rs` | ~30 行 | 移除外部活动聚合逻辑 |
| `src/components/layout/Dashboard.vue` | ~5 行 | 移除外部统计面板引用 |
| `src/components/settings/OutputSettings.vue` | ~35 行 | 移除外部统计配置区域 |
| `src/types/tauri.ts` | ~20 行 | 移除外部统计类型定义 |
| **总计** | **~1,453 行** | **删除** |

**重构范围（分析管线）**：

| 模块 | 变更类型 | 说明 |
|------|---------|------|
| `auto_perception/mod.rs` | 重构 | `capture_and_store()` 移除 `analyze_screen()` 调用，改为存储 pending 记录 |
| 新增 `session_manager/mod.rs` | 新建 | 时段检测、创建、结束、批量分析触发 |
| `memory_storage/schema.rs` | 扩展 | 新增 sessions 表、records 表新字段 |
| `synthesis/mod.rs` | 重构 | 日报生成改为基于时段分析结果，优先使用 user_summary |
| 前端若干组件 | 重构/新建 | SessionView、ScreenshotGallery 分组、编辑 UI |

---

## 第3节：推荐路径

**选项**：直接调整（Option 1）

### 理由

1. **已有基础设施**：`capture_only_mode`（仅截图不分析）已经实现了"捕获与分析解耦"的雏形，代码中已有 placeholder content 和离线队列机制
2. **数据层就绪**：`user_notes` 字段已存在于 records 表，用户编辑的数据基础已就绪
3. **无需回滚**：不删除任何用户数据，不影响已生成的日报
4. **风险可控**：分阶段实施，每个 Story 独立可测试

### 评估

| 维度 | 评级 | 说明 |
|------|------|------|
| 工作量 | Medium-High | 外部统计集成移除简单；分析管线重构需 3-4 个 Sprint Stories |
| 风险 | Medium | 分析管线是核心路径，需要完善的测试覆盖 |
| 时间线影响 | 1 个 Sprint | 可在一个 Sprint 内完成全部变更 |
| 用户价值 | 极高 | 直接解决"分析不准确"这一核心体验痛点 |

---

## 第4节：变更提案详情

### 变更 A：移除外部统计集成

**操作**：删除上述移除清单中的全部文件和代码引用。

**数据库字段处理**：保留遗留外部平台字段以维持旧数据库文件兼容性，但不在 UI 中暴露，后续版本可通过 migration 清理。

---

### 变更 B：新增 Epic 8 — 工作时段感知分析

**新架构设计**：

```
【捕获管线】— 不调用 AI，高频运行
┌──────────────────────────────────────────────────┐
│ capture_screen()                                  │
│     → fingerprint + should_capture()             │
│     → quality_filter()                           │
│     → save_screenshot()                          │
│     → detect_or_create_session()                 │
│     → add_record(status: "pending", session_id)  │
└──────────────────────────────────────────────────┘

【时段管理】— 后台监控
┌──────────────────────────────────────────────────┐
│ session_monitor (后台任务)                         │
│     → 每次截图后检查：距上一张截图 > 30min？       │
│        YES → 结束当前时段 → 触发时段分析           │
│        NO  → 继续当前时段                         │
│     → 30min 间隔可在设置中配置                    │
└──────────────────────────────────────────────────┘

【分析管线】— 三种触发方式
┌──────────────────────────────────────────────────┐
│ B. 时段自动分析（时段结束时自动触发）               │
│    → 收集该时段所有截图 + 上一时段的 context_summary│
│    → 批量发送 AI（多图 + 文本上下文）               │
│    → 返回：每张截图的分析 + 时段摘要                │
│    → 更新 records.content + sessions.ai_summary   │
│                                                    │
│ A. 日报生成时（汇总所有时段分析）                    │
│    → 收集当天所有 session 的摘要                    │
│    → 优先使用 user_summary，fallback 到 ai_summary│
│    → 生成日报 Markdown                             │
│                                                    │
│ D. 手动触发                                        │
│    → 用户选择时间范围或指定时段                     │
│    → 执行与 B 相同的分析流程                       │
└──────────────────────────────────────────────────┘

【用户编辑层】— 覆盖 AI 结果
┌──────────────────────────────────────────────────┐
│ 截图级：                                          │
│   AI 分析 → records.content (JSON)                │
│   用户自写 → records.user_notes (优先级更高)       │
│                                                    │
│ 时段级：                                          │
│   AI 摘要 → sessions.ai_summary                   │
│   用户自写 → sessions.user_summary (优先级更高)    │
│                                                    │
│ 日报生成 / UI 展示：                               │
│   user 层有内容 → 使用 user 层                     │
│   user 层为空 → fallback 到 AI 层                  │
└──────────────────────────────────────────────────┘
```

**数据库变更**：

```sql
-- 新增 sessions 表
CREATE TABLE sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL,                    -- YYYY-MM-DD
    start_time TEXT NOT NULL,              -- RFC3339
    end_time TEXT,                         -- RFC3339, NULL = ongoing
    ai_summary TEXT,                       -- AI 生成的时段摘要
    user_summary TEXT,                     -- 用户自写的时段摘要
    context_for_next TEXT,                 -- 传递给下一时段分析的上下文
    status TEXT DEFAULT 'active'           -- active / ended / analyzed
);

CREATE INDEX idx_sessions_date ON sessions(date);

-- records 表新增字段
ALTER TABLE records ADD COLUMN session_id INTEGER REFERENCES sessions(id);
ALTER TABLE records ADD COLUMN analysis_status TEXT DEFAULT 'pending';
-- analysis_status: 'pending' | 'analyzed' | 'user_edited'

-- user_notes 已存在，直接复用
```

---

### 变更 C：Epic 8 Stories 定义

#### Epic 8: 工作时段感知分析 (Session-Aware Analysis)

**目标**: 将截图分析从"逐张即时分析"重构为"工作时段批量上下文分析"，同时支持用户编辑和手动触发

**优先级**: P0
**预计周期**: 1 Sprint

> **设计原则**: 工作是连续的。分析必须有上下文才有意义。用户对自己的工作最了解，AI 只是辅助。

| ID | 故事 | 优先级 | 估算 | 说明 |
|----|------|--------|------|------|
| SESSION-001 | 捕获与分析解耦 + 时段管理 | P0 | 5pts | 重构 capture_and_store() 移除即时分析；新增 sessions 表和 session_manager 模块；实现时段检测（30min 间隔可配置） |
| SESSION-002 | 时段批量上下文分析 | P0 | 5pts | 实现 analyze_session()：收集时段截图 + 上一时段上下文 → 批量发送 AI → 返回每张截图分析 + 时段摘要；时段结束时自动触发 |
| SESSION-003 | 分析结果用户编辑 | P0 | 3pts | 截图级：编辑 user_notes；时段级：编辑 user_summary；前端 UI（ScreenshotModal 编辑、SessionView 编辑）；日报和 UI 优先展示用户内容 |
| SESSION-004 | 手动触发分析 | P1 | 2pts | 用户选择时间范围或指定时段手动触发分析；复用 SESSION-002 的分析管线 |
| SESSION-005 | 日报生成适配 | P1 | 3pts | synthesis 模块改为基于时段分析结果生成日报；优先使用 user_summary；按时段组织内容而非按时间线平铺 |
| CLEAN-001 | 移除外部统计集成 | P0 | 2pts | 删除外部统计模块与面板及所有引用；移除 synthesis 中的相关活动聚合逻辑 |

**Story 依赖关系**：
```
SESSION-001 (基础) ─→ SESSION-002 (分析) ─→ SESSION-005 (日报)
                  ─→ SESSION-003 (编辑)
                  ─→ SESSION-004 (手动触发)
CLEAN-001 (独立，无依赖)
```

**验收条件摘要**：

**SESSION-001**:
- [ ] 截图捕获后不调用 AI，仅保存截图和 pending 记录
- [ ] 截图间隔 > 30min 自动创建新时段
- [ ] sessions 表正确记录时段开始/结束时间
- [ ] 30min 间隔可在设置中配置
- [ ] 向后兼容：现有 records 数据不受影响（session_id 可为 NULL）

**SESSION-002**:
- [ ] 时段结束时自动触发批量分析
- [ ] AI 接收该时段所有截图 + 上一时段的 context_for_next
- [ ] AI 返回每张截图的分析结果（更新 records.content）
- [ ] AI 返回时段摘要（存入 sessions.ai_summary）
- [ ] 生成 context_for_next 供下一时段使用
- [ ] 分析完成后 records.analysis_status 更新为 'analyzed'

**SESSION-003**:
- [ ] 用户可编辑每张截图的分析内容（写入 user_notes）
- [ ] 用户可编辑每个时段的摘要（写入 user_summary）
- [ ] 编辑后 analysis_status 更新为 'user_edited'
- [ ] UI 展示优先级：user_notes/user_summary > AI 分析结果

**SESSION-004**:
- [ ] 用户可手动选择时段触发分析
- [ ] 复用 SESSION-002 的分析管线
- [ ] 支持重新分析已分析过的时段

**SESSION-005**:
- [ ] 日报按时段分组组织内容
- [ ] 每个时段使用 user_summary（如有），否则 fallback 到 ai_summary
- [ ] 截图级使用 user_notes（如有），否则 fallback 到 content
- [ ] 日报质量应优于当前"逐条拼接"的方式

---

### 变更 D：PRD Section 6.1 更新

**旧内容**：

```
#### 6.1 自动感知 (Auto-Perception)

需求描述: 定时截取屏幕，调用 AI Vision API 分析工作内容

功能规格:
- 可配置截图间隔（默认 5 分钟）
- 截图自动 Base64 编码发送至 AI
- AI 分析结果存入数据库
- 可随时启动/停止

验收条件:
- Given 用户启动自动感知，When 到达设定间隔，Then 自动截图并分析
- Given 截图成功，When AI 分析完成，Then 结果存入数据库并显示在列表
```

**新内容**：

```
#### 6.1 自动感知 (Auto-Perception)

需求描述: 定时截取屏幕并按工作时段组织，通过批量上下文分析提供准确的工作记录

功能规格:
- 可配置截图间隔（默认 5 分钟）
- 截图捕获后保存到本地，不立即调用 AI
- 系统自动检测工作时段（截图间隔 > 30 分钟 = 新时段，间隔可配置）
- 时段结束后自动触发批量分析（将该时段所有截图 + 上一时段上下文一起发送 AI）
- 支持用户手动触发指定时段的分析
- 截图级和时段级分析结果均支持用户编辑/自写
- UI 和日报优先展示用户自写内容，AI 分析作为 fallback

验收条件:
- Given 用户启动自动感知，When 到达设定间隔，Then 自动截图并保存（状态：待分析）
- Given 当前时段有截图，When 距上一张截图超过 30 分钟，Then 自动结束当前时段并触发分析
- Given 时段分析触发，When 分析完成，Then 每张截图和时段摘要均更新
- Given 用户编辑分析结果，When 保存，Then 优先展示用户版本
- Given 用户手动触发分析，When 选择时段，Then 执行批量分析并更新结果

优先级: P0 (核心功能)
```

---

## 第5节：实施交接计划

### 变更范围分类：Major

本次变更涉及核心分析管线的架构重构，影响后端核心模块、数据库 schema、前端多个组件和日报生成流程。

### 实施顺序

| 阶段 | Story | 负责 | 前置条件 |
|------|-------|------|---------|
| 1 | CLEAN-001 — 移除外部统计集成 | Developer | 无 |
| 2 | SESSION-001 — 捕获解耦 + 时段管理 | Developer | 无 |
| 3 | SESSION-002 — 时段批量分析 | Developer | SESSION-001 完成 |
| 4 | SESSION-003 — 用户编辑 UI | Developer | SESSION-001 完成 |
| 5 | SESSION-004 — 手动触发分析 | Developer | SESSION-002 完成 |
| 6 | SESSION-005 — 日报生成适配 | Developer | SESSION-002 完成 |

**阶段 1-2 可并行**：CLEAN-001 与 SESSION-001 无依赖关系。

### 成功标准

- [ ] 外部统计相关代码完全移除，CI 通过
- [ ] 截图捕获不再触发即时 AI 分析
- [ ] 时段检测和自动分析正常工作
- [ ] 用户可编辑截图级和时段级分析结果
- [ ] 手动触发分析功能可用
- [ ] 日报生成基于时段分析，优先使用用户自写内容
- [ ] 所有已有 records 数据向后兼容（session_id 为 NULL 不报错）

### 版本号

本次变更为 **MAJOR** 版本（v3.0.0），因为：
- 核心分析管线架构变更
- 数据库 schema 新增表和字段
- 移除已有功能（外部统计集成）

---

**提案起草者**: Claude (Scrum Master)
**审批者**: Weiyicheng
**审批日期**: 2026-03-22
**审批状态**: ✅ 已批准
**下一步**: 开始 CLEAN-001 + SESSION-001 实施
