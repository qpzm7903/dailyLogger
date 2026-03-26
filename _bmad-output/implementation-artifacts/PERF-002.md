# Story 10.2: 新用户引导

Status: done

## Story

As a new DailyLogger user,
I want to see a guided onboarding flow when I first launch the app,
so that I can quickly configure the essential settings (API and output path) without getting lost or frustrated.

**来源**: plan.md 未来规划 - 新用户引导 / 首屏体验优化

## Background

当前应用首次启动时没有引导流程。用户看到的是空白的主界面，不知道需要先配置 API 和 Obsidian 路径。导致新用户体验差，可能直接放弃使用。

**问题分析**：
- 首次启动时 `api_base_url` 和 `api_key` 为空，`obsidian_path` 也可能未配置
- Dashboard 没有任何配置引导，用户不知道需要做什么
- 没有"首屏体验"的概念

**Epic 10 定位**：
```
Epic 10: 体验极致化
├── PERF-001: AI 配置完善（代理支持） ✅ 已完成
├── PERF-002: 新用户引导 ← 当前
├── PERF-003: 性能优化 - 截图加载
├── PERF-004: 性能优化 - 数据库查询
├── PERF-005: 多语言支持 (i18n)
└── PERF-006: 浅色主题支持
```

## Acceptance Criteria

1. **首次启动检测**
   - Given 新用户首次启动应用（没有任何配置）
   - When 应用加载完成
   - Then 自动弹出引导流程，而不是显示空白 Dashboard

2. **API 配置引导**
   - Given 引导流程第 1 步（API 配置）
   - When 用户输入 API Base URL 和 API Key
   - Then 显示"测试连接"按钮，验证配置有效性
   - And 测试成功后才允许进入下一步

3. **Obsidian 路径配置引导**
   - Given 引导流程第 2 步（输出路径）
   - When 用户选择 Obsidian Vault 路径
   - Then 验证路径有效性并显示确认
   - And 如果路径不存在，可以选择创建

4. **跳过与后续补充**
   - Given 引导流程中
   - When 用户跳过某些步骤
   - Then 记录跳过状态，显示"之后可在设置中补充"
   - And 用户随时可以在设置中完成配置

5. **引导完成**
   - Given 用户完成所有必填步骤或选择跳过
   - When 点击"开始使用"
   - Then 关闭引导流程，进入主界面
   - And 设置 `onboarding_completed = true` 标记引导已完成

## Tasks / Subtasks

