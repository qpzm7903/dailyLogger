# UX-001 改进错误提示和用户反馈

**版本**: v1.40.0
**优先级**: HIGH
**关联**: MAINT-009

## 功能需求

改善 DailyLogger 中所有异步操作（截图分析、日报生成、数据保存等）的错误提示体验，确保用户始终清楚操作结果，遇到错误时能获得可操作的提示信息。

**当前问题**:
- 部分错误仅在控制台输出，用户界面无任何反馈
- 错误消息直接暴露技术细节（如 Rust 错误字符串），对用户无意义
- 成功操作缺乏反馈（用户不确定操作是否完成）

## 不在范围内

- 不修改错误的产生逻辑（Rust 后端错误处理）
- 不新增功能入口
- 不修改数据库 schema

## 接口定义

### Toast 消息类型扩展

```typescript
// src/types/toast.ts
type ToastLevel = 'success' | 'error' | 'warning' | 'info'

interface ToastOptions {
  level: ToastLevel
  message: string        // 用户友好的消息（中文）
  detail?: string        // 可选技术细节（折叠显示）
  duration?: number      // 显示时长（ms），默认 error=5000, success=3000
  action?: {
    label: string
    handler: () => void
  }
}
```

### 错误消息映射规则

| 原始错误模式 | 用户友好消息 |
|---|---|
| `*network*` / `*connection*` | 网络连接失败，请检查网络设置 |
| `*api key*` / `*unauthorized*` | API Key 无效，请在设置中重新配置 |
| `*database*` / `*sqlite*` | 数据存储失败，请重启应用 |
| `*screenshot*` / `*capture*` | 截图失败，请检查屏幕录制权限 |
| 其他 | 操作失败，请稍后重试 |

## 验收条件（Given/When/Then）

### AC1 - 异步操作成功反馈

- Given 用户触发任意异步操作（如手动记录、生成日报）
- When 操作成功完成
- Then 界面显示绿色 success Toast，持续 3 秒后自动消失

### AC2 - 错误消息用户友好

- Given 后端返回错误字符串（如 `"Error: network timeout"`）
- When Toast 展示错误
- Then 用户看到映射后的中文消息，技术细节折叠在"详情"中

### AC3 - 错误 Toast 持续时间更长

- Given 出现错误
- When Toast 显示
- Then error 级别的 Toast 持续 5 秒（而非 success 的 3 秒），且可手动关闭

### AC4 - 不阻断操作流程

- Given 非致命错误发生
- When Toast 显示
- Then 用户仍可继续操作，Toast 不阻挡主界面交互区域

## 技术约束

- 使用现有 `toast` store（`src/stores/toast.ts`）扩展，不新建 store
- 只使用 TailwindCSS，无内联样式
- 前端测试须覆盖错误消息映射函数
- `npm run test` 全部通过
