# DailyLogger 项目规划

> 最后更新: 2026-03-20
> 当前版本: v1.31.0 ✅ 已发布
> 下一版本: v1.32.0（维护版本）

---

## 下一迭代: v1.32.0（维护版本）📋 规划中

**目标**: 代码质量改进、文档完善、用户体验优化

**版本类型**: MINOR（维护优化）

**需求清单**:

| ID | 需求 | 故事点 | 优先级 | 状态 |
|----|------|--------|--------|------|
| MAINT-007 | 更新 README.md 版本历史（保留最新10个版本） | 1pt | MEDIUM | ✅ 完成 |
| MAINT-008 | 检查并清理未使用的代码/依赖 | 2pts | LOW | ✅ 完成（无需清理） |
| MAINT-009 | 改进错误提示和用户反馈 | 2pts | MEDIUM | ⏳ 待开始 |

**备注**:
- 所有 specs 目录中的功能规格已实现完成
- 无未关闭的 GitHub Issues
- CI 全部通过（531 frontend tests + 480 Rust tests）
- Tailwind CSS v4 可作为后续版本考虑（major 更新需谨慎）

---

## 后续版本规划

### v1.40.0 — 即时体验修复 📋 规划中

**目标**: 修复最影响日常使用的 UX 痛点，提升基础交互质量

**版本类型**: MINOR（UX 优化）

| ID | 需求 | 故事点 | 优先级 | 状态 | Spec |
|----|------|--------|--------|------|------|
| UX-001 | 改进错误提示和用户反馈（完成 MAINT-009） | 2pts | HIGH | ⏳ 待开始 | [specs/UX-001-error-feedback.md](specs/UX-001-error-feedback.md) |
| UX-002 | 报告生成按钮状态互锁 + 加载状态 | 2pts | HIGH | ⏳ 待开始 | [specs/UX-002-report-lock.md](specs/UX-002-report-lock.md) |
| UX-003 | 离线状态改为顶部横幅 Banner | 1pt | MEDIUM | ⏳ 待开始 | [specs/UX-003-offline-banner.md](specs/UX-003-offline-banner.md) |
| UX-004 | 截图分析完成显示简短应用摘要 | 2pts | MEDIUM | ⏳ 待开始 | [specs/UX-004-screenshot-summary.md](specs/UX-004-screenshot-summary.md) |
| UX-005 | 标签过滤条超出时折叠为 "+N 个标签" | 1pt | LOW | ⏳ 待开始 | [specs/UX-005-tag-filter-collapse.md](specs/UX-005-tag-filter-collapse.md) |

### v1.41.0 — 核心交互重构 📋 规划中

**目标**: 重构状态管理模式，消除模态对话框管理混乱，提升大数据量下的渲染性能

**版本类型**: MINOR（架构优化）

| ID | 需求 | 故事点 | 优先级 | 状态 | Spec |
|----|------|--------|--------|------|------|
| UX-010 | useModal composable 替代 21 个 showXxx ref | 3pts | HIGH | ⏳ 待开始 | [specs/UX-010-use-modal.md](specs/UX-010-use-modal.md) |
| UX-011 | 报告生成入口整合为单一下拉菜单 | 2pts | HIGH | ⏳ 待开始 | [specs/UX-011-report-dropdown.md](specs/UX-011-report-dropdown.md) |
| UX-012 | HistoryViewer 虚拟滚动 | 3pts | MEDIUM | ⏳ 待开始 | [specs/UX-012-history-virtual-scroll.md](specs/UX-012-history-virtual-scroll.md) |
| UX-013 | 快速记录键盘快捷键气泡提示 | 1pt | LOW | ⏳ 待开始 | — |

### v1.42.0 — 设置体验与标签系统 📋 规划中

**目标**: 重构设置界面结构，将标签颜色从硬编码改为后端可配置，提升大搜索结果渲染性能

**版本类型**: MINOR（功能增强）

| ID | 需求 | 故事点 | 优先级 | 状态 | Spec |
|----|------|--------|--------|------|------|
| UX-020 | SettingsModal 拆分为 4 个标签页子组件 | 3pts | HIGH | ⏳ 待开始 | [specs/UX-020-settings-split.md](specs/UX-020-settings-split.md) |
| UX-021 | 标签颜色移除硬编码，改为后端可配置 | 3pts | MEDIUM | ⏳ 待开始 | [specs/UX-021-tag-color-config.md](specs/UX-021-tag-color-config.md) |
| UX-022 | SearchPanel 虚拟滚动 | 2pts | MEDIUM | ⏳ 待开始 | — |
| UX-023 | ScreenshotGallery 懒加载缩略图 | 2pts | LOW | ⏳ 待开始 | — |

