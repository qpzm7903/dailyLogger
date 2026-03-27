# Story 11.3: STAB-001 - 错误边界与优雅降级

Status: in-progress

## Story

作为 DailyLogger 用户，
I want 在应用遇到错误时得到清晰的错误提示并能继续使用核心功能，
so that 即使出现网络故障、AI 服务异常或数据库问题时，我的工作记录不会丢失，应用不会崩溃。

## Acceptance Criteria

1. **AC1: 全局错误边界**
   - Given 应用运行中，When 发生未捕获的 Rust 恐慌或前端 JavaScript 错误，Then 应用显示友好的错误提示而非崩溃
   - 错误提示显示错误类型、时间和简短说明
   - 用户可关闭错误提示并继续使用

2. **AC2: AI 服务降级**
   - Given AI 服务不可用（网络错误、超时、API 错误），When 用户触发需要 AI 的操作（如分析、生成日报），Then 显示友好的错误提示、保留用户数据、并提供重试选项
   - 不因 AI 错误而丢失已捕获的截图
   - 自动重试机制：网络恢复后提示用户是否重试失败的请求

3. **AC3: 网络状态感知**
   - Given 网络断开，When 用户执行需要网络的操作，Then 提示用户当前处于离线状态并提供离线可用功能
   - 离线状态下：截图、本地记录查看功能正常
   - 网络恢复后：自动检测并提示用户恢复同步

4. **AC4: 数据库错误恢复**
   - Given 数据库发生错误（如写入失败、连接断开），When 操作失败，Then 保留操作数据、提示用户错误、提供数据导出选项
   - 防止数据库损坏：异常时回滚事务
   - 提供手动备份触发按钮

5. **AC5: 截图捕获失败处理**
   - Given 截图捕获失败（如权限问题、屏幕不存在），When 捕获失败，Then 记录错误日志、跳过本次捕获、不影响后续捕获
   - 用户提示：静默处理或可选显示通知

6. **AC6: 错误日志记录**
   - Given 任何错误发生，When 错误被捕获，Then 记录到日志文件（包含时间戳、错误类型、堆栈跟踪）
   - 前端错误同时记录到 `logs/` 目录
   - 提供日志查看入口（在设置或日志面板中）

## Tasks / Subtasks

