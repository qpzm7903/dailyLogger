# Story 10.2: 新用户引导 (New User Onboarding)

Status: ready-for-dev

## Story

As a new DailyLogger user,
I want to see a guided onboarding flow on first launch,
so that I can quickly configure the app and start using it without confusion.

**来源**: plan.md 未来规划 - 新用户引导 / 首屏体验优化

## Background

新用户首次启动 DailyLogger 时，面临着"无从下手"的困境。当前应用没有引导流程，用户需要自己找到设置、配置 API、设置 Obsidian 路径等，体验不够友好。

**Epic 10 定位**:
```
Epic 10: 体验极致化
├── PERF-001: AI 配置完善（代理支持） ✅ 已完成
├── PERF-002: 新用户引导 ← 当前
├── PERF-003: 性能优化 - 截图加载
├── PERF-004: 性能优化 - 数据库查询
├── PERF-005: 多语言支持 (i18n)
└── PERF-006: 浅色主题支持
```

**参考已有组件模式**:
- `QuickNoteModal.vue`: 全屏遮罩 + 居中卡片 + 步骤流程 UI
- `SettingsModal.vue`: Tab 切换式设置界面
- `BasicSettings.vue`: API 配置 UI (API Base URL, API Key, Test Connection)
- `OutputSettings.vue`: Obsidian 路径配置 UI

## Acceptance Criteria

1. **首次启动检测**
   - Given 新用户首次启动应用
   - When 应用加载完成
   - Then 自动弹出引导流程（不显示主界面）
   - And 检测方式：settings 表中 `first_run_completed = false` 或 `api_base_url` 为空

2. **引导步骤一：欢迎页**
   - Given 引导流程启动
   - When 用户看到欢迎页
   - Then 显示应用简介和"开始配置"按钮

3. **引导步骤二：API 配置**
   - Given 欢迎页点击"下一步"
   - When 用户进入 API 配置步骤
   - Then 显示 API Base URL 输入框、API Key 输入框、Test Model 输入框（可选）
   - And 显示"测试连接"按钮
   - And 测试连接成功后才允许进入下一步

4. **引导步骤三：Obsidian 路径配置**
   - Given API 配置完成并通过测试
   - When 用户进入 Obsidian 配置步骤
   - Then 显示路径输入框 + 浏览按钮
   - And 验证路径有效性（目录存在或可创建）
   - And 显示确认信息

5. **引导步骤四：完成**
   - Given Obsidian 配置完成（或跳过）
   - When 用户点击"完成"
   - Then 保存 `first_run_completed = true`
   - And 关闭引导模态框，显示主界面
   - And 启动自动截图感知

6. **跳过机制**
   - Given 引导流程中任意步骤
   - When 用户点击"跳过"
   - Then 保存已填写内容（若有），标记 `first_run_completed = true`
   - And 关闭引导模态框
   - And 之后用户可在设置中补充配置

## Tasks / Subtasks

