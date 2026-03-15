# Story 5.3: REPORT-003 - 自定义报告周期

Status: in-progress

## Story

As a DailyLogger user,
I want to define custom report periods beyond weekly and monthly,
so that I can generate reports that match my specific reporting needs (bi-weekly, quarterly, or arbitrary date ranges).

## Acceptance Criteria

1. **Given** 用户在报告界面，**When** 用户选择"自定义周期"，**Then** 显示日期范围选择器
2. **Given** 用户选择日期范围，**When** 用户点击"生成报告"，**Then** 系统汇总指定范围内的记录并生成报告
3. **Given** 用户选择"双周报"预设，**When** 生成报告，**Then** 自动计算最近两周的日期范围
4. **Given** 用户选择"季度报"预设，**When** 生成报告，**Then** 自动计算当前季度的日期范围
5. **Given** 报告生成成功，**When** 用户查看，**Then** 显示文件路径并提供打开选项
6. **Given** 指定范围无记录，**When** 用户点击生成，**Then** 提示"所选时间范围内无记录"

## Tasks / Subtasks

- [ ] Task 1: 数据库扩展 - 自定义报告配置 (AC: #1)
  - [ ] 1.1 在 Settings 表添加 `custom_report_templates` 字段 (TEXT, JSON 格式存储预设模板)
  - [ ] 1.2 定义预设模板结构:
    ```json
    {
      "presets": [
        {"id": "biweekly", "name": "双周报", "days": 14},
        {"id": "quarterly", "name": "季度报", "calc": "quarter"}
      ],
      "custom": []
    }
    ```
  - [ ] 1.3 更新 Settings struct 和 CRUD 函数
  - [ ] 1.4 添加数据库迁移逻辑

- [ ] Task 2: Rust 后端 - 自定义周期报告生成 (AC: #2, #3, #4, #6)
  - [ ] 2.1 在 memory_storage/mod.rs 添加 `get_records_by_date_range_sync()` 函数
    - 参数: start_date, end_date (NaiveDate)
    - 返回指定范围内的所有记录
  - [ ] 2.2 在 synthesis/mod.rs 添加 `generate_custom_report()` Tauri command
    - 参数: start_date, end_date, report_name (可选)
    - 复用 `format_records_for_summary()` 格式化记录
    - 复用 LLM 调用模式
    - 生成文件名: `{report_name}-{start_date}-to-{end-date}.md`
  - [ ] 2.3 添加日期范围计算辅助函数:
    - `get_biweekly_range()` - 计算双周范围
    - `get_quarter_range()` - 计算季度范围
  - [ ] 2.4 在 main.rs 的 `generate_handler![]` 中注册新命令
  - [ ] 2.5 编写单元测试
    - 测试日期边界
    - 测试空记录处理
    - 测试跨月/跨年范围

- [ ] Task 3: 前端 - 自定义周期报告 UI (AC: #1, #2, #3, #4, #5)
  - [ ] 3.1 创建 `CustomReportModal.vue` 组件
    - 日期范围选择器 (两个 date input)
    - 预设选项: 双周报、季度报
    - 报告名称输入 (可选)
  - [ ] 3.2 在 App.vue 添加"自定义报告"入口按钮
  - [ ] 3.3 实现日期选择和预设切换逻辑
  - [ ] 3.4 添加报告生成 loading 状态和成功/错误提示
  - [ ] 3.5 显示最近报告路径和打开按钮

- [ ] Task 4: 端到端测试 (AC: All)
  - [ ] 4.1 前端 Vitest 测试: 日期选择器交互
  - [ ] 4.2 Rust 集成测试: 完整自定义报告生成流程

## Dev Notes

### Architecture Context

**关键架构决策**:
- 复用现有 `synthesis/mod.rs` 的报告生成模式
- 自定义报告使用通用的日期范围参数
- 与日报/周报共用 Prompt 模板机制

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
- 参考 `generate_daily_summary()` 和 `generate_weekly_report()` 实现

**memory_storage/mod.rs** - 参考以下函数:
- `get_today_records_sync()` - 时间范围查询模式
- `get_week_records_sync()` - 周记录查询模式 (REPORT-001 实现)

### Date Range Calculation Logic

```rust
// 计算双周范围
fn get_biweekly_range() -> (NaiveDate, NaiveDate) {
    let today = chrono::Local::now().date_naive();
    let end = today;
    let start = today - chrono::Duration::days(13);
    (start, end)
}

// 计算季度范围
fn get_quarter_range() -> (NaiveDate, NaiveDate) {
    let today = chrono::Local::now().date_naive();
    let month = today.month();
    let quarter = (month - 1) / 3;  // 0, 1, 2, 3
    let start_month = quarter * 3 + 1;

    let start = NaiveDate::from_ymd_opt(today.year(), start_month, 1).unwrap();
    let end = if quarter == 3 {
        NaiveDate::from_ymd_opt(today.year() + 1, 1, 1).unwrap() - chrono::Duration::days(1)
    } else {
        NaiveDate::from_ymd_opt(today.year(), start_month + 3, 1).unwrap() - chrono::Duration::days(1)
    };
    (start, end)
}

// 通用日期范围记录查询
pub fn get_records_by_date_range_sync(start_date: NaiveDate, end_date: NaiveDate) -> Result<Vec<Record>, String> {
    let start_dt = start_date.and_hms_opt(0, 0, 0).unwrap()
        .and_local_timezone(chrono::Local).unwrap()
        .with_timezone(&chrono::Utc);
    let end_dt = end_date.and_hms_opt(23, 59, 59).unwrap()
        .and_local_timezone(chrono::Local).unwrap()
        .with_timezone(&chrono::Utc);

    // Query records between start_dt and end_dt
}
```

### Default Custom Report Prompt

```rust
const DEFAULT_CUSTOM_REPORT_PROMPT: &str = r#"你是一个工作日志助手。请根据以下指定时间段的工作记录，生成一份结构化的 Markdown 格式报告。

要求：
1. 按日期分组展示工作内容
2. 提取该时间段的关键成果和技术亮点
3. 总结遇到的问题和解决方案
4. 列出后续待跟进事项
5. 输出纯 Markdown 格式，不要有其他说明文字

时间段：{start_date} 至 {end_date}
记录：
{records}

请生成报告："#;
```

### File Naming Convention

自定义报告文件名格式: `{report_name}-{YYYY-MM-DD}-to-{YYYY-MM-DD}.md`
- 默认 report_name: "自定义报告"
- 例如: `自定义报告-2026-03-01-to-2026-03-14.md`
- 双周报: `双周报-2026-03-01-to-2026-03-14.md`
- 季度报: `季度报-2026-01-01-to-2026-03-31.md`

### Database Migration

在 `init_database()` 中添加:
```rust
let _ = conn.execute(
    "ALTER TABLE settings ADD COLUMN custom_report_templates TEXT",
    [],
);
```

### Project Structure Notes

**需要修改的文件**:
- `src-tauri/src/memory_storage/mod.rs` - 添加日期范围记录查询
- `src-tauri/src/synthesis/mod.rs` - 添加自定义报告生成命令
- `src-tauri/src/main.rs` - 注册新命令
- `src/App.vue` - 添加自定义报告入口
- `src/components/CustomReportModal.vue` - 新建自定义报告模态框

**前端组件参考**:
- 复用 `DailySummaryViewer.vue` 模式处理报告显示
- 参考 `SettingsModal.vue` 的日期输入实现

### Testing Requirements

**必须测试的场景**:
1. 日期边界: 起始日 00:00:00 和结束日 23:59:59 的记录是否正确包含
2. 空记录: 指定范围无记录时返回空列表
3. 跨月/跨年: 范围跨越月份或年份时正确包含所有记录
4. 双周预设: 正确计算最近 14 天
5. 季度预设: 正确计算当前季度首尾日期
6. 时区转换: 本地时间正确转换为 UTC 查询

**测试模式** (参考现有测试):
```rust
#[test]
fn finds_records_at_custom_range_boundaries() {
    setup_test_db();
    // 测试起始日 00:00 的记录
    // 测试结束日 23:59 的记录
}

#[test]
fn calculates_quarter_range_correctly() {
    // 测试 Q1: 1/1 - 3/31
    // 测试 Q4: 10/1 - 12/31
}
```

### Dependencies

**依赖关系**:
- REPORT-001 (周报生成) 应已完成，可复用其生成逻辑
- REPORT-002 (月报生成) 应已完成，可参考其月份范围计算

### UI/UX Considerations

**日期选择器要求**:
- 使用 HTML5 native `<input type="date">` 或第三方库
- 结束日期不能早于起始日期
- 预设选项应即时更新日期范围
- 显示已选天数 (如 "已选择 14 天")

### References

- [Source: architecture.md#2.2] - 后端模块架构
- [Source: architecture.md#3.2] - 日报生成流程（报告复用此模式）
- [Source: architecture.md#4.3] - 时区处理正确方式
- [Source: PRD.md#11] - 周报月报功能规划
- [Source: epics.md#Epic 5] - 周报月报功能 Epic
- [Source: REPORT-001 story] - 周报实现参考
- [Source: REPORT-002 story] - 月报实现参考

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

## Dev Agent Record

### Agent Model Used

- Claude Sonnet 4.6

### Implementation Notes

1. Database: Added `custom_report_templates` field to Settings table with migration
2. Backend: Implemented `generate_custom_report` Tauri command in synthesis/mod.rs
3. Helper functions: Added `get_biweekly_range()`, `get_quarter_range()`, `generate_custom_report_filename()`
4. Frontend: Created CustomReportModal.vue with date picker and preset selection

### Files Modified

- `src-tauri/src/memory_storage/mod.rs` - Database schema and Settings struct
- `src-tauri/src/synthesis/mod.rs` - Custom report generation logic
- `src-tauri/src/main.rs` - Registered Tauri command
- `src/App.vue` - Added custom report button and state
- `src/components/CustomReportModal.vue` - New component for custom report UI

## Change Log

- 2026-03-15: Implemented Tasks 1-3 (database, backend, frontend)

### File List