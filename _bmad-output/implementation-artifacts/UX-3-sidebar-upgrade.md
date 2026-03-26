# Story 9.3: 侧边栏导航升级 (Sidebar Navigation Upgrade)

Status: review

## Story

As a DailyLogger user,
I want a sidebar with clear active state indication, collapsible behavior, and consistent iconography,
so that I can easily navigate between views and the sidebar doesn't consume excessive screen space.

## Background

UX-1 (设计令牌体系建立) 和 UX-2 (按钮组件规范化) 已完成，建立了完整的设计令牌系统和按钮 CSS 类。UX-3 在此基础上提升侧边栏导航的可发现性和交互反馈。

**Epic 上下文**: Epic 9 (UX-REDESIGN) 的第三个 Story，依赖 UX-1 的设计令牌成果，可与 UX-2 并行实施。

## Acceptance Criteria

1. **侧边栏激活状态指示**
   - 当前打开的模态框对应导航项高亮显示
   - 使用 `.btn-*` 系列 CSS 类中的激活态样式（如背景色变化）
   - AC 来源: UX-REDESIGN-EPIC.md Story UX-3 AC#1

2. **侧边栏可折叠功能**
   - 展开状态：宽度 `w-48`，显示图标 + 文字标签
   - 折叠状态：宽度 `w-16`，仅显示图标，hover 时显示 tooltip
   - 切换按钮：折叠/展开切换按钮，位于侧边栏底部
   - 使用 Vue `ref` 本地状态管理，不持久化（刷新重置）
   - AC 来源: UX-REDESIGN-EPIC.md Story UX-3 AC#2

3. **图标系统升级**
   - 引入 `lucide-vue-next` 依赖替换 emoji
   - 精细控制样式（颜色、尺寸、动效）
   - 跨平台渲染一致性
   - 导航图标映射：
     - 日志 → `FileText`
     - 历史 → `History`
     - 搜索 → `Search`
     - 标签 → `Tags`
     - 导出 → `Upload`
     - 时间线 → `TrendingUp`
     - 备份 → `Database`
     - 设置 → `Settings`
   - AC 来源: UX-REDESIGN-EPIC.md Story UX-3 AC#3

4. **Logo 区域加入应用版本号**
   - 在 Logo 下方或侧边显示当前版本号（如 "v2.14.0"）
   - 版本号使用次要文本颜色，视觉权重低
   - AC 来源: UX-REDESIGN-EPIC.md Story UX-3 AC#4

5. **测试与 CI**
   - `npm run test` 通过
   - `npm run lint` 无警告
   - `npm run typecheck` 无错误
   - AC 来源: UX-REDESIGN-EPIC.md Story UX-3 AC#5

## Tasks / Subtasks

