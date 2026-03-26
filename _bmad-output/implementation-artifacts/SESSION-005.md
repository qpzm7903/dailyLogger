# Story 8.5: 日报生成适配

Status: review

## Story

As a DailyLogger user,
I want my daily report to be organized by work sessions with AI summaries enriched by my personal edits,
so that the report reflects the natural rhythm of my work day and prioritizes my own understanding over AI interpretations.

## Background

SESSION-001 完成了捕获与分析解耦，SESSION-002 完成了批量上下文分析，SESSION-003 完成了用户编辑功能。本 Story 实现日报生成与时段分析结果的适配：

**核心价值**：
- 日报应按工作时段组织，而非扁平记录列表
- 用户自写的摘要（user_summary）应优先于 AI 生成的摘要（ai_summary）
- 每个时段内的截图也应优先显示用户备注（user_notes）
- 未分析的时段应在生成日报前自动触发分析

**前置依赖**：
- SESSION-001 已完成：sessions 表、session_manager 模块
- SESSION-002 已完成：analyze_session() 批量分析、ai_summary 和 context_for_next 字段
- SESSION-003 已完成：user_summary 和 user_notes 用户编辑、update_session_user_summary 命令

**SESSION-005 在 Epic-8 中的位置**：
```
SESSION-001 (基础) ─→ SESSION-002 (分析) ─→ SESSION-005 (日报) ← 当前
                  ─→ SESSION-003 (编辑)
                  ─→ SESSION-004 (手动触发)
```

## Acceptance Criteria

1. **基于时段的日报内容组织**
   - 日报按工作时段（Session）组织，而非扁平记录列表
   - 每个时段显示为一个章节，包含时段时间和摘要
   - 时段按时间顺序排列

2. **用户摘要优先展示**
   - 时段摘要使用 `COALESCE(user_summary, ai_summary)`
   - 若两者都为空，显示"暂无摘要"
   - 用户已编辑的时段标记特殊标识（如 ✏️）

3. **时段内截图内容优先**
   - 每个时段下显示其截图的用户备注优先内容
   - 截图使用 `COALESCE(user_notes, content)` 优先展示用户内容
   - 未分析的截图不展示（pending 状态跳过）

4. **未分析时段自动处理**
   - 生成日报时，若存在 pending/ended 状态的未分析时段
   - 自动触发 `analyze_session()` 进行分析
   - 分析完成后再组织日报内容

5. **Tauri Commands 暴露**
   - 复用现有的 `get_today_sessions` 命令
   - 复用现有的 `analyze_session` 命令
   - 可能需要新增 `get_session_records(session_id)` 获取时段内记录

6. **向后兼容**
   - 若当天无任何 session（legacy 数据），回退到现有的扁平记录格式
   - 日报输出格式保持 Markdown 兼容

7. **测试覆盖**
   - `cargo test --no-default-features` 通过
   - `cargo clippy -- -D warnings` 无警告
   - 新增 session-based 日报生成的单元测试

## Tasks / Subtasks

