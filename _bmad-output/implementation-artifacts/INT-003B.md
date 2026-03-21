# Story 6.3B: GitHub 工时统计展示

Status: done

## Story

As a DailyLogger 用户,
I want 在应用界面中独立查看 GitHub 工时统计，而不是仅在日报中查看,
so that 我可以实时了解今日代码工作情况，无需等到生成日报时才能看到统计。

## 背景

INT-003A 已验证 GitHub API 集成完整可用：
- 后端 `github.rs` 实现了完整的 API 集成和工时统计算法（提交聚类）
- 统计数据已集成到日报生成流程（synthesis/mod.rs:486-506）
- 前端配置 UI 已存在（OutputSettings.vue:100-132）
- 测试覆盖完善（16 个测试）

但当前 GitHub 工时统计**仅嵌入在日报 Markdown 中**，用户无法在应用界面独立查看实时统计。

本 Story 需要添加独立的工时统计展示功能。

## Acceptance Criteria

1. 新增 Tauri 命令 `get_github_work_stats` 返回今日 GitHub 工时统计（JSON 格式）
2. 前端新增 `GitHubStatsPanel.vue` 组件展示今日 GitHub 活动：
   - 提交数、PR 数、预估工时
   - 活跃仓库列表
   - 提交时间分布（可选：时间线图表）
3. 组件在 Dashboard 中显示，仅当 GitHub 已配置时可见
4. 支持手动刷新统计数据
5. 加载状态和错误处理友好
6. 单元测试覆盖新增命令和组件

## Tasks / Subtasks

- [x] Task 1: 后端命令实现 (AC: 1)
  - [x] 1.1 在 `github.rs` 添加 `#[command] get_github_work_stats()`
  - [x] 1.2 定义 `GitHubWorkStatsResponse` 结构体用于 JSON 序列化
  - [x] 1.3 在 `main.rs` 注册新命令
  - [x] 1.4 添加单元测试

- [x] Task 2: 前端组件开发 (AC: 2, 3, 5)
  - [x] 2.1 创建 `src/components/GitHubStatsPanel.vue` 组件
  - [x] 2.2 实现数据获取逻辑（调用 `get_github_work_stats`）
  - [x] 2.3 实现加载状态和错误处理 UI
  - [x] 2.4 在 Dashboard 中集成组件（条件渲染）

- [x] Task 3: 交互功能 (AC: 4)
  - [x] 3.1 添加刷新按钮和刷新逻辑
  - [x] 3.2 添加自动刷新选项（可选）

- [x] Task 4: 测试与验证 (AC: 6)
  - [x] 4.1 后端单元测试
  - [x] 4.2 前端组件测试（Vitest）
  - [x] 4.3 手动测试各场景

## Dev Notes

### 现有代码位置

**后端 (Rust)**:
- `src-tauri/src/github.rs`:
  - `GitHubWorkStats` - 工时统计结构体（line 58-72）
  - `fetch_today_github_activity()` - 获取今日统计（line 347-494）
  - `format_github_activity_for_report()` - Markdown 格式化（line 295-344）
  - `is_github_configured()` - 检查配置状态（line 75-77）
- `src-tauri/src/main.rs`:
  - Tauri 命令注册（line 375-472）

**前端 (Vue)**:
- `src/components/layout/Dashboard.vue` - 主面板
- `src/components/settings/OutputSettings.vue` - GitHub 配置 UI（line 100-132）
- `src/types/` - TypeScript 类型定义

### 实现参考

**新增 Tauri 命令示例**:

```rust
// github.rs
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct GitHubWorkStatsResponse {
    pub configured: bool,
    pub stats: Option<GitHubWorkStatsJson>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GitHubWorkStatsJson {
    pub commit_count: usize,
    pub pr_count: usize,
    pub estimated_hours: f64,
    pub active_repos: Vec<String>,
    pub commits_by_hour: std::collections::HashMap<u32, Vec<String>>,
    pub pull_requests: Vec<String>,
}

#[command]
pub async fn get_github_work_stats() -> Result<GitHubWorkStatsResponse, String> {
    let settings = crate::memory_storage::get_settings_sync()?;

    if !is_github_configured(&settings) {
        return Ok(GitHubWorkStatsResponse {
            configured: false,
            stats: None,
        });
    }

    match fetch_today_github_activity(&settings).await {
        Ok(stats) => Ok(GitHubWorkStatsResponse {
            configured: true,
            stats: Some(GitHubWorkStatsJson {
                commit_count: stats.commit_count,
                pr_count: stats.pr_count,
                estimated_hours: stats.estimated_hours,
                active_repos: stats.active_repos,
                commits_by_hour: stats.commits_by_hour,
                pull_requests: stats.pull_requests,
            }),
        }),
        Err(e) => {
            tracing::warn!("Failed to fetch GitHub stats: {}", e);
            Err(e)
        }
    }
}
```

**前端组件结构**:

