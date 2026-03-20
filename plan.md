# DailyLogger 项目规划

> 最后更新: 2026-03-20
> 当前版本: v1.45.0 ✅ 已发布
> 上个发布版本: v1.44.0 ✅ 已发布

---

## 后续版本规划

### v1.46.0 待定

**状态**: 等待需求输入

**潜在需求**:
- 用户反馈收集后的新需求
- 进一步优化 SettingsModal.vue（将模态框逻辑提取为独立组件）
- 性能优化或用户体验改进

**需求来源检查 (2026-03-20)**:
- ✅ GitHub Issues: 无未关闭问题
- ✅ GitHub Actions: 最近 workflow 全部通过
- ✅ 代码质量: clippy 无警告
- ⏳ 等待用户反馈或新功能需求

**需求来源检查 (2026-03-20 晚)**:
- ✅ GitHub Issues: 无未关闭问题
- ✅ GitHub Actions: 最新 5 个 workflow 全部通过
- ✅ 代码质量: `cargo clippy -- -D warnings` 无警告
- ⏳ 等待用户反馈或新功能需求

**需求来源检查 (2026-03-20 Session 21:22)**:
- ✅ GitHub Issues: 无未关闭问题
- ✅ GitHub Actions: 最新 5 个 workflow 全部通过（Build and Release + Test）
- ✅ 代码质量: `cargo clippy -- -D warnings` 无警告
- ✅ 前端测试: 583 个测试全部通过
- ⏳ 等待用户反馈或新功能需求

**需求来源检查 (2026-03-20 Session 当前)**:
- ✅ GitHub Issues: 无未关闭问题
- ✅ GitHub Actions: 最新 workflow 全部通过
- ✅ 代码质量: `cargo clippy -- -D warnings` 无警告
- ✅ 安全审计: npm audit 显示 0 个漏洞
- ✅ 依赖状态: Tailwind CSS v4 可用（3.4.19 → 4.2.2），需谨慎评估
- ⏳ 等待用户反馈或新功能需求

**需求来源检查 (2026-03-20 Session 最新)**:
- ✅ GitHub Issues: 无未关闭问题
- ✅ GitHub Actions: 最新 5 个 workflow 全部通过
- ✅ 代码质量: `cargo clippy -- -D warnings` 无警告
- ✅ 前端测试: 583 个测试全部通过
- ✅ 安全审计: npm audit 显示 0 个漏洞
- ⏳ 等待用户反馈或新功能需求

**需求来源检查 (2026-03-20 20:03)**:
- ✅ GitHub Issues: 无未关闭问题
- ✅ GitHub Actions: 最新 workflow 全部通过
- ✅ 代码质量: `cargo clippy -- -D warnings` 无警告
- ✅ 前端测试: 583 个测试全部通过
- ✅ 安全审计: npm audit 显示 0 个漏洞
- ⏳ 等待用户反馈或新功能需求

---

**潜在改进点（待用户确认）**:
- Tailwind CSS v4 大版本更新（需谨慎评估，当前 3.4.19 → 4.2.2）

---

## v1.46.0 需求决策

当前无明确需求输入，需确认以下事项：
1. 是否执行 Tailwind CSS v4 升级？（major 更新，可能有破坏性变更）
2. 是否有新的功能需求或改进点？
3. 是否需要进行其他维护工作？

---

## v1.45.0（组件重构）✅ 已发布

**目标**: 拆分大型组件 SettingsModal.vue，提升代码可维护性

**版本类型**: MINOR（代码质量优化）

| ID | 需求 | 故事点 | 优先级 | 状态 | Spec |
|----|------|--------|--------|------|------|
| MAINT-011 | SettingsModal.vue 拆分为独立子组件（当前 2750 行） | 5pts | LOW | ✅ 完成 | [specs/MAINT-011-settings-modal-split.md](specs/MAINT-011-settings-modal-split.md) |

**需求详情**:
- **MAINT-011** ✅: 将 2750 行的 SettingsModal.vue 拆分为独立子组件
  - ✅ `src/components/settings/shared/types.ts` — 共享类型定义和工具函数
  - ✅ `src/components/settings/BasicSettings.vue` (476 行) — API 配置、模型选择、Ollama 管理、语言切换
  - ✅ `src/components/settings/AISettings.vue` (329 行) — 分析模型配置、报告模型配置、Prompt 配置
  - ✅ `src/components/settings/CaptureSettings.vue` (353 行) — 截图间隔、静默检测、窗口过滤、多显示器
  - ✅ `src/components/settings/OutputSettings.vue` (413 行) — Obsidian、Logseq、Notion、GitHub、Slack 配置
  - ✅ `src/components/SettingsModal.vue` (691 行) — 容器组件，集成子组件

**验收结果**:
- ✅ AC1: 所有 583 个前端测试 + 435 个 Rust 测试通过
- ✅ AC2: 每个子组件不超过 500 行
- ✅ AC3: 功能完全保留
- ✅ AC4: 无 TypeScript 错误
- ⚠️ AC5: SettingsModal.vue 精简至 691 行（目标 300 行，因保留模态框逻辑略超）

---

**目标**: 移除硬编码标签颜色，优化大型组件结构，提升代码可维护性

**版本类型**: MINOR（代码质量优化）

| ID | 需求 | 故事点 | 优先级 | 状态 | Spec |
|----|------|--------|--------|------|------|
| MAINT-010 | 移除 App.vue 中硬编码的 tagColors，统一使用手动标签系统 | 2pts | MEDIUM | ✅ 完成 | — |
| MAINT-011 | SettingsModal.vue 拆分为独立子组件（当前 2750 行） | 5pts | LOW | ⏳ 延后 | — |
| UX-022 | SearchPanel 虚拟滚动 | 2pts | LOW | ✅ 完成 | — |
| UX-023 | ScreenshotGallery 懒加载缩略图 | 2pts | LOW | ✅ 完成 | — |

