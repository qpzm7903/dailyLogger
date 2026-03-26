# Story 9.2: 按钮组件规范化 (Button Component Normalization)

Status: ready-for-dev

## Story

As a DailyLogger user,
I want consolidated report action buttons with clear hierarchy,
so that the dashboard is less cluttered and I can easily find both frequent and advanced report operations.

## Background

UX-1 (设计令牌体系建立) 已完成，建立了完整的设计令牌系统和按钮 CSS 类。UX-2 在此基础上：
1. 将散落的 5 个报告操作按钮合并为统一的入口
2. 建立按钮使用规范文档
3. 确保 Dashboard.vue 所有按钮使用规范化 CSS 类

**Epic 上下文**: Epic 9 (UX-REDESIGN) 的第二个 Story，依赖 UX-1 的设计令牌成果。

## Acceptance Criteria

1. **按钮变体使用文档化**
   - 在 `src/styles/main.css` 中为每个按钮变体添加注释，说明使用场景
   - AC 来源: UX-REDESIGN-EPIC.md Story UX-2 AC#1

2. **报告操作按钮合并**
   - ReportDropdown 扩展支持更多操作选项
   - 保留直接可见按钮：日报（通过 ReportDropdown 主按钮）、周报（通过 ReportDropdown 下拉）、月报（通过 ReportDropdown 下拉）
   - 以下操作移入 ReportDropdown 下拉菜单：
     - 自定义报告 (`@open='customReport'`)
     - 对比分析 (`@open='comparisonReport'`)
     - 按日期重新分析 (`@open='reanalyzeByDate'`)
     - 重新分析今天 (`@reanalyzeToday'`)
   - 时段管理按钮保持独立（因为是导航操作而非报告生成）
   - AC 来源: UX-REDESIGN-EPIC.md Story UX-2 AC#2

3. **Dashboard.vue 按钮使用规范化**
   - 所有按钮使用 `.btn-*` CSS 类（已在 UX-1 中完成）
   - 验证按钮变体语义正确：
     - 主操作（生成日报）→ `.btn-primary`
     - 辅助操作（时段管理）→ `.btn-secondary`
     - 工具操作（截图、分析触发）→ `.btn-ghost`
     - 危险操作（停止捕获）→ `.btn-danger`
   - AC 来源: UX-REDESIGN-EPIC.md Story UX-2 AC#3

4. **测试与 CI**
   - `npm run test` 通过
   - `npm run lint` 无警告
   - AC 来源: UX-REDESIGN-EPIC.md Story UX-2 AC#4

## Tasks / Subtasks

