# Story 5.2: REPORT-002 - 月报生成

Status: ready-for-dev

## Story

As a DailyLogger user,
I want to generate a monthly work summary report,
so that I can have a comprehensive overview of my work across the entire month for performance reviews and long-term planning.

## Acceptance Criteria

1. **Given** 用户有本月的记录，**When** 用户点击"生成月报"，**Then** 系统汇总本月记录并调用 AI 生成结构化 Markdown 月报
2. **Given** 月报生成成功，**When** 用户查看，**Then** 显示文件路径并提供打开选项
3. **Given** 月报生成失败，**When** 错误发生，**Then** 显示具体错误信息
4. **Given** 本月无记录，**When** 用户点击生成月报，**Then** 提示"本月无记录"
5. **Given** 用户有自定义月报模板，**When** 生成月报，**Then** 使用自定义模板
6. **Given** 月报生成完成，**When** 用户选择打开，**Then** 在默认应用中打开月报文件
7. **Given** 月报内容，**When** 查看月报，**Then** 包含月度趋势分析（按周统计工作量）

## Tasks / Subtasks

- [ ] Task 1: 数据库扩展 - 月报配置字段 (AC: #5)
  - [ ] 1.1 在 Settings 表添加 `monthly_report_prompt` 字段 (TEXT, 可为空)
  - [ ] 1.2 更新 Settings struct 和相关 CRUD 函数
  - [ ] 1.3 添加数据库迁移逻辑 (ALTER TABLE)

- [ ] Task 2: Rust 后端 - 月报生成核心逻辑 (AC: #1, #3, #4, #7)
  - [ ] 2.1 在 memory_storage/mod.rs 添加 `get_month_records_sync()` 函数
    - 获取本月 1 日 00:00:00 到本月最后一天 23:59:59 的记录
    - 正确处理跨时区问题 (参考 `get_today_records_sync` 模式)
  - [ ] 2.2 在 synthesis/mod.rs 添加 `generate_monthly_report()` Tauri command
    - 复用 `format_records_for_summary()` 格式化记录
    - 复用 LLM 调用模式 (reqwest + OpenAI API)
    - 生成文件名: `月报-{YYYY-MM}.md`
  - [ ] 2.3 添加 `get_default_monthly_report_prompt()` 函数
    - 包含月度趋势分析指令
  - [ ] 2.4 添加 `format_records_by_week()` 辅助函数
    - 按周分组记录，用于趋势分析
  - [ ] 2.5 在 main.rs 的 `generate_handler![]` 中注册新命令
  - [ ] 2.6 编写单元测试
    - 测试时间边界（1 日 00:00, 月末 23:59）
    - 测试空记录处理
    - 测试跨月边界

- [ ] Task 3: 前端 - 月报生成 UI (AC: #1, #2, #5, #6)
  - [ ] 3.1 在 App.vue 添加"生成月报"按钮
  - [ ] 3.2 创建 `MonthlyReportViewer.vue` 组件或复用 `DailySummaryViewer.vue` 模式
  - [ ] 3.3 添加月报生成 loading 状态和成功/错误提示
  - [ ] 3.4 显示最近月报路径和打开按钮
  - [ ] 3.5 在 SettingsModal.vue 添加月报模板配置入口（可选）

- [ ] Task 4: 端到端测试 (AC: All)
  - [ ] 4.1 前端 Vitest 测试: 月报生成按钮交互
  - [ ] 4.2 Rust 集成测试: 完整月报生成流程

## Dev Notes

### Architecture Context

**关键架构决策**:
- 复用现有 `synthesis/mod.rs` 的日报生成模式
- 月报使用独立的 prompt 模板，包含趋势分析
- 文件输出到同一 Obsidian 路径
- **依赖 REPORT-001**: 如果周报功能已实现，可复用周记录查询逻辑

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
/// 计算本月起始和结束时间 (UTC RFC3339)
fn get_month_bounds() -> (DateTime<Utc>, DateTime<Utc>) {
    let now = chrono::Local::now();
    let first_day = now.date_naive().with_day(1).unwrap();

    // 本月开始: 本月 1 日 00:00:00 本地时间
    let month_start = first_day
        .and_hms_opt(0, 0, 0).unwrap()
        .and_local_timezone(chrono::Local).unwrap()
        .with_timezone(&chrono::Utc);

    // 本月结束: 下月 1 日 00:00:00 - 1 秒
    let next_month = if now.month() == 12 {
        chrono::NaiveDate::from_ymd_opt(now.year() + 1, 1, 1).unwrap()
    } else {
        chrono::NaiveDate::from_ymd_opt(now.year(), now.month() + 1, 1).unwrap()
    };

    let month_end = next_month
        .and_hms_opt(0, 0, 0).unwrap()
        .and_local_timezone(chrono::Local).unwrap()
        .with_timezone(&chrono::Utc)
        - chrono::Duration::seconds(1);

    (month_start, month_end)
}
```

### Database Migration

在 `init_database()` 中添加:
```rust
let _ = conn.execute(
    "ALTER TABLE settings ADD COLUMN monthly_report_prompt TEXT",
    [],
);
```

### Default Monthly Report Prompt

```rust
const DEFAULT_MONTHLY_REPORT_PROMPT: &str = r#"你是一个工作日志助手。请根据以下本月工作记录，生成一份结构化的 Markdown 格式月报。

要求：
1. 按周分组展示工作内容
2. 提取本月关键成果和技术亮点
3. 总结遇到的问题和解决方案
4. 分析月度工作趋势（哪些方面投入更多时间）
5. 列出下月待跟进事项
6. 输出纯 Markdown 格式，不要有其他说明文字

本月记录：
{records}

请生成月报："#;
```

### File Naming Convention

月报文件名格式: `月报-{YYYY-MM}.md`
- 例如: `月报-2026-03.md`

### Week Grouping Helper

```rust
/// 将记录按周分组，用于趋势分析
fn format_records_by_week(records: &[Record]) -> String {
    // 按周分组 (第1周、第2周...)
    // 每周统计记录数量
    // 用于 AI 进行趋势分析
}
```

### Project Structure Notes

**需要修改的文件**:
- `src-tauri/src/memory_storage/mod.rs` - 添加月记录查询
- `src-tauri/src/synthesis/mod.rs` - 添加月报生成命令
- `src-tauri/src/main.rs` - 注册新命令
- `src/App.vue` - 添加月报按钮
- `src/components/` - 可选：新建月报查看组件

**前端组件参考**: 复用 `DailySummaryViewer.vue` 模式处理月报显示

### Testing Requirements

**必须测试的场景**:
1. 时间边界: 1 日 00:00:00 和月末 23:59:59 的记录是否正确包含
2. 空记录: 本月无记录时返回空列表
3. 跨月边界: 上月末和下月初的记录不应包含
4. 跨时区: 本地时间正确转换为 UTC 查询

**测试模式** (参考现有测试):
```rust
#[test]
fn finds_records_at_month_boundaries() {
    setup_test_db();
    // 测试 1 日 00:00 的记录
    // 测试月末 23:59 的记录
}

#[test]
fn excludes_records_from_previous_month() {
    setup_test_db();
    // 确保上月末记录不包含在本月中
}
```

### Previous Story Intelligence (REPORT-001)

从 REPORT-001 (周报生成) 学到的经验:
- 时间边界计算使用 `and_local_timezone()` 而非 `.and_utc()`
- 文件命名使用中文前缀（周报-、月报-）保持一致性
- 复用 `format_records_for_summary()` 减少代码重复
- Settings 表使用 ALTER TABLE 添加新字段时忽略错误（兼容已有列）

### References

- [Source: architecture.md#2.2] - 后端模块架构
- [Source: architecture.md#3.2] - 日报生成流程（月报复用此模式）
- [Source: architecture.md#4.3] - 时区处理正确方式
- [Source: PRD.md#11] - 周报月报功能规划
- [Source: epics.md#Epic 5] - 周报月报功能 Epic
- [Source: synthesis/mod.rs] - 现有日报生成代码
- [Source: REPORT-001 story] - 周报生成实现模式

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List