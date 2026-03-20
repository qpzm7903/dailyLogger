# UX-010 useModal Composable 统一模态管理

**版本**: v1.41.0
**优先级**: HIGH

## 功能需求

创建 `useModal` composable，替代 `App.vue` 中分散的 21 个 `showXxx` ref。通过集中的模态管理，确保同一时刻只有一个模态框处于打开状态，简化模态的打开/关闭逻辑，消除状态泄漏（如关闭一个模态意外影响另一个）。

**当前问题**:
- `App.vue` 中存在大量 `showXxx = ref(false)` 声明，难以维护
- 模态框的打开/关闭逻辑分散在各处
- 可能出现多个模态同时打开的情况
- 测试模态状态需要了解每个 `showXxx` 变量的名称

## 不在范围内

- 不修改各模态框组件的内部逻辑
- 不修改模态框的视觉样式
- 不引入新的状态管理库（使用 Vue Composition API 实现）

## 接口定义

### useModal 接口

```typescript
// src/composables/useModal.ts

type ModalId =
  | 'settings'
  | 'quickNote'
  | 'historyViewer'
  | 'reportGenerator'
  | 'screenshotGallery'
  | 'tagManager'
  // ... 其他模态 ID

interface UseModalReturn {
  activeModal: Readonly<Ref<ModalId | null>>
  isOpen: (id: ModalId) => boolean
  open: (id: ModalId) => void
  close: (id?: ModalId) => void   // 无参数时关闭当前模态
  toggle: (id: ModalId) => void
}

export function useModal(): UseModalReturn
```

### 迁移映射

| 旧 ref | 新调用 |
|--------|--------|
| `showSettings = ref(false)` | `isOpen('settings')` |
| `showSettings.value = true` | `open('settings')` |
| `showSettings.value = false` | `close('settings')` |

### 行为规范

- `open(id)` 调用时，若已有其他模态打开，先自动关闭再打开新模态
- `close()` 无参数版本关闭当前活跃模态
- 同一 `id` 重复 `open()` 不触发重新渲染

## 验收条件（Given/When/Then）

### AC1 - 同一时刻只有一个模态打开

- Given 设置模态已经打开
- When 用户触发打开历史记录模态
- Then 设置模态自动关闭，历史记录模态打开

### AC2 - close 无参数关闭当前模态

- Given 报告生成模态处于打开状态
- When 调用 `close()`（无参数）
- Then 报告生成模态关闭，activeModal 变为 null

### AC3 - isOpen 返回正确状态

- Given quickNote 模态已打开
- When 检查 `isOpen('quickNote')` 和 `isOpen('settings')`
- Then 前者返回 true，后者返回 false

### AC4 - 无模态打开时 activeModal 为 null

- Given 所有模态均已关闭
- When 读取 activeModal
- Then 返回 null

### AC5 - App.vue 中无残留 showXxx ref

- Given 迁移完成
- When 检查 App.vue 源码
- Then 不存在 `showXxx = ref(false)` 形式的模态控制变量

## 技术约束

- 实现为单例 composable（模块级 ref，非组件级）
- 前端测试须覆盖所有方法（open、close、toggle、isOpen）
- 迁移须保持所有现有模态功能正常，无行为回归
- `npm run test` 全部通过
- TypeScript 严格模式，ModalId 须为字面量联合类型（不允许 `string`）
