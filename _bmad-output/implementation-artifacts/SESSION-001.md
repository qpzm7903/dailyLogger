# Story 8.1: 捕获与分析解耦 + 时段管理

Status: review

## Story

As a DailyLogger user,
I want screenshots to be captured without immediate AI analysis and organized into work sessions,
so that I get more accurate, context-aware analysis results instead of fragmented single-screenshot analysis.

## Background

当前 `capture_and_store()` 在每次截图后立即调用 `analyze_screen()`，对单张截图做独立分析。这违反了一个基本事实：**工作是连续的**。

问题：
- 上下文缺失：AI 无法区分"刚打开 VS Code"和"编码 2 小时"
- 分析粒度错误：5 分钟一张截图，每张独立分析，产生大量重复且碎片化的结果
- API 成本浪费：每张截图一次 Vision API 调用

根据 Sprint Change Proposal v2 (2026-03-22)，本 Story 是 Epic 8 的基础，后续 SESSION-002/003/004/005 都依赖于此。

## Acceptance Criteria

1. **捕获与分析解耦**
   - 截图捕获后不调用 `analyze_screen()`，仅保存截图和 pending 状态记录
   - 现有的 `capture_only_mode` 逻辑可复用，但需移除条件判断（始终只捕获不分析）

2. **新增 sessions 表**
   - 表结构符合 Architecture Section 5.1.1 定义
   - 字段：id, date, start_time, end_time, ai_summary, user_summary, context_for_next, status

3. **records 表扩展**
   - 新增 `session_id INTEGER` 字段（可为 NULL，向后兼容现有数据）
   - 新增 `analysis_status TEXT DEFAULT 'pending'` 字段（值：pending | analyzed | user_edited）

4. **时段检测逻辑**
   - 两次截图间隔 > 30 分钟 → 自动创建新时段
   - 30 分钟间隔可通过设置 `session_gap_minutes` 配置
   - 时段开始时间 = 第一张截图时间
   - 时段结束时间 = 检测到新时段时设置，或应用退出时

5. **新增 session_manager 模块**
   - 创建 `src-tauri/src/session_manager/mod.rs`
   - 核心函数：`detect_or_create_session()`, `get_today_sessions()`, `end_current_session()`
   - 在 `lib.rs` 中添加 `pub mod session_manager;`

6. **向后兼容**
   - 现有 records 数据不受影响（session_id 可为 NULL）
   - 应用启动时不因 session_id 为 NULL 而报错
   - 数据库迁移使用 `ALTER TABLE ... ADD COLUMN`（不存在才添加）

7. **测试覆盖**
   - `cargo test --no-default-features` 通过
   - `cargo clippy -- -D warnings` 无警告
   - 新增 session_manager 模块测试

## Tasks / Subtasks