- [x] Task 1: 修改日报内容组织逻辑 (AC: #1, #2, #3)
  - [x] 1.1 在 `synthesis/mod.rs` 中新增 `format_session_for_summary()` 函数
  - [x] 1.2 修改 `generate_daily_summary()` 使用 sessions 而非 flat records
  - [x] 1.3 实现时段章节格式化（时段时间 + 摘要 + 截图列表）

- [x] Task 2: 用户摘要优先逻辑 (AC: #2)
  - [x] 2.1 实现 `get_session_display_summary(session)` 返回 user_summary > ai_summary
  - [x] 2.2 为用户编辑过的时段添加 ✏️ 标识
  - [x] 2.3 确保空摘要情况正确处理

- [x] Task 3: 时段内截图内容优先 (AC: #3)
  - [x] 3.1 实现 `get_session_records_for_summary(session_id)` 获取时段内记录
  - [x] 3.2 时段内截图使用 `user_notes > content` 优先逻辑
  - [x] 3.3 跳过 pending 状态的截图

- [x] Task 4: 未分析时段自动分析 (AC: #4)
  - [x] 4.1 在 `generate_daily_summary()` 中检查 pending/ended 时段
  - [x] 4.2 对未分析时段调用 `analyze_session()`
  - [x] 4.3 分析完成后组织日报内容

- [x] Task 5: 向后兼容 (AC: #6)
  - [x] 5.1 检测当天是否有 sessions 数据
  - [x] 5.2 若无 sessions，回退到现有扁平记录格式
  - [x] 5.3 测试 fallback 路径

- [x] Task 6: 测试验证 (AC: #7)
  - [x] 6.1 编写 session-based 日报生成单元测试
  - [x] 6.2 运行 `cargo fmt`
  - [x] 6.3 运行 `cargo clippy -- -D warnings`
  - [x] 6.4 运行 `cargo test --no-default-features`

## Dev Notes

### 关键文件位置

```
src-tauri/src/
├── main.rs                             # Tauri commands 注册
├── session_manager/
│   └── mod.rs                          # analyze_session(), get_today_sessions()
├── memory_storage/
│   ├── mod.rs                          # Record 结构体
│   └── records.rs                      # get_records_by_session_id()
└── synthesis/
    └── mod.rs                          # generate_daily_summary() 修改

src/
├── components/
│   └── DailySummaryViewer.vue          # 日报查看组件（可能需要适配）
```

### 现有代码参考

**sessions 表结构** (`schema.rs` lines 290-310):
```sql
CREATE TABLE sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL,
    start_time TEXT NOT NULL,
    end_time TEXT,
    ai_summary TEXT,                       -- AI 生成的时段摘要
    user_summary TEXT,                     -- 用户自写的时段摘要（优先）
    context_for_next TEXT,                 -- 传递给下一时段分析的上下文
    status TEXT DEFAULT 'active'           -- active | ended | analyzed
);
```

**records 表关键字段**:
```sql
-- records 表已有字段
session_id INTEGER REFERENCES sessions(id),
user_notes TEXT,                           -- 用户自写备注（优先于 content）
analysis_status TEXT DEFAULT 'pending'     -- pending | analyzed | user_edited
```

**get_today_sessions 命令** (SESSION-001):
```rust
// session_manager/mod.rs
#[command]
pub async fn get_today_sessions() -> Result<Vec<Session>, String>
```

**analyze_session 命令** (SESSION-002):
```rust
// session_manager/mod.rs
#[command]
pub async fn analyze_session(session_id: i64) -> Result<(), String>
```

**get_records_by_session_id** (SESSION-002):
```rust
// memory_storage/records.rs
pub fn get_records_by_session_id(session_id: i64) -> Result<Vec<Record>, String>
```

### 核心函数设计

**修改后的 generate_daily_summary 流程**:
```rust
#[command]
pub async fn generate_daily_summary() -> Result<String, String> {
    // 1. 检查网络
    if !crate::network_status::is_online() {
        // ... 离线队列处理
    }

    // 2. 获取设置
    let settings = memory_storage::get_settings_sync()?;
    let obsidian_path = settings.get_obsidian_output_path()?;
    let api_config = load_api_config(&settings)?;

    // 3. 获取今日所有时段
    let sessions = session_manager::get_today_sessions()?;

    // 4. 处理未分析的时段
    for session in &sessions {
        if session.status == "active" || session.status == "ended" {
            session_manager::analyze_session(session.id).await?;
        }
    }

    // 5. 重新获取最新状态
    let sessions = session_manager::get_today_sessions()?;

    if sessions.is_empty() {
        // 5a. Fallback: 无 sessions，回退到扁平记录格式
        return generate_daily_summary_legacy().await;
    }

    // 6. 按会话组织日报内容
    let content = build_session_based_report(&sessions);

    // 7. 生成最终日报
    let prompt = build_report_prompt(&content, &settings);
    let summary = call_llm_api(&api_config, &prompt, 2000, "generate_daily_summary").await?;

    // 8. 写入文件并通知
    // ... (保持现有逻辑)
}
```

**时段章节格式化**:
```rust
/// 按会话组织日报内容
fn build_session_based_report(sessions: &[Session]) -> String {
    let mut content = String::new();

    for session in sessions {
        // 时段标题
        let time_range = format_time_range(&session.start_time, &session.end_time);
        let summary = get_display_summary(session);
        let is_edited = session.user_summary.is_some();

        content.push_str(&format!(
            "## {} - {}\n\n{}{}\n\n",
            time_range,
            session.status,
            if is_edited { "✏️ " } else { "" },
            summary
        ));

        // 时段内截图
        let records = memory_storage::get_records_by_session_id(session.id)
            .unwrap_or_default();

        for record in records {
            // 跳过未分析的截图
            if record.analysis_status.as_deref() == Some("pending") {
                continue;
            }

            let display_content = record.user_notes
                .as_ref()
                .filter(|n| !n.is_empty())
                .map(|n| n.as_str())
                .unwrap_or(&record.content);

            let time = extract_time(&record.timestamp);
            content.push_str(&format!("- [{}] {}\n", time, display_content));
        }

        content.push_str("\n");
    }

    content
}

/// 获取时段显示摘要（user_summary > ai_summary > "暂无摘要"）
fn get_display_summary(session: &Session) -> String {
    session
        .user_summary
        .as_ref()
        .filter(|s| !s.is_empty())
        .or(session.ai_summary.as_ref())
        .filter(|s| !s.is_empty())
        .cloned()
        .unwrap_or_else(|| "暂无摘要".to_string())
}
```

### 与 SESSION-002/SESSION-003 的集成点

1. **SESSION-002 分析管线**：
   - 复用 `analyze_session()` 对未分析时段进行批量分析
   - 复用 `get_today_sessions()` 获取时段列表

2. **SESSION-003 用户编辑**：
   - 复用 `user_summary` 优先展示逻辑
   - 复用 `user_notes` 优先展示逻辑
   - 复用 `analysis_status` 判断截图是否已分析

3. **前端状态管理**：
   - 日报查看器可能需要适配新的 session-based 格式

### Project Structure Notes

- 遵循项目现有 Rust 代码风格
- 使用 `cargo fmt` 格式化
- 日志使用 `tracing` crate
- 异步函数使用 `async fn`，Tauri commands 使用 `#[command]`
- 复用现有 `call_llm_api` 进行 LLM 调用

### References

- [Source: _bmad-output/planning-artifacts/architecture.md#Section-3.3] - 日报生成流程（需适配）
- [Source: _bmad-output/planning-artifacts/architecture.md#Section-5.1.1] - sessions 表结构和优先级规则
- [Source: _bmad-output/planning-artifacts/architecture.md#Section-5.1] - records 表 user_notes 字段
- [Source: _bmad-output/implementation-artifacts/SESSION-001.md] - SESSION-001 实现细节
- [Source: _bmad-output/implementation-artifacts/SESSION-002.md] - SESSION-002 实现细节
- [Source: _bmad-output/implementation-artifacts/SESSION-003.md] - SESSION-003 实现细节
- [Source: src-tauri/src/session_manager/mod.rs] - analyze_session 和 get_today_sessions
- [Source: src-tauri/src/memory_storage/records.rs] - get_records_by_session_id
- [Source: src-tauri/src/synthesis/mod.rs] - 现有 generate_daily_summary 实现

## Dev Agent Record

### Implementation Notes

**Date**: 2026-03-26

**Implementation Summary**:
- 在 `synthesis/mod.rs` 中实现了 `build_session_based_report()` 函数，按时段组织日报内容
- 实现了 `get_session_display_summary()` 函数，实现 user_summary > ai_summary > "暂无摘要" 优先级
- 实现了 `get_session_records_for_summary()` 函数，获取时段内记录，优先展示 user_notes
- 实现了 `format_session_for_summary()` 函数，格式化单个时段为章节
- 修改了 `generate_daily_summary()` 函数，优先使用 session-based 方式生成日报
- 添加了 AC#4 自动分析未分析时段的逻辑
- 添加了 AC#6 向后兼容，当无 sessions 时回退到扁平记录格式
- 新增 10 个单元测试覆盖新功能

**Files Modified/Created**:
- `src-tauri/src/synthesis/mod.rs` (修改)

### Completion Notes

所有任务已完成并通过测试验证：
- 454 Rust 测试通过（包含 10 个新增测试）
- Clippy 无警告
- Cargo fmt 通过

## Change Log

- 2026-03-26: feat(SESSION-005): 实现日报生成适配 - 基于时段的日报内容组织、用户摘要优先、未分析时段自动分析、向后兼容
