# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

必须参考 @prompt.md

## Build & Development Commands

```bash
# Frontend only (Vite hot-reload, localhost:1420)
npm run dev

# Full Tauri dev with hot-reload (opens desktop window)
npm run tauri dev

# Production build → outputs DailyLogger.app + .dmg
npm run tauri build
```

### Rust (run from `src-tauri/`)
```bash
cargo check                      # Syntax check without building
cargo test                       # All unit tests
cargo test <test_name>           # Single test
cargo clippy -- -D warnings      # Lint (CI-enforced)
cargo fmt                        # Format
```

### Frontend tests
```bash
npm run test          # Vitest (run once)
npm run test:watch    # Watch mode
```

### Quality Gate Workflow（强制，每次代码变更后必须完整执行）

每次完成代码变更后，**必须**按顺序执行以下四个步骤，绝不能跳过任何一步。

#### 步骤 1：Code Formatting（代码格式化）
```bash
# Rust
cd src-tauri && cargo fmt

# Frontend
npm run format  # 如无此命令可跳过
```
若格式化命令报错，必须自主修复，不允许手动改行绕过。

#### 步骤 2：Static Analysis（静态分析）
```bash
# Rust — 零警告容忍
cd src-tauri && cargo clippy -- -D warnings

# Frontend
npm run lint  # 如有配置
```
所有 warning 和 error 必须修复后才能进入下一步。

#### 步骤 3：Local Test Execution（本地测试）
```bash
cd src-tauri && cargo test
npm run test
```
不允许使用 mock workaround 绕过核心逻辑测试。测试全绿后方可提交。

#### 步骤 4：CI 状态验证（提交后必须检查）
执行 `git push` 后，**必须主动监控 GitHub Actions CI 状态**：
```bash
# 查看最新 workflow 运行状态
gh run list --limit 5

# 持续等待并查看结果（直到 completed）
gh run watch

# 若有失败，查看详情
gh run view --log-failed
```
- CI **通过** → 任务完成
- CI **失败** → 立即分析报错、修复、重新推送，再次验证 CI，直到通过为止
- **禁止**在 CI 红灯状态下进行下一个任务

> **原则**：每次 `git push` 都要看到 CI 绿灯，才算本次变更真正完成。

### Pre-commit checklist（快速参考）
```bash
cd src-tauri && cargo fmt && cargo clippy -- -D warnings && cargo test
npm run test
```

CI runs these automatically on PRs. All checks must pass before merging. No direct pushes to `main`.

### Git pre-commit hook (install once per clone)
```bash
git config core.hooksPath .githooks
```
The hook at `.githooks/pre-commit` runs `cargo fmt --check` and `cargo clippy` automatically on every commit, catching CI failures locally before push.

> **`cargo fmt` is CI-blocking.** If `cargo fmt -- --check` fails in CI, fix it with `cd src-tauri && cargo fmt`, then re-commit. Never manually reformat lines to match rustfmt — always let the tool do it.

## Architecture

DailyLogger is a Tauri v2 desktop app: Vue 3 frontend + Rust backend + SQLite.

```
Vue 3 (src/)
  └─ invoke() / listen()    ← Tauri IPC
Rust (src-tauri/src/)
  ├─ main.rs                ← App init, tray, plugin registration, invoke_handler![]
  ├─ lib.rs                 ← AppState (global Mutex), module exports, init_app()
  ├─ auto_perception/       ← Scheduled screenshots + OpenAI Vision analysis
  ├─ manual_entry/          ← Quick notes + file reading
  ├─ memory_storage/        ← SQLite CRUD (records + settings tables)
  └─ synthesis/             ← AI daily summary generation → Obsidian output
```

**Frontend → Backend flow**: Vue components call `invoke('command_name', args)` from `@tauri-apps/api/core`. All Tauri commands must be registered in `main.rs` inside `generate_handler![]`.

**AppState**: A `Lazy<Mutex<AppState>>` singleton in `lib.rs` holds the SQLite connection (`Mutex<Option<Connection>>`) and an `AtomicBool` for auto-capture state. Use the existing mutex pattern — never open a second DB connection.

**Auto-capture**: Runs as a `tokio::spawn()` background task. Uses `AtomicBool` to signal stop — do not use `sleep()` loops that block the Tauri thread.

