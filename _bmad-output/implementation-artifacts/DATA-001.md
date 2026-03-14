# Story 4.1: 历史记录浏览

Status: ready-for-dev

## Story

As a DailyLogger 用户,
I want 浏览和搜索历史记录,
so that 我可以回顾过去的工作内容并管理存储的数据.

## Acceptance Criteria

1. **AC1: 日期范围浏览**
   - Given 用户打开历史记录界面, When 用户选择日期范围, Then 显示该范围内的所有记录
   - 默认显示最近 7 天的记录
   - 支持自定义开始和结束日期

2. **AC2: 来源类型筛选**
   - Given 历史记录已加载, When 用户选择筛选条件, Then 仅显示符合条件的记录
   - 支持筛选：全部 / 自动捕获 / 手动记录
   - 筛选切换后立即更新列表

3. **AC3: 删除单条记录**
   - Given 记录列表显示, When 用户删除某条记录, Then 该记录从数据库和列表中移除
   - 删除前需确认
   - 删除成功后显示 Toast 提示

4. **AC4: 分页加载**
   - Given 记录数量超过 50 条, When 用户滚动到底部, Then 加载下一页记录
   - 每页 50 条记录
   - 显示加载状态指示器

## Tasks / Subtasks

- [ ] Task 1: 后端 API 扩展 (AC: 1, 2, 3)
  - [ ] 1.1 在 `memory_storage/mod.rs` 添加 `get_records_by_date_range` 函数
  - [ ] 1.2 添加 `delete_record` 函数
  - [ ] 1.3 在 `main.rs` 注册新 Tauri 命令
  - [ ] 1.4 添加单元测试覆盖边界条件

- [ ] Task 2: 前端历史记录组件 (AC: 1, 2, 4)
  - [ ] 2.1 创建 `HistoryViewer.vue` 组件
  - [ ] 2.2 实现日期范围选择器
  - [ ] 2.3 实现来源类型筛选器
  - [ ] 2.4 实现分页加载逻辑

- [ ] Task 3: 删除功能实现 (AC: 3)
  - [ ] 3.1 添加删除按钮和确认对话框
  - [ ] 3.2 实现前端删除调用
  - [ ] 3.3 添加删除成功的 Toast 提示

- [ ] Task 4: 集成与入口 (AC: 全部)
  - [ ] 4.1 在 App.vue 添加历史记录入口按钮
  - [ ] 4.2 集成 HistoryViewer 到主界面
  - [ ] 4.3 添加组件测试

## Dev Notes

### 架构约束

1. **数据库操作**: 使用现有的 `DB_CONNECTION` 全局 Mutex，不创建新的数据库连接
2. **Tauri 命令**: 所有新命令必须在 `main.rs` 的 `generate_handler![]` 中注册
3. **前端风格**: 使用 TailwindCSS，遵循 `bg-dark`、`bg-darker`、`text-primary` 主题色
4. **时区处理**: 查询时使用与 `get_today_records_sync` 相同的本地时区转换逻辑

### 关键代码参考

**现有记录查询** (`src-tauri/src/memory_storage/mod.rs:149-186`):
```rust
pub fn get_today_records_sync() -> Result<Vec<Record>, String> {
    let today_start = chrono::Local::now()
        .date_naive()
        .and_hms_opt(0, 0, 0).unwrap()
        .and_local_timezone(chrono::Local)
        .unwrap()
        .with_timezone(&chrono::Utc)
        .to_rfc3339();
    // ... query with timestamp >= today_start
}
```

**Record 结构体** (`src-tauri/src/memory_storage/mod.rs:99-106`):
```rust
pub struct Record {
    pub id: i64,
    pub timestamp: String,
    pub source_type: String,
    pub content: String,
    pub screenshot_path: Option<String>,
}
```

### 新增 API 设计

**Rust 端**:
```rust
// 获取指定日期范围的记录
pub fn get_records_by_date_range(
    start_date: &str,  // RFC3339 UTC
    end_date: &str,    // RFC3339 UTC
    source_type: Option<&str>,  // None = 全部, Some("auto"/"manual")
    offset: i64,
    limit: i64,
) -> Result<Vec<Record>, String>

// 删除单条记录
pub fn delete_record(id: i64) -> Result<(), String>
```

**Tauri 命令**:
```rust
#[command]
pub async fn get_history_records(
    start_date: String,
    end_date: String,
    source_type: Option<String>,
    page: i64,
) -> Result<Vec<Record>, String>

#[command]
pub async fn delete_record(id: i64) -> Result<(), String>
```

### 前端组件结构

```
HistoryViewer.vue
├─ Header: 标题 + 关闭按钮
├─ Filters:
│  ├─ DateRangePicker (开始/结束日期)
│  └─ SourceTypeSelect (全部/自动/手动)
├─ RecordList:
│  ├─ RecordItem (v-for)
│  │  ├─ 时间戳
│  │  ├─ 来源标签
│  │  ├─ 内容摘要
│  │  └─ 删除按钮
│  └─ LoadingIndicator
└─ EmptyState (无记录时显示)
```

### 项目结构 Notes

- 新组件位置: `src/components/HistoryViewer.vue`
- 后端修改: `src-tauri/src/memory_storage/mod.rs`, `src-tauri/src/main.rs`
- 遵循现有命名规范: snake_case (Rust), PascalCase (Vue)

### 测试要求

**Rust 测试** (`memory_storage/mod.rs`):
- 测试日期范围边界 (跨越本地午夜)
- 测试 source_type 筛选
- 测试分页 offset/limit
- 测试删除不存在的记录返回错误

**前端测试** (Vitest):
- 组件挂载时加载默认数据
- 日期选择触发重新加载
- 筛选条件变化更新列表
- 删除确认流程

### 性能考虑

- 数据库已存在 `idx_timestamp` 索引，日期范围查询高效
- 分页避免一次性加载大量数据
- 前端使用虚拟滚动 (可选优化)

### References

- [Source: architecture.md#5.1] 数据库 schema 和索引
- [Source: architecture.md#6] API 端点设计模式
- [Source: PRD.md#6.4] 截图回顾功能 (参考现有 ScreenshotGallery 实现)
- [Source: epics.md#Epic 4] 数据管理 Epic 上下文

## Dev Agent Record

### Agent Model Used

(待实现时填写)

### Debug Log References

(待实现时填写)

### Completion Notes List

(待实现时填写)

### File List

(待实现时填写)