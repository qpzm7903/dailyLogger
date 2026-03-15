# Story 1.4: 错误处理与用户提示

Status: done

## Story

作为一个 DailyLogger 用户，
我希望在发生错误时能够看到明确、友好的提示信息，并支持网络错误重试和错误日志导出，
以便我能够快速定位问题并解决，提升应用的可靠性和用户体验。

## Acceptance Criteria

### AC1 - API 调用失败有明确错误信息
- Given 用户执行需要调用 AI API 的操作（如截图分析、日报生成）
- When API 调用失败（网络错误、认证失败、配额超限等）
- Then 显示明确、用户友好的错误信息，包括错误类型和建议操作

### AC2 - 网络错误支持重试
- Given 用户执行操作时发生网络错误
- When 错误提示显示后
- Then 提供重试按钮，点击后重新执行失败的操作

### AC3 - 错误日志可导出
- Given 用户需要排查问题或反馈 Bug
- When 点击导出日志按钮
- Then 将最近的应用日志导出为文件，便于分享或存档

### AC4 - 设置保存失败有明确提示
- Given 用户修改设置并点击保存
- When 保存失败（如无效的 API URL 格式）
- Then 显示具体错误原因，不自动关闭设置窗口

## Tasks / Subtasks

- [x] Task 1: 创建统一错误处理工具模块 (AC: 1)
  - [x] 创建 `src/utils/errors.js` 错误处理工具
  - [x] 定义 `ErrorType` 枚举（NETWORK, AUTH, QUOTA, VALIDATION, UNKNOWN）
  - [x] 实现 `parseError(error)` 函数解析错误类型
  - [x] 实现 `getErrorMessage(errorType)` 返回用户友好的错误消息
  - [x] 实现 `getSuggestedAction(errorType)` 返回建议操作

- [x] Task 2: 实现 Toast 通知组件 (AC: 1, 2)
  - [x] 创建 `src/components/Toast.vue` 通用通知组件
  - [x] 支持不同类型：success, error, warning, info
  - [x] 支持显示操作按钮（如重试）
  - [x] 支持自动消失和手动关闭
  - [x] 添加动画效果（滑入滑出）

- [x] Task 3: 创建全局状态管理 (AC: 1, 2)
  - [x] 创建 `src/stores/toast.js` Toast 状态管理
  - [x] 实现 `showToast(message, options)` 函数
  - [x] 实现 `showError(error, retryCallback)` 函数
  - [x] 实现队列管理，支持多个通知排队显示

- [x] Task 4: 重构 App.vue 错误处理 (AC: 1, 2)
  - [x] 替换现有的 captureError/summaryError 为统一 Toast 系统
  - [x] 为截图、分析、日报生成操作添加重试支持
  - [x] 使用 parseError 解析错误并显示友好消息

- [x] Task 5: 实现日志导出功能 (AC: 3)
  - [x] 添加 Rust 后端 `get_logs_for_export` 和 `get_log_file_path` 命令
  - [x] 使用 Tauri save dialog 选择导出位置
  - [x] 复制日志文件到用户指定路径
  - [x] 在设置界面添加"导出日志"按钮
  - [x] 显示导出成功/失败提示

- [x] Task 6: 改进设置保存错误提示 (AC: 4)
  - [x] 添加前端表单验证（API URL 格式、必填项检查）
  - [x] 后端返回结构化验证错误
  - [x] 在设置界面显示具体错误，不自动关闭

- [x] Task 7: 编写测试 (All ACs)
  - [x] 前端单元测试：parseError 函数
  - [x] 前端单元测试：Toast 组件渲染
  - [x] 前端单元测试：错误状态显示和重试
  - [x] Rust 单元测试：export_logs 命令
  - [x] 端到端测试：错误场景用户流程

## Dev Notes

### 技术需求

1. **前端框架** - Vue 3 Composition API + `<script setup>`
2. **状态管理** - 使用 reactive/ref 实现，无需引入 Pinia
3. **样式** - TailwindCSS only，无内联样式
4. **测试** - Vitest 前端测试 + Rust cargo test

