# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

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

### Pre-commit checklist (required before every PR)
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

## Common Pitfalls

- **Database locked**: Always acquire the global `Mutex` — never create a separate `Connection`
- **Tauri command not found at runtime**: Register new commands in `generate_handler![]` in `main.rs`
- **Screenshot path**: Stored relative to the app data dir; use `app.path().app_data_dir()` to resolve
- **OpenAI calls**: Screenshots are Base64-encoded before being sent to the Vision API
