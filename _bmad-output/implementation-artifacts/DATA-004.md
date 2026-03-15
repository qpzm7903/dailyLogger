# Story 4.4: 数据导出 (JSON/MD)

Status: review

## Story

As a DailyLogger 用户,
I want 导出历史记录为 JSON 或 Markdown 格式,
so that 我可以备份数据或与其他工具进行数据分析.

## Acceptance Criteria

1. **AC1: JSON 格式导出**
   - Given 用户选择导出功能, When 用户选择 JSON 格式, Then 生成包含所有记录的 JSON 文件
   - JSON 结构包含完整的记录信息 (id, timestamp, source_type, content, screenshot_path)
   - 文件名格式: `dailylogger-export-YYYY-MM-DD.json`

2. **AC2: Markdown 格式导出**
   - Given 用户选择导出功能, When 用户选择 Markdown 格式, Then 生成结构化的 Markdown 文件
   - 按日期分组记录，每日包含时间线和内容摘要
   - 文件名格式: `dailylogger-export-YYYY-MM-DD.md`

3. **AC3: 日期范围选择**
   - Given 用户打开导出界面, When 用户选择日期范围, Then 仅导出该范围内的记录
   - 默认导出最近 7 天的记录
   - 支持自定义开始和结束日期

4. **AC4: 导出进度反馈**
   - Given 导出数据量较大, When 导出进行中, Then 显示进度指示器
   - 导出成功后显示文件路径
   - 支持打开导出文件所在目录

## Tasks / Subtasks

- [x] Task 1: 后端导出 API (AC: 1, 2, 3)
  - [x] 1.1 在 `memory_storage/mod.rs` 添加 `get_records_for_export` 函数
  - [x] 1.2 创建 `src-tauri/src/export/mod.rs` 模块
  - [x] 1.3 实现 `export_to_json` Tauri 命令
  - [x] 1.4 实现 `export_to_markdown` Tauri 命令
  - [x] 1.5 在 `main.rs` 注册新命令
  - [x] 1.6 添加单元测试

- [x] Task 2: 前端导出组件 (AC: 3, 4)
  - [x] 2.1 创建 `ExportModal.vue` 组件
  - [x] 2.2 实现日期范围选择器
  - [x] 2.3 实现格式选择 (JSON/Markdown)
  - [x] 2.4 实现导出进度显示
  - [x] 2.5 实现导出成功后的路径显示和目录打开

- [x] Task 3: 集成入口 (AC: 全部)
  - [x] 3.1 在 App.vue 添加导出按钮
  - [x] 3.2 集成 ExportModal 到主界面
  - [x] 3.3 添加组件测试

## Dev Notes

### 架构约束

1. **数据库操作**: 使用现有的 `DB_CONNECTION` 全局 Mutex，不创建新的数据库连接
2. **Tauri 命令**: 所有新命令必须在 `main.rs` 的 `generate_handler![]` 中注册
3. **前端风格**: 使用 TailwindCSS，遵循 `bg-dark`、`bg-darker`、`text-primary` 主题色
4. **时区处理**: 导出时间使用本地时区显示，数据库存储保持 UTC RFC3339
5. **文件位置**: 导出文件默认保存到应用数据目录的 `exports/` 子目录

### 关键代码参考

**现有记录查询** (`src-tauri/src/memory_storage/mod.rs:149-186`):
```rust
pub fn get_today_records_sync() -> Result<Vec<Record>, String> {
    // 复用此模式实现日期范围查询
    let today_start = chrono::Local::now()
        .date_naive()
        .and_hms_opt(0, 0, 0).unwrap()
        .and_local_timezone(chrono::Local)
        .unwrap()
        .with_timezone(&chrono::Utc)
        .to_rfc3339();
}
```

**记录格式化参考** (`src-tauri/src/synthesis/mod.rs:44-63`):
```rust
pub fn format_records_for_summary(records: &[Record]) -> String {
    records.iter().map(|r| {
        let time = chrono::DateTime::parse_from_rfc3339(&r.timestamp)
            .map(|dt| dt.with_timezone(&chrono::Local).format("%H:%M").to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        // ...
    }).collect::<Vec<_>>().join("\n")
}
```

**文件写入模式** (`src-tauri/src/synthesis/mod.rs:226-231`):
```rust
let output_dir = PathBuf::from(&obsidian_path);
std::fs::create_dir_all(&output_dir)
    .map_err(|e| format!("Failed to create output directory: {}", e))?;
let output_path = output_dir.join(&filename);
std::fs::write(&output_path, summary).map_err(|e| format!("Failed to write: {}", e))?;
```

### 新增 API 设计

**Rust 端** (`src-tauri/src/export/mod.rs`):
```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 导出请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportRequest {
    pub start_date: String,     // RFC3339 UTC
    pub end_date: String,       // RFC3339 UTC
    pub format: String,         // "json" | "markdown"
}

/// 导出结果
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportResult {
    pub path: String,           // 导出文件路径
    pub record_count: usize,    // 导出记录数
    pub file_size: u64,         // 文件大小 (bytes)
}

/// 查询指定日期范围的记录 (复用 memory_storage 的记录结构)
pub fn get_records_for_export(
    start_date: &str,
    end_date: &str,
) -> Result<Vec<Record>, String>

/// 导出为 JSON 格式
pub fn export_to_json(records: &[Record]) -> Result<String, String>

/// 导出为 Markdown 格式
pub fn export_to_markdown(records: &[Record]) -> Result<String, String>

/// 获取导出目录路径
pub fn get_export_dir() -> PathBuf
```

**Tauri 命令**:
```rust
#[command]
pub async fn export_records(request: ExportRequest) -> Result<ExportResult, String>
```