---

## 已完成版本

### v1.31.0（前端代码全量迁移至 TypeScript）✅ 已发布

**目标**: 将 `src/` 下所有 `.js` 文件迁移为 `.ts`，所有 Vue 组件添加 `lang="ts"`，建立完整的类型定义。

**版本类型**: MINOR（代码质量提升）

**迁移完成情况**:

| 类别 | 文件 | 状态 |
|------|------|------|
| 入口 | `src/main.js` → `src/main.ts` | ✅ 已迁移 |
| 入口 | `src/quick-note.js` → `src/quick-note.ts` | ✅ 已迁移 |
| 国际化 | `src/i18n.js` → `src/i18n.ts` | ✅ 已迁移 |
| 测试配置 | `src/setupTests.js` → `src/setupTests.ts` | ✅ 已迁移 |
| Store | `src/stores/toast.js` → `src/stores/toast.ts` | ✅ 已迁移 |
| Composable | `src/composables/usePlatform.js` → `src/composables/usePlatform.ts` | ✅ 已迁移 |
| 工具函数 | `src/utils/errors.js` → `src/utils/errors.ts` | ✅ 已迁移 |
| Vue 组件 | `src/App.vue`（添加 `lang="ts"`） | ✅ 已迁移 |
| Vue 组件 | `src/components/*.vue`（添加 `lang="ts"`，约 15 个） | ✅ 已迁移 |
| 测试文件 | `src/**/__tests__/*.{spec,test}.js`（约 25 个） | ✅ 已迁移 |

### v1.30.0（TypeScript 基础设施）✅ 已发布

**目标**: 为前端引入 TypeScript 基础设施，为后续全量代码迁移做准备。

**版本类型**: MINOR（技术基础设施升级）

| ID | 需求 | 故事点 | 优先级 | 状态 |
|----|------|--------|--------|------|
| TS-001 | 安装 TypeScript 及相关依赖（typescript、vue-tsc、@vue/tsconfig、@types/node） | 1pt | HIGH | ✅ 完成 |
| TS-002 | 创建 tsconfig.json（启用 strict 模式，allowJs: true 支持渐进迁移） | 1pt | HIGH | ✅ 完成 |
| TS-003 | 更新 vite.config.js 支持 TypeScript 路径别名和类型检查 | 1pt | HIGH | ✅ 完成 |
| TS-004 | 更新 vitest 配置，使测试文件支持 `.ts`/`.spec.ts` | 1pt | MEDIUM | ✅ 完成 |
| TS-005 | 在 CI workflow 中增加 `vue-tsc --noEmit` 类型检查步骤 | 1pt | HIGH | ✅ 完成 |
| TS-006 | 验证现有所有测试（531 frontend + 480 Rust）在新配置下仍通过 | 1pt | HIGH | ✅ 完成 |

### v1.29.0（依赖维护）✅ 已发布

**目标**: 更新前端依赖到最新版本，保持项目健康。

**版本类型**: MINOR（依赖维护）

| ID | 需求 | 故事点 | 优先级 | 状态 |
|----|------|--------|--------|------|
| MAINT-005 | 更新 vitest 到 v4 | 2pts | MEDIUM | ✅ 完成 |
| MAINT-006 | 更新 jsdom 到 v29 | 1pts | LOW | ✅ 完成 |

### v1.28.0（vue-i18n v11 迁移）✅ 已发布

**目标**: 将 vue-i18n 从 v9 迁移到 v11，保持依赖处于受支持状态。

| ID | 需求 | 故事点 | 优先级 | 状态 |
|----|------|--------|--------|------|
| MAINT-004 | 升级 vue-i18n 从 v9 到 v11 | 2pts | HIGH | ✅ 完成 |

### v1.27.0（依赖维护）✅ 已发布

### v1.0.0 ~ v1.9.0（Sprint 1 完成）

Sprint 1 完成了 5 大 Epic（87 故事点，24 个 Story），覆盖核心功能、智能捕获、AI 能力、数据管理和报告功能。

**已交付功能汇总**:

| Epic | 故事数 | 状态 |
|------|--------|------|
| 核心功能（自动捕获、手动记录） | 6 | ✅ |
| 智能捕获（静默检测、窗口过滤） | 4 | ✅ |
| AI 能力（日报、周报、月报生成） | 6 | ✅ |
| 数据管理（存储、搜索、导出） | 5 | ✅ |
| 报告功能（时间线、对比分析） | 3 | ✅ |