### 架构合规要求

- 遵循现有组件结构（components/ 目录）
- 后端命令注册在 `main.rs` 的 `generate_handler![]`
- 错误消息使用中文（用户语言）
- 使用 Tauri dialog API 进行文件保存

### 现有实现分析

**当前错误处理问题：**
1. 错误仅 `console.error` + 简单红色文字显示
2. 无错误分类，用户难以理解问题原因
3. 无重试机制，用户需手动重新操作
4. 无日志导出，排查问题困难
5. 设置保存失败仍自动关闭窗口

**需要修改的文件：**
1. `src/App.vue` - 替换错误处理逻辑
2. `src/components/SettingsModal.vue` - 改进保存错误处理
3. `src-tauri/src/manual_entry/mod.rs` - 添加 export_logs 命令
4. `src-tauri/src/main.rs` - 注册新命令

### 错误类型与消息映射

| 错误类型 | 用户消息 | 建议操作 |
|---------|---------|---------|
| NETWORK | 网络连接失败，请检查网络设置 | 重试 |
| AUTH | API Key 无效或已过期 | 检查设置 |
| QUOTA | API 调用次数已达上限 | 检查账户 |
| VALIDATION | 输入内容格式不正确 | 修改输入 |
| UNKNOWN | 操作失败，请稍后重试 | 重试 |

### 错误解析规则

从错误字符串中识别类型：
- 包含 "network"、"timeout"、"ECONNREFUSED" → NETWORK
- 包含 "401"、"403"、"unauthorized"、"api key" → AUTH
- 包含 "429"、"rate limit"、"quota" → QUOTA
- 包含 "invalid"、"validation"、"format" → VALIDATION
- 其他 → UNKNOWN

### 文件结构要求

**新增文件：**
```
src/
├── utils/
│   └── errors.js           # 错误处理工具函数
├── stores/
│   └── toast.js            # Toast 状态管理
├── components/
│   └── Toast.vue           # Toast 通知组件

src-tauri/src/
├── manual_entry/
│   └── mod.rs              # 添加 export_logs 函数
```

**修改文件：**
```
src/App.vue                   # 替换错误处理
src/components/SettingsModal.vue  # 改进保存错误
src-tauri/src/main.rs         # 注册 export_logs 命令
```

### 测试要求

**前端测试重点：**
1. parseError 正确识别各类型错误
2. Toast 组件正确渲染各类型消息
3. 重试按钮点击后调用正确的回调
4. 设置保存失败不关闭窗口

**Rust 测试重点：**
1. export_logs 正确返回日志内容
2. 空日志文件处理

### Toast 组件设计

```vue
<!-- Toast.vue 设计规范 -->
<div class="fixed bottom-4 right-4 z-50">
  <div class="bg-dark border rounded-lg px-4 py-3 shadow-lg max-w-sm">
    <!-- 类型图标 -->
    <span v-if="type === 'error'" class="text-red-400">⚠️</span>
    <span v-if="type === 'success'" class="text-green-400">✓</span>
    <!-- 消息内容 -->
    <p class="text-sm text-gray-200">{{ message }}</p>
    <!-- 建议操作 -->
    <p v-if="suggestion" class="text-xs text-gray-400 mt-1">{{ suggestion }}</p>
    <!-- 操作按钮 -->
    <div v-if="retryCallback" class="mt-2 flex gap-2">
      <button @click="retryCallback">重试</button>
      <button @click="close">关闭</button>
    </div>
  </div>
</div>
```

## Previous Story Intelligence

### 从 CORE-001 学习的经验

1. **设置保存模式**：成功后 800ms 自动关闭，显示绿色勾号
2. **失败处理**：显示红色 ✗ 图标 + "保存失败" 文字
3. **Tailwind 类名**：`text-red-400` 用于错误文字，`bg-red-900/30` 用于错误背景
4. **hover 效果**：所有按钮添加 hover 状态

