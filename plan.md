# DailyLogger 项目规划

> 最后更新: 2026-03-30
> 当前版本: v4.4.2 ✅ (已发布)
> 项目状态: 508 Rust + 1220 前端测试全部通过 ✅ | CI 全部通过 ✅ | 无待处理 issue

---

## 当前进行中的工作

### DEBT-003 完成: CSS 变量化 & 性能优化
- ✅ 所有组件的硬编码 `gray-*` 颜色迁移到 CSS 变量
- ✅ 新增 `--color-primary-hover` CSS 变量，15 个按钮 hover 状态迁移
- ✅ Dashboard `getRecordTags` 改为 computed Map 缓存，避免重复 JSON.parse
- ✅ StatisticsPanel 图表函数转为 computed 属性
- ✅ 设置子组件双向 watch 循环优化 (isUpdatingFromProps 标志位)
- ✅ v-for 使用稳定 key 替代 index (customHeaders 用 _id，tags 用字符串值)

### 未来优化方向 (已识别，待排期)
（当前无待处理优化项）

---

## v4.3.5 ✅ 已发布

**目标**: Windows 启动修复 (issue #84)

**状态**: ✅ 已完成并发布 (v4.3.5)

**修复内容**：
- 使用 Tauri 异步运行时替代 tokio::spawn
- 解决 Windows 便携版 "there is no reactor running" 启动崩溃问题

---

## v4.3.4 ✅ 已发布

**目标**: Migration 幂等性修复 (issue #83)

**状态**: ✅ 已完成并发布

**修复内容**：
- 移除 batch SQL 中 settings 表的重复 `ALTER TABLE ADD COLUMN` 语句
- 解决旧版本数据库迁移时出现的 "duplicate column name: summary_model_name" 错误

---

## Epic 16: 用户体验优化 (UX-OPT) ✅

**目标**: 优化用户界面交互体验和性能

| Story | 名称 | 状态 |
|-------|------|------|
| UX-012 | HistoryViewer 虚拟滚动 | ✅ done (v4.2.0) |

---

## Epic 15: 稳定性与错误处理 (STAB-ERR) ✅

| Story | 名称 | 状态 |
|-------|------|------|
| STAB-ERR-001 | 改进 SettingsModal 错误处理 | ✅ done |
| STAB-ERR-002 | capture_service mutex 错误处理改进 | ✅ done |
| STAB-ERR-003 | 统一 Rust 错误类型 | ✅ done |

---

## Epic 11: 数据增强与稳定性 (STAB) ✅

| Story | 名称 | 状态 |
|-------|------|------|
| data-007 | 数据库架构统一与测试隔离 | ✅ done |
| data-008 | 数据库迁移系统完善 | ✅ done |
| stab-001 | 错误边界与优雅降级 | ✅ done |
| stab-002 | 自动备份与恢复 | ✅ done |

## Epic 12: 多维度输出与标签管理 (OUTPUT) ✅

| Story | 名称 | 状态 |
|-------|------|------|
| VAULT-001 | 多 Vault 自动选择 | ✅ done (v3.9.0) |
| TAG-001 | 标签颜色后端化 | ✅ done (v3.7.1) |
| EXPORT-001 | 自定义导出模板 | ✅ done (v3.8.0) |

## Epic 13: 技术债务清偿 (DEBT) ✅

| Story | 名称 | 状态 |
|-------|------|------|
| DEBT-001 | 统一测试数据库 schema | ✅ done (v4.0.0) |
| DEBT-002 | 数据库版本迁移机制 | ✅ done (v4.0.0) |
| DEBT-003 | 组件颜色 CSS 变量化 | ✅ done (v4.0.0) |

---

## Epic 14: 前端框架升级 (TAIL) ✅

| Story | 名称 | 状态 |
|-------|------|------|
| TAIL-001 | Tailwind v4 依赖升级与配置迁移 | ✅ done (v4.1.1) |
| TAIL-002 | 样式渲染验证与渐变类名更新 | ✅ done (v4.1.1) |
| TAIL-003 | 构建验证与测试通过 | ✅ done (v4.1.1) |

---

## Epic 17: 性能优化 (PERF) ✅

**目标**: 优化启动速度，减少冷启动时间

| Story | 名称 | 状态 |
|-------|------|------|
| PERF-007 | 启动速度优化 | ✅ done (v4.3.6) |

---

## v4.3.6 (已废弃 - 被 v4.3.7 替代)

**目标**: 启动速度优化 (PERF-007)

**状态**: ⚠️ 已废弃 - v4.3.6 从未正式发布，已被 v4.3.7 替代

---

## v4.3.7 ✅ 已发布

**目标**: 启动速度优化 (PERF-007) + 数据库迁移修复 (issue #85)

**状态**: ✅ 已完成并发布

**修复内容**：
- 实现懒加载机制 - `silent_tracker` 和 `work_time` 首次访问时才加载
- 缓冲诊断写入 - 减少启动时多次文件 I/O 操作
- 延迟 Tray 和 Backup Scheduler 初始化到窗口显示后
- 修复旧数据库 sessions 表缺失列问题 (`start_time`, `end_time`, `ai_summary`, `user_summary`, `context_for_next`, `status`)

**关联 issue**: #85

---

## Epic 18: 数据分析增强 (ANALYTICS) ✅

**目标**: 可视化统计、productivity 趋势分析

| Story | 名称 | 状态 |
|-------|------|------|
| ANALYTICS-001 | 生产力趋势与周期对比分析 | ✅ done |

---

## v4.4.2 ✅ 已发布

**目标**: 跨平台路径与除零修复

**状态**: ✅ 已完成并发布

**修复内容**：
- StatisticsPanel: 修复单数据点时除零问题 (`length === 0` → `length <= 1`)
- DailySummaryViewer: 修复跨平台路径分隔符处理 (使用 `lastIndexOf` 替代 `split/join`)

---

## v4.4.1 ✅ 已发布

**目标**: StatisticsPanel SVG viewBox 修复

**状态**: ✅ 已完成并发布

**修复内容**：
- 修复 SVG viewBox 使用 Mustache 模板语法导致图表无法正确缩放的问题
- 改用 Vue 动态绑定：`:viewBox="'0 0 ' + chartWidth + ' ' + chartHeight"`

---

## v4.4.0 ✅ 已发布

**目标**: Epic 18 数据分析增强 (ANALYTICS-001)

**状态**: ✅ 已完成并发布

**功能内容**：
- 新增 `get_productivity_trend` 后端命令
- StatisticsPanel 新增"趋势对比" Tab
- 支持"本周 vs 上周"、"本月 vs 上月"对比视图
- SVG 折线图展示日趋势数据
- 高峰时段分布显示

---

## 未来版本方向（待细化）

> 以下为初步方向，具体优先级需根据用户反馈和实际使用情况调整

| 方向 | 说明 | 优先级 |
|------|------|--------|
| Epic 18 (续): 数据分析增强 | 可视化统计、 productivity 分析 | P2 |
| Epic 19: 移动端支持 | iOS/Android 应用适配 | P3 |

---

## 最近 10 个已完成版本摘要

### v4.4.2 — 跨平台路径与除零修复 ✅
- StatisticsPanel: 修复单数据点时除零问题 (length check === 0 → <= 1)
- DailySummaryViewer: 修复跨平台路径分隔符处理 (支持 Windows `\` 和 Unix `/`)

### v4.4.1 — StatisticsPanel SVG viewBox 修复 ✅
- 修复 SVG viewBox Mustache 模板语法问题
- 改用 Vue 动态绑定确保图表正确缩放

### v4.4.0 — 数据分析增强 (Epic 18: ANALYTICS-001) ✅
- 新增生产力趋势命令 `get_productivity_trend` 支持周/月对比
- StatisticsPanel 新增"趋势对比" Tab 和 SVG 折线图
- 高峰时段分布显示 top 5  busiest hours
- 508 Rust + 1180 前端测试全部通过

### v4.3.7 — 启动速度优化 + 数据库迁移修复 ✅
- 实现懒加载机制 - `silent_tracker` 和 `work_time` 首次访问时才加载
- 缓冲诊断写入 - 减少启动时多次文件 I/O 操作
- 延迟 Tray 和 Backup Scheduler 初始化到窗口显示后 (PERF-007)
- 修复旧数据库 sessions 表缺失列问题 (issue #85)

### v4.3.5 — Windows 启动修复 ✅
- 使用 Tauri 异步运行时替代 tokio::spawn
- 解决 Windows 便携版 "there is no reactor running" 启动崩溃问题（issue #84）

### v4.3.4 — Migration 幂等性修复 ✅
- 移除 batch SQL 中 settings 表的重复 `ALTER TABLE ADD COLUMN` 语句
- 解决旧版本数据库迁移时出现的 "duplicate column name: summary_model_name" 错误（issue #83）

### v4.3.3 — 代码质量与文档同步 ✅
- Rust clippy 无警告通过，代码格式检查通过
- README 与代码功能同步（移除已删除的 GitHub integration 描述）
- 错误处理与日志链路审查完成，未发现高风险问题

### v4.3.2 — Migration Duplicate Column 修复 ✅
- 移除 batch SQL 中重复的 `ALTER TABLE records ADD COLUMN` 语句
- 将扩展列定义移至 `CREATE TABLE IF NOT EXISTS records` 避免二次添加
- 解决旧版本数据库迁移时出现的 "duplicate column name: monitor_info" 错误（issue #82）

### v4.3.1 — Migration 幂等性修复 ✅
- 扩展 v4.2.2 的 sessions.date 修复到 records 和 settings 表的所有扩展列
- 添加幂等的列添加辅助函数，优雅处理 "duplicate column name" 错误
- 解决旧版本数据库升级时的迁移失败问题（issue #81）

### v4.3.0 — AI Settings 模板导入/导出 ✅
- 完成 AISettings 模板导入/导出功能
- 使用 Tauri 文件对话框实现模板文件的保存和加载
- 新增 i18n 国际化支持（en.json, zh-CN.json）

### v4.2.x - v4.1.x — 维护版本集合 ✅
- v4.2.2: 修复 legacy sessions 表缺少 date 列导致的启动失败问题
- v4.2.1: TimelineWidget 防御性修复，添加 Array.isArray() 检查
- v4.2.0: HistoryViewer 虚拟滚动 (UX-012)，@tanstack/vue-virtual 实现
- v4.1.1: Timeline 时区转换错误处理修复 + Tailwind CSS v4 升级

---

