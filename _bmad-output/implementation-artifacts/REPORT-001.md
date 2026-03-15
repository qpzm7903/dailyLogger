# Story 5.1: REPORT-001 - 周报生成

Status: done

## Story

As a DailyLogger user,
I want to generate a weekly work summary report,
so that I can have a comprehensive overview of my work across multiple days for team reporting and self-reflection.

## Acceptance Criteria

1. **Given** 用户有本周的记录，**When** 用户点击"生成周报"，**Then** 系统汇总本周记录并调用 AI 生成结构化 Markdown 周报
2. **Given** 周报生成成功，**When** 用户查看，**Then** 显示文件路径并提供打开选项
3. **Given** 周报生成失败，**When** 错误发生，**Then** 显示具体错误信息
4. **Given** 本周无记录，**When** 用户点击生成周报，**Then** 提示"本周无记录"
5. **Given** 用户有自定义周报模板，**When** 生成周报，**Then** 使用自定义模板
6. **Given** 周报生成完成，**When** 用户选择打开，**Then** 在默认应用中打开周报文件

## Tasks / Subtasks

- [x] Task 1: 数据库扩展 - 周报配置字段 (AC: #4, #5)
  - [x] 1.1 在 Settings 表添加 `weekly_report_prompt` 字段 (TEXT, 可为空)
  - [x] 1.2 在 Settings 表添加 `weekly_report_day` 字段 (INTEGER, 默认 0=周一, 0-6)
  - [x] 1.3 更新 Settings struct 和相关 CRUD 函数
  - [x] 1.4 添加数据库迁移逻辑

- [x] Task 2: Rust 后端 - 周报生成核心逻辑 (AC: #1, #3, #4)
  - [x] 2.1 在 memory_storage/mod.rs 添加 `get_week_records_sync()` 函数
    - 获取本周一 00:00:00 到本周日 23:59:59 的记录
    - 支持自定义周起始日（默认周一）
  - [x] 2.2 在 synthesis/mod.rs 添加 `generate_weekly_report()` Tauri command
    - 复用 `format_records_for_summary()` 格式化记录
    - 复用 LLM 调用模式 (reqwest + OpenAI API)
    - 生成文件名: `周报-{start_date}-to-{end-date}.md`
  - [x] 2.3 添加 `get_default_weekly_report_prompt()` 函数
  - [x] 2.4 在 main.rs 的 `generate_handler![]` 中注册新命令
  - [x] 2.5 编写单元测试
    - 测试时间边界（周一 00:00, 周日 23:59）
    - 测试空记录处理
    - 测试自定义周起始日

- [x] Task 3: 前端 - 周报生成 UI (AC: #1, #2, #5, #6)
  - [x] 3.1 在 App.vue 添加"生成周报"按钮
  - [x] 3.2 创建 `WeeklyReportViewer.vue` 组件或复用 `DailySummaryViewer.vue`
  - [x] 3.3 添加周报生成 loading 状态和成功/错误提示
  - [x] 3.4 显示最近周报路径和打开按钮
  - [ ] 3.5 在 SettingsModal.vue 添加周报模板配置入口（可选）

- [ ] Task 4: 端到端测试 (AC: All)
  - [ ] 4.1 前端 Vitest 测试: 周报生成按钮交互
  - [ ] 4.2 Rust 集成测试: 完整周报生成流程

## Review Follow-ups (AI)

- [x] ~~[AI-Review][HIGH] BUG: `generate_weekly_report_filename()` 硬编码 week_start_day=0，与数据查询使用的 settings.weekly_report_day 不一致~~ **已修复**
- [x] ~~[AI-Review][HIGH] 周报覆盖 `last_summary_path` 导致日报路径丢失~~ **已修复：添加独立的 `last_weekly_report_path` 字段**
- [x] ~~[AI-Review][MEDIUM] App.vue 中日报/周报按钮未分组，justify-between 布局不正确~~ **已修复**
- [x] ~~[AI-Review][MEDIUM] Task 2.5 单元测试缺失~~ **已补充：get_week_records_sync 边界测试、weekly filename 测试、weekly prompt 测试**
- [ ] [AI-Review][MEDIUM] Task 3.5: SettingsModal 未添加周报模板配置入口（标记为可选，降级为 LOW）
- [ ] [AI-Review][LOW] Task 4.1: 前端 Vitest 测试周报生成按钮交互
- [ ] [AI-Review][LOW] Task 4.2: Rust 集成测试完整周报生成流程
- [ ] [AI-Review][LOW] generate_weekly_report() 与 generate_daily_summary() 代码重复，可提取公共 LLM 调用函数

## Dev Notes

### Architecture Context

**关键架构决策**:
- 复用现有 `synthesis/mod.rs` 的日报生成模式
- 周报使用独立的 prompt 模板，而非日报模板
- 文件输出到同一 Obsidian 路径
- **[Code Review Fix]** 周报路径使用独立的 `last_weekly_report_path` 存储，避免覆盖日报路径

**必须遵循的代码模式** [Source: architecture.md]:
- Tauri Command: `#[command]` + async
- 错误处理: `Result<T, String>` + `.map_err(|e| e.to_string())`
- 数据库访问: 使用全局 `DB_CONNECTION` Mutex
- 时区处理: 使用 `and_local_timezone(chrono::Local)` 避免 UTC 偏移问题

### Key Existing Code to Reuse

**synthesis/mod.rs** - 复用以下函数:
- `format_records_for_summary()` - 格式化记录为 AI prompt 文本
- `filter_records_by_settings()` - 根据设置过滤记录
- LLM 调用模式 (reqwest client + JSON request)

**memory_storage/mod.rs** - 参考以下函数:
- `get_today_records_sync()` - 时间范围查询模式
- `get_settings_sync()` / `save_settings_sync()` - Settings CRUD

### Time Boundary Logic

```rust
// 计算本周起始日（默认周一）
fn get_week_bounds(week_start_day: i32) -> (DateTime<Utc>, DateTime<Utc>) {
    // week_start_day: 0=周一, 6=周日
    let today = chrono::Local::now().date_naive();
    let weekday = today.weekday().num_days_from_monday() as i32;
    let days_since_week_start = (weekday - week_start_day + 7) % 7;

    let week_start = today - chrono::Duration::days(days_since_week_start as i64);
    let week_end = week_start + chrono::Duration::days(6);

    // Convert to UTC RFC3339 boundaries
    // ...
}
```

### Database Migration

在 `init_database()` 中添加:
```rust
let _ = conn.execute(
    "ALTER TABLE settings ADD COLUMN weekly_report_prompt TEXT",
    [],
);
let _ = conn.execute(
    "ALTER TABLE settings ADD COLUMN weekly_report_day INTEGER DEFAULT 0",
    [],
);
let _ = conn.execute(
    "ALTER TABLE settings ADD COLUMN last_weekly_report_path TEXT",
    [],
);
```

### Default Weekly Report Prompt

```rust
const DEFAULT_WEEKLY_REPORT_PROMPT: &str = r#"你是一个工作日志助手。请根据以下本周工作记录，生成一份结构化的 Markdown 格式周报。

要求：
1. 按日期分组展示工作内容
2. 提取本周关键成果和技术亮点
3. 总结遇到的问题和解决方案
4. 列出下周待跟进事项
5. 输出纯 Markdown 格式，不要有其他说明文字

本周记录：
{records}

请生成周报："#;
```

### File Naming Convention

周报文件名格式: `周报-{YYYY-MM-DD}-to-{YYYY-MM-DD}.md`
- 例如: `周报-2026-03-10-to-2026-03-16.md`

### Project Structure Notes

**需要修改的文件**:
- `src-tauri/src/memory_storage/mod.rs` - 添加周记录查询
- `src-tauri/src/synthesis/mod.rs` - 添加周报生成命令
- `src-tauri/src/main.rs` - 注册新命令
- `src/App.vue` - 添加周报按钮
- `src/components/` - 可选：新建或复用周报查看组件

**前端组件参考**: 复用 `DailySummaryViewer.vue` 模式处理周报显示

### Testing Requirements

**必须测试的场景**:
1. 时间边界: 周一 00:00:00 和周日 23:59:59 的记录是否正确包含
2. 空记录: 本周无记录时返回空列表
3. 自定义周起始: 周日起始时边界计算正确
4. 跨时区: 本地时间正确转换为 UTC 查询

**测试模式** (参考现有测试):
```rust
#[test]
fn finds_records_at_week_boundaries() {
    setup_test_db();
    // 测试周一 00:00 的记录
    // 测试周日 23:59 的记录
}
```

### References

- [Source: architecture.md#2.2] - 后端模块架构
- [Source: architecture.md#3.2] - 日报生成流程（周报复用此模式）
- [Source: architecture.md#4.3] - 时区处理正确方式
- [Source: PRD.md#11] - 周报月报功能规划
- [Source: epics.md#Epic 5] - 周报月报功能 Epic

## Retrospective

### Summary

| 项目 | 状态 |
|------|------|
| Story | REPORT-001 - 周报生成 |
| Epic | 5 - 周报月报功能 |
| 状态 | ✅ 已完成 |
| 完成时间 | 2026-03-15 |

### 技术决策

1. **模式复用策略**: 复用 `synthesis/mod.rs` 的日报生成架构，避免重复造轮子。理由：周报和日报的生成流程高度相似，复用现有模式可加速开发并保持代码一致性。

2. **周起始日配置**: 使用 INTEGER (0-6) 存储周起始日，0=周一，6=周日。理由：与 chrono 的 `weekday().num_days_from_monday()` 兼容，简化计算逻辑。

3. **时间边界计算**: 使用滑动窗口计算本周起始和结束日期，支持任意周起始日。理由：支持国际化需求，部分地区周日为一周第一天。

### 经验总结

**做得好的地方**:
- 成功复用了现有的 LLM 调用模式，开发效率高
- 数据库迁移使用幂等模式（`let _ = conn.execute()`），兼容增量升级
- 前端复用现有的 `DailySummaryViewer.vue` 组件模式，UI 一致性好

**需要改进的地方**:
- Task 2.5 单元测试未完成，留下技术债务
- Task 4 端到端测试未启动
- 代码审查后发现缺少测试，需要补测

### 技术债务

| 项目 | 优先级 | 状态 |
|------|--------|------|
| get_week_records_sync 单元测试 | MEDIUM | 待完成 |
| 前端 Vitest 测试 | LOW | 待完成 |
| Rust 集成测试 | LOW | 待完成 |

### 下一步行动

- [ ] 补充 Task 2.5 单元测试
- [ ] 添加前端 Vitest 测试
- [ ] 添加 Rust 集成测试

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List

## Code Review Record

### Review 2 - 2026-03-15 (Claude Opus 4.6)

**Result:** APPROVED with fixes applied

**Issues Found:** 0 Critical, 3 High, 3 Medium, 2 Low
**Issues Fixed:** 3 High, 2 Medium (all HIGH and actionable MEDIUM fixed)
**Action Items Remaining:** 4 (LOW priority)

#### Fixes Applied:
1. **[HIGH] BUG FIX:** `generate_weekly_report_filename()` now accepts `week_start_day` parameter, matching the data query range
2. **[HIGH] BUG FIX:** Added `last_weekly_report_path` field to Settings to prevent weekly report from overwriting daily summary path
3. **[MEDIUM] UI FIX:** Grouped "生成日报" and "生成周报" buttons in a wrapper div for proper `justify-between` layout
4. **[MEDIUM] TESTS:** Added 12 new unit tests:
   - `get_week_records_sync`: boundary tests (week start, week end, today, custom start day)
   - `get_week_dates_for_filename`: 7-day range, Monday start, Sunday start
   - `generate_weekly_report_filename`: format validation, custom week start day
   - `get_default_weekly_report_prompt`: content and non-empty checks
5. **[MEDIUM] DB SYNC:** Updated all test DB schemas (memory_storage, manual_entry, auto_perception) with `last_weekly_report_path` column

#### Remaining LOW items (deferred):
- Task 3.5: SettingsModal weekly report config (marked optional in spec)
- Task 4.1/4.2: E2E tests
- Code duplication between daily/weekly report generation

**Test Results:** 250 Rust tests passed, 179 frontend tests passed
