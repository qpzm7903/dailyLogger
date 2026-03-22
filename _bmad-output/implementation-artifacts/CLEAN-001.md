# Story 8.CLEAN: 移除 GitHub 集成

Status: review

## Story

As a DailyLogger developer,
I want to remove the GitHub integration feature,
so that the codebase is simplified and aligned with the product strategy (GitHub integration removed in v3.0.0).

## Background

根据 Sprint Change Proposal v2 (2026-03-22)，GitHub 集成功能已被移除出产品规划。此功能属于 P3 集成类功能，在核心体验未达极致前不优先。当前需要清理相关代码以减少维护负担。

## Acceptance Criteria

1. **删除 Rust 后端模块**
   - 删除 `src-tauri/src/github.rs` 文件（865 行代码）
   - 从 `src-tauri/src/lib.rs` 移除 `pub mod github;` 声明
   - 从 `src-tauri/src/main.rs` 移除 `test_github_connection` 和 `get_github_work_stats` 命令注册

2. **删除前端组件**
   - 删除 `src/components/GitHubStatsPanel.vue` 文件
   - 从 `src/components/layout/Dashboard.vue` 移除 GitHubStatsPanel 导入和使用
   - 从 `src/components/settings/OutputSettings.vue` 移除 GitHub Work Time Statistics 配置区域

3. **删除类型定义**
   - 从 `src/types/tauri.ts` 移除 `GitHubStats`, `RepositoryStats`, `GetGitHubStatsArgs`, `GitHubWorkStatsJson`, `GitHubWorkStatsResponse` 类型

4. **清理设置字段**
   - 从 `src-tauri/src/memory_storage/mod.rs` 的 Settings 结构体移除 `github_token` 和 `github_repositories` 字段

5. **删除测试文件**
   - 删除 `src/components/__tests__/GitHubStatsPanel.test.ts`
   - 删除 `tests/e2e/fixtures/test-data.ts` 中的 GitHub 相关 mock 数据（如有）

6. **验证测试通过**
   - `cargo test --no-default-features` 通过
   - `npm test` 通过
   - `cargo clippy -- -D warnings` 无警告

## Tasks / Subtasks

