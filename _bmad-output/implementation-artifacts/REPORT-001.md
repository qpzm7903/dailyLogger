# Story 5.1: REPORT-001 - 周报生成

Status: ready-for-dev

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

- [ ] Task 1: 数据库扩展 - 周报配置字段 (AC: #4, #5)
  - [ ] 1.1 在 Settings 表添加 `weekly_report_prompt` 字段 (TEXT, 可为空)
  - [ ] 1.2 在 Settings 表添加 `weekly_report_day` 字段 (INTEGER, 默认 0=周一, 0-6)
  - [ ] 1.3 更新 Settings struct 和相关 CRUD 函数
  - [ ] 1.4 添加数据库迁移逻辑

- [ ] Task 2: Rust 后端 - 周报生成核心逻辑 (AC: #1, #3, #4)
  - [ ] 2.1 在 memory_storage/mod.rs 添加 `get_week_records_sync()` 函数
    - 获取本周一 00:00:00 到本周日 23:59:59 的记录
    - 支持自定义周起始日（默认周一）
  - [ ] 2.2 在 synthesis/mod.rs 添加 `generate_weekly_report()` Tauri command
    - 复用 `format_records_for_summary()` 格式化记录
    - 复用 LLM 调用模式 (reqwest + OpenAI API)
    - 生成文件名: `周报-{start_date}-to-{end-date}.md`
  - [ ] 2.3 添加 `get_default_weekly_report_prompt()` 函数
  - [ ] 2.4 在 main.rs 的 `generate_handler![]` 中注册新命令
  - [ ] 2.5 编写单元测试
    - 测试时间边界（周一 00:00, 周日 23:59）
    - 测试空记录处理
    - 测试自定义周起始日

- [ ] Task 3: 前端 - 周报生成 UI (AC: #1, #2, #5, #6)
  - [ ] 3.1 在 App.vue 添加"生成周报"按钮
  - [ ] 3.2 创建 `WeeklyReportViewer.vue` 组件或复用 `DailySummaryViewer.vue`
  - [ ] 3.3 添加周报生成 loading 状态和成功/错误提示
  - [ ] 3.4 显示最近周报路径和打开按钮
  - [ ] 3.5 在 SettingsModal.vue 添加周报模板配置入口（可选）

- [ ] Task 4: 端到端测试 (AC: All)
  - [ ] 4.1 前端 Vitest 测试: 周报生成按钮交互
  - [ ] 4.2 Rust 集成测试: 完整周报生成流程

## Dev Notes

### Architecture Context

**关键架构决策**:
- 复用现有 `synthesis/mod.rs` 的日报生成模式
- 周报使用独立的 prompt 模板，而非日报模板
- 文件输出到同一 Obsidian 路径

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

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List