## Database Schema

Two tables in `~/.local/share/DailyLogger/data/local.db`:

```sql
records(id, timestamp TEXT NOT NULL,  -- RFC3339
        source_type TEXT NOT NULL,    -- 'auto' | 'manual'
        content TEXT NOT NULL,        -- JSON or plain text
        screenshot_path TEXT)         -- nullable

settings(id INTEGER PRIMARY KEY CHECK(id = 1),  -- single-row
         api_base_url, api_key, model_name,
         screenshot_interval INTEGER DEFAULT 5,
         summary_time TEXT DEFAULT '18:00',
         obsidian_path, auto_capture_enabled INTEGER DEFAULT 0,
         last_summary_path)
```

Always use `params![]` macro for parameterized queries. Never string-interpolate SQL.

## Code Style

**Rust**
- Module structure follows DDD — each domain in its own `mod.rs`
- `snake_case` for functions/variables, `PascalCase` for structs/enums
- Error handling: `Result<T, String>` with `.map_err(|e| e.to_string())`
- Tests in `#[cfg(test)]` blocks within the same file as the code under test

**Vue 3**
- Use `<script setup>` syntax
- TailwindCSS only — no inline styles, no CSS files per component
- Custom theme colors: `bg-dark`, `bg-darker`, `text-primary` (defined in `tailwind.config.js`)

## TDD Requirement

All new features and bug fixes must follow Red → Green → Refactor:
1. Write a failing test that defines the expected behavior
2. Write the minimal implementation to make it pass
3. Refactor under green tests

**Prohibited**: Submitting business logic code without a corresponding test. Modifying test assertions to make tests pass (unless requirements changed).

### Bug Fix Testing Rule

Every bug fix commit **must** include a regression test that reproduces the bug. The test must:
1. Fail on the old code (proves the test catches the bug)
2. Pass on the new code (proves the fix works)
3. Target the boundary condition or root cause, not just the symptom

A bug fix PR without a reproducing test will not be merged.

### Test Priority Guide

Focus testing effort on code with **high bug probability** — not on what's easy to test:
- **Must test**: Time/timezone conversions, query boundaries, state transitions, data format parsing
- **Must test**: Any function that converts between representations (local↔UTC, JSON↔struct, path↔string)
- **Should test**: Business logic with multiple code paths, error handling branches
- **Low priority**: Simple CRUD wrappers, configuration loading, straightforward delegation

### Test Isolation with Global State

`DB_CONNECTION` is a global `Lazy<Mutex<…>>` — Rust runs tests in parallel within the same process. Tests that use the shared DB must:
- Never assert on `records.len()` or `records[0]` — other tests may have inserted data
- Use `.iter().any()` or `.find()` to locate specific records by content
- Use `setup_test_db()` (creates a fresh in-memory DB) but accept that parallel tests may interleave

## Common Pitfalls

- **Database locked**: Always acquire the global `Mutex` — never create a separate `Connection`
- **Tauri command not found at runtime**: Register new commands in `generate_handler![]` in `main.rs`
- **Screenshot path**: Stored relative to the app data dir; use `app.path().app_data_dir()` to resolve
- **OpenAI calls**: Screenshots are Base64-encoded before being sent to the Vision API
- **Timezone**: Never use `.and_utc()` on a `NaiveDateTime` derived from `Local::now()`. Use `.and_local_timezone(chrono::Local).unwrap().with_timezone(&chrono::Utc)` to correctly convert local time to UTC. The `records` table stores timestamps in UTC RFC3339 format.

## BMAD 开发工作流

本项目使用 BMAD 方法进行开发管理。所有功能开发任务须遵循以下工作流：

### 快速开发工作流（标准任务）
使用 `/bmad-bmm-quick-dev` 

### 任务就绪标准（开始实施前必须满足）
- 每个任务必须有对应的 spec 文件（`specs/<task-id>.md`）
- Spec 文件需包含：功能需求、接口定义、验收条件（Given/When/Then 格式）
- 无占位符、无 TBD、具体可操作

### BMAD Spec 文件模板
参考 `specs/add.md` 格式，在 specs/ 目录下为每个任务创建规格文件。