### JSON 导出格式

```json
{
  "exported_at": "2026-03-14T10:30:00Z",
  "date_range": {
    "start": "2026-03-07",
    "end": "2026-03-14"
  },
  "total_records": 42,
  "records": [
    {
      "id": 1,
      "timestamp": "2026-03-14T02:30:00Z",
      "source_type": "auto",
      "content": "正在编写 Rust 后端代码...",
      "screenshot_path": "screenshots/screenshot_20260314_103000.png"
    }
  ]
}
```

### Markdown 导出格式

```markdown
# DailyLogger 数据导出

导出时间: 2026-03-14 10:30
日期范围: 2026-03-07 至 2026-03-14
总记录数: 42

---

## 2026-03-14

### 时间线

- **10:30** 🖥️ 自动感知
  正在编写 Rust 后端代码...

- **11:15** ⚡ 闪念
  需要实现数据库连接...

---

## 2026-03-13

### 时间线

...
```

### 前端组件结构

```
ExportModal.vue
├─ Header: 标题 + 关闭按钮
├─ DateRangePicker:
│  ├─ 开始日期选择
│  └─ 结束日期选择
├─ FormatSelector:
│  ├─ JSON 选项
│  └─ Markdown 选项
├─ Actions:
│  ├─ 导出按钮
│  └─ 取消按钮
└─ ResultDisplay (导出后显示):
   ├─ 文件路径
   ├─ 记录数量
   └─ 打开目录按钮
```

### 项目结构 Notes

- 新模块位置: `src-tauri/src/export/mod.rs`
- 新组件位置: `src/components/ExportModal.vue`
- 后端修改: `src-tauri/src/memory_storage/mod.rs`, `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`
- 导出目录: `~/.local/share/DailyLogger/exports/`
- 遵循现有命名规范: snake_case (Rust), PascalCase (Vue)

### 代码复用机会

1. **synthesis 模块的 `format_records_for_summary`**: 可以参考其时间格式化和来源图标逻辑
2. **memory_storage 的 `get_today_records_sync`**: 复用其时区转换和数据库查询模式
3. **synthesis 模块的文件写入**: 复用目录创建和文件写入的错误处理模式

### 测试要求

**Rust 测试** (`export/mod.rs`):
- 测试 JSON 导出格式正确性
- 测试 Markdown 导出格式正确性
- 测试日期范围边界
- 测试空记录导出
- 测试特殊字符转义 (JSON)

**前端测试** (Vitest):
- 组件挂载时初始化默认日期范围
- 格式选择切换
- 导出按钮状态 (日期有效时启用)
- 导出成功后显示结果

### 边界条件考虑

- 空记录导出: 生成有效但内容为空的文件
- 大数据量导出: 前端显示进度，后端考虑分批写入 (当前数据量预期不大)
- 特殊字符: JSON 内容需正确转义，Markdown 内容需处理特殊字符
- 跨时区: 导出文件使用本地时区显示，JSON 中保留 UTC 时间戳

### 性能考虑

- 数据库已存在 `idx_timestamp` 索引，日期范围查询高效
- 导出文件写入到本地目录，避免网络 IO
- 前端导出过程使用 loading 状态，防止用户重复点击

### 错误处理

- 日期范围无效 (开始日期晚于结束日期): 前端验证，禁用导出按钮
- 导出目录创建失败: 返回错误信息，提示用户检查权限
- 文件写入失败: 返回错误信息，建议用户重试

### References

- [Source: architecture.md#5.1] 数据库 schema 和索引
- [Source: architecture.md#6] API 端点设计模式
- [Source: synthesis/mod.rs] 文件写入和记录格式化参考
- [Source: memory_storage/mod.rs] 数据库查询和时区处理参考
- [Source: epics.md#Epic 4] 数据管理 Epic 上下文

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

No issues encountered during implementation.

### Completion Notes List

1. **后端导出模块** (`src-tauri/src/export/mod.rs`): 实现了完整的数据导出功能
   - `ExportRequest` / `ExportResult` 数据结构
   - `export_records_to_json()`: JSON 格式导出，包含完整记录信息和元数据
   - `export_records_to_markdown()`: Markdown 格式导出，按日期分组，时间线排列
   - `export_records` Tauri 命令：日期范围校验、格式校验、文件写入
   - 13 个单元测试覆盖 JSON/Markdown 导出格式、空记录、特殊字符等

2. **前端导出组件** (`src/components/ExportModal.vue`): 完整的导出 UI
   - 日期范围选择器（默认最近 7 天）
   - JSON / Markdown 格式切换
   - 日期校验（开始日期不能晚于结束日期）
   - 导出进度状态和成功/失败反馈
   - 10 个 Vitest 测试覆盖组件交互

3. **集成**: 在 App.vue 头部添加"📤 导出"按钮，注册 ExportModal 组件

4. **复用现有代码**:
   - 复用 `get_records_by_date_range_sync()` 进行日期范围查询
   - 复用 synthesis 模块的时间格式化和来源图标模式
   - 遵循现有 TailwindCSS 主题和组件模式

### File List

- `src-tauri/src/export/mod.rs` (新增) - 导出模块核心逻辑 + 13 测试
- `src-tauri/src/lib.rs` (修改) - 注册 export 模块
- `src-tauri/src/main.rs` (修改) - 注册 export_records Tauri 命令
- `src/components/ExportModal.vue` (新增) - 导出弹窗组件
- `src/App.vue` (修改) - 添加导出按钮和 ExportModal 集成
- `src/__tests__/ExportModal.spec.js` (新增) - 前端组件测试 (10 tests)

### Change Log

- 2026-03-15: Implemented DATA-004 data export feature (JSON/Markdown) with full TDD coverage