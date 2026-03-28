# ARCH-010 架构约束规范

**目标版本**: v3.6.0
**优先级**: P2
**类型**: 架构文档

## 背景

v3.6.0 是架构收口三期的最后阶段。前两期已完成：
- v3.4.0: 前端应用壳提取、IPC 调用边界统一
- v3.5.0: 后端 service 边界建立

本期需要建立明确的架构约束文档，防止架构劣化。

## 架构分层

```
┌─────────────────────────────────────────────────────────────┐
│                      Frontend (Vue 3)                        │
├─────────────────────────────────────────────────────────────┤
│  components/      - UI 组件，不直接调用 Tauri 命令           │
│  features/*/     - 业务动作，通过 feature actions 调用 IPC   │
│  shared/api/     - 统一 IPC Client，封装 invoke 调用        │
│  stores/         - 状态管理，通过 actions 修改状态           │
└─────────────────────────────────────────────────────────────┘
                            │ IPC (invoke)
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                    Tauri Backend (Rust)                      │
├─────────────────────────────────────────────────────────────┤
│  commands/       - 薄命令层，只做参数转换和错误映射          │
│  services/       - 业务逻辑，不直接处理 Tauri 上下文        │
│  infrastructure/ - 公共基础设施 (state, errors)              │
│  */              - 领域模块 (memory_storage, synthesis...)  │
└─────────────────────────────────────────────────────────────┘
```

## 硬性约束

### 1. 命令层约束

**规则**: `commands/` 目录下的 `#[tauri::command]` 函数必须满足：
- 只做参数验证和转换
- 调用对应的 service 函数
- 将 service 返回的错误映射为用户可读的消息
- **禁止**在命令层编写业务逻辑

**示例**:
```rust
// ✅ 正确：薄命令层
#[tauri::command]
pub async fn get_settings() -> Result<Settings, String> {
    get_settings_service().map_err(|e| format!("Failed to get settings: {}", e))
}

// ❌ 错误：命令层包含业务逻辑
#[tauri::command]
pub async fn get_settings() -> Result<Settings, String> {
    let db = get_db_connection()?;  // 业务逻辑！
    let settings = query_settings(&db)?;  // 业务逻辑！
    Ok(settings)
}
```

### 2. Service 层约束

**规则**: `services/` 目录下的模块必须满足：
- 包含纯业务逻辑，不直接处理 Tauri 命令上下文
- 返回 `Result<T, AppError>` 或 `Result<T, String>`
- 不直接调用 `invoke()` 或处理 IPC
- 编写单元测试时可以用 mock

### 3. 前端 IPC 约束

**规则**: `components/` 下的 Vue 组件必须满足：
- **禁止**直接调用 `invoke()`
- 使用 `feature actions` 封装业务动作
- 通过 `stores/` 管理状态，不直接修改共享状态

**示例**:
```typescript
// ✅ 正确：使用 feature actions
import { sessionActions } from '@/features/sessions/actions'
const sessions = await sessionActions.getTodaySessions()

// ❌ 错误：组件直接调用 invoke
import { invoke } from '@tauri-apps/api/tauri'
const sessions = await invoke('get_today_sessions')
```

### 4. 全局状态约束

**规则**:
- 模块私有状态使用 `Lazy<Mutex<...>>`，配合 accessor 函数
- 跨模块共享状态添加到 `AppState` (lib.rs)
- 新增全局状态前必须先问：能否作为参数传递？能否限制在单模块内？

参见: `src-tauri/src/infrastructure/state.rs`

### 5. 错误处理约束

**规则**:
- Service 层优先返回 `Result<T, AppError>`
- 命令层将 `AppError` 转换为用户可读消息
- 前端使用 `createErrorInfo()` 处理错误，根据错误类型显示不同 UI

参见: `src-tauri/src/errors.rs`, `src/utils/errors.ts`

## 新增功能检查清单

新增功能时，确保满足以下检查：

- [ ] 后端逻辑写在 `services/` 而不是 `commands/`
- [ ] 前端业务动作写在 `features/*/actions.ts`
- [ ] 组件不直接调用 `invoke()`
- [ ] 全局状态变化有明确来源
- [ ] 错误使用 `AppError` 类型

## 架构守护

以下情况属于架构违规，应在 PR 中指出：

1. **命令层业务化**: `commands/` 下的函数出现数据库查询、业务计算
2. **组件直接 IPC**: Vue 组件中出现 `invoke()` 调用
3. **散落的全局状态**: 新增 `Lazy<Mutex<...>>` 前未评估是否必须
4. **字符串错误**: Service 层返回 `Err(String)` 而非 `Err(AppError)`

## 文档维护

- 本文档与 ARCH-001 保持一致
- 架构规则更新需同步到 `specs/ARCH-010.md`
- 违反约束需要有合理的例外记录
