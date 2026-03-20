# UX-020 SettingsModal 拆分为标签页子组件

**版本**: v1.42.0
**优先级**: HIGH

## 功能需求

将现有 `SettingsModal.vue` 中所有设置项拆分到 4 个独立的标签页子组件中，减少单个组件的代码量（当前约 500+ 行），提高可维护性，同时改善用户在大量设置项中的导航体验。

**当前问题**:
- SettingsModal.vue 单文件包含所有设置项，维护困难
- 用户打开设置后需要滚动很长才能找到特定设置
- 添加新设置项时影响范围不清晰

## 不在范围内

- 不修改任何设置项的功能逻辑
- 不增加新设置项（属于功能需求，独立规划）
- 不修改设置数据的持久化方式（仍使用 SQLite `settings` 表）

## 接口定义

### 标签页划分

| 标签 ID | 标签名称 | 包含设置项 |
|---------|---------|-----------|
| `general` | 通用 | 语言、主题、应用行为 |
| `ai` | AI 配置 | api_base_url、api_key、model_name |
| `capture` | 截图 | screenshot_interval、auto_capture_enabled |
| `export` | 导出 | obsidian_path、summary_time、last_summary_path |

### 组件层次结构

```
SettingsModal.vue              # 外层容器：标签页导航 + 当前活跃 Tab 渲染
├── SettingsTabGeneral.vue     # 通用设置
├── SettingsTabAI.vue          # AI 配置
├── SettingsTabCapture.vue     # 截图配置
└── SettingsTabExport.vue      # 导出配置
```

### 标签导航组件接口

```typescript
// SettingsModal.vue 内部状态
type SettingsTabId = 'general' | 'ai' | 'capture' | 'export'

const activeTab = ref<SettingsTabId>('general')

// 标签页子组件 Props（统一接口）
interface SettingsTabProps {
  settings: SettingsData    // 传入当前设置值
}

interface SettingsTabEmits {
  update: (patch: Partial<SettingsData>) => void  // 局部更新
}
```

### 标签页样式规范

```
标签导航容器：flex border-b border-gray-700 mb-4
标签按钮（未选中）：px-4 py-2 text-sm text-gray-400 hover:text-primary cursor-pointer
标签按钮（选中）：px-4 py-2 text-sm text-primary border-b-2 border-blue-500
```

## 验收条件（Given/When/Then）

### AC1 - 默认显示第一个标签页

- Given 用户打开设置模态框
- When 模态框渲染完成
- Then 默认激活"通用"标签页，其他标签页不渲染（或隐藏）

### AC2 - 切换标签页正确渲染

- Given 设置模态框已打开
- When 用户点击"AI 配置"标签
- Then AI 配置相关设置项显示，其他标签页内容隐藏

### AC3 - 各标签页设置独立保存

- Given 用户在"AI 配置"标签修改了 api_key
- When 用户切换到"截图"标签再切回"AI 配置"
- Then 修改的 api_key 值仍然保留（未被清除）

### AC4 - 原有功能无回归

- Given 拆分前所有设置项均可正常保存和读取
- When 完成拆分后
- Then 所有设置项的保存、读取、验证逻辑保持不变

### AC5 - SettingsModal 主文件行数减少

- Given 重构完成
- When 检查 SettingsModal.vue 文件
- Then 主文件行数不超过 150 行（不含子组件）

## 技术约束

- 只使用 TailwindCSS，无内联样式
- 每个子组件须有对应测试文件
- 重构须通过现有所有 SettingsModal 相关测试
- `npm run test` 全部通过
- TypeScript 严格模式，所有 Props 须有明确类型定义
