<!-- Windows build fix test PR -->
# AGENTS.md — DailyLogger Agent Instructions

> 架构与命令详见 [CLAUDE.md](./CLAUDE.md)。本文件只包含 AI Agent 专属约束与快速反馈命令。

## Quick Commands

```bash
# 前端测试（单文件快速反馈）
npx vitest run src/__tests__/<file>.test.ts

# Rust 单测（快速反馈）
cd src-tauri && cargo test <test_name>

# 提交前必过检查
cd src-tauri && cargo fmt && cargo clippy -- -D warnings && cargo test
npm run test
```

## TDD — 强制要求

**所有新业务逻辑必须先写测试**，流程：Red → Green → Refactor

**Rust**: 测试写在同文件的 `#[cfg(test)]` 模块内
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_record_returns_rowid() {
        // 先定义期望，再写实现
        let id = add_record("manual", "content", None).unwrap();
        assert!(id > 0);
    }
}
```

**Vue 3**: 测试放 `src/__tests__/` 或与组件同目录（`*.spec.ts`），使用 Vitest + `@vue/test-utils`

## DO

- 新增 Tauri 命令后**必须**在 `src-tauri/src/main.rs` 的 `generate_handler![]` 中注册
- 数据库操作**必须**通过全局 `DB_CONNECTION`（`Mutex<Option<Connection>>`），使用 `params![]` 参数化查询
- Rust 错误处理统一使用 `Result<T, String>` + `.map_err(|e| e.to_string())`
- Vue 组件统一使用 `<script setup>` 语法 + TailwindCSS（自定义色：`bg-dark` / `bg-darker` / `text-primary`）
- 截图路径通过 `dirs::data_dir()` 解析，不硬编码绝对路径

## DON'T

- **禁止**在没有对应测试的情况下提交业务逻辑代码
- **禁止**为让测试通过而修改测试断言（需求变更除外）
- **禁止**新建第二个 `rusqlite::Connection` — 数据库只有一个全局连接
- **禁止**直接推送到 `main`，所有变更必须通过 PR
- **禁止**在 SQL 语句中字符串拼接参数（SQL 注入风险）
- **禁止**在 Rust 模块外暴露 `DB_CONNECTION`，数据库访问只通过 `memory_storage` 公开函数

## Common Mistakes (从已修复 Bug 提炼)

| 现象 | 原因 | 修复 |
|------|------|------|
| Tauri 调用时 "command not found" | 忘记在 `generate_handler![]` 注册 | 在 `main.rs` 添加命令引用 |
| "database is locked" 错误 | 创建了第二个 DB 连接 | 通过 `memory_storage` 函数访问 |
| 自动捕获阻塞 UI 线程 | 用了阻塞式 sleep | 改用 `tokio::spawn` + `AtomicBool` 信号 |
| Vue 调用 Tauri 命令报类型错误 | 参数名与 Rust 函数签名不一致 | Rust 参数用 `camelCase` 对应 JS 侧 |

## PR & CI

PR 创建后自动运行（`.github/workflows/test.yml`）：
- `test-rust`: `cargo fmt --check` → `cargo clippy -D warnings` → `cargo test`
- `test-frontend`: `npm run test`

两个 job 全部通过方可合并。
