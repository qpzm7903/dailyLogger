# Story 8.2: 时段批量上下文分析

Status: ready-for-dev

## Story

As a DailyLogger user,
I want screenshots within a work session to be analyzed together with context from the previous session,
so that AI analysis produces more accurate, context-aware results instead of fragmented single-screenshot analysis.

## Background

SESSION-001 完成了捕获与分析解耦，截图现在只保存不立即分析（`analysis_status = 'pending'`）。本 Story 实现核心分析管线 `analyze_session()`：

**核心价值**：工作是连续的。AI 分析必须理解上下文才能给出有意义的结果。
- 单张截图无法区分"刚打开 VS Code"和"编码 2 小时"
- 批量分析可传递上下文：上一时段摘要 → 当前时段理解
- 减少 API 调用：10 张截图从 10 次 Vision API 调用变为 1 次批量调用

**前置依赖**：
- SESSION-001 已完成：`sessions` 表、`session_manager` 模块、`records.session_id` 和 `analysis_status` 字段

## Acceptance Criteria

1. **时段截图收集**
   - `get_session_screenshots(session_id)` 返回该时段所有 `analysis_status = 'pending'` 的截图记录
   - 返回格式：`Vec<SessionScreenshot>` 包含 `record_id`, `timestamp`, `screenshot_path`

2. **上一时段上下文获取**
   - `get_previous_session_context(session_id)` 返回上一时段的 `context_for_next` 字段
   - 若无上一时段或 `context_for_next` 为空，返回 `None`

3. **批量 Vision API 调用**
   - 构建 multi-image prompt：每张截图带时间戳，附加上一时段上下文
   - 使用现有 `analyze_screen` 模式的 Vision API 调用（支持自定义 Headers）
   - Prompt 要求返回结构化 JSON：`per_screenshot_analysis` + `session_summary` + `context_for_next`

4. **分析结果解析与存储**
   - 解析 API 返回的 JSON
   - 更新每张截图的 `records.content` 和 `records.analysis_status = 'analyzed'`
   - 更新 `sessions.ai_summary` 和 `sessions.context_for_next`
   - 更新 `sessions.status = 'analyzed'`

5. **Tauri Commands 暴露**
   - `analyze_session(session_id: i64)` - 异步分析指定时段
   - `get_session_screenshots(session_id: i64)` - 获取时段截图列表

6. **错误处理**
   - API 调用失败时记录日志，不抛出异常导致应用崩溃
   - 支持重试机制（返回 Error 让调用方决定是否重试）
   - 截图文件不存在时跳过该截图，继续分析其他截图

7. **测试覆盖**
   - `cargo test --no-default-features` 通过
   - `cargo clippy -- -D warnings` 无警告
   - 新增 `analyze_session` 相关单元测试

## Tasks / Subtasks

