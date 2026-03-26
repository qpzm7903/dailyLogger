# Story 11.2: DATA-008 - 数据统计面板

Status: in-progress

## Story

As a DailyLogger 用户，
I want 查看使用数据统计面板，
so that 我可以直观了解每日/每周/每月的记录情况，包括截图数量、工作时段数量、AI 分析成功率等数据，帮助我评估工作模式和 productivity。

## Acceptance Criteria

1. **AC1: 统计面板入口**
   - Given 用户打开应用，When 用户点击统计入口，Then 显示统计面板模态框或侧边栏
   - 入口位置：在 Dashboard 或侧边栏提供"数据统计"按钮

2. **AC2: 今日统计概览**
   - Given 用户打开统计面板，When 显示今日数据，Then 展示：截图数量、工作时段数量、AI 分析成功率、生成日报次数
   - 统计数据来源：records 表和 sessions 表

3. **AC3: 本周/本月统计**
   - Given 用户选择时间范围，When 切换到本周/本月，Then 显示对应时间范围内的统计数据
   - 支持：今日、本周、本月、自定义日期范围

4. **AC4: 统计图表可视化**
   - Given 有历史数据，When 用户查看统计数据，Then 显示可视化图表（如柱状图或折线图）
   - 图表类型：每日截图数量趋势、工作时段分布、AI 分析成功率变化

5. **AC5: 数据导出**
   - Given 用户查看统计数据，When 点击导出，Then 生成统计报告（CSV 或 Markdown）
   - 导出内容包括：日期范围、各项统计数据明细

## Tasks / Subtasks

