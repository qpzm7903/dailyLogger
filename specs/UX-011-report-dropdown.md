# UX-011 报告生成整合为下拉菜单

**版本**: v1.41.0
**优先级**: HIGH

## 功能需求

将当前分散的日报、周报、月报三个独立生成按钮，整合为一个带下拉菜单的"生成报告"按钮。主按钮触发最常用的操作（日报），下拉菜单提供周报和月报选项。减少界面按钮数量，降低操作复杂度。

**当前问题**:
- 三个按钮占用较大界面空间
- 用户需要理解每个按钮的区别
- 生成进行中时三个按钮需要分别处理状态

## 不在范围内

- 不修改报告生成的 Rust 后端逻辑
- 不修改报告文件格式或存储位置
- 不实现自定义时间范围报告（未来版本）

## 接口定义

### 下拉菜单组件规格

```typescript
// src/components/ReportDropdown.vue

interface ReportOption {
  id: 'daily' | 'weekly' | 'monthly'
  label: string          // 显示文字（如"生成日报"）
  shortcut?: string      // 键盘快捷键提示（如"今日"）
  description?: string   // 简短说明（如"生成今天的工作总结"）
}

// Props
interface Props {
  isGenerating: boolean   // 来自父组件的生成状态锁（与 UX-002 联动）
}

// Emits
interface Emits {
  generate: (type: ReportOption['id']) => void
}
```

### 下拉菜单样式规范

```
主按钮：bg-blue-600 hover:bg-blue-500 text-white rounded-l-md px-4 py-2
箭头按钮：bg-blue-600 hover:bg-blue-500 border-l border-blue-400 rounded-r-md px-2 py-2
下拉面板：absolute bg-darker border border-gray-600 rounded-md shadow-lg z-20
菜单项：px-4 py-2 hover:bg-dark text-sm cursor-pointer
```

### 交互行为

- 点击主按钮：直接触发日报生成
- 点击箭头按钮：展开/收起下拉菜单
- 点击菜单项：触发对应类型报告生成，同时关闭菜单
- 点击菜单外部：关闭菜单（onClickOutside）
- `isGenerating` 为 true 时：主按钮和箭头按钮均禁用

## 验收条件（Given/When/Then）

### AC1 - 主按钮直接生成日报

- Given 报告生成界面加载完成
- When 用户点击主按钮（非箭头区域）
- Then 直接触发日报生成，不显示下拉菜单

### AC2 - 箭头按钮展开菜单

- Given 下拉菜单处于收起状态
- When 用户点击箭头按钮
- Then 下拉菜单展开，显示日报、周报、月报三个选项

### AC3 - 选择菜单项触发对应报告

- Given 下拉菜单已展开
- When 用户点击"生成周报"
- Then 触发周报生成，菜单自动收起

### AC4 - 生成中时全部禁用

- Given 报告生成正在进行中（isGenerating = true）
- When 用户查看报告生成区域
- Then 主按钮和箭头按钮均显示禁用样式，无法触发新操作

### AC5 - 点击菜单外关闭

- Given 下拉菜单已展开
- When 用户点击菜单以外的区域
- Then 菜单收起，不触发任何报告生成

## 技术约束

- 只使用 TailwindCSS，无内联样式
- 使用 VueUse 的 `onClickOutside` 处理外部点击关闭
- 前端测试须覆盖：展开/收起、各选项触发、禁用状态
- 与 UX-002（按钮互锁）配合，isGenerating 状态由父组件传入
- `npm run test` 全部通过