- [ ] Task 1: 数据库查询函数 (AC: #1, #2)
  - [ ] 1.1 在 `memory_storage/records.rs` 添加 `get_records_by_session_id(session_id: i64)`
  - [ ] 1.2 在 `session_manager/mod.rs` 添加 `get_previous_session_context(session_id: i64)`
  - [ ] 1.3 添加 `SessionScreenshot` 结构体

- [ ] Task 2: 批量分析 Prompt 设计 (AC: #3)
  - [ ] 2.1 设计 multi-image prompt 结构
  - [ ] 2.2 定义 API 返回 JSON Schema（`SessionAnalysisResponse`）
  - [ ] 2.3 编写 DEFAULT_SESSION_ANALYSIS_PROMPT 常量

- [ ] Task 3: 实现 analyze_session 核心函数 (AC: #3, #4)
  - [ ] 3.1 实现 `collect_session_screenshots()` - 收集时段截图
  - [ ] 3.2 实现 `build_multi_image_request()` - 构建多图 API 请求
  - [ ] 3.3 实现 `call_vision_api_batch()` - 批量 Vision API 调用
  - [ ] 3.4 实现 `parse_and_store_results()` - 解析并存储结果
  - [ ] 3.5 组装 `analyze_session()` 主函数

- [ ] Task 4: Tauri Commands 暴露 (AC: #5)
  - [ ] 4.1 添加 `#[command] async fn analyze_session(session_id: i64)`
  - [ ] 4.2 添加 `#[command] async fn get_session_screenshots(session_id: i64)`
  - [ ] 4.3 在 `lib.rs` 的 `generate_handler![]` 中注册新命令

- [ ] Task 5: 错误处理与边界情况 (AC: #6)
  - [ ] 5.1 处理截图文件不存在的情况
  - [ ] 5.2 处理 API 调用失败的情况
  - [ ] 5.3 处理空时段（无截图）的情况
  - [ ] 5.4 处理 JSON 解析失败的情况

- [ ] Task 6: 测试验证 (AC: #7)
  - [ ] 6.1 编写 `analyze_session` 单元测试
  - [ ] 6.2 运行 `cargo fmt`
  - [ ] 6.3 运行 `cargo clippy -- -D warnings`
  - [ ] 6.4 运行 `cargo test --no-default-features`

## Dev Notes

### 关键文件位置

```
src-tauri/src/
├── lib.rs                              # 注册新 Tauri commands
├── session_manager/
│   └── mod.rs                          # analyze_session() 实现
├── memory_storage/
│   ├── mod.rs                          # Record 结构体
│   └── records.rs                      # get_records_by_session_id()
└── auto_perception/
    └── mod.rs                          # 参考 Vision API 调用模式
```

### 现有代码参考

**Vision API 多图调用模式** (`auto_perception/mod.rs` lines 605-767):
```rust
async fn analyze_screen(settings: &CaptureSettings, image_base64: &str) -> Result<ScreenAnalysis, String> {
    let request_body = serde_json::json!({
        "model": settings.model_name,
        "messages": [
            {
                "role": "user",
                "content": [
                    {"type": "text", "text": prompt},
                    {"type": "image_url", "image_url": {"url": format!("data:image/png;base64,{}", image_base64)}}
                ]
            }
        ],
        "max_tokens": 500
    });
    // ... HTTP client setup with custom headers support
}
```

**多图 API 请求格式**（OpenAI Vision API 支持）:
```json
{
  "model": "gpt-4o",
  "messages": [
    {
      "role": "user",
      "content": [
        {"type": "text", "text": "分析以下截图，每张带时间戳..."},
        {"type": "image_url", "image_url": {"url": "data:image/png;base64,<img1>"}},
        {"type": "image_url", "image_url": {"url": "data:image/png;base64,<img2>"}},
        ...
      ]
    }
  ],
  "max_tokens": 2000
}
```

**Sessions 表结构** (`schema.rs` lines 290-310):
```sql
CREATE TABLE sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL,
    start_time TEXT NOT NULL,
    end_time TEXT,
    ai_summary TEXT,           -- AI 生成的时段摘要
    user_summary TEXT,         -- 用户自写的时段摘要（优先）
    context_for_next TEXT,     -- 传递给下一时段分析的上下文
    status TEXT DEFAULT 'active'  -- active | ended | analyzed
);
```

**Records 表结构** (`schema.rs` lines 302-320):
```sql
-- records 表已有字段
session_id INTEGER REFERENCES sessions(id),
analysis_status TEXT DEFAULT 'pending'  -- pending | analyzed | user_edited
```

### 批量分析 Prompt 设计

```rust
const DEFAULT_SESSION_ANALYSIS_PROMPT: &str = r#"你是一个工作分析助手。用户在一段时间内连续工作了 N 分钟，期间截取了多张屏幕截图。

请分析这些截图，理解用户在这段时间内的工作内容，返回以下 JSON 格式：

{
  "per_screenshot_analysis": [
    {
      "timestamp": "2026-03-22T10:05:00Z",
      "current_focus": "正在编写 Rust 代码",
      "active_software": "VS Code",
      "context_keywords": ["Rust", "Tauri", "异步"],
      "tags": ["开发"]
    },
    ...
  ],
  "session_summary": "用户在这段时间主要进行 Rust 后端开发，实现了工作时段管理功能...",
  "context_for_next": "正在开发 session_manager 模块，下一步需要实现 analyze_session 函数..."
}

注意：
1. per_screenshot_analysis 数组长度必须与输入截图数量一致
2. session_summary 应概括整个时段的工作内容
3. context_for_next 用于帮助下一时段理解连续性工作

上一时段上下文（如有）：
{previous_context}

返回纯 JSON，不要添加任何其他文字。"#;
```

### 返回 JSON Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionAnalysisResponse {
    pub per_screenshot_analysis: Vec<ScreenshotAnalysis>,
    pub session_summary: String,
    pub context_for_next: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotAnalysis {
    pub timestamp: String,
    pub current_focus: String,
    pub active_software: String,
    pub context_keywords: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionScreenshot {
    pub record_id: i64,
    pub timestamp: String,
    pub screenshot_path: String,
}
```

### 核心函数签名

```rust
// session_manager/mod.rs

/// 分析指定时段的所有截图
///
/// # Arguments
/// * `session_id` - 时段 ID
///
/// # Returns
/// * `Ok(())` - 分析成功，结果已写入数据库
/// * `Err(String)` - 分析失败
pub async fn analyze_session(session_id: i64) -> Result<(), String> {
    // 1. 收集时段截图
    let screenshots = collect_session_screenshots(session_id)?;

    if screenshots.is_empty() {
        return Err("No pending screenshots in session".to_string());
    }

    // 2. 获取上一时段上下文
    let previous_context = get_previous_session_context(session_id)?;

    // 3. 构建多图请求
    let request = build_multi_image_request(&screenshots, previous_context.as_deref())?;

    // 4. 调用 Vision API
    let response = call_vision_api_batch(&request).await?;

    // 5. 解析并存储结果
    parse_and_store_results(session_id, &screenshots, &response)?;

    Ok(())
}

/// 获取时段内所有待分析的截图
pub fn get_session_screenshots(session_id: i64) -> Result<Vec<SessionScreenshot>, String>

/// 获取上一时段的上下文
fn get_previous_session_context(session_id: i64) -> Result<Option<String>, String>
```

### Tauri Commands

```rust
// lib.rs

tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
        // ... 现有命令 ...
        session_manager::analyze_session,
        session_manager::get_session_screenshots,
    ])
```

### 错误处理策略

1. **截图文件不存在**：跳过该截图，记录 `tracing::warn!`，继续处理其他截图
2. **API 调用失败**：返回 `Err(String)`，让调用方决定是否重试
3. **JSON 解析失败**：返回 `Err(String)`，记录原始响应内容便于调试
4. **空时段**：返回 `Err("No pending screenshots in session")`

### 与现有代码的集成点

1. **Settings 复用**：使用 `memory_storage::get_settings_sync()` 获取 API 配置
2. **HTTP Client 复用**：使用 `crate::create_http_client()` 创建客户端
3. **Custom Headers**：复用 AI-006 的自定义 Headers 支持
4. **API Key Masking**：复用 `crate::mask_api_key()` 日志脱敏

### 性能考虑

- 批量分析可减少 API 调用次数（10 张图从 10 次变为 1 次）
- 注意 Vision API 的 token 限制（GPT-4o: 128k input tokens）
- 如果时段截图过多（> 20 张），考虑分批处理
- 大图需考虑内存占用

### 向后兼容

- `records.analysis_status = 'pending'` 的记录才会被分析
- 已分析的记录（`status = 'analyzed'`）会被跳过
- 用户编辑过的记录（`status = 'user_edited'`）也会被跳过（保留用户内容）

### Project Structure Notes

- 遵循项目现有 Rust 代码风格
- 使用 `cargo fmt` 格式化
- 日志使用 `tracing` crate
- 异步函数使用 `async fn`，Tauri commands 使用 `#[command]`

### References

- [Source: _bmad-output/planning-artifacts/architecture.md#Section-2.2] - session_manager 模块定义
- [Source: _bmad-output/planning-artifacts/architecture.md#Section-3.2] - 分析管线流程
- [Source: src-tauri/src/session_manager/mod.rs] - SESSION-001 实现的基础函数
- [Source: src-tauri/src/auto_perception/mod.rs#L605-767] - Vision API 调用模式
- [Source: src-tauri/src/synthesis/mod.rs#L62-187] - LLM API 调用共享函数
- [Source: src-tauri/src/memory_storage/records.rs] - Record 结构体和查询函数

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6 (claude-opus-4-6)

### Debug Log References

(待开发时填写)

### Completion Notes List

(待开发时填写)

### File List

(待开发时填写)

## Change Log

- 2026-03-22: Story 创建，状态设置为 ready-for-dev