- [ ] Task 1: 数据库添加首次引导完成标记 (AC: #1, #6)
  - [ ] 在 `schema.rs` 添加 `first_run_completed INTEGER DEFAULT 0` 字段
  - [ ] 在 `memory_storage/mod.rs` 的 `Settings` 结构体添加 `first_run_completed: bool` 字段
  - [ ] 在 `memory_storage/settings.rs` 的 `get_settings` 和 `update_settings` 中处理该字段
  - [ ] 初始化时设置默认值为 `false`

- [ ] Task 2: 创建 OnboardingModal.vue 组件 (AC: #1-#6)
  - [ ] 创建 `src/components/OnboardingModal.vue`
  - [ ] 实现多步骤引导流程（欢迎 → API 配置 → Obsidian 路径 → 完成）
  - [ ] 使用 vue-i18n 实现中文/英文国际化
  - [ ] 实现步骤进度指示器
  - [ ] 每个步骤支持"上一步"/"下一步"/"跳过"按钮

- [ ] Task 3: API 配置步骤实现 (AC: #3)
  - [ ] 复用 `BasicSettings.vue` 中的 API 配置 UI 设计模式
  - [ ] 实现"测试连接"功能（调用后端 `test_api_connection` 命令）
  - [ ] 测试成功/失败状态显示
  - [ ] 测试通过后才能进入下一步

- [ ] Task 4: Obsidian 路径配置步骤实现 (AC: #4)
  - [ ] 复用 `OutputSettings.vue` 中的 Obsidian 路径 UI 设计模式
  - [ ] 实现目录选择器（调用后端 `select_directory` 命令）
  - [ ] 路径有效性验证
  - [ ] 显示选中路径确认信息

- [ ] Task 5: 首次启动检测与引导触发 (AC: #1)
  - [ ] 在 `App.vue` 的 `onMounted` 中检测是否首次启动
  - [ ] 若是首次启动，打开 `OnboardingModal` 而非主界面
  - [ ] 引导完成后标记 `first_run_completed = true`

- [ ] Task 6: 国际化文案 (AC: all)
  - [ ] 在 `en.json` 添加引导流程所有文案
  - [ ] 在 `zh-CN.json` 添加引导流程所有中文文案
  - [ ] 确保所有用户可见文本都有对应翻译

- [ ] Task 7: 集成测试 (AC: all)
  - [ ] 手动测试：首次启动 → 引导流程 → 完成 → 主界面
  - [ ] 手动测试：跳过 Obsidian 配置 → 设置中可补充
  - [ ] 手动测试：API 测试连接成功/失败状态
  - [ ] 确认旧用户升级不受影响（`first_run_completed = true` 保持）

## Dev Notes

### 关键技术栈约束

1. **前端技术栈**：Vue 3 Composition API + `<script setup>`，TailwindCSS（无独立 CSS 文件），vue-i18n
2. **后端技术栈**：Rust + Tauri v2
3. **组件模式**：参考 `QuickNoteModal.vue` 的全屏遮罩 + 居中卡片模式
4. **Tauri IPC**：使用 `invoke()` 调用后端命令，使用 `emit()`/`listen()` 进行事件通信

### 文件树组件（需修改/新增）

```
src/
├── components/
│   └── OnboardingModal.vue          # 新增：引导流程主组件
├── App.vue                          # 修改：添加首次启动检测
src-tauri/src/
├── memory_storage/
│   ├── mod.rs                      # 修改：Settings 结构体添加字段
│   ├── schema.rs                   # 修改：ALTER TABLE 添加字段
│   └── settings.rs                 # 修改：get/update settings 处理字段
src/locales/
├── en.json                         # 修改：添加引导文案
└── zh-CN.json                      # 修改：添加引导文案
```

### 数据库 Schema 变更

```sql
ALTER TABLE settings ADD COLUMN first_run_completed INTEGER DEFAULT 0;
```

### 引导流程步骤定义

```typescript
interface OnboardingStep {
  id: 'welcome' | 'api-config' | 'obsidian-path' | 'complete';
  titleKey: string;      // i18n key
  canSkip: boolean;
  requiresValidation: boolean;
}

const STEPS: OnboardingStep[] = [
  { id: 'welcome', titleKey: 'onboarding.welcome.title', canSkip: false, requiresValidation: false },
  { id: 'api-config', titleKey: 'onboarding.apiConfig.title', canSkip: false, requiresValidation: true },
  { id: 'obsidian-path', titleKey: 'onboarding.obsidianPath.title', canSkip: true, requiresValidation: false },
  { id: 'complete', titleKey: 'onboarding.complete.title', canSkip: false, requiresValidation: false },
];
```

### 首次启动检测逻辑

```typescript
// App.vue onMounted
const settings = await invoke<Settings>('get_settings');
const isFirstRun = !settings.first_run_completed && !settings.api_base_url;

if (isFirstRun) {
  open('onboarding');
} else {
  // 正常加载主界面
  loadTodayRecords();
  startAutoCaptureIfEnabled();
}
```

### 引导完成后的处理

```typescript
// OnboardingModal.vue emit 'complete'
async function handleComplete() {
  // 保存 first_run_completed
  await invoke('update_first_run_completed', { completed: true });
  close('onboarding');
  // 通知 App.vue 切换到主界面
  emit('complete');
}
```

### i18n 文案结构

```json
// en.json / zh-CN.json
{
  "onboarding": {
    "welcome": {
      "title": "Welcome to DailyLogger",
      "subtitle": "Your AI-powered work memory assistant",
      "getStarted": "Get Started"
    },
    "apiConfig": {
      "title": "Configure AI API",
      "subtitle": "Enter your API credentials to enable AI analysis",
      "baseUrl": "API Base URL",
      "apiKey": "API Key",
      "testModel": "Test Model (optional)",
      "testConnection": "Test Connection",
      "testing": "Testing...",
      "testSuccess": "Connection successful!",
      "testFailed": "Connection failed: {error}",
      "next": "Next",
      "skip": "Skip for now"
    },
    "obsidianPath": {
      "title": "Set Obsidian Path",
      "subtitle": "Choose where to save your daily reports",
      "pathPlaceholder": "Select Obsidian vault folder...",
      "browse": "Browse",
      "validPath": "Valid path selected",
      "next": "Next",
      "skip": "Skip for now"
    },
    "complete": {
      "title": "You're all set!",
      "subtitle": "DailyLogger is ready to help you record your work",
      "startUsing": "Start Using DailyLogger"
    }
  }
}
```

### 组件设计要点

1. **全屏遮罩**: `fixed inset-0 bg-black/60 backdrop-blur-sm`
2. **居中卡片**: `max-w-lg w-full bg-darker rounded-2xl shadow-2xl`
3. **步骤指示器**: 顶部圆点进度指示，已完成(蓝)、当前(蓝+放大)、未完成(灰)
4. **按钮样式**: `.btn-primary` / `.btn-ghost` 符合现有设计系统
5. **过渡动画**: 使用 Vue Transition，步骤切换使用淡入淡出

## Testing Requirements

1. **单元测试**：
   - `OnboardingModal.vue`: 测试步骤切换逻辑
   - `App.vue`: 测试首次启动检测分支

2. **集成测试**：
   - 完整引导流程：首次启动 → 完成所有步骤 → 主界面
   - 跳过 Obsidian 配置后可在设置中补充
   - 旧用户升级后 `first_run_completed = true` 不触发引导

3. **测试覆盖**：
   - API 测试连接成功/失败
   - Obsidian 路径有效性验证
   - 步骤导航（上一步/下一步/跳过）
   - 引导完成状态持久化

## References

- [Source: src/components/QuickNoteModal.vue] - 全屏遮罩模态框参考
- [Source: src/components/settings/BasicSettings.vue] - API 配置 UI 参考
- [Source: src/components/settings/OutputSettings.vue] - Obsidian 路径 UI 参考
- [Source: src/App.vue] - 首次启动检测逻辑
- [Source: src-tauri/src/memory_storage/schema.rs] - 数据库 Schema
- [Source: src/locales/en.json] - i18n 英文文案
- [Source: src/locales/zh-CN.json] - i18n 中文文案
- [PRD Section 5.1: 新用户入职流程] - 产品要求的入职流程

## Dev Agent Record

### Agent Model Used

claude-opus-4-6

### Debug Log References

### Completion Notes List

### File List
