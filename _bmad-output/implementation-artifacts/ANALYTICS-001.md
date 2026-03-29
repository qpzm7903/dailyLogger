# Story 18.1: 生产力趋势与周期对比分析

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

作为用户，我希望查看工作生产力的时间趋势和与上一周期的对比，以便了解我的工作效率变化模式。

## Implementation Summary

- Backend: Added `ProductivityTrend` struct with `PeriodComparison`, `HourlyDistribution`, `DailyTrendPoint`
- Backend: Added `get_productivity_trend` command for week/month comparison
- Frontend: Added "趋势对比" tab to StatisticsPanel with SVG line chart
- Frontend: Added period selector (本周 vs 上周, 本月 vs 上月)
- Frontend: Added peak hours display showing top 5 busiest hours
- i18n: Added translation keys for en.json and zh-CN.json
- Export: Extended CSV export to support productivity trend data

## Acceptance Criteria

1. [AC-1] 在 StatisticsPanel 中新增"趋势对比"Tab，显示生产力趋势图和周期对比数据
2. [AC-2] 支持选择"本周 vs 上周"、"本月 vs 上月"对比视图
3. [AC-3] 趋势图使用折线图展示 screenshot_count、record_count 随时间变化
4. [AC-4] 周期对比卡片显示：当前周期总数、上一周期总数、变化百分比（带颜色指示）
5. [AC-5] 后端新增 `get_productivity_trend` 命令，计算日均记录数、高峰时段分布
6. [AC-6] 前端新增 `ProductivityTrend` 类型定义
7. [AC-7] 新增翻译键：`statistics.trendsAndComparison`、`statistics.vsLastPeriod`、`statistics.changePercent` 等
8. [AC-8] 导出功能扩展支持 productivity trend 数据

## Tasks / Subtasks

- [x] Task 1: 后端 - 新增 ProductivityTrend 类型和 get_productivity_trend 命令 (AC: #5, #6)
  - [x] Subtask 1.1: 在 memory_storage/mod.rs 中定义 ProductivityTrend 数据结构
  - [x] Subtask 1.2: 实现 get_productivity_trend 异步命令函数
  - [x] Subtask 1.3: 在 commands/mod.rs 中注册新命令
  - [x] Subtask 1.4: 在 src/types/tauri.ts 中添加 ProductivityTrend TypeScript 类型
- [x] Task 2: 前端 - StatisticsPanel 新增趋势对比视图 (AC: #1, #2, #3, #4)
  - [x] Subtask 2.1: 在 StatisticsPanel.vue 中新增 Tab 切换（明细/趋势对比）
  - [x] Subtask 2.2: 实现 TrendComparisonView 子组件
  - [x] Subtask 2.3: 实现 PeriodComparisonCard 组件
  - [x] Subtask 2.4: 使用 CSS/SVG 实现简单折线图（复用现有 bar chart 样式模式）
  - [x] Subtask 2.5: 实现周期选择器（本周vs上周、本月vs上月）
- [x] Task 3: 国际化 - 新增翻译键 (AC: #7)
  - [x] Subtask 3.1: 在 zh-CN.json 和 en.json 中新增 statistics.trendsAndComparison 相关翻译
- [x] Task 4: 导出增强 (AC: #8)
  - [x] Subtask 4.1: 扩展 generateCsv 函数支持 productivity trend 数据

## Senior Developer Review (AI)

**审查日期**: 2026-03-29
**审查结论**: ✅ 通过

**验证结果**:
- AC-1 ~ AC-8: 全部实现 ✓
- Rust clippy: 通过 ✓
- Rust tests: 508 passed ✓
- TypeScript typecheck: 通过 ✓
- Frontend tests: 1180 passed ✓
- 错误处理完善 ✓
- SQL 注入防护：参数化查询 ✓

**问题**: Tasks 标记为 [ ] 但实际已实现（文档同步问题）

## Dev Notes

### 项目结构
- 后端核心: `src-tauri/src/memory_storage/mod.rs` (现有 get_statistics 函数所在)
- 后端命令注册: `src-tauri/src/commands/mod.rs`
- 前端组件: `src/components/StatisticsPanel.vue`
- 类型定义: `src/types/tauri.ts` (已有 Statistics, DailyStatistic 类型)
- 国际化: `src/locales/zh-CN.json`, `src/locales/en.json`

### 现有模式参考
- StatisticsPanel 当前使用简单的 CSS bar chart（3列并排），趋势图使用类似模式但改为折线
- 后端 get_statistics 返回 `Statistics` 结构，包含 `daily_breakdown: Vec<DailyStatistic>`
- 前端通过 `invoke<Statistics>('get_statistics', args)` 调用

### 关键约束
- 使用 Tauri v2 插件系统
- 所有核心逻辑在 Rust 端实现
- SQLite 数据库，查询现有统计函数可复用
- 使用 Composition API 和 `<script setup>`
- 唯一样式方案：Tailwind CSS（无独立 CSS 文件）

### 测试要求
- Rust: `cargo test --no-default-features` 通过
- 前端: `npm run test` 通过
- 类型检查: `npm run typecheck` 通过

## References

- [Source: src/components/StatisticsPanel.vue] - 现有统计面板实现，300+ 行
- [Source: src/types/tauri.ts#Statistics] - 统计数据类型定义
- [Source: src-tauri/src/memory_storage/mod.rs] - 后端统计查询函数实现
- [Source: src-tauri/src/commands/mod.rs] - Tauri 命令注册
- [Source: _bmad-output/implementation-artifacts/PERF-007.md] - 最近 story 示例（参考格式）