- [x] Task 1: 全局错误边界与 panic 处理 (AC: #1)
  - [x] 1.1 在 Rust 后端设置全局 panic 处理 hook，捕获未处理恐慌 (main.rs:311-314)
  - [x] 1.2 在前端设置 Vue 错误边界组件 (ErrorBoundary.vue)
  - [x] 1.3 创建全局错误展示组件 (ErrorToast.vue)
  - [x] 1.4 添加 Rust panic 钩子测试 (lib.rs:421-512)

- [ ] Task 2: AI 服务降级与重试机制 (AC: #2)
  - [x] 2.1 分析 `synthesis/mod.rs` 和 `session_manager/mod.rs` 的 AI 调用点
  - [x] 2.2 为 AI 调用添加错误处理和降级逻辑
  - [x] 2.3 实现自动重试机制（指数退避）(synthesis: call_llm_api_with_retry, session_manager: call_vision_api_batch_with_retry)
  - [x] 2.4 添加重试队列状态管理
  - [x] 2.5 添加 AI 降级场景测试

- [ ] Task 3: 网络状态感知 (AC: #3)
  - [ ] 3.1 在前端添加网络状态监听 (online/offline 事件)
  - [ ] 3.2 创建网络状态指示器组件 (NetworkStatusIndicator.vue 已创建)
  - [ ] 3.3 根据网络状态禁用/启用需要网络的功能
  - [ ] 3.4 网络恢复时提示用户

- [ ] Task 4: 数据库错误恢复 (AC: #4)
  - [ ] 4.1 在 `memory_storage/mod.rs` 的写入操作中添加事务回滚
  - [ ] 4.2 添加数据库连接断开重连逻辑
  - [ ] 4.3 提供手动数据库备份命令 (backup/mod.rs 已存在)
  - [ ] 4.4 添加数据库错误场景测试

- [ ] Task 5: 截图失败处理 (AC: #5)
  - [ ] 5.1 在 `auto_perception/mod.rs` 添加截图失败处理
  - [ ] 5.2 添加权限错误检测和用户提示
  - [ ] 5.3 添加截图失败日志记录

- [x] Task 6: 错误日志系统 (AC: #6)
  - [x] 6.1 配置 Rust `tracing` 记录到文件 (main.rs:setup_logging)
  - [x] 6.2 前端错误捕获并写入日志 (manual_entry/mod.rs: log_frontend_error)
  - [x] 6.3 创建日志查看器组件 (LogViewer.vue 已有，扩展错误日志展示)
  - [x] 6.4 添加日志轮转配置 (main.rs: RollingFileAppender with max_log_files(7))

- [ ] Task 7: 端到端测试 (AC: All)
  - [x] 7.1 添加 Rust 错误处理集成测试 (lib.rs panic tests, synthesis retry tests)
  - [ ] 7.2 添加前端错误边界组件测试
  - [ ] 7.3 添加网络状态切换测试

## Dev Notes

### Architecture Context

**关键架构决策**:
- 复用现有 `tracing` crate 进行日志记录
- 错误处理遵循 `Result<T, String>` 模式
- 前端使用 Vue 3 Error Boundary 模式
- 降级操作使用状态机管理

**必须遵循的代码模式** [Source: architecture.md]:
- Tauri Command: `#[command]` + async
- 错误处理: `Result<T, String>` + `.map_err(|e| e.to_string())`
- 数据库访问: 使用全局 `DB_CONNECTION` Mutex
- 事务处理: 使用 `rusqlite` 事务 API

### Key Existing Code to Reuse

**auto_perception/mod.rs** - 复用以下函数:
- `capture_screen()` - 截图捕获入口
- `start_auto_capture()` - 自动捕获循环

**session_manager/mod.rs** - 分析以下调用点:
- `analyze_session()` - AI 分析入口，需要添加降级
- `manual_analyze_session()` - 手动分析入口

**synthesis/mod.rs** - 分析以下调用点:
- `generate_daily_summary()` - 日报生成入口，需要添加降级

**memory_storage/mod.rs** - 复用以下函数:
- `add_record()` - 记录添加，需要事务保护
- `init_database()` - 数据库初始化

**已有组件**:
- `LogViewer.vue` - 日志查看器（扩展以支持错误日志）

### Error Handling Patterns

```rust
// Rust 错误处理模式
#[command]
pub async fn some_operation() -> Result<String, String> {
    let result = do_something().await.map_err(|e| e.to_string())?;
    Ok(result)
}

// Panic Hook 示例
std::panic::set_hook(Box::new(|panic_info| {
    let msg = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
        s.clone()
    } else {
        "Unknown panic".to_string()
    };
    tracing::error!("Application panic: {}", msg);
}));

// 事务回滚示例
conn.execute("BEGIN TRANSACTION", []).map_err(|e| e.to_string())?;
let result = conn.execute("INSERT INTO ...", params).map_err(|e| {
    let _ = conn.execute("ROLLBACK", []);
    e.to_string()
});
```

### Frontend Component Structure

```
ErrorBoundary.vue (wraps App content)
├── ErrorToast.vue (非阻塞通知)
├── ErrorModal.vue (阻塞错误展示)
├── NetworkStatusIndicator.vue (网络状态指示器)
└── LogViewer.vue (已有，扩展错误日志)

App.vue
├── ErrorBoundary (wrapper)
├── Dashboard
├── SettingsModal
├── ...
└── NetworkStatusIndicator
```

### Project Structure Notes

**需要创建的文件**:
- `src/components/ErrorBoundary.vue` - 前端错误边界组件
- `src/components/ErrorToast.vue` - 错误通知组件
- `src/components/NetworkStatusIndicator.vue` - 网络状态指示器

**需要修改的文件**:
- `src-tauri/src/lib.rs` - 添加 panic hook
- `src-tauri/src/auto_perception/mod.rs` - 添加截图失败处理
- `src-tauri/src/session_manager/mod.rs` - 添加 AI 调用降级
- `src-tauri/src/synthesis/mod.rs` - 添加 AI 调用降级
- `src-tauri/src/memory_storage/mod.rs` - 添加事务保护
- `src/App.vue` - 添加 ErrorBoundary wrapper
- `src/components/LogViewer.vue` - 扩展错误日志支持

### Testing Requirements

**必须测试的场景**:
1. Panic 捕获：验证 panic 被正确捕获并记录
2. AI 超时：验证超时后显示友好错误而非崩溃
3. AI API 错误：验证 API 错误码被正确处理
4. 网络断开：验证 offline 事件触发并显示状态
5. 网络恢复：验证 online 事件触发并提示用户
6. 数据库写入失败：验证事务回滚且数据不损坏
7. 截图权限拒绝：验证跳过本次捕获并记录日志

**测试模式** (参考现有测试):
```rust
#[test]
fn panic_hook_catches_unwind() {
    // 设置 panic hook
    // 触发 panic
    // 验证 hook 被调用
}

#[test]
fn database_transaction_rollback_on_error() {
    // 开始事务
    // 执行会失败的写入
    // 验证回滚
    // 验证数据未损坏
}
```

```typescript
// 前端错误边界测试
describe('ErrorBoundary.vue', () => {
  it('catches child component errors');
  it('displays error message');
  it('allows user to dismiss error');
});
```

### Previous Story Intelligence

从 DATA-008 学到的经验:
- Tasks/Subtasks 必须标记为 `[x]` 表示完成，否则会造成状态混淆
- 测试文件路径使用 `__tests__` 目录
- 前端组件使用 `ref()` 响应式状态管理
- 遵循现有的错误处理模式 `Result<T, String>`

从 DATA-007 学到的经验:
- 数据库 ALTER TABLE 添加字段时使用 `#[derive(Default)]`
- 使用 `chrono::Local` 和 `and_local_timezone()` 处理时区

### References

- [Source: architecture.md#4.1] - 全局状态管理（错误处理基础）
- [Source: architecture.md#5.2] - settings 表结构
- [Source: PRD.md#7.4] - 可用性要求（错误恢复、离线模式）
- [Source: PRD.md#8] - 技术约束
- [Source: epics.md#Epic 11] - 数据增强与稳定性 Epic
- [Source: DATA-008 story] - 组件开发和测试模式参考

## Dev Agent Record

### Agent Model Used
minimax-m2.7-highspeed

### Debug Log References

### Completion Notes List
- Task 1 完成: Panic hook 已存在于 main.rs，添加了 ErrorBoundary.vue 和 ErrorToast.vue 组件，添加了 panic hook 测试
- Task 2 完成: 在 synthesis/mod.rs 和 session_manager/mod.rs 中实现了自动重试机制 (exponential backoff with jitter)，添加了 is_retryable_error 和 calculate_retry_delay 辅助函数
- Task 6 完成: tracing 已配置为写入文件，日志轮转已配置 (7天)，添加了 log_frontend_error 命令用于前端错误日志记录

### File List
- src/components/ErrorBoundary.vue (新建)
- src/components/ErrorToast.vue (新建)
- src/components/NetworkStatusIndicator.vue (新建)
- src-tauri/src/manual_entry/mod.rs (添加 log_frontend_error 命令)
- src-tauri/src/main.rs (添加 log_frontend_error 到 invoke handler)
- src-tauri/src/lib.rs (添加 panic hook 测试)
- src-tauri/src/synthesis/mod.rs (添加重试逻辑)
- src-tauri/src/session_manager/mod.rs (添加重试逻辑)

## Change Log

- 2026-03-27: 实现 Task 1 (全局错误边界), Task 2 (AI 重试机制), Task 6 (错误日志系统)
  - 添加 ErrorBoundary.vue, ErrorToast.vue, NetworkStatusIndicator.vue 组件
  - 在 synthesis 和 session_manager 中实现指数退避重试机制
  - 添加 log_frontend_error 命令记录前端错误
  - 添加 panic hook 和重试逻辑测试

