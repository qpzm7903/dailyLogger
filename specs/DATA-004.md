# DATA-004 数据导出 (JSON/MD)

## 功能需求

导出 DailyLogger 数据用于备份或分析，支持 JSON 和 Markdown 格式。

## 接口定义

### Rust Tauri 命令

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    pub format: String,           // "json" | "markdown"
    pub start_date: Option<String>, // RFC3339 或 YYYY-MM-DD
    pub end_date: Option<String>,   // RFC3339 或 YYYY-MM-DD
    pub include_screenshots: bool,  // 是否包含截图路径
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub file_path: String,  // 导出文件路径
    pub record_count: i64,  // 导出记录数
    pub file_size: i64,     // 文件大小 (bytes)
}

#[command]
pub async fn export_data(options: ExportOptions) -> Result<ExportResult, String>
```

## 验收条件 (Given/When/Then)

### AC1: 导出为 JSON 格式
- Given 用户选择 JSON 格式导出, When 执行导出, Then 生成有效的 JSON 文件包含所有选定记录
- JSON 结构包含: records 数组、export_time、record_count
- 每条记录包含: id, timestamp, source_type, content, screenshot_path

### AC2: 导出为 Markdown 格式
- Given 用户选择 Markdown 格式导出, When 执行导出, Then 生成格式化的 .md 文件
- Markdown 结构: 标题、导出时间、记录列表 (时间戳 + 来源 + 内容)
- 自动记录和手动记录有区分标识

### AC3: 日期范围筛选
- Given 用户指定日期范围, When 导出, Then 仅导出该范围内的记录
- 支持单日导出 (start_date == end_date)
- 支持不指定日期 (导出全部)

### AC4: 导出结果反馈
- Given 导出完成, When 用户查看结果, Then 显示文件路径和记录数量
- 提供打开文件/打开所在目录选项

## 约束

- 使用现有的 `DB_CONNECTION` 全局 Mutex
- 导出文件保存到应用数据目录的 exports/ 子目录
- 大数据量导出时显示进度 (可选优化)
- 导出操作不修改原始数据