**需求详情**:
- **MAINT-010** ✅: 已创建统一的标签颜色系统 (src/utils/tagColors.ts)，App.vue 和 TagCloud.vue 已迁移
- **MAINT-011** ⏳: 大型重构任务（5pts），延后至后续版本
- **UX-022** ✅: SearchPanel 已添加虚拟滚动，搜索结果上限提升至 200 条
- **UX-023** ✅: ScreenshotGallery 实现缩略图懒加载，仅加载可见页的缩略图

---

## v1.43.0（功能增强）✅ 已发布

**目标**: 增强截图分析功能，移除不需要的登录注册模块

**版本类型**: MINOR（功能增强）

| ID | 需求 | 故事点 | 优先级 | 状态 | Spec |
|----|------|--------|--------|------|------|
| FEAT-001 | 支持重新分析已分析的截图记录 (#53) | 3pts | HIGH | ✅ 完成 | [specs/FEAT-001-reanalyze-screenshot.md](specs/FEAT-001-reanalyze-screenshot.md) |
| FEAT-002 | 移除登录/注册功能 (#55) | 3pts | MEDIUM | ✅ 完成 | [specs/FEAT-002-remove-auth.md](specs/FEAT-002-remove-auth.md) |

**需求详情**:
- **FEAT-001 (#53)**: 用户希望能够重新分析已有的截图记录（如更换模型后）。已在 ScreenshotModal 中添加"重新分析"按钮。
- **FEAT-002 (#55)**: 移除了登录/注册功能及团队协作模块，简化应用架构。

---

## v1.42.0（问题修复 + 功能增强）✅ 已发布

## v1.41.0（核心交互重构 + 问题修复）✅ 已发布

**目标**: 重构状态管理模式，消除模态对话框管理混乱，提升大数据量下的渲染性能，修复关键问题

**版本类型**: MINOR（架构优化 + 问题修复）

| ID | 需求 | 故事点 | 优先级 | 状态 | Spec |
|----|------|--------|--------|------|------|
| UX-010 | useModal composable 替代 21 个 showXxx ref | 3pts | HIGH | ✅ 完成 | [specs/UX-010-use-modal.md](specs/UX-010-use-modal.md) |
| UX-011 | 报告生成入口整合为单一下拉菜单 | 2pts | HIGH | ✅ 完成 | [specs/UX-011-report-dropdown.md](specs/UX-011-report-dropdown.md) |
| UX-012 | HistoryViewer 虚拟滚动 | 3pts | MEDIUM | ✅ 完成 | [specs/UX-012-history-virtual-scroll.md](specs/UX-012-history-virtual-scroll.md) |
| UX-013 | 快速记录键盘快捷键气泡提示 | 1pt | LOW | ⏳ 延后 | — |
| FIX-004 | Windows portable 版本离线状态问题 (#49) | 3pts | HIGH | ✅ 完成 | — |
| FIX-005 | 设置浮窗点击外部关闭导致内容丢失 (#51) | 2pts | HIGH | ✅ 完成 | — |

---

## v1.40.0（即时体验修复）✅ 已发布

**目标**: 修复最影响日常使用的 UX 痛点，提升基础交互质量

**版本类型**: MINOR（UX 优化）

| ID | 需求 | 故事点 | 优先级 | 状态 | Spec |
|----|------|--------|--------|------|------|
| UX-001 | 改进错误提示和用户反馈（Toast 时长/错误处理修复） | 2pts | HIGH | ✅ 完成 | [specs/UX-001-error-feedback.md](specs/UX-001-error-feedback.md) |
| UX-002 | 报告生成按钮状态互锁 + 加载状态 | 2pts | HIGH | ✅ 完成 | [specs/UX-002-report-lock.md](specs/UX-002-report-lock.md) |
| UX-003 | 离线状态改为顶部横幅 Banner | 1pt | MEDIUM | ✅ 完成 | [specs/UX-003-offline-banner.md](specs/UX-003-offline-banner.md) |
| UX-004 | 截图分析完成显示简短应用摘要 | 2pts | MEDIUM | ✅ 完成 | [specs/UX-004-screenshot-summary.md](specs/UX-004-screenshot-summary.md) |
| UX-005 | 标签过滤条超出时折叠为 "+N 个标签" | 1pt | LOW | ✅ 完成 | [specs/UX-005-tag-filter-collapse.md](specs/UX-005-tag-filter-collapse.md) |

---

## v1.32.0（维护版本）✅ 已发布

**目标**: 代码质量改进、文档完善、用户体验优化

**版本类型**: MINOR（维护优化）

**需求清单**:

| ID | 需求 | 故事点 | 优先级 | 状态 |
|----|------|--------|--------|------|
| MAINT-007 | 更新 README.md 版本历史（保留最新10个版本） | 1pt | MEDIUM | ✅ 完成 |
| MAINT-008 | 检查并清理未使用的代码/依赖 | 2pts | LOW | ✅ 完成（无需清理） |
| MAINT-009 | 改进错误提示和用户反馈 | 2pts | MEDIUM | ✅ 完成 |

**备注**:
- 所有 specs 目录中的功能规格已实现完成
- 无未关闭的 GitHub Issues
- CI 全部通过（531 frontend tests + 480 Rust tests）
- Tailwind CSS v4 可作为后续版本考虑（major 更新需谨慎）

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