### 从 CORE-002 学习的经验

1. **视图状态管理**：使用 ref() 管理组件状态
2. **组件复用**：优先修改现有组件而非创建新组件
3. **错误处理模式**：console.error + 简单文字显示

### 从 CORE-003 学习的经验

1. **数据库迁移**：使用 ALTER TABLE 添加新字段
2. **测试模式**：每个 AC 对应多个测试用例
3. **代码风格**：遵循现有模块结构和命名规范

## Project Structure Notes

### 现有项目结构

```
src-tauri/src/
├── lib.rs                     # 应用入口
├── main.rs                    # Tauri 主进程，命令注册
├── manual_entry/
│   └── mod.rs                 # 手动输入模块（添加 export_logs）
├── memory_storage/
│   └── mod.rs                 # 数据存储
├── synthesis/
│   └── mod.rs                 # 日报生成
└── auto_perception/
    └── mod.rs                 # 自动感知

src/
├── App.vue                    # 主界面容器
├── components/
│   ├── SettingsModal.vue      # 设置模态框
│   ├── LogViewer.vue          # 日志查看器
│   └── ...
```

### 日志文件位置

```
~/.local/share/DailyLogger/logs/daily-logger.log
```

## References

- [Source: architecture.md#2.2 后端模块] - manual_entry 职责描述
- [Source: architecture.md#7. 文件系统] - 日志文件路径
- [Source: PRD.md#7.4 可用性要求] - 离线模式、自动重连、错误恢复
- [Source: epics.md#Epic 1] - 所属 Epic 信息
- [Source: src/App.vue] - 现有错误处理实现
- [Source: src/components/SettingsModal.vue] - 现有设置保存逻辑
- [Source: src-tauri/src/manual_entry/mod.rs] - get_recent_logs 实现
- [Source: CLAUDE.md] - 项目开发规范

## Dev Agent Record

### Agent Model Used

Claude glm-5

### Debug Log References

None - implementation proceeded smoothly following TDD approach.

### Completion Notes List

1. **Error Handling Utility**: Created `src/utils/errors.js` with comprehensive error type detection (NETWORK, AUTH, QUOTA, VALIDATION, UNKNOWN) and user-friendly Chinese messages.

2. **Toast Component**: Implemented `src/components/Toast.vue` with support for multiple toast types (success, error, warning, info), retry functionality, and smooth animations.

3. **Toast Store**: Created `src/stores/toast.js` with queue management, auto-dismiss for non-error toasts, and retry callback support.

4. **App.vue Refactoring**: Replaced inline error displays with unified Toast system, added retry support for screenshot, analysis, and summary operations.

5. **Log Export**: Added Rust commands `get_logs_for_export` and `get_log_file_path`, integrated with Tauri save dialog for file export.

6. **Settings Validation**: Added frontend form validation for API URL format, screenshot interval, and other numeric fields. Validation errors prevent auto-close of settings modal.

7. **Tests**: All 87 frontend tests pass, 39 Rust tests pass. Added new test files for error handling (`errors.spec.js`) and retry functionality (`errorRetry.spec.js`).

### File List

**New Files:**
- `src/utils/errors.js` - Error handling utility module
- `src/stores/toast.js` - Toast notification state management
- `src/components/Toast.vue` - Toast notification component
- `src/__tests__/errors.spec.js` - Error parsing unit tests
- `src/__tests__/Toast.spec.js` - Toast component unit tests
- `src/__tests__/errorRetry.spec.js` - Error retry functionality tests

**Modified Files:**
- `src/App.vue` - Integrated Toast system, removed inline error displays
- `src/components/SettingsModal.vue` - Added log export button, validation, separated save/export errors
- `src-tauri/src/manual_entry/mod.rs` - Added get_logs_for_export, get_log_file_path commands with tests
- `src-tauri/src/main.rs` - Registered new commands in generate_handler![]
- `package.json` - Added @tauri-apps/plugin-dialog and @tauri-apps/plugin-fs dependencies