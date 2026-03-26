# Story 9.4: 仪表板信息架构重组 (Dashboard Reorganization)

Status: review

## Story

As a DailyLogger user,
I want a dashboard with real-time status indicators, optimized record lists, and cleaner report tabs,
so that I can quickly understand my current work status and navigate the interface efficiently.

## Background

UX-1 (设计令牌体系建立) 和 UX-2 (按钮组件规范化) 已完成，建立了完整的设计令牌系统和按钮 CSS 类。UX-3 (侧边栏导航升级) 已完成，提升了导航可发现性和激活状态。

UX-4 在此基础上重组仪表板信息架构，优化数据呈现和空间利用效率，解决以下问题：
- Dashboard 操作按钮过载（5个按钮挤在一起）
- 记录列表双层滚动问题（max-h-80 限制）
- 输出文件卡片视觉噪音（多个"尚未生成"空状态）
- Header 资源浪费（仅显示文字+时钟，无实时状态）

**Epic 上下文**: Epic 9 (UX-REDESIGN) 的第四个 Story，依赖 UX-1/2/3 的成果。

## Acceptance Criteria

1. **Header 实时状态栏 (AC#1)**
   - 自动捕获状态指示（运行中/暂停），使用语义化颜色
   - 今日记录数实时统计（从数据库实时查询）
   - 状态变化时有平滑过渡动画
   - AC 来源: UX-REDESIGN-EPIC.md Story UX-4 AC#1

2. **今日工作流记录列表优化 (AC#2)**
   - 去掉 `max-h-80` 限制，改为全高度或分页
   - 实现虚拟滚动或分页（每页 20 条）
   - 消除双层滚动问题
   - 保持响应式设计
   - AC 来源: UX-REDESIGN-EPIC.md Story UX-4 AC#2

3. **输出文件卡片改为 Tab 式 (AC#3)**
   - 日报/周报/月报 Tab 切换
   - 统一空状态展示（单个"尚未生成"提示代替多个分散的空状态）
   - Tab 切换动画平滑
   - AC 来源: UX-REDESIGN-EPIC.md Story UX-4 AC#3

4. **自动捕获卡片和速记卡片样式更新 (AC#4)**
   - 更紧凑的操作面板布局
   - 视觉权重调整（主操作更突出）
   - 保持与设计令牌的语义一致性
   - AC 来源: UX-REDESIGN-EPIC.md Story UX-4 AC#4

5. **测试与 CI (AC#5)**
   - `npm run test` 通过
   - `npm run lint` 无警告
   - `npm run typecheck` 无错误
   - AC 来源: UX-REDESIGN-EPIC.md Story UX-4 AC#5

## Tasks / Subtasks

- [x] Task 1: Header 实时状态栏 (AC: #1)
  - [x] 1.1 分析现有 Header.vue 结构
  - [x] 1.2 添加自动捕获状态指示器（useAutoCapture 或后端状态）
  - [x] 1.3 添加今日记录数统计（从数据库实时查询）
  - [x] 1.4 添加状态过渡动画
  - [x] 1.5 使用语义化颜色（--color-status-success/running/paused）

- [x] Task 2: 记录列表优化 (AC: #2)
  - [x] 2.1 分析 Dashboard.vue 中记录列表的当前实现
  - [x] 2.2 移除 max-h-80 限制
  - [x] 2.3 实现虚拟滚动或分页（每页 20 条）
  - [x] 2.4 确保滚动行为流畅，消除双层滚动
  - [x] 2.5 响应式设计适配

- [x] Task 3: 输出文件卡片 Tab 式改造 (AC: #3)
  - [x] 3.1 分析当前 ReportCard/DailySummaryViewer 结构
  - [x] 3.2 设计 Tab 切换组件（日报/周报/月报）
  - [x] 3.3 统一空状态展示
  - [x] 3.4 添加 Tab 切换动画

- [x] Task 4: 卡片样式更新 (AC: #4)
  - [x] 4.1 优化自动捕获卡片布局
  - [x] 4.2 优化速记卡片布局
  - [x] 4.3 调整视觉权重（主操作更突出）
  - [x] 4.4 使用设计令牌保持一致性

- [x] Task 5: 验证与测试 (AC: #5)
  - [x] 5.1 运行 `npm run test` 确保测试通过
  - [x] 5.2 运行 `npm run lint` 确保无警告
  - [x] 5.3 运行 `npm run typecheck` 确保无错误

## Dev Notes

### 关键文件分析

**Header.vue 当前状态（来自 UX-3 完成后）**
- 位置: `src/components/layout/Header.vue`
- 当前实现:
  - 仅显示 "DailyLogger" 文字 + 时钟
  - 27 行代码
  - 无实时状态反馈
- 需要添加:
  - 自动捕获状态指示器
  - 今日记录数统计

**Dashboard.vue 当前状态（来自 UX-1/2/3 完成后）**
- 位置: `src/components/layout/Dashboard.vue`
- 当前实现:
  - 300+ 行代码
  - 5 个报告操作按钮（已部分合并）
  - 记录列表有 max-h-80 限制
  - 多个输出文件卡片（日报/周报/月报分立）
- 需要优化:
  - 记录列表滚动行为
  - 输出文件卡片整合为 Tab 式

**设计令牌使用（来自 UX-1）**

```css
/* 语义化令牌（已建立） */
--color-action-primary: /* 主操作蓝色 */
--color-action-secondary: /* 辅助操作灰色 */
--color-action-danger: /* 破坏性操作红色 */
--color-action-neutral: /* 工具操作深灰 */
--color-status-success: /* 成功绿色 */
--color-status-warning: /* 警告橙色 */
--color-status-error: /* 错误红色 */
--color-status-info: /* 信息蓝色 */
--color-surface-0: /* 背景底层 */
--color-surface-1: /* 卡片层 */
--color-surface-2: /* 表面层 */

/* 按钮 CSS 类（已建立） */
.btn-primary / .btn-secondary / .btn-ghost / .btn-danger
.btn-sm / .btn-md / .btn-lg
```

### 组件结构参考

**Header 布局目标**:
```
┌──────────────────────────────────────────────────────────────┐
│ [DailyLogger]  🟢 运行中 | 📝 42 条记录     2026-03-26 10:30 │
└──────────────────────────────────────────────────────────────┘
```

**Dashboard 布局目标**:
```
┌──────────────────────────────────────────────────────────────┐
│  Header (已优化)                                             │
├────────────┬─────────────────────────────────────────────────┤
│            │  今日工作流                                      │
│  Sidebar   │  ┌─────────────────────────────────────────┐   │
│  (已优化)   │  │ 记录列表 (全高/分页)                      │   │
│            │  │ - 截图1: 10:30 - AI 分析结果...           │   │
│            │  │ - 截图2: 10:15 - AI 分析结果...           │   │
│            │  │ ...                                      │   │
│            │  └─────────────────────────────────────────┘   │
│            │                                                 │
│            │  输出文件 (Tab 式)                              │
│            │  ┌─────────────────────────────────────────┐   │
│            │  │ [日报] [周报] [月报]                     │   │
│            │  │                                         │   │
│            │  │ 报告内容或空状态                         │   │
│            │  └─────────────────────────────────────────┘   │
└────────────┴─────────────────────────────────────────────────┘
```

### 技术约束

1. **滚动优化**: 使用 Vue 的虚拟滚动或分页，避免一次性渲染大量 DOM
2. **状态同步**: 自动捕获状态需要与 Rust 后端同步（通过 Tauri 命令）
3. **数据库查询**: 今日记录数需要高效的 SQL 查询（带索引）
4. **动画性能**: 使用 CSS transition 而非 JavaScript 动画

### 关键文件

| 文件 | 改动幅度 | 说明 |
|------|---------|------|
| `src/components/layout/Dashboard.vue` | 较大（重构） | 记录列表优化、输出卡片 Tab 化、卡片布局调整 |
| `src/components/layout/Header.vue` | 中等（扩展） | 添加实时状态栏 |
| `src/styles/main.css` | 无 | 使用已有设计令牌 |

### Project Structure Notes

- Vue 3 Composition API + `<script setup>`
- TypeScript strict mode
- Tailwind CSS v4（通过 CSS 变量使用设计令牌）
- Tauri v2 后端通信（Rust）
- 虚拟滚动考虑使用 `@vueuse/core` 或自定义实现

### 依赖版本

无新增依赖。使用已有：
- `vue` 3.4+
- `@vueuse/core`（如果需要）
- `lucide-vue-next` 1.0.0（UX-3 已添加）

### 设计决策（来自 Epic 文档）

**决策 1: 记录列表滚动方案**
- 方案 A: 虚拟滚动（vue-virtual-scroller）
  - 优点：性能好，无分页感
  - 缺点：增加依赖，复杂
- 方案 B: 分页（每页 20 条）
  - 优点：简单，无需新依赖
  - 缺点：用户需要点击加载更多
- **决策**: 待实现时根据实际数据量选择，可先用分页

**决策 2: Tab 组件**
- 使用 Tailwind 实现基础 Tab
- 不引入额外 UI 库依赖
- Tab 切换使用 CSS transition

### References

- [Source: _bmad-output/implementation-artifacts/UX-REDESIGN-EPIC.md#Story UX-4] - Epic 规格说明
- [Source: _bmad-output/implementation-artifacts/UX-1-design-tokens.md] - UX-1 成果（设计令牌和按钮系统）
- [Source: _bmad-output/implementation-artifacts/UX-2-button-normalization.md] - UX-2 成果（按钮规范化）
- [Source: _bmad-output/implementation-artifacts/UX-3-sidebar-upgrade.md] - UX-3 成果（侧边栏升级）
- [Source: src/components/layout/Dashboard.vue] - 当前仪表板组件
- [Source: src/components/layout/Header.vue] - 当前 Header 组件
- [Source: src/styles/main.css#行1-100] - 设计令牌定义

## Dev Agent Record

### Agent Model Used

claude-opus-4-6

### Debug Log References

### Completion Notes List

- **Task 1 (Header 实时状态栏)**: 添加了 `autoCaptureEnabled` 和 `todayRecordsCount` props 到 Header.vue，显示自动捕获状态（运行中/已暂停）和今日记录数。使用语义化颜色 `--color-status-success` 和过渡动画。
- **Task 2 (记录列表优化)**: 移除了 `max-h-80` 限制，实现了分页功能（每页20条），添加了"加载更多"按钮。解决了双层滚动问题。
- **Task 3 (输出文件 Tab 式改造)**: 将分散的日报/周报/月报卡片整合为 Tab 切换组件，统一了空状态展示为"尚未生成"。自定义报告和对比报告移至下方单独区域。
- **Task 4 (卡片样式更新)**: 优化了自动捕获卡片和速记卡片的布局，将 padding 从 p-5 减小到 p-4，移除了描述文字以减少视觉噪音，按钮更紧凑。
- **Task 5 (验证与测试)**: 所有前端测试通过（927个），类型检查通过，linting 通过。Rust 测试通过。

## File List

- `src/components/layout/Header.vue` - 添加实时状态栏（自动捕获状态 + 记录数）
- `src/components/layout/Dashboard.vue` - 记录列表分页 + 输出文件 Tab 化 + 卡片样式优化
- `src/locales/zh-CN.json` - 添加 header.running/paused/records, dashboard.loadMore, outputTabs.* 翻译
- `src/locales/en.json` - 添加对应英译翻译
- `src/__tests__/Dashboard.test.ts` - 更新测试以适配 Tab 布局

## Change Log

- 2026-03-26: 完成 UX-4-dashboard-reorganization 全部任务，标记为 review 状态
