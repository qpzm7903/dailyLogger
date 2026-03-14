# Story 4.4: 数据导出 (JSON/MD)

Status: ready-for-dev

## Story

As a DailyLogger 用户,
I want 导出数据为 JSON 或 Markdown 格式,
so that 我可以备份工作记录或进行离线分析.

## Acceptance Criteria

1. **AC1: 导出为 JSON 格式**
   - Given 用户选择 JSON 格式导出, When 执行导出, Then 生成有效的 JSON 文件包含所有选定记录
   - JSON 结构包含: `records` 数组、`export_time`、`record_count`
   - 每条记录包含: `id`, `timestamp`, `source_type`, `content`, `screenshot_path`

2. **AC2: 导出为 Markdown 格式**
   - Given 用户选择 Markdown 格式导出, When 执行导出, Then 生成格式化的 .md 文件
   - Markdown 结构: 标题、导出时间、记录列表 (时间戳 + 来源 + 内容)
   - 自动记录和手动记录使用不同图标/标记区分

3. **AC3: 日期范围筛选**
   - Given 用户指定日期范围, When 导出, Then 仅导出该范围内的记录
   - 支持单日导出 (start_date == end_date)
   - 支持不指定日期 (导出全部)

4. **AC4: 导出结果反馈**
   - Given 导出完成, When 用户查看结果, Then 显示文件路径和记录数量
   - 提供打开文件和打开所在目录选项

## Tasks / Subtasks

- [ ] Task 1: 后端导出 API (AC: 1, 2, 3)
  - [ ] 1.1 定义 `ExportOptions` 和 `ExportResult` 结构体
  - [ ] 1.2 实现 `export_data` 函数 (JSON 格式)
  - [ ] 1.3 实现 Markdown 格式导出逻辑
  - [ ] 1.4 实现日期范围筛选查询
  - [ ] 1.5 在 `main.rs` 注册 `export_data` Tauri 命令
  - [ ] 1.6 添加单元测试覆盖各格式和边界条件

- [ ] Task 2: 前端导出组件 (AC: 1, 2, 3, 4)
  - [ ] 2.1 创建 `ExportModal.vue` 组件
  - [ ] 2.2 实现格式选择 (JSON/Markdown)
  - [ ] 2.3 实现日期范围选择器
  - [ ] 2.4 实现导出按钮和进度显示
  - [ ] 2.5 实现导出结果展示和操作按钮

- [ ] Task 3: 集成与测试 (AC: 全部)
  - [ ] 3.1 在 `App.vue` 添加导出入口按钮
  - [ ] 3.2 集成 ExportModal 到主界面
  - [ ] 3.3 添加组件测试
  - [ ] 3.4 手动测试大数据量导出性能

## Dev Notes

### 架构约束

1. **数据库操作**: 使用现有的 `DB_CONNECTION` 全局 Mutex，不创建新的数据库连接
2. **Tauri 命令**: 所有新命令必须在 `main.rs` 的 `generate_handler![]` 中注册
3. **前端风格**: 使用 TailwindCSS，遵循 `bg-dark`、`bg-darker`、`text-primary` 主题色
4. **文件系统**: 导出文件保存到应用数据目录的 `exports/` 子目录

### 关键代码参考

**现有 Record 结构体** (`src-tauri/src/memory_storage/mod.rs`):
```rust
pub struct Record {
    pub id: i64,
    pub timestamp: String,
    pub source_type: String,
    pub content: String,
    pub screenshot_path: Option<String>,
}
```

**日期范围查询参考** (`get_today_records` 逻辑):
```rust
// 扩展为支持自定义日期范围
let query = if start_date.is_some() || end_date.is_some() {
    "SELECT id, timestamp, source_type, content, screenshot_path
     FROM records
     WHERE timestamp >= ?1 AND timestamp <= ?2
     ORDER BY timestamp DESC"
} else {
    "SELECT id, timestamp, source_type, content, screenshot_path
     FROM records
     ORDER BY timestamp DESC"
};
```

### 新增 API 设计

**Rust 端** (`memory_storage/mod.rs` 或新建 `export/mod.rs`):
```rust
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    pub format: String,           // "json" | "markdown"
    pub start_date: Option<String>, // YYYY-MM-DD 格式
    pub end_date: Option<String>,   // YYYY-MM-DD 格式
    pub include_screenshots: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub file_path: String,
    pub record_count: i64,
    pub file_size: i64,
}

/// 导出数据
pub async fn export_data(
    app: tauri::AppHandle,
    options: ExportOptions,
) -> Result<ExportResult, String>
```