- [x] Task 1: 删除 Rust 后端代码 (AC: #1)
  - [x] 删除 `src-tauri/src/github.rs`
  - [x] 从 `lib.rs` 移除 `pub mod github;`
  - [x] 从 `main.rs` 移除 `daily_logger_lib::github::test_github_connection` 和 `daily_logger_lib::github::get_github_work_stats`

- [x] Task 2: 删除前端组件和引用 (AC: #2)
  - [x] 删除 `src/components/GitHubStatsPanel.vue`
  - [x] 更新 `Dashboard.vue` 移除 import 和 `<GitHubStatsPanel />` 使用
  - [x] 更新 `OutputSettings.vue` 移除 GitHub 配置区域

- [x] Task 3: 清理类型定义 (AC: #3)
  - [x] 从 `src/types/tauri.ts` 移除 GitHub 相关类型

- [x] Task 4: 清理设置字段 (AC: #4)
  - [x] 从 Settings 结构体移除 `github_token` 和 `github_repositories`

- [x] Task 5: 清理测试 (AC: #5)
  - [x] 删除 `src/components/__tests__/GitHubStatsPanel.test.ts`
  - [x] 检查并清理其他测试文件中的 GitHub 相关代码

- [x] Task 6: 验证 (AC: #6)
  - [x] 运行 `cargo fmt`
  - [x] 运行 `cargo clippy -- -D warnings`
  - [x] 运行 `cargo test --no-default-features`
  - [x] 运行 `npm test`

## Dev Notes

### 文件清单

**删除文件：**
```
src-tauri/src/github.rs                      # 865 行 Rust 后端
src/components/GitHubStatsPanel.vue         # 181 行 Vue 组件
src/components/__tests__/GitHubStatsPanel.test.ts  # 测试文件
```

**修改文件：**
```
src-tauri/src/lib.rs                        # 移除 github 模块声明
src-tauri/src/main.rs                       # 移除 github 命令注册 (lines 433-434)
src/components/layout/Dashboard.vue         # 移除 GitHubStatsPanel 引用 (lines 77, 308)
src/components/settings/OutputSettings.vue  # 移除 GitHub 配置区域 (lines 100-132, 220-225, 256, 260-261, 371-397)
src/types/tauri.ts                          # 移除 GitHub 类型 (lines 184-220)
src-tauri/src/memory_storage/mod.rs         # 移除 github_token 和 github_repositories 字段 (lines 84-85)
```

### 代码模式

**Dashboard.vue 变更：**
```vue
<!-- 删除这行 -->
<GitHubStatsPanel @open-settings="$emit('open', 'settings')" />

<!-- 删除这个 import -->
import GitHubStatsPanel from '../GitHubStatsPanel.vue'
```

**OutputSettings.vue 变更：**
- 移除 GitHub Work Time Statistics 整个配置区块（约 30 行模板 + 相关脚本）
- 移除 `testGithubConnection` 函数
- 移除 `isTestingGithubConnection` 和 `githubConnectionStatus` ref
- 移除 Props 中的 `github_token` 和 `github_repositories`

**Settings 结构体变更：**
```rust
// 移除这两行
pub github_token: Option<String>,
pub github_repositories: Option<String>,
```

### 风险提示

1. **数据库迁移**：用户数据库中可能存在 `github_token` 和 `github_repositories` 设置值。由于 Settings 使用 Option 类型，移除字段后旧数据会被忽略，无需数据库迁移。
2. **加密密钥**：`github_token` 可能被加密存储，移除后不影响加密模块的其他用途（如 API Key 加密）。

### Project Structure Notes

- 遵循项目现有代码风格
- Rust 使用 `cargo fmt` 格式化
- 前端使用项目配置的 ESLint/Prettier 规则

### References

- [Source: _bmad-output/planning-artifacts/sprint-change-proposal-2026-03-22-v2.md]
- [Source: _bmad-output/planning-artifacts/epics.md#Epic-8]
- [Source: _bmad-output/planning-artifacts/architecture.md]

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6 (claude-opus-4-6)

### Debug Log References

无

### Completion Notes List

1. 成功删除 `src-tauri/src/github.rs` (865 行 Rust 后端代码)
2. 从 `lib.rs` 移除 `pub mod github;` 声明
3. 从 `main.rs` 移除 `test_github_connection` 和 `get_github_work_stats` 命令注册
4. 删除 `src/components/GitHubStatsPanel.vue` (181 行 Vue 组件)
5. 从 `Dashboard.vue` 移除 GitHubStatsPanel import 和使用
6. 从 `OutputSettings.vue` 移除 GitHub 配置区域、测试函数和相关状态
7. 从 `src/types/tauri.ts` 移除 GitHub 相关类型定义
8. 从 `memory_storage/mod.rs` Settings 结构体移除 `github_token` 和 `github_repositories` 字段
9. 从 `memory_storage/settings.rs` 移除 GitHub 相关的读写和加解密代码
10. 从 `synthesis/mod.rs` 移除 GitHub 活动集成代码
11. 删除 `src/components/__tests__/GitHubStatsPanel.test.ts`
12. 更新 `OutputSettings.test.ts` 移除 GitHub 测试用例并修复按钮索引
13. 从 `SettingsModal.vue` 移除 GitHub 相关的设置和事件处理
14. 从 `en.json` 和 `zh-CN.json` 移除 GitHub 相关翻译

### File List

**删除的文件：**
- src-tauri/src/github.rs
- src/components/GitHubStatsPanel.vue
- src/components/__tests__/GitHubStatsPanel.test.ts

**修改的文件：**
- src-tauri/src/lib.rs
- src-tauri/src/main.rs
- src-tauri/src/synthesis/mod.rs
- src-tauri/src/memory_storage/mod.rs
- src-tauri/src/memory_storage/settings.rs
- src/components/layout/Dashboard.vue
- src/components/settings/OutputSettings.vue
- src/components/settings/__tests__/OutputSettings.test.ts
- src/components/SettingsModal.vue
- src/types/tauri.ts
- src/locales/en.json
- src/locales/zh-CN.json
- _bmad-output/implementation-artifacts/sprint-status.yaml

## Change Log

- 2026-03-22: 完成 CLEAN-001 实现，移除 GitHub 集成功能