# Story 8.CLEAN: 移除 GitHub 集成

Status: ready-for-dev

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

- [ ] Task 1: 删除 Rust 后端代码 (AC: #1)
  - [ ] 删除 `src-tauri/src/github.rs`
  - [ ] 从 `lib.rs` 移除 `pub mod github;`
  - [ ] 从 `main.rs` 移除 `daily_logger_lib::github::test_github_connection` 和 `daily_logger_lib::github::get_github_work_stats`

- [ ] Task 2: 删除前端组件和引用 (AC: #2)
  - [ ] 删除 `src/components/GitHubStatsPanel.vue`
  - [ ] 更新 `Dashboard.vue` 移除 import 和 `<GitHubStatsPanel />` 使用
  - [ ] 更新 `OutputSettings.vue` 移除 GitHub 配置区域

- [ ] Task 3: 清理类型定义 (AC: #3)
  - [ ] 从 `src/types/tauri.ts` 移除 GitHub 相关类型

- [ ] Task 4: 清理设置字段 (AC: #4)
  - [ ] 从 Settings 结构体移除 `github_token` 和 `github_repositories`

- [ ] Task 5: 清理测试 (AC: #5)
  - [ ] 删除 `src/components/__tests__/GitHubStatsPanel.test.ts`
  - [ ] 检查并清理其他测试文件中的 GitHub 相关代码

- [ ] Task 6: 验证 (AC: #6)
  - [ ] 运行 `cargo fmt`
  - [ ] 运行 `cargo clippy -- -D warnings`
  - [ ] 运行 `cargo test --no-default-features`
  - [ ] 运行 `npm test`

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

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List