```vue
<!-- GitHubStatsPanel.vue -->
<template>
  <div class="github-stats-panel">
    <div class="header">
      <h3>GitHub 今日活动</h3>
      <button @click="refresh" :disabled="loading">刷新</button>
    </div>

    <div v-if="loading" class="loading">加载中...</div>
    <div v-else-if="error" class="error">{{ error }}</div>
    <div v-else-if="!configured" class="not-configured">
      请在设置中配置 GitHub Token
    </div>
    <div v-else class="stats-content">
      <div class="summary">
        <div class="stat-item">
          <span class="label">提交</span>
          <span class="value">{{ stats?.commit_count || 0 }}</span>
        </div>
        <div class="stat-item">
          <span class="label">PR</span>
          <span class="value">{{ stats?.pr_count || 0 }}</span>
        </div>
        <div class="stat-item">
          <span class="label">预估工时</span>
          <span class="value">{{ stats?.estimated_hours?.toFixed(1) || 0 }}h</span>
        </div>
      </div>

      <div v-if="stats?.active_repos?.length" class="repos">
        活跃仓库: {{ stats.active_repos.join(', ') }}
      </div>
    </div>
  </div>
</template>
```

### 关联 Story

- **INT-003A** (已完成): GitHub API 集成验证 - 提供后端 API 基础
- **INT-001** (已完成): Notion 导出 - 可参考前端组件结构
- **CORE-006** (已完成): API Key 加密 - GitHub Token 复用加密机制

### 注意事项

1. **避免重复请求**: 组件挂载时加载数据，刷新按钮手动触发
2. **错误处理**: API 调用失败时显示友好提示，不阻塞界面
3. **条件渲染**: 仅在 GitHub 已配置时显示组件或显示配置提示
4. **响应式设计**: 组件应适应 Dashboard 的布局
5. **类型定义**: 前端添加对应的 TypeScript 接口

### 项目结构注意事项

- 遵循现有 Vue 组件结构和命名规范
- 使用 TailwindCSS 样式（项目无独立 CSS 文件）
- 测试运行使用 `cargo test --no-default-features` 避免 xcap 依赖

### 测试运行命令

```bash
# 后端测试
cd src-tauri && cargo test --no-default-features github

# 前端测试
npm run test

# 格式化和 Lint
cd src-tauri && cargo fmt && cargo clippy -- -D warnings
```

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6 (claude-opus-4-6)

### Debug Log References

无

### Completion Notes List

- **Task 1 完成**: 添加了 `get_github_work_stats` Tauri 命令，返回 `GitHubWorkStatsResponse` 结构体
- **Task 2 完成**: 创建了 `GitHubStatsPanel.vue` 组件，支持显示提交数、PR数、预估工时、活跃仓库和提交时间分布
- **Task 3 完成**: 实现了刷新按钮和刷新逻辑
- **Task 4 完成**: 添加了 3 个后端单元测试和 13 个前端组件测试，所有 928 个测试通过

### File List

- `src-tauri/src/github.rs` - 添加 `GitHubWorkStatsJson`、`GitHubWorkStatsResponse` 结构体和 `get_github_work_stats` 命令
- `src-tauri/src/main.rs` - 注册新命令
- `src/types/tauri.ts` - 添加 TypeScript 类型定义
- `src/components/GitHubStatsPanel.vue` - 新增前端组件
- `src/components/layout/Dashboard.vue` - 集成 GitHubStatsPanel 组件
- `src/components/__tests__/GitHubStatsPanel.test.ts` - 新增前端测试文件

## Change Log

- 2026-03-21: Story created - GitHub work time statistics display
- 2026-03-21: Implementation complete - All tasks completed, tests passing (928/928)
- 2026-03-21: Code review passed - All ACs verified, 19 backend + 13 frontend tests passing, no issues found

---

## Code Review Findings (2026-03-21)

**Reviewer:** Claude Opus 4.6
**Verdict:** ✅ PASSED

### Acceptance Criteria Verification

| AC | Status | Evidence |
|----|--------|----------|
| AC1: Tauri 命令 | ✅ | `github.rs:142-170` - `get_github_work_stats` command |
| AC2: 前端组件 | ✅ | `GitHubStatsPanel.vue` - 提交/PR/工时/时间分布 |
| AC3: Dashboard 集成 | ✅ | `Dashboard.vue:77` - 组件已集成 |
| AC4: 手动刷新 | ✅ | `GitHubStatsPanel.vue:10-18` - 刷新按钮 |
| AC5: 状态处理 | ✅ | 加载/错误/未配置三种状态 |
| AC6: 测试覆盖 | ✅ | 19 后端 + 13 前端测试通过 |

### Code Quality

- **安全**: 无安全漏洞，Token 加密存储
- **性能**: 合理的 API 调用频率
- **代码风格**: Clippy 无警告
- **测试质量**: 真实断言，覆盖主要场景

### Issues Found

无 High/Medium/Low 问题发现。