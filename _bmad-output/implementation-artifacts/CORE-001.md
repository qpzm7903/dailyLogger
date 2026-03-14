# Story 1.1: 设置界面优化

Status: review

## Story

作为一个 DailyLogger 用户，
我希望设置界面有清晰直观的交互体验，
以便我能够快速配置 API 和捕获参数，减少配置错误。

## Acceptance Criteria

### AC1 - 颜色一致性
- Given 应用启动后打开设置界面
- When 用户查看设置表单
- Then 所有背景色使用 `bg-dark` 或 `bg-darker`，所有文字使用 `text-primary` 或 Tailwind 标准颜色

### AC2 - 按钮交互反馈
- Given 用户将鼠标悬停在按钮上
- When hover 事件发生
- Then 按钮有明显的视觉反馈（颜色/透明度变化）

### AC3 - 设置表单可用性
- Given 用户打开设置界面
- When 查看各输入字段
- Then 每个字段有清晰的 label 和 placeholder 说明，输入聚焦时有边框高亮

### AC4 - API Key 显示/隐藏切换
- Given 用户正在配置 API Key
- When 点击"显示/隐藏"按钮
- Then API Key 输入框在明文/密文之间切换，并提供视觉反馈

### AC5 - 保存状态反馈
- Given 用户点击保存按钮
- When 保存操作完成
- Then 显示成功（绿色）或失败（红色）的明确提示

## Tasks / Subtasks

- [x] Task 1: 优化 SettingsModal 表单布局和视觉层次 (AC: 1, 3)
  - [x] 统一所有 label 使用 `text-gray-300`，字体大小 `text-xs`
  - [x] 统一所有 placeholder 使用 `text-gray-500`（通过 placeholder:text-gray-500 和 text-gray-100 实现）
  - [x] 优化 input 字段间距，确保组与组之间有清晰分隔（space-y-3）
- [x] Task 2: 改进按钮 hover/active 状态 (AC: 2)
  - [x] 为所有按钮添加 hover 效果（hover:bg-gray-700, hover:text-white, hover:bg-blue-600）
  - [x] 为关闭按钮添加 hover:text-white 反馈
  - [x] 确保保存按钮有 disabled:opacity-50 状态
- [x] Task 3: API Key 显示/隐藏功能优化 (AC: 4)
  - [x] 确保显示/隐藏按钮有清晰的文字提示（"显示"/"隐藏"）
  - [x] 添加按钮 hover 效果（hover:text-gray-300, hover:bg-gray-700）
  - [x] 确保切换时状态即时生效
- [x] Task 4: 保存状态提示优化 (AC: 5)
  - [x] 保存成功显示绿色勾号图标和"已保存"文本
  - [x] 保存失败显示红色警告图标和错误信息
  - [x] 成功后 0.8 秒自动关闭模态框

## Dev Notes

### 技术需求

1. **只修改前端 Vue 组件** - 不修改 Rust 后端逻辑
2. **TailwindCSS only** - 不使用内联样式或 per-component CSS
3. **不新增 Vue 组件** - 只修改 `SettingsModal.vue`
4. **前端测试必须通过** - `npm run test`

### 架构合规要求

- 遵循现有组件结构 (`src/components/SettingsModal.vue`)
- 使用 Tailwind 自定义颜色：`primary`, `secondary`, `dark`, `darker`
- 保持现有 `invoke()` / `emit()` 通信模式不变

### 文件结构要求

- 修改文件：`src/components/SettingsModal.vue`
- 测试文件（如有）：`src/components/__tests__/SettingsModal.test.js`

### 测试要求

组件测试验证：
- 设置界面加载时正确显示所有字段
- API Key 显示/隐藏切换功能正常
- 保存按钮点击触发 `save_settings` Tauri 命令
- 保存状态正确显示（成功/失败）

## Project Structure Notes

### 现有项目结构

```
src/
├── App.vue                    # 主界面容器
├── components/
│   ├── SettingsModal.vue      # 设置模态框 (本次修改目标)
│   ├── QuickNoteModal.vue     # 速记输入
│   ├── ScreenshotModal.vue    # 截图查看
│   ├── ScreenshotGallery.vue  # 截图画廊
│   └── ...
tailwind.config.js             # Tailwind 配置（定义自定义颜色）
```

### Tailwind 自定义颜色

```js
colors: {
  primary: '#3b82f6',    // 蓝色主色
  secondary: '#64748b',  // 灰色辅助色
  dark: '#1e293b',       // 深色背景
  darker: '#0f172a'      // 更深的背景
}
```

## References

- [Source: architecture.md#2.1 前端模块] - 组件职责描述
- [Source: specs/ui001-ui-improvement.md] - UI 交互优化规格
- [Source: tailwind.config.js] - 自定义颜色定义
- [Source: src/components/SettingsModal.vue] - 现有设置组件代码

## Dev Agent Record

### Agent Model Used

BMAD dev-story Workflow

### Implementation Summary

所有任务已完成，实现内容：

1. **Task 1 - 表单布局优化**：
   - 所有 label 改为 `text-xs text-gray-300`
   - 所有 h3 标题保持 `text-sm font-medium text-gray-300`
   - 所有 input/textarea 添加 `text-gray-100 placeholder:text-gray-500`
   - 使用 `space-y-3` 确保字段间距

2. **Task 2 - 按钮交互改进**：
   - 关闭按钮：`hover:text-white`
   - 保存按钮：`hover:bg-blue-600 disabled:opacity-50`
   - 取消按钮：`hover:bg-gray-700 hover:text-white`
   - API Key 切换按钮：`hover:text-gray-300 hover:bg-gray-700`

3. **Task 3 - API Key 显示/隐藏**：
   - 保持清晰的"显示"/"隐藏"文字提示
   - 添加 hover:bg-gray-700 背景反馈
   - 调整 padding 为 `px-2 py-1` 改善点击区域

4. **Task 4 - 保存状态提示**：
   - 成功：绿色勾号 SVG + "已保存"
   - 失败：红色警告 SVG + "保存失败"
   - 成功后 800ms 自动关闭

### Tests Added

- AC1 颜色一致性测试（4 个）
- AC2 按钮交互反馈测试（4 个）
- AC3 表单可用性测试（4 个）

所有 32 个测试通过（新增 12 个，原有 20 个）

### File List

- Modified: `src/components/SettingsModal.vue`
- Modified: `src/__tests__/SettingsModal.spec.js`

### Change Log

- 设置界面 UI 优化完成 (Date: 2026-03-14)
- 所有 AC 验证通过
- 前端测试全部通过

### File List

- Created: `_bmad-output/implementation-artifacts/CORE-001.md`
- To Modify: `src/components/SettingsModal.vue`

## 设计参考

### 当前 SettingsModal 状态

现有组件结构已包含：
- API 配置区域（Base URL, API Key）
- 截图分析配置（分析模型，分析 Prompt）
- 日报生成配置（日报模型，日报 Prompt）
- 时间策略配置（截图间隔，总结时间）
- 智能去重配置（变化阈值，最大静默时间）
- 输出配置（Obsidian 路径）
- 快捷键信息

### 本次优化重点

1. **视觉层次**：使用统一的颜色和字体大小创建清晰的组间分隔
2. **交互反馈**：所有可点击元素必须有 hover 状态
3. **可用性**：确保所有 label 和 placeholder 清晰易懂
4. **API Key 切换**：改进显示/隐藏按钮的视觉设计
5. **保存反馈**：明确的成功/失败状态提示
