# Agent Plan: Add Function Implementation

## 1. 任务理解

实现一个简单的 `add` 函数，接收两个 `i32` 参数，返回它们的和。

**规格要求**（来自 `specs/add.md`）：
- 接口：`pub fn add(a: i32, b: i32) -> i32`
- 正常相加：`add(2, 3) == 5`
- 负数：`add(-1, 1) == 0`
- 零：`add(0, 0) == 0`

这是一个基础的纯函数，无副作用，无需处理 overflow（Rust 的 i32 加法在 debug 模式下会 panic，release 模式下会 wrap，这是预期行为）。

## 2. 当前状态

- **仓库类型**: Tauri v2 桌面应用（Vue 3 + Rust + SQLite）
- **代码位置**: Rust 代码位于 `src-tauri/`，库入口在 `src-tauri/src/lib.rs`
- **测试模式**: 在 `lib.rs` 中有 `#[cfg(test)]` 测试模块
- **相关文档**:
  - `specs/add.md` - 函数规格
  - `plan.md` - 任务列表
- **现有测试**: `lib.rs` 中有 `mask_api_key` 函数的测试示例

## 3. 行动计划

1. 在 `src-tauri/src/lib.rs` 中添加 `add` 函数
2. 在同一文件中的 `#[cfg(test)]` 模块中添加测试用例
3. 运行 `cargo test` 验证测试通过
4. 编写审查报告到 `/workspace/review_result.json`

## 4. 技术决策

**放置位置**: 将 `add` 函数放在 `lib.rs` 中，因为：
- `lib.rs` 是库的入口点
- 现有的 `mask_api_key` 工具函数也在此处
- 这是一个通用的纯函数，不属于任何特定业务模块

**测试策略**: 遵循 TDD 原则，但鉴于任务简单，将同时编写实现和测试：
- 测试覆盖规格中的所有约束
- 使用 `assert_eq!` 宏进行断言

## 5. 验证方式

- `cargo test` 所有测试通过
- 特别验证 `add` 相关测试通过
- 运行 `cargo clippy -- -D warnings` 无警告
- 运行 `cargo fmt --check` 格式正确
