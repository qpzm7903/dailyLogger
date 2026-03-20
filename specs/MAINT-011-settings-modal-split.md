# MAINT-011 SettingsModal.vue 组件拆分

**版本**: v1.45.0
**优先级**: LOW

## 功能需求

将 2750 行的 `SettingsModal.vue` 拆分为独立的子组件，每个 Tab 独立为单独的 Vue 组件，提升代码可维护性和可测试性。

**当前问题**:
- 单文件 2750 行，难以维护和审查
- 模板和逻辑高度耦合
- 难以单独测试各个设置模块
- 多人协作时容易产生合并冲突

## 不在范围内

- 不修改各设置项的功能逻辑
- 不修改设置的视觉样式
- 不改变设置的数据结构或 API

## 组件拆分方案

### 新组件结构

```
src/components/settings/
├── index.ts                    # 导出所有设置组件
├── BasicSettings.vue           # 基础设置（API 配置、模型选择、测试连接）
├── AISettings.vue              # AI 设置（分析模型、报告模型、Prompt 配置）
├── CaptureSettings.vue         # 截图设置（间隔、静默检测、窗口过滤、多显示器）
├── OutputSettings.vue          # 输出设置（Obsidian、Logseq、Notion、Slack）
└── shared/
    ├── SettingSection.vue      # 通用设置区块容器
    └── types.ts                # 共享类型定义
```

### 父子组件通信模式

```typescript
// SettingsModal.vue 作为容器
// 子组件通过 v-model 与父组件双向绑定 settings 对象

interface SettingsProps {
  settings: AppSettings  // 从父组件传入
}

interface SettingsEmits {
  (e: 'update:settings', value: AppSettings): void
}

// 每个子组件
<BasicSettings v-model="settings" />
```

### 拆分边界

| 组件 | 包含的设置项 | 模板行数 |
|------|-------------|----------|
| BasicSettings | API Base URL、API Key、模型选择、测试连接、Ollama 模型管理、语言切换 | ~255 |
| AISettings | 分析模型配置、报告模型配置、Prompt 配置、标签分类配置 | ~182 |
| CaptureSettings | 截图间隔、智能静默检测、窗口过滤、多显示器选择 | ~346 |
| OutputSettings | Obsidian Vault、Logseq、Notion、Slack、数据备份恢复 | ~173 |

## 接口定义

### BasicSettings.vue

```vue
<script setup lang="ts">
import type { AppSettings } from '@/types/tauri'

const props = defineProps<{
  settings: AppSettings
}>()

const emit = defineEmits<{
  (e: 'update:settings', value: AppSettings): void
}>()

// 暴露给父组件的方法（可选）
defineExpose({
  validate: () => boolean
})
</script>
```

### 共享类型

```typescript
// src/components/settings/shared/types.ts

export interface ModelInfo {
  context_window?: number
  max_tokens?: number
}

export interface ConnectionTestResult {
  success: boolean
  message: string
  latency_ms?: number
}

export type SettingsTab = 'basic' | 'ai' | 'capture' | 'output'
```

## 验收条件（Given/When/Then）

### AC1 - 功能完全保留

- Given 拆分完成
- When 用户打开设置模态框并操作任意设置项
- Then 所有设置项功能与拆分前完全一致

### AC2 - 单组件行数合理

- Given 拆分完成
- When 检查各子组件文件行数
- Then 每个子组件不超过 500 行

### AC3 - 测试全部通过

- Given 拆分完成
- When 运行 `npm run test`
- Then 所有 583 个测试通过

### AC4 - 无 TypeScript 错误

- Given 拆分完成
- When 运行 `vue-tsc --noEmit`
- Then 无类型错误

### AC5 - 父组件精简

- Given 拆分完成
- When 检查 SettingsModal.vue 行数
- Then 不超过 300 行（仅保留容器逻辑）

## 技术约束

- 子组件使用 `v-model` 实现双向绑定
- 共享状态通过 `provide/inject` 或 props 传递，避免全局状态污染
- 保持现有的 i18n 国际化支持
- 保持现有的响应式布局

## 实施顺序

1. 创建 `src/components/settings/` 目录结构
2. 提取共享类型到 `types.ts`
3. 创建 `BasicSettings.vue` 并迁移相关代码
4. 创建 `AISettings.vue` 并迁移相关代码
5. 创建 `CaptureSettings.vue` 并迁移相关代码
6. 创建 `OutputSettings.vue` 并迁移相关代码
7. 重构 `SettingsModal.vue` 为容器组件
8. 运行测试确保无回归
9. 更新相关测试文件（如有必要）

## 风险评估

- **低风险**：此重构仅改变代码组织，不改变业务逻辑
- **回滚策略**：Git 可随时回滚到重构前版本