**Tauri 命令**:
```rust
#[command]
pub async fn export_data(
    app: tauri::AppHandle,
    format: String,
    start_date: Option<String>,
    end_date: Option<String>,
    include_screenshots: Option<bool>,
) -> Result<ExportResult, String>
```

### JSON 导出格式

```json
{
  "export_time": "2026-03-14T10:30:00Z",
  "record_count": 42,
  "records": [
    {
      "id": 1,
      "timestamp": "2026-03-14T09:00:00Z",
      "source_type": "auto",
      "content": "正在编写 Rust 代码...",
      "screenshot_path": "screenshots/screenshot_20260314_090000.png"
    },
    {
      "id": 2,
      "timestamp": "2026-03-14T09:15:00Z",
      "source_type": "manual",
      "content": "需要实现数据库连接",
      "screenshot_path": null
    }
  ]
}
```

### Markdown 导出格式

```markdown
# DailyLogger 数据导出

**导出时间**: 2026-03-14 10:30:00
**记录数量**: 42 条

---

## 记录列表

### 🤖 2026-03-14 09:00:00 (自动)

正在编写 Rust 代码...

### 📝 2026-03-14 09:15:00 (手动)

需要实现数据库连接

---
*由 DailyLogger 自动生成*
```

### 前端组件结构

```
ExportModal.vue
├─ Header: 导出标题 + 关闭按钮
├─ FormatSelect:
│  ├─ JsonRadio (JSON 格式)
│  └─ MarkdownRadio (Markdown 格式)
├─ DateRangePicker:
│  ├─ StartDateInput
│  └─ EndDateInput
├─ OptionsToggle:
│  └─ IncludeScreenshotsCheckbox
├─ ExportButton:
│  └─ 导出按钮 (显示记录数)
└─ ResultPanel (导出完成后显示):
   ├─ 文件路径显示
   ├─ 打开文件按钮
   └─ 打开目录按钮
```

### 项目结构 Notes

- 新组件位置: `src/components/ExportModal.vue`
- 后端新增文件: `src-tauri/src/export/mod.rs` (推荐) 或扩展现有 `memory_storage/mod.rs`
- 后端修改: `src-tauri/src/lib.rs` (导出模块), `src-tauri/src/main.rs` (注册命令)
- 导出目录: `~/.local/share/DailyLogger/exports/`
- 遵循现有命名规范: snake_case (Rust), PascalCase (Vue)

### 文件命名规则

导出文件自动命名：
- JSON: `export_YYYYMMDD_HHMMSS.json`
- Markdown: `export_YYYYMMDD_HHMMSS.md`

示例：
- `export_20260314_103000.json`
- `export_20260314_103000.md`

### 测试要求

**Rust 测试**:
- 测试 JSON 格式输出正确性
- 测试 Markdown 格式输出正确性
- 测试日期范围筛选边界
- 测试空数据导出
- 测试特殊字符转义 (JSON/Markdown)

**前端测试** (Vitest):
- 组件挂载时显示格式选择
- 日期选择器正常工作
- 导出按钮触发 API 调用
- 结果面板正确显示文件路径

### 与其他 Story 的关系

- **DATA-001** (历史记录浏览): 可共享日期选择器组件逻辑
- **DATA-002** (全文搜索): 共享 Record 类型定义
- **DATA-005** (数据备份与恢复): 导出功能是备份的基础

### 性能考虑

- 大数据量 (1000+ 条) 时考虑流式写入而非内存拼接
- 前端显示导出进度 (可选优化)
- 导出操作为异步，不阻塞 UI

### References

- [Source: architecture.md#5.1] 数据库 schema 和索引
- [Source: architecture.md#6] API 端点设计模式
- [Source: architecture.md#7] 文件系统路径约定
- [Source: PRD.md#11] 未来规划 - 数据导出 P3 优先级
- [Source: epics.md#Epic 4] 数据管理 Epic 上下文
- [Source: specs/DATA-004.md] 功能规格定义

## Dev Agent Record

### Agent Model Used

(待实现时填写)

### Debug Log References

(待实现时填写)

### Completion Notes List

(待实现时填写)

### File List

(待实现时填写)