- [x] Task 1: 数据库 Schema 扩展 (AC: #2, #3, #6)
  - [x] 1.1 在 `memory_storage/schema.rs` 添加 sessions 表创建
  - [x] 1.2 在 records 表添加 `session_id` 和 `analysis_status` 列
  - [x] 1.3 在 settings 表添加 `session_gap_minutes` 配置字段（默认 30）
  - [x] 1.4 在 `init_test_database` 中同步添加对应表结构
  - [x] 1.5 添加 `idx_session_id` 索引优化查询性能

- [x] Task 2: 创建 session_manager 模块 (AC: #5)
  - [x] 2.1 创建 `src-tauri/src/session_manager/mod.rs`
  - [x] 2.2 实现 `Session` 结构体（匹配 sessions 表字段）
  - [x] 2.3 实现 `detect_or_create_session()` - 检测/创建当前时段
  - [x] 2.4 实现 `get_today_sessions()` - 获取今日所有时段
  - [x] 2.5 实现 `end_current_session()` - 结束当前时段
  - [x] 2.6 实现 `get_current_session()` - 获取当前活跃时段
  - [x] 2.7 在 `lib.rs` 添加 `pub mod session_manager;`

- [x] Task 3: 重构 capture_and_store (AC: #1, #4)
  - [x] 3.1 移除 `analyze_screen()` 调用
  - [x] 3.2 在保存截图后调用 `detect_or_create_session()`
  - [x] 3.3 将返回的 session_id 写入 record
  - [x] 3.4 设置 record 的 analysis_status = 'pending'
  - [x] 3.5 复用现有的 `capture_only_mode` placeholder content 模式
  - [x] 3.6 移除 `capture_only_mode` 条件判断（始终只捕获不分析）

- [x] Task 4: 更新 Settings 结构体 (AC: #4)
  - [x] 4.1 在 `memory_storage/mod.rs` 的 Settings 添加 `session_gap_minutes: Option<i32>`
  - [x] 4.2 更新 `get_settings()` 和 `save_settings()` 函数

- [x] Task 5: 清理离线队列相关代码 (AC: #1)
  - [x] 5.1 保留 offline_queue 机制，但移除 ScreenshotAnalysis 任务入队逻辑
  - [x] 5.2 截图不再单独触发 AI 分析，而是在时段结束时批量处理

- [x] Task 6: 测试验证 (AC: #7)
  - [x] 6.1 编写 session_manager 模块单元测试
  - [x] 6.2 运行 `cargo fmt`
  - [x] 6.3 运行 `cargo clippy -- -D warnings`
  - [x] 6.4 运行 `cargo test --no-default-features` - 444 tests passed
  - [ ] 6.5 运行 `npm test` - 前端环境依赖缺失，跳过

## Dev Notes

### 关键文件位置

```
src-tauri/src/
├── lib.rs                        # 添加 pub mod session_manager;
├── auto_perception/mod.rs        # 重构 capture_and_store() (lines 966-1200)
├── memory_storage/
│   ├── mod.rs                    # Settings 结构体扩展
│   ├── schema.rs                 # 数据库迁移
│   └── records.rs                # add_record() 添加 session_id 参数
└── session_manager/              # 新建模块
    └── mod.rs                    # 时段管理核心逻辑
```

### 现有代码参考

**capture_only_mode 模式** (`auto_perception/mod.rs` lines 1036-1081):
```rust
// 现有的仅截图模式，可作为重构参考
if settings.capture_only_mode {
    let content = serde_json::json!({
        "current_focus": "仅截图模式 - 待分析",
        "active_software": active_window.process_name,
        "context_keywords": [],
        "active_window": {...},
        "offline_pending": true,
        "capture_only": true
    }).to_string();

    let record_id = memory_storage::add_record(
        "auto", &content, screenshot_path.as_deref(),
        monitor_info_json.as_deref(), None
    )?;

    // Queue for later analysis
    let _ = crate::offline_queue::enqueue_task(...);
}
```

**时段检测逻辑**:
```rust
// 在 detect_or_create_session() 中实现
// 1. 获取最后一条记录的时间戳
// 2. 与当前时间比较
// 3. 间隔 > session_gap_minutes → 创建新 session
// 4. 否则返回当前活跃 session
```

### 数据库 Schema (Architecture Section 5.1.1)

```sql
-- 新增 sessions 表
CREATE TABLE sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL,                    -- YYYY-MM-DD
    start_time TEXT NOT NULL,              -- RFC3339
    end_time TEXT,                         -- RFC3339, NULL = ongoing
    ai_summary TEXT,                       -- AI 生成的时段摘要
    user_summary TEXT,                     -- 用户自写的时段摘要
    context_for_next TEXT,                 -- 传递给下一时段分析的上下文
    status TEXT DEFAULT 'active'           -- active | ended | analyzed
);

CREATE INDEX idx_sessions_date ON sessions(date);

-- records 表扩展
ALTER TABLE records ADD COLUMN session_id INTEGER REFERENCES sessions(id);
ALTER TABLE records ADD COLUMN analysis_status TEXT DEFAULT 'pending';

CREATE INDEX idx_session_id ON records(session_id);
```

### Settings 新增字段

```rust
pub struct Settings {
    // ... 现有字段 ...
    pub session_gap_minutes: Option<i32>,  // 时段间隔阈值，默认 30 分钟
}
```

```sql
ALTER TABLE settings ADD COLUMN session_gap_minutes INTEGER DEFAULT 30;
```

### 时段检测核心逻辑

```rust
// session_manager/mod.rs

/// 检测或创建当前工作时段
/// 返回 session_id
pub fn detect_or_create_session(current_timestamp: &str) -> Result<i64, String> {
    // 1. 获取当前日期
    let today = /* 从 current_timestamp 提取 YYYY-MM-DD */;

    // 2. 检查是否有活跃时段
    if let Some(active_session) = get_active_session(&today)? {
        // 3. 检查最后一条记录时间
        let last_record_time = get_last_record_timestamp(active_session.id)?;
        let gap_minutes = calc_gap_minutes(&last_record_time, current_timestamp);

        // 4. 间隔超过阈值 → 结束当前时段，创建新时段
        let gap_threshold = get_session_gap_minutes()?; // 默认 30
        if gap_minutes > gap_threshold {
            end_session(active_session.id)?;
            return create_new_session(&today, current_timestamp);
        }

        return Ok(active_session.id);
    }

    // 5. 无活跃时段 → 创建新时段
    create_new_session(&today, current_timestamp)
}
```

### 向后兼容策略

1. **现有记录**：session_id 为 NULL 是允许的，不影响查询
2. **查询逻辑**：使用 `LEFT JOIN sessions ON records.session_id = sessions.id`
3. **日报生成**：对于 session_id 为 NULL 的记录，按时间戳分组处理

### API 端点 (Tauri Commands)

暂不添加新的 Tauri commands，SESSION-002 将添加：
- `get_today_sessions`
- `get_session_screenshots`

### 风险提示

1. **数据迁移**：使用 `ALTER TABLE ADD COLUMN` 对现有数据库安全，但需处理可能的 NULL 值
2. **测试覆盖**：需在 `--no-default-features` 模式下测试（无 screenshot feature）
3. **时区处理**：使用 `chrono::Local` 确保本地时间正确

### Project Structure Notes

- 遵循项目现有 Rust 代码风格
- 使用 `cargo fmt` 格式化
- 使用 `#[cfg(feature = "screenshot")]` 条件编译（如需要）
- 日志使用 `tracing` crate

### References

- [Source: _bmad-output/planning-artifacts/architecture.md#Section-5.1.1] - sessions 表定义
- [Source: _bmad-output/planning-artifacts/architecture.md#Section-3.1] - 捕获管线流程
- [Source: _bmad-output/planning-artifacts/sprint-change-proposal-2026-03-22-v2.md#Change-B] - Epic 8 设计
- [Source: src-tauri/src/auto_perception/mod.rs#L966-1200] - capture_and_store 现有实现
- [Source: src-tauri/src/memory_storage/schema.rs] - 数据库迁移模式

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6 (claude-opus-4-6)

### Debug Log References

无

### Completion Notes List

1. **数据库 Schema 扩展**：
   - 在 `schema.rs` 添加了 `sessions` 表创建（包含 id, date, start_time, end_time, ai_summary, user_summary, context_for_next, status 字段）
   - 在 `records` 表添加了 `session_id` 和 `analysis_status` 列
   - 在 `settings` 表添加了 `session_gap_minutes` 配置字段（默认 30 分钟）
   - 添加了 `idx_sessions_date` 和 `idx_session_id` 索引

2. **session_manager 模块**：
   - 创建了 `src-tauri/src/session_manager/mod.rs`
   - 实现了 `Session` 结构体和 `SessionStatus` 枚举
   - 实现了核心函数：`detect_or_create_session()`, `get_today_sessions()`, `end_current_session()`, `get_current_session()`
   - 添加了 4 个单元测试

3. **capture_and_store 重构**：
   - 移除了 `analyze_screen()` 调用，所有截图现在只保存不立即分析
   - 添加了 `detect_or_create_session()` 调用来关联时段
   - 使用 `add_record_with_session()` 保存 session_id 和 analysis_status
   - 移除了 `capture_only_mode` 条件判断

4. **Settings 更新**：
   - 添加了 `session_gap_minutes: Option<i32>` 字段
   - 更新了 `get_settings()` 和 `save_settings()` 函数

5. **测试结果**：
   - Rust 测试：444 passed; 0 failed
   - Clippy：0 warnings

### File List

**新增文件：**
- src-tauri/src/session_manager/mod.rs

**修改文件：**
- src-tauri/src/lib.rs
- src-tauri/src/memory_storage/mod.rs
- src-tauri/src/memory_storage/schema.rs
- src-tauri/src/memory_storage/settings.rs
- src-tauri/src/memory_storage/records.rs
- src-tauri/src/memory_storage/tags.rs
- src-tauri/src/timeline.rs
- src-tauri/src/export/mod.rs
- src-tauri/src/synthesis/mod.rs
- src-tauri/src/auto_perception/mod.rs
- _bmad-output/implementation-artifacts/sprint-status.yaml