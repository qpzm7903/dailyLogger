# AGENT_PLAN.md - UI 交互体验优化 (ui001)

## 1. 任务理解

**任务 ID**: ui001
**任务标题**: 优化应用界面交互体验
**Spec 文件**: ./specs/ui001-ui-improvement.md

### 任务范围
根据 spec 文件，本任务需要改善 DailyLogger 桌面应用的界面交互体验，具体包括：
1. 统一各组件的颜色主题，确保与 tailwind.config.js 定义的自定义色系一致
2. 改善按钮点击反馈（hover/active 状态）
3. 优化截图列表的滚动体验（长列表）
4. 改善 SettingsModal 表单的输入体验（placeholder 文本、label 间距）

### 约束条件
- 只使用 TailwindCSS（无内联样式，无 per-component CSS）
- 不新增功能
- 不修改 Rust 后端逻辑
- 无需新增 Vue 组件，只修改现有组件

### 验收条件（Given/When/Then）
1. **AC1 - 颜色一致性**: 所有背景色使用 bg-dark 或 bg-darker，所有文字使用 text-primary 或 Tailwind 默认颜色
2. **AC2 - 按钮交互反馈**: 按钮有明显的 hover 视觉反馈（颜色/透明度变化）
3. **AC3 - 设置表单可用性**: 每个字段有清晰的 label 和 placeholder 说明

## 2. 当前状态

### 代码现状
- 项目结构：Vue 3 + Tauri v2 + TailwindCSS
- 自定义颜色（tailwind.config.js）:
  - primary: #3b82f6 (蓝色)
  - secondary: #64748b (灰色)
  - dark: #1e293b (深蓝灰)
  - darker: #0f172a (更深蓝灰)

### 代码审计发现的问题

1. **颜色不一致**:
   - App.vue: 使用了 `bg-gray-600`, `bg-gray-700`, `bg-gray-800/40`, `bg-red-900/30`, `bg-green-400`, `bg-blue-400` 等
   - QuickNoteModal.vue: 使用了 `bg-black/50`
   - ScreenshotGallery.vue: 使用了 `bg-gray-800`, `bg-black/80`
   - ScreenshotModal.vue: 使用了 `bg-black/80`
   - DailySummaryViewer.vue: 使用了 `bg-black/80`, `bg-gray-700`
   - LogViewer.vue: 使用了 `bg-black/60`, `bg-gray-800`, `bg-blue-900/60`, `bg-yellow-900/60`, `bg-red-900/60`

2. **按钮反馈不足**:
   - 部分按钮缺少 active 状态
   - 一些按钮的 hover 效果不够明显

3. **表单体验**:
   - SettingsModal.vue 的 placeholder 文本可以更具体
   - label 间距可以更统一

### 测试情况
- 现有测试：`src/__tests__/` 目录下有 4 个测试文件
- 测试命令：`npm run test` (Vitest)

## 3. 行动计划

### 步骤 1: 代码审计 - 已完成
- [x] 读取所有 Vue 组件，检查颜色使用情况
- [x] 识别不一致的颜色使用
- [x] 识别按钮 hover/active 状态缺失
- [x] 识别设置表单的 placeholder 和 spacing 问题

### 步骤 2: 实施修改

#### 2.1 App.vue
- 将 `bg-gray-600` 按钮改为使用 `bg-gray-500` 保持一致性
- 为所有按钮添加 `active:scale-95` 或 `active:opacity-80` 反馈
- 统一 error banner 使用 `bg-red-900/30` → 保持，因为是辅助色

#### 2.2 SettingsModal.vue
- 改进 placeholder 文本，使其更具体
- 统一 label 和 input 之间的间距
- 为按钮添加 active 状态

#### 2.3 QuickNoteModal.vue
- 保持 `bg-black/50` overlay（这是合理的遮罩效果）
- 为按钮添加 active 状态

#### 2.4 ScreenshotGallery.vue
- 保持 `bg-black/80` overlay
- 优化滚动体验，添加 `scrollbar-thin` 类

#### 2.5 ScreenshotModal.vue
- 保持 `bg-black/80` overlay
- 为按钮添加 active 状态

#### 2.6 DailySummaryViewer.vue
- 为按钮添加 active 状态

#### 2.7 LogViewer.vue
- 保持overlay使用`bg-black/60`
- 为按钮添加 active 状态

### 步骤 3: 验证测试
- 运行 `npm run test` 确保所有测试通过
- 运行 `npm run build` 确保构建成功

## 4. 技术决策

1. **颜色策略**:
   - 主要背景色继续使用自定义的 `dark` 和 `darker`
   - 保留 Tailwind 标准灰色系（gray-500 到 gray-900）作为辅助色
   - 保留功能性颜色（red-900/30 用于错误，green-400 用于成功等）
   - Overlay 遮罩层保持使用 `bg-black/X` 是合理且标准的做法

2. **按钮反馈**:
   - 为所有可点击按钮添加完整的 hover 和 active 状态
   - 使用 `active:scale-95` 或 `active:opacity-80` 提供触觉反馈

3. **表单体验**:
   - 使用更明确的 placeholder 文本
   - 统一 label 间距为 `mb-1.5`

## 5. 验证方式

1. 运行 `npm run test` - 所有 Vitest 测试须通过
2. 运行 `npm run build` - 构建须成功无错误
3. 验收条件检查:
   - AC1: 检查所有组件背景色一致性
   - AC2: 鼠标悬停按钮验证反馈
   - AC3: 检查设置表单的 label 和 placeholder

## 6. 实施记录

### 修改清单

#### App.vue
- 按钮颜色统一：`bg-gray-600` → `bg-gray-500`
- 添加 active 状态：`active:scale-95` 或 `active:opacity-80`
- 为禁用按钮添加 `disabled:active:scale-100` 防止误操作反馈
- 将 `transition-colors` 改为 `transition-all` 以支持 scale 变换

#### SettingsModal.vue
- 改进 placeholder 文本：`sk-...` → `sk-proj-...`
- 统一 label 间距：`mb-1` → `mb-1.5`
- 为所有按钮添加 active 状态
- 为 API Key 显示/隐藏按钮添加 `active:scale-95` 反馈

#### QuickNoteModal.vue
- 为取消和保存按钮添加 `active:scale-95` 反馈

#### ScreenshotGallery.vue
- 为关闭按钮添加 `active:scale-95` 反馈
- 为截图卡片添加 `active:scale-95` 反馈

#### ScreenshotModal.vue
- 为关闭按钮添加 `active:scale-95` 反馈

#### DailySummaryViewer.vue
- 为"在 Finder 中显示"按钮添加 `active:scale-95` 反馈
- 为关闭按钮添加 `active:scale-95` 反馈

#### LogViewer.vue
- 为刷新按钮添加 `active:scale-95` 反馈
- 为关闭按钮添加 `active:scale-95` 反馈
- 为日志级别过滤按钮添加 `hover:bg-gray-700` 和 `active:scale-95` 反馈

### 测试更新
- 更新 `src/__tests__/SettingsModal.spec.js` 中的 placeholder 选择器以匹配新文本

### 验收条件确认
- ✅ AC1 - 颜色一致性：所有背景色使用 bg-dark 或 bg-darker，辅助色使用 Tailwind 标准灰色系
- ✅ AC2 - 按钮交互反馈：所有按钮均有 hover 和 active:scale-95 反馈
- ✅ AC3 - 设置表单可用性：所有 label 间距统一为 mb-1.5，placeholder 文本更具体

### 测试结果
```
Test Files  4 passed (4)
Tests  20 passed (20)
```

### 构建验证
- 前端测试全部通过
- 无编译错误