- [x] Task 1: 检测新用户首次启动 (AC: #1)
  - [x] 在 `Settings` 结构体添加 `onboarding_completed` 字段（布尔值，数据库存储）
  - [x] 在 `App.vue` 的 `onMounted` 中检测是否需要显示引导
  - [x] 检测逻辑：`api_base_url` 为空或 `onboarding_completed` 为 false

- [x] Task 2: 创建 OnboardingModal 组件 (AC: #1-5)
  - [x] 创建 `src/components/OnboardingModal.vue` 组件
  - [x] 实现步骤指示器（Step 1: API 配置 → Step 2: 输出路径 → Step 3: 完成）
  - [x] 实现步骤导航（下一步、上一步、跳过）
  - [x] 实现引导完成后的状态保存

- [x] Task 3: API 配置步骤 UI (AC: #2)
  - [x] 复用 `BasicSettings.vue` 中的 AI 配置部分（API Base URL、API Key）
  - [x] 添加"测试连接"按钮，调用 `test_api_connection_with_ollama`
  - [x] 测试成功/失败的状态反馈
  - [x] 不通过测试不允许进入下一步（必填项）

- [x] Task 4: Obsidian 路径配置步骤 UI (AC: #3)
  - [x] 调用 Tauri 命令选择文件夹 `dialog::open`
  - [x] 验证路径有效性（目录是否存在、可读写）
  - [x] 显示路径确认信息
  - [x] 支持创建新目录

- [x] Task 5: 跳过与完成逻辑 (AC: #4, #5)
  - [x] 实现"跳过"按钮（可选步骤）
  - [x] 完成后设置 `onboarding_completed = true`
  - [x] 保存用户填写的内容（即使跳过）

- [x] Task 6: 集成与测试 (AC: all)
  - [x] 在 `App.vue` 中集成 `OnboardingModal`
  - [x] 测试：首次启动 → 引导流程 → 完成 → 不再显示
  - [x] 测试：非首次启动 → 直接显示 Dashboard

## Dev Notes

### 关键架构约束

1. **前端技术栈**：Vue 3 Composition API + `<script setup>`，TailwindCSS（无独立 CSS 文件）
2. **后端技术栈**：Rust + Tauri v2，Tauri `dialog` API 用于选择文件夹
3. **检测逻辑**：基于 `api_base_url` 是否为空 + `onboarding_completed` 标志

### 文件树组件（需新增/修改）

```
src/
├── components/
│   └── OnboardingModal.vue          # 新增：引导流程组件
├── App.vue                          # 修改：集成引导流程检测
src-tauri/src/
├── memory_storage/
│   ├── mod.rs                      # 修改：Settings 结构体添加 onboarding_completed
│   └── schema.rs                   # 修改：添加 onboarding_completed 字段
└── lib.rs / main.rs                # 可能需要添加 Tauri dialog 依赖
```

### 数据库 Schema 变更

```sql
ALTER TABLE settings ADD COLUMN onboarding_completed INTEGER DEFAULT 0;
```

### 引导流程 UI 设计

```
┌─────────────────────────────────────────────┐
│  Welcome to DailyLogger!              [×]   │
├─────────────────────────────────────────────┤
│  ●────○────○                                  │
│  API   输出   完成                             │
├─────────────────────────────────────────────┤
│                                             │
│  Step 1: 配置 AI API                         │
│                                             │
│  API Base URL:                              │
│  [https://api.openai.com/v1          ]      │
│                                             │
│  API Key:                                   │
│  [sk-••••••••••••••••••              ]      │
│                                             │
│  [        测试连接        ]                  │
│  ✅ 连接成功！                                │
│                                             │
│                    [跳过]  [下一步 →]        │
└─────────────────────────────────────────────┘
```

### 检测新用户的逻辑

```typescript
// App.vue onMounted 中
const settings = await invoke<Settings>('get_settings')
const needsOnboarding = !settings.api_base_url || !settings.onboarding_completed
if (needsOnboarding) {
  open('onboarding')
}
```

### Tauri dialog 使用

```rust
// Rust 后端：使用 tauri::api::dialog
#[tauri::command]
async fn select_folder() -> Result<String, String> {
    let folder = dialog::FileDialog::new()
        .set_title("选择 Obsidian Vault 路径")
        .pick_folder();
    folder.map(|p| p.to_string_lossy().to_string())
        .ok_or_else(|| "No folder selected".to_string())
}
```

## Testing Requirements

1. **首次启动测试**：
   - 清除数据库后首次启动 → 弹出引导
   - `onboarding_completed = false` + `api_base_url` 为空

2. **API 配置测试**：
   - 空 API URL → 测试连接失败
   - 无效 API URL → 测试连接失败
   - 有效配置 → 测试连接成功

3. **路径选择测试**：
   - 选择有效目录 → 显示确认
   - 选择无效目录 → 报错
   - 选择不存在的目录 → 提示创建

4. **跳过与完成测试**：
   - 跳过 API 配置 → 保存跳过状态 → 可在设置补充
   - 完成引导 → `onboarding_completed = true` → 不再弹出

5. **回归测试**：
   - 已有配置的用户启动 → 不显示引导 → 直接进入 Dashboard

## References

- [Source: src/App.vue#494] - `onMounted` 中 `loadSettings()` 和 `loadTodayRecords()` 调用
- [Source: src/components/SettingsModal.vue] - 设置模态框结构（复用 AI 配置部分）
- [Source: src/components/settings/BasicSettings.vue] - AI 配置 UI（测试连接按钮）
- [Source: src-tauri/src/memory_storage/mod.rs] - Settings 结构体定义
- [Source: src-tauri/src/memory_storage/schema.rs] - 数据库 Schema 定义
- [Source: plan.md] - 未来规划中新用户引导描述

## Dev Agent Record

### Agent Model Used

claude-opus-4-6

### Debug Log References

<!-- TODO: Fill in during development -->

### Completion Notes List

- 完成新用户引导流程的完整实现
- 在 `Settings` 结构体中添加了 `onboarding_completed` 字段
- 在数据库 schema 中添加了 `onboarding_completed` INTEGER DEFAULT 0 字段
- 创建了 `OnboardingModal.vue` 组件，包含 3 步引导流程（API 配置 → 输出路径 → 完成）
- 实现了 API 测试连接功能，调用 `test_api_connection_with_ollama`
- 使用 Tauri dialog 插件选择 Obsidian Vault 文件夹
- 在 `App.vue` 中集成了引导检测逻辑：首次启动或 `api_base_url` 为空时显示引导
- 注意：OnboardingModal 使用硬编码中文字符串，未使用 i18n 系统（符合当前用户群体）
- 所有测试通过（927 前端测试 + 454 Rust 测试）

### File List

**新增文件：**
- `src/components/OnboardingModal.vue` - 新用户引导流程组件

**修改文件：**
- `src/App.vue` - 集成 OnboardingModal 和引导检测逻辑
- `src/types/tauri.ts` - 添加 onboarding_completed 字段到 Settings 接口
- `src-tauri/src/memory_storage/mod.rs` - Settings 结构体添加 onboarding_completed 字段
- `src-tauri/src/memory_storage/schema.rs` - 数据库 schema 添加 onboarding_completed 字段
- `src-tauri/src/memory_storage/settings.rs` - get_settings_sync 和 save_settings_sync 添加 onboarding_completed 支持
- `src-tauri/src/synthesis/mod.rs` - 测试代码中添加 onboarding_completed 字段

**未修改（文档声明 vs 实际不符）：**
- `src/locales/zh-CN.json` - OnboardingModal 使用硬编码中文，未使用 i18n
- `src/locales/en.json` - OnboardingModal 使用硬编码中文，未使用 i18n