- [x] Task 1: 数据库统计查询函数 (AC: #2, #3)
  - [x] 1.1 在 `memory_storage/mod.rs` 添加 `get_statistics()` 函数
  - [x] 1.2 实现按日期范围查询统计数据逻辑
  - [x] 1.3 添加 Rust 单元测试

- [x] Task 2: Tauri 命令注册 (AC: #1)
  - [x] 2.1 在 `main.rs` 注册 `get_statistics` 命令
  - [x] 2.2 定义统计结果的数据结构 `Statistics`

- [x] Task 3: 前端统计面板 UI (AC: #1, #2, #3, #4)
  - [x] 3.1 创建 `StatisticsPanel.vue` 组件
  - [x] 3.2 实现时间范围选择器（今日/本周/本月/自定义）
  - [x] 3.3 实现统计卡片展示（截图数、时段数、分析成功率）
  - [x] 3.4 实现简单柱状图展示每日趋势（可复用 ScreenshotGallery 的展示模式）

- [x] Task 4: 数据导出功能 (AC: #5)
  - [x] 4.1 在前端添加导出按钮
  - [x] 4.2 实现 CSV/Markdown 格式导出逻辑 (仅实现 CSV，Markdown 待完成)

- [ ] Task 5: 端到端测试 (AC: All)
  - [ ] 5.1 前端组件测试
  - [ ] 5.2 Rust 集成测试

## Dev Notes

### Architecture Context

**关键架构决策**:
- 复用现有 `memory_storage/mod.rs` 的数据库查询模式
- 统计面板作为独立模态框组件实现
- 复用 `session_manager/mod.rs` 的时段统计逻辑
- 可复用 `synthesis/mod.rs` 的日报文件名格式用于导出

**必须遵循的代码模式** [Source: architecture.md]:
- Tauri Command: `#[command]` + async
- 错误处理: `Result<T, String>` + `.map_err(|e| e.to_string())`
- 数据库访问: 使用全局 `DB_CONNECTION` Mutex
- 时区处理: 使用 `and_local_timezone(chrono::Local)` 避免 UTC 偏移问题

### Key Existing Code to Reuse

**memory_storage/mod.rs** - 复用以下函数:
- `get_today_records()` - 今日记录查询模式
- `get_settings_sync()` - 设置查询
- `init_database()` - 数据库初始化模式

**session_manager/mod.rs** - 复用以下函数:
- `get_today_sessions()` - 时段查询模式
- `detect_or_create_session()` - 时段检测逻辑

**synthesis/mod.rs** - 参考:
- 文件导出模式（日报生成的文件路径处理）

### Database Queries

```rust
// 统计数据查询示例

/// 获取日期范围内的截图数量
fn count_screenshots_in_range(conn: &Connection, start: &str, end: &str) -> Result<i64, String> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM records WHERE timestamp >= ? AND timestamp < ? AND screenshot_path IS NOT NULL",
            [start, end],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    Ok(count)
}

/// 获取日期范围内的工作时段数量
fn count_sessions_in_range(conn: &Connection, start: &str, end: &str) -> Result<i64, String> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sessions WHERE date >= ? AND date <= ?",
            [start, end],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    Ok(count)
}

/// 获取 AI 分析成功率
fn get_analysis_success_rate(conn: &Connection, start: &str, end: &str) -> Result<f64, String> {
    let total: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM records WHERE timestamp >= ? AND timestamp < ?",
            [start, end],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let analyzed: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM records WHERE timestamp >= ? AND timestamp < ? AND analysis_status = 'analyzed'",
            [start, end],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    if total == 0 {
        return Ok(0.0);
    }
    Ok((analyzed as f64) / (total as f64) * 100.0)
}
```

### Data Structure

```rust
// src-tauri/src/memory_storage/mod.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statistics {
    pub date_range: DateRange,
    pub screenshot_count: i64,
    pub session_count: i64,
    pub record_count: i64,
    pub analysis_success_rate: f64,  // 百分比 0-100
    pub daily_breakdown: Vec<DailyStatistic>,  // 每日明细
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: String,  // RFC3339
    pub end: String,    // RFC3339
    pub label: String,  // "今日" / "本周" / "本月" / "自定义"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyStatistic {
    pub date: String,  // YYYY-MM-DD
    pub screenshot_count: i64,
    pub session_count: i64,
    pub record_count: i64,
}
```

### Frontend Component Structure

```
StatisticsPanel.vue (Modal)
├── TimeRangeSelector
│   ├── Today button
│   ├── This Week button
│   ├── This Month button
│   └── Custom Range picker
├── StatisticsCards
│   ├── ScreenshotCount card
│   ├── SessionCount card
│   ├── RecordCount card
│   └── AnalysisSuccessRate card
├── DailyTrendChart (simple bar chart)
│   └── Bar items for each day
└── ExportButton
```

### Project Structure Notes

**需要创建/修改的文件**:
- `src-tauri/src/memory_storage/mod.rs` - 添加 Statistics 结构体和统计查询函数
- `src-tauri/src/main.rs` - 注册 `get_statistics` 命令
- `src/types/tauri.ts` - 添加 Statistics 相关类型
- `src/components/StatisticsPanel.vue` - 新建统计面板组件

**前端组件参考**:
- SettingsModal.vue - 模态框实现模式
- ScreenshotGallery.vue - 网格展示模式
- ReportDropdown.vue - 下拉菜单选择模式

### Testing Requirements

**必须测试的场景**:
1. 今日统计：验证日期范围正确、数据准确
2. 本周统计：验证包含完整的周数据
3. 本月统计：验证包含完整的月数据
4. 分析成功率：验证计算公式正确
5. 空数据处理：无记录时正确显示 0 或空状态
6. 导出功能：验证 CSV/Markdown 格式正确

**测试模式** (参考现有测试):
```rust
#[test]
fn get_statistics_for_today_returns_correct_counts() {
    setup_test_db_with_records();
    let stats = get_statistics("today").unwrap();
    assert_eq!(stats.screenshot_count, 10);
    assert_eq!(stats.session_count, 3);
}

#[test]
fn empty_database_returns_zero_counts() {
    setup_empty_test_db();
    let stats = get_statistics("today").unwrap();
    assert_eq!(stats.screenshot_count, 0);
    assert_eq!(stats.session_count, 0);
}
```

### Previous Story Intelligence

从 DATA-007 学到的经验:
- 数据库 ALTER TABLE 添加字段时使用 `#[derive(Default)]` 让新字段有默认值
- 使用 `chrono::Local` 和 `and_local_timezone()` 处理时区
- 前端组件使用 `ref()` 响应式状态管理
- 复用现有组件的模式（如 ReportDropdown 的下拉菜单）加速开发

从 DATA-006 学到的经验:
- 组件测试使用 `mount()` 和 `wrapper.findAll()` 选择元素
- 事件测试使用 `await wrapper.find().trigger('click')`
- 模拟异步操作使用 `vi.useFakeTimers()` 处理定时器

### References

- [Source: architecture.md#2.2] - 后端模块架构
- [Source: architecture.md#5.1] - records 表结构
- [Source: architecture.md#5.1.1] - sessions 表结构
- [Source: PRD.md#6] - 功能需求概览
- [Source: epics.md#Epic 11] - 数据增强与稳定性 Epic
- [Source: DATA-007 story] - 前端组件开发模式参考
- [Source: DATA-006 story] - 数据库查询模式参考

## Dev Agent Record

### Agent Model Used

claude-opus-4-6

### Debug Log References

### Completion Notes List

### File List

- `src-tauri/src/memory_storage/mod.rs` - Statistics struct, DateRange, DailyStatistic, get_statistics()
- `src-tauri/src/main.rs` - Registered get_statistics command
- `src/types/tauri.ts` - Statistics, DateRange, DailyStatistic, GetStatisticsArgs types
- `src/components/StatisticsPanel.vue` - Statistics panel modal component
- `src/App.vue` - Added StatisticsPanel modal registration
- `src/components/layout/Dashboard.vue` - Added statistics button
- `src/locales/zh-CN.json` - Added statistics translations
- `src/locales/en.json` - Added statistics translations

### Completion Notes List

- [2026-03-26] Implemented backend Statistics struct and get_statistics() Tauri command
- [2026-03-26] Added frontend StatisticsPanel.vue component with time range selector, statistics cards, daily breakdown chart
- [2026-03-26] Added data export (CSV) functionality
- [2026-03-26] Added i18n translations for statistics panel

## Review Findings

### 🔴 CRITICAL ISSUES

1. **Tasks not checked off but story claims completion**
   - All Tasks/Subtasks in the story are marked `[ ]` (unchecked), but story status is "review" with completion notes claiming work is done
   - This creates a misleading state - the story appears incomplete when it may actually be complete
   - **Fix**: Update Tasks/Subtasks to reflect actual completion status

### 🟡 MEDIUM ISSUES

2. **Markdown export not implemented (AC5 partially incomplete)**
   - AC5 says "CSV 或 Markdown" (CSV or Markdown) export
   - Only CSV export is implemented in `generateCsv()` function
   - No `generateMarkdown()` function exists
   - **Fix**: Either add Markdown export functionality or update AC5 to say "CSV" only

3. **Missing frontend component test (Task 5.1 incomplete)**
   - Task 5.1 mentions "前端组件测试" (frontend component tests)
   - No `StatisticsPanel.test.ts` file exists in `src/components/__tests__/`
   - **Fix**: Create `StatisticsPanel.test.ts` or update task to reflect actual test coverage

4. **Missing Rust integration tests for get_statistics (Task 5.2 incomplete)**
   - Story describes tests like `get_statistics_for_today_returns_correct_counts` and `empty_database_returns_zero_counts`
   - Only unit tests for helper functions exist (get_last_day_of_month, parse_date, etc.)
   - No actual integration test that calls `get_statistics()` with a test database
   - **Fix**: Add integration tests for `get_statistics()` or update task to reflect actual test coverage

### 🟢 LOW ISSUES

5. **Git vs Story Discrepancies**
   - File List in story matches actual git changes - no discrepancies found
   - Story documentation is accurate in this regard

### Summary

| Category | Count |
|----------|-------|
| CRITICAL | 1 |
| MEDIUM | 3 |
| LOW | 1 |

**Overall Assessment**: Implementation covers all major acceptance criteria (AC1-AC4 fully, AC5 partially). The critical issue is the Tasks/Subtasks section not being updated to reflect completed work, which creates confusion about actual story state.