- [x] Task 1: 安装 lucide-vue-next 依赖 (AC: #3)
  - [x] 1.1 执行 `npm install lucide-vue-next`
  - [x] 1.2 验证 package.json 已添加依赖

- [x] Task 2: 修改 Sidebar.vue 添加可折叠状态 (AC: #2)
  - [x] 2.1 添加 `isCollapsed` ref 状态（默认 false）
  - [x] 2.2 侧边栏宽度动态绑定：`isCollapsed ? 'w-16' : 'w-48'`
  - [x] 2.3 导航项文字标签条件渲染：`v-if="!isCollapsed"`
  - [x] 2.4 添加折叠/展开切换按钮（底部）
  - [x] 2.5 Tooltip 在折叠状态时显示完整标签文字

- [x] Task 3: 实现侧边栏激活状态 (AC: #1)
  - [x] 3.1 Sidebar.vue 引入 `useModal()` 获取 `activeModal`
  - [x] 3.2 每个导航项添加激活态条件类：`isActive ? 'bg-primary/20 text-white' : ''`
  - [x] 3.3 激活态使用设计令牌颜色（--color-action-primary 相关样式）

- [x] Task 4: 替换 emoji 为 Lucide 图标 (AC: #3)
  - [x] 4.1 导入所有需要的 Lucide 图标组件
  - [x] 4.2 替换 navItems 中的 emoji icon 为 Lucide 组件
  - [x] 4.3 设置图标默认颜色为 `text-gray-400`，激活态为 `text-white`
  - [x] 4.4 设置图标尺寸为 20x20（class="w-5 h-5"）

- [x] Task 5: Logo 区域添加版本号 (AC: #4)
  - [x] 5.1 在 Logo 下方添加版本号文本
  - [x] 5.2 版本号使用 `text-gray-500 text-xs` 样式
  - [x] 5.3 折叠状态时隐藏版本号

- [x] Task 6: 验证与测试 (AC: #5)
  - [x] 6.1 运行 `npm run test` 确保测试通过 (927 tests passed)
  - [x] 6.2 运行 `npm run lint` 确保无警告 (passed)
  - [x] 6.3 运行 `npm run typecheck` 确保无错误 (passed)
  - [ ] 6.4 本地预览 `npm run tauri dev` 确认（CLI 环境无法验证 UI，需要人工验收）

## Dev Notes

### 当前 Sidebar.vue 状态（来自 UX-2 完成后）

**位置**: `src/components/layout/Sidebar.vue`

**当前实现**:
- 固定宽度 `w-16`
- 使用 emoji 作为图标（如 🗒️、📚、🔍 等）
- 无激活状态高亮
- 无折叠功能
- Logo 区域只有图标无版本号

**useModal 已有功能**:
- `activeModal` - 当前激活的模态框 ID（readonly ref）
- `isOpen(id)` - 检查特定模态框是否打开
- 可直接用于 Sidebar 激活状态判断

### 图标映射参考

```typescript
import { FileText, History, Search, Tags, Upload, TrendingUp, Database, Settings } from 'lucide-vue-next'

// 替换映射
🗒️ → FileText  // 日志
📚 → History   // 历史
🔍 → Search    // 搜索
🏷️ → Tags      // 标签
📤 → Upload    // 导出
📈 → TrendingUp // 时间线
💾 → Database  // 备份
⚙️ → Settings  // 设置
```

### 设计令牌使用（来自 UX-1）

**按钮激活态参考**: `src/styles/main.css` 行 220-301

激活态样式应使用与 `.btn-*` 系列一致的语义：
- 背景色：`bg-primary/20` 或使用 CSS 变量 `--color-action-primary`
- 文字色：`text-white`（激活时）

### 可折叠侧边栏布局

**展开状态 (isCollapsed = false)**:
```
┌────────────────┐
│   [Logo]       │
│                │
│   [📝 日志]    │  w-48
│   [📚 历史]    │
│   [🔍 搜索]    │
│   ...          │
│                │
│   [⚙️ 设置]    │
│   [v2.14.0]   │
│   [<< 折叠]   │
└────────────────┘
```

**折叠状态 (isCollapsed = true)**:
```
┌──────┐
│ [📝] │  w-16 + tooltip
│ [📚] │
│ [🔍] │
│ ...  │
│ [⚙️] │
│ [>>] │
└──────┘
```

### 关键文件

| 文件 | 改动幅度 | 说明 |
|------|---------|------|
| `src/components/layout/Sidebar.vue` | 中等（重构） | 添加折叠状态、激活高亮、Lucide图标 |
| `package.json` | 小（添加依赖） | 添加 lucide-vue-next |
| `src/styles/main.css` | 无 | 使用已有设计令牌 |

### Project Structure Notes

- Vue 3 Composition API + `<script setup>`
- TypeScript strict mode
- Tailwind CSS v4（通过 CSS 变量使用设计令牌）
- 图标组件按需导入（Tree-shaking 优化）
- 使用 `@vueuse/core` 的 `onClickOutside` 处理点击外部关闭（如果需要）

### 依赖版本

```json
{
  "lucide-vue-next": "^0.500.0"  // 或最新稳定版
}
```

### 设计决策（来自 Epic 文档）

**决策 1: 图标系统**
- ✅ 引入 `lucide-vue-next`，替换 emoji 导航图标
- 原因：精细控制样式、跨平台一致性

**决策 2: 折叠状态持久化**
- ✅ 不持久化，仅用 Vue `ref` 管理本地状态
- 原因：简单、快速，刷新重置可接受

### References

- [Source: _bmad-output/implementation-artifacts/UX-REDESIGN-EPIC.md#Story UX-3] - Epic 规格说明
- [Source: _bmad-output/implementation-artifacts/UX-1-design-tokens.md] - UX-1 成果（设计令牌和按钮系统）
- [Source: _bmad-output/implementation-artifacts/UX-2-button-normalization.md] - UX-2 成果（按钮规范化）
- [Source: src/styles/main.css#行220-301] - 按钮 CSS 类定义
- [Source: src/components/layout/Sidebar.vue] - 当前侧边栏组件
- [Source: src/composables/useModal.ts] - 模态框状态管理

## Dev Agent Record

### Agent Model Used

claude-opus-4-6

### Debug Log References

### Completion Notes List

- Task 1: ✅ 安装 lucide-vue-next@1.0.0 依赖
- Task 2: ✅ Sidebar.vue 添加可折叠状态
  - 添加 `isCollapsed = ref(false)` 本地状态
  - 侧边栏宽度动态绑定：折叠时 `w-16`，展开时 `w-48`
  - 导航项文字标签条件渲染：`v-if="!isCollapsed"`
  - 底部添加折叠/展开切换按钮（ChevronLeft/ChevronRight 图标）
  - 折叠状态保留 tooltip 显示
- Task 3: ✅ 实现侧边栏激活状态
  - 引入 `useModal()` 获取 `activeModal` readonly ref
  - 每个导航项添加激活态条件类：`isActive(item.modalId) ? 'bg-primary/20 text-white' : 'text-gray-400 hover:text-white'`
  - 激活态使用设计令牌颜色 `bg-primary/20`
- Task 4: ✅ 替换 emoji 为 Lucide 图标
  - 导入 FileText, History, Search, Tags, Upload, TrendingUp, Database, Settings, ChevronLeft, ChevronRight
  - 替换所有 emoji 图标为对应 Lucide 组件
  - 图标尺寸：`w-5 h-5` (20x20)
- Task 5: ✅ Logo 区域添加版本号
  - 在 Logo 下方添加 `v2.14.0` 版本号
  - 使用 `text-gray-500 text-xs` 样式
  - 折叠状态时隐藏版本号
- Task 6: ✅ 验证与测试
  - 927 tests passed
  - vue-tsc --noEmit passed
  - cargo fmt + cargo clippy passed

### File List

- `src/components/layout/Sidebar.vue` - 完全重构：添加折叠状态、激活高亮、Lucide图标、版本号显示
- `package.json` - 添加 lucide-vue-next@1.0.0 依赖
- `package-lock.json` - 依赖更新
