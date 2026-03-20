# UX-003 离线状态顶部横幅 Banner

**版本**: v1.40.0
**优先级**: MEDIUM

## 功能需求

将当前离线状态的显示方式从模态提示或内嵌组件改为顶部固定横幅（Banner），横幅在网络恢复后自动消失。Banner 不阻断用户操作，但明确告知用户当前处于离线状态，AI 相关功能不可用。

**当前问题**:
- 离线状态指示位置不固定，用户可能忽略
- 离线提示可能阻断操作流程
- 网络恢复后没有明确的恢复通知

## 不在范围内

- 不修改网络状态检测逻辑
- 不修改离线时功能的限制规则（AI 功能仍不可用）
- 不实现离线数据缓存

## 接口定义

### Banner 组件规格

```typescript
// src/components/OfflineBanner.vue
// Props: 无（直接读取网络状态 store 或 composable）

// 样式规范
// - 位置：fixed top-0 left-0 right-0，z-index: 50
// - 高度：py-2（约 36px）
// - 颜色：bg-yellow-600 text-white（离线）/ bg-green-600 text-white（恢复，显示 3s）
// - 内容：图标 + 文字 + 可选的重试按钮
```

### 状态机

```
隐藏 → [网络断开] → 显示离线 Banner
显示离线 Banner → [网络恢复] → 显示"已重新连接" Banner（3s）→ 隐藏
```

### 主布局适配

顶部内容区域需预留 Banner 高度，防止 Banner 遮挡主内容：
- Banner 显示时：主容器添加 `pt-9`（或动态 padding）
- Banner 隐藏时：主容器恢复正常

## 验收条件（Given/When/Then）

### AC1 - 离线时显示横幅

- Given 应用正常运行
- When 网络连接断开
- Then 顶部固定横幅在 2 秒内出现，显示"当前处于离线状态，AI 功能暂不可用"

### AC2 - 横幅不阻断操作

- Given 离线横幅正在显示
- When 用户尝试进行非 AI 操作（如手动记录笔记）
- Then 操作正常执行，横幅不弹出模态框阻断流程

### AC3 - 网络恢复时提示

- Given 离线横幅正在显示
- When 网络连接恢复
- Then 横幅变为绿色"网络已恢复"，3 秒后自动消失

### AC4 - 主内容不被遮挡

- Given 离线横幅显示时
- When 用户滚动主界面
- Then 主内容区域顶部有足够间距，第一行内容不被横幅遮挡

## 技术约束

- 只使用 TailwindCSS，无内联样式
- Banner 使用 Vue Transition 实现滑入/滑出动画
- 复用现有网络状态检测逻辑（不新建检测机制）
- `npm run test` 全部通过