- [ ] Task 1: 文档化按钮变体使用规范 (AC: #1)
  - [ ] 1.1 在 `src/styles/main.css` 的按钮系统注释区添加使用场景说明
  - [ ] 1.2 验证每个 `.btn-*` 类都有对应的使用文档

- [ ] Task 2: 扩展 ReportDropdown 组件 (AC: #2)
  - [ ] 2.1 修改 ReportDropdown.vue Props，添加 `additionalOptions` 数组 prop
  - [ ] 2.2 在下拉菜单中渲染额外选项（分隔线 + 高级操作）
  - [ ] 2.3 添加 `option.type` 区分 'report' (生成) 和 'action' (直接执行)
  - [ ] 2.4 验证日报/周报/月报仍然正常工作

- [ ] Task 3: 合并 Dashboard.vue 按钮 (AC: #2, #3)
  - [ ] 3.1 移除独立的自定义报告按钮，改为通过 ReportDropdown 的 additionalOptions 传入
  - [ ] 3.2 移除独立的对比分析按钮
  - [ ] 3.3 移除独立的按日期重新分析按钮
  - [ ] 3.4 移除独立的重新分析今天按钮
  - [ ] 3.5 保留时段管理按钮（因为是导航操作）
  - [ ] 3.6 更新 Dashboard.vue 中对应的 emit 定义，移除 customReport/comparisonReport/reanalyzeByDate/reanalyzeToday 事件（这些改由 ReportDropdown 内部处理）

- [ ] Task 4: 验证与测试 (AC: #4)
  - [ ] 4.1 运行 `npm run test` 确保测试通过
  - [ ] 4.2 运行 `npm run lint` 确保无警告
  - [ ] 4.3 本地预览 `npm run tauri dev` 确认：
    - ReportDropdown 包含所有 7 个选项（日报/周报/月报 + 4 个高级操作）
    - 按钮样式与 UX-1 保持一致
    - 点击各选项能正确触发对应功能

## Dev Notes

### 当前设计令牌和按钮系统（来自 UX-1）

**设计令牌位置**: `src/styles/main.css` 行 3-25

**按钮 CSS 类**: `src/styles/main.css` 行 220-301

```css
/* 按钮变体 */
.btn-primary   /* 主操作：蓝色 (#3b82f6) */
.btn-secondary /* 辅助操作：深灰 (#475569) */
.btn-ghost     /* 工具操作：透明背景 hover 显示灰色 */
.btn-danger    /* 危险操作：红色 (#ef4444) */
.btn-success   /* 成功操作：绿色 (#22c55e) */

/* 按钮尺寸 */
.btn-sm  /* 小型：px-3 py-1.5 text-xs */
.btn-md  /* 中型：px-4 py-2 text-sm - 默认 */
.btn-lg  /* 大型：px-6 py-3 text-base */
```

### ReportDropdown.vue 当前接口

**位置**: `src/components/ReportDropdown.vue`

**当前 Props**:
```typescript
interface Props {
  isGeneratingDaily?: boolean
  isGeneratingWeekly?: boolean
  isGeneratingMonthly?: boolean
}
```

**当前 Emits**:
```typescript
interface Emits {
  (e: 'generate', type: 'daily' | 'weekly' | 'monthly'): void
}
```

**当前 Options**:
```typescript
const reportOptions = [
  { id: 'daily' as const, label: '生成日报', shortcut: '今日工作总结' },
  { id: 'weekly' as const, label: '生成周报', shortcut: '本周工作汇总' },
  { id: 'monthly' as const, label: '生成月报', shortcut: '本月工作汇总' },
]
```

### 推荐的 ReportDropdown 扩展方案

**新增 Props**:
```typescript
interface AdditionalOption {
  id: string
  label: string
  shortcut?: string
  type: 'report' | 'action'  // report=emit generate, action=emit对应事件
  icon?: string
}

interface Props {
  // ... 现有 props
  additionalOptions?: AdditionalOption[]
}
```

**新增 Emits**:
```typescript
// 扩展 Emits 支持更多操作类型
(e: 'openModal', modalId: ModalId): void  // 打开模态框
(e: 'customAction', actionId: string): void  // 自定义操作
```

### Dashboard.vue 当前按钮布局（行 97-133）

```vue
<div class="flex items-center gap-2">
  <ReportDropdown ... />
  <button @click="$emit('open', 'customReport')" class="btn btn-secondary btn-sm">自定义报告</button>
  <button @click="$emit('open', 'comparisonReport')" class="btn btn-secondary btn-sm">对比分析</button>
  <button @click="$emit('open', 'reanalyzeByDate')" class="btn btn-secondary btn-sm">按日期重新分析</button>
  <button @click="$emit('open', 'sessionList')" class="btn btn-secondary btn-sm">时段管理</button>  <!-- 保留 -->
  <button @click="$emit('reanalyzeToday')" class="btn btn-secondary btn-sm">重新分析今天</button>
</div>
```

**合并后目标布局**:
```vue
<div class="flex items-center gap-2">
  <ReportDropdown
    :additionalOptions="[
      { id: 'customReport', label: '自定义报告', type: 'action', icon: '📄' },
      { id: 'comparisonReport', label: '对比分析', type: 'action', icon: '📊' },
      { id: 'reanalyzeByDate', label: '按日期重新分析', type: 'action', icon: '📅' },
      { id: 'reanalyzeToday', label: '重新分析今天', type: 'action', icon: '🔄' }
    ]"
    ...
  />
  <button @click="$emit('open', 'sessionList')" class="btn btn-secondary btn-sm">时段管理</button>
</div>
```

### 关键文件

| 文件 | 改动幅度 | 说明 |
|------|---------|------|
| `src/styles/main.css` | 小（添加注释） | 文档化按钮使用规范 |
| `src/components/ReportDropdown.vue` | 中（扩展接口） | 支持 additionalOptions |
| `src/components/layout/Dashboard.vue` | 小（简化按钮） | 合并报告操作到 ReportDropdown |

### Project Structure Notes

- Vue 3 Composition API + `<script setup>`
- TypeScript strict mode
- Tailwind CSS v4（通过 CSS 变量使用设计令牌）
- 按钮类使用原生 CSS（不是 Tailwind @apply），位于 `main.css` 行 220-301
- 使用 `@vueuse/core` 的 `onClickOutside` 处理点击外部关闭

### 设计决策（来自 Epic 文档）

**报告按钮统一策略**:
- 日报/周报/月报 → 通过 ReportDropdown（保留高频可见）
- 自定义报告、对比分析、按日期重新分析、重新分析今天 → 移入 ReportDropdown 下拉菜单
- 时段管理 → 保留独立按钮（因为是导航操作而非报告生成）

**按钮语义映射**:
- 主操作（生成日报）→ `.btn-primary`
- 辅助操作（时段管理）→ `.btn-secondary`
- 工具操作（截图、分析触发）→ `.btn-ghost`

### References

- [Source: _bmad-output/implementation-artifacts/UX-REDESIGN-EPIC.md#Story UX-2] - Epic 规格说明
- [Source: _bmad-output/implementation-artifacts/UX-1-design-tokens.md] - UX-1 成果（设计令牌和按钮系统）
- [Source: src/styles/main.css#行220-301] - 按钮 CSS 类定义
- [Source: src/components/ReportDropdown.vue] - 现有 ReportDropdown 组件
- [Source: src/components/layout/Dashboard.vue#行97-133] - 当前按钮布局

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List
- `src/styles/main.css` - 添加按钮使用文档注释
- `src/components/ReportDropdown.vue` - 扩展支持 additionalOptions
- `src/components/layout/Dashboard.vue` - 合并报告按钮到 ReportDropdown
