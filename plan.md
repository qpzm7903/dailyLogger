# DailyLogger 项目规划

> 最后更新: 2026-03-20
> 当前版本: v1.29.0 ✅ 已发布
> 下一版本: v1.30.0（TypeScript 基础设施）

---

## 下一迭代: v1.30.0（TypeScript 基础设施）🚧 待开发

**目标**: 为前端引入 TypeScript 基础设施，为后续全量代码迁移做准备。

**版本类型**: MINOR（技术基础设施升级）

**背景**: 项目当前前端代码（`src/`）全部为 JavaScript（`.js`），缺乏类型安全保障，随着功能复杂度增长，维护风险上升。本版本完成基础设施层面的 TS 接入，下一版本（v1.31.0）完成代码层面的全量迁移。

| ID | 需求 | 故事点 | 优先级 | 状态 |
|----|------|--------|--------|------|
| TS-001 | 安装 TypeScript 及相关依赖（typescript、vue-tsc、@vue/tsconfig、@types/node） | 1pt | HIGH | ⬜ 待开发 |
| TS-002 | 创建 tsconfig.json（启用 strict 模式，allowJs: true 支持渐进迁移） | 1pt | HIGH | ⬜ 待开发 |
| TS-003 | 更新 vite.config.js 支持 TypeScript 路径别名和类型检查 | 1pt | HIGH | ⬜ 待开发 |
| TS-004 | 更新 vitest 配置，使测试文件支持 `.ts`/`.spec.ts` | 1pt | MEDIUM | ⬜ 待开发 |
| TS-005 | 在 CI workflow 中增加 `vue-tsc --noEmit` 类型检查步骤 | 1pt | HIGH | ⬜ 待开发 |
| TS-006 | 验证现有所有测试（531 frontend + 480 Rust）在新配置下仍通过 | 1pt | HIGH | ⬜ 待开发 |

---

## 下下迭代: v1.31.0（前端代码全量迁移至 TypeScript）📋 规划中

**目标**: 将 `src/` 下所有 `.js` 文件迁移为 `.ts`，所有 Vue 组件添加 `lang="ts"`，建立完整的类型定义。

**版本类型**: MINOR（代码质量提升）

**迁移范围**（当前 JS 文件清单）:

| 类别 | 文件 | 状态 |
|------|------|------|
| 入口 | `src/main.js` → `src/main.ts` | ⬜ 待迁移 |
| 入口 | `src/quick-note.js` → `src/quick-note.ts` | ⬜ 待迁移 |
| 国际化 | `src/i18n.js` → `src/i18n.ts` | ⬜ 待迁移 |
| 测试配置 | `src/setupTests.js` → `src/setupTests.ts` | ⬜ 待迁移 |
| Store | `src/stores/toast.js` → `src/stores/toast.ts` | ⬜ 待迁移 |
| Composable | `src/composables/usePlatform.js` → `src/composables/usePlatform.ts` | ⬜ 待迁移 |
| 工具函数 | `src/utils/errors.js` → `src/utils/errors.ts` | ⬜ 待迁移 |
| Vue 组件 | `src/App.vue`（添加 `lang="ts"`） | ⬜ 待迁移 |
| Vue 组件 | `src/components/*.vue`（添加 `lang="ts"`，约 15 个） | ⬜ 待迁移 |
| 测试文件 | `src/**/__tests__/*.{spec,test}.js`（约 25 个） | ⬜ 待迁移 |

**迁移原则**:
- 迁移时为所有函数参数、返回值、组件 Props/Emits 补充类型标注
- 使用 `interface` 定义 Tauri IPC 命令的入参和返回类型（集中放 `src/types/tauri.ts`）
- 不允许使用 `any`（除极少数无法避免的第三方库边界情况，必须加 `// eslint-disable-next-line @typescript-eslint/no-explicit-any` 注释）
- 每迁移一个文件须保证对应测试仍通过

---

## 已完成版本

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