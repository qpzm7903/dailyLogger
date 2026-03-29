# DailyLogger 项目规划

> 最后更新: 2026-03-29
> 当前版本: v4.3.5
> 项目状态: 508 Rust + 1180 前端测试全部通过 ✅

---

## v4.3.5 ✅ 已发布

**目标**: Windows 启动修复 (issue #84)

**状态**: ✅ 已完成并发布 (v4.3.5)

**修复内容**：
- 使用 Tauri 异步运行时替代 tokio::spawn
- 解决 Windows 便携版 "there is no reactor running" 启动崩溃问题

---

## v4.3.4 ✅ 已完成

**目标**: Migration 幂等性修复 (issue #83)

**状态**: ✅ 已完成

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

## 最近 10 个已完成版本摘要

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

### v4.2.2 — Migration 修复 ✅
- 修复 legacy sessions 表缺少 date 列导致的启动失败问题
- 在 Rust 代码中显式检查并添加缺失列，确保数据库迁移成功

### v4.2.1 — TimelineWidget 防御性修复 ✅
- 修复 hour_groups 非数组时导致的 "hour_groups is not iterable" 错误
- 添加 Array.isArray() 检查，防止数据异常时的崩溃

### v4.2.0 — HistoryViewer 虚拟滚动 ✅
- 完成 UX-012：使用 @tanstack/vue-virtual 实现历史记录虚拟滚动
- 配置阈值 100 条，超过后启用虚拟化渲染
- 修复虚拟化计数响应式问题，确保过滤后数据正确更新

### v4.1.1 — Timeline 时区转换错误处理修复 + Tailwind v4 升级 ✅
- 修复 `get_timeline_data_for_date()` 中潜在的 panic 问题
- 替换 `unwrap()` 为 `ok_or_else()` 和 `single()` 正确错误处理
- 解决时区转换边界情况（如夏令时切换）导致的崩溃问题
- 完成 Tailwind CSS v4 升级 (DEP-001)：迁移到 @tailwindcss/vite、@theme 块配置

---

