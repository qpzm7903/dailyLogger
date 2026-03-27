# Story 11.4: STAB-002 - 自动备份与恢复

Status: ready-for-dev

## Story

作为 DailyLogger 用户，
I want 系统自动定期备份我的数据，
so that 即使发生意外情况（如硬盘故障、误操作），我也不会丢失重要的工作记录和截图。

## Acceptance Criteria

1. **AC1: 自动备份开关**
   - Given 用户在设置中启用了自动备份，When 应用运行时，Then 系统按设定的时间间隔自动执行备份
   - Given 用户在设置中禁用了自动备份，When 应用运行时，Then 系统不执行自动备份
   - 自动备份设置位于设置界面的"数据备份"区域

2. **AC2: 自动备份间隔配置**
   - Given 自动备份已启用，When 用户设置了备份间隔（如每天、每周），Then 系统按设定间隔执行备份
   - 支持的间隔选项：每天、每周、每月
   - 默认间隔为每天

3. **AC3: 自动备份保留策略**
   - Given 自动备份已启用，When 备份数量超过保留上限，Then 系统自动删除最旧的备份
   - 默认保留最近 5 个自动备份
   - 用户可在设置中配置保留数量（3-20 个）

4. **AC4: 自动备份状态展示**
   - Given 自动备份已启用，When 备份执行时，Then 在设置界面显示最近一次备份的时间和状态
   - 备份成功时显示"备份成功"和备份时间
   - 备份失败时显示"备份失败"和错误摘要

5. **AC5: 备份过程不影响应用正常运行**
   - Given 自动备份正在执行，When 用户正在进行其他操作，Then 备份在后台执行，不阻塞主线程
   - 截图捕获、AI 分析等核心功能不受影响

6. **AC6: 与手动备份共存**
   - Given 用户同时使用手动备份和自动备份，When 系统执行自动备份，Then 不影响手动备份创建的备份
   - 手动备份不计入自动备份保留数量限制

## Tasks / Subtasks

- [ ] Task 1: 自动备份设置 UI (AC: #1, #2, #3, #4)
  - [ ] 1.1 在 BasicSettings.vue 中添加"自动备份"开关和配置区域
  - [ ] 1.2 添加备份间隔选择器（每天/每周/每月）
  - [ ] 1.3 添加保留数量配置输入框（3-20）
  - [ ] 1.4 显示最近一次自动备份的状态和时间
  - [ ] 1.5 添加 i18n 翻译（zh-CN.json, en.json）

- [ ] Task 2: 后端自动备份调度器 (AC: #1, #2, #5)
  - [ ] 2.1 在 `backup/mod.rs` 中添加 `AutoBackupConfig` 结构体（启用状态、间隔、保留数）
  - [ ] 2.2 添加备份任务调度器模块 `auto_backup_scheduler.rs`
  - [ ] 2.3 实现基于间隔的定时触发逻辑（使用 tokio 的 interval 或定时器）
  - [ ] 2.4 在 `lib.rs` 中初始化调度器并注册为后台任务
  - [ ] 2.5 添加启动时检查：如果距上次备份超过设定间隔，立即执行一次备份

- [ ] Task 3: 自动备份保留策略 (AC: #3)
  - [ ] 3.1 在 `backup/mod.rs` 中添加 `cleanup_old_auto_backups()` 函数
  - [ ] 3.2 备份完成后调用清理函数，只清理标记为"自动备份"的文件
  - [ ] 3.3 修改备份文件命名：添加 `auto-` 前缀区分手动和自动备份
  - [ ] 3.4 在 `BackupInfo` 中添加 `is_auto: bool` 字段

- [ ] Task 4: 设置持久化 (AC: #1, #2, #3)
  - [ ] 4.1 在 `settings` 表中添加自动备份相关字段
  - [ ] 4.2 修改 `save_settings` / `get_settings` 命令以支持新字段
  - [ ] 4.3 在 Rust 端读取和应用这些设置

- [ ] Task 5: 集成测试 (AC: All)
  - [ ] 5.1 添加自动备份调度器的 Rust 单元测试
  - [ ] 5.2 添加备份保留策略的 Rust 单元测试
  - [ ] 5.3 添加前端自动备份设置的 Vue 组件测试

## Dev Notes

### Architecture Context

**关键架构决策**:
- 复用现有的 `backup/mod.rs` 模块（已实现手动备份的核心逻辑）
- 使用 tokio 的定时器功能实现调度，不引入额外依赖
- 前端使用 Vue 3 Composition API + TailwindCSS（符合现有模式）
- 设置通过 SQLite settings 表持久化（符合现有架构）

**必须遵循的代码模式** [Source: architecture.md]:
- Tauri Command: `#[command]` + async
- 错误处理: `Result<T, String>` + `.map_err(|e| e.to_string())`
- 数据库访问: 使用全局 `DB_CONNECTION` Mutex
- 事务处理: 使用 `rusqlite` 事务 API
- 前端状态管理: 使用 `ref()` 响应式变量

### Key Existing Code to Reuse

**backup/mod.rs** - 复用以下函数:
- `create_backup()` - 创建备份的核心逻辑
- `list_backups()` - 列出备份历史
- `delete_backup()` - 删除备份文件
- `get_default_backup_dir()` - 获取默认备份目录

**BackupModal.vue** - 参考以下设计模式:
- UI 布局和样式
- 与后端 `create_backup`, `list_backups`, `delete_backup` 命令的交互
- 格式化函数（formatSize, formatDate）

**settings 表** - 扩展以下字段:
- 添加 `auto_backup_enabled: INTEGER DEFAULT 0`
- 添加 `auto_backup_interval: TEXT DEFAULT 'daily'`
- 添加 `auto_backup_retention: INTEGER DEFAULT 5`
- 添加 `last_auto_backup_at: TEXT`

### Project Structure Notes

**需要创建的文件**:
- `src-tauri/src/auto_backup_scheduler.rs` - 自动备份调度器模块

**需要修改的文件**:
- `src-tauri/src/backup/mod.rs` - 添加自动备份标记和清理逻辑
- `src-tauri/src/lib.rs` - 初始化自动备份调度器
- `src-tauri/src/main.rs` - 确保调度器在启动时启动
- `src/components/settings/BasicSettings.vue` - 添加自动备份设置 UI
- `src/locales/zh-CN.json` - 添加中文翻译
- `src/locales/en.json` - 添加英文翻译
- `src-tauri/src/memory_storage/mod.rs` - 添加 settings 字段读取

**不影响现有手动备份**:
- 手动备份（BackupModal.vue）完全独立工作
- 自动备份使用不同文件名前缀（`auto-`）以区分
- 保留策略只清理自动备份，不影响手动备份

### Testing Requirements

**必须测试的场景**:
1. 自动备份启用/禁用切换正常工作
2. 不同备份间隔（每天/每周/每月）的调度逻辑
3. 保留数量限制正确执行
4. 自动备份在后台执行不阻塞主操作
5. 备份失败时不影响应用正常运行
6. 与手动备份共存时互不影响

**测试模式** (参考现有测试):
```rust
#[test]
fn auto_backup_cleanup_preserves_manual_backups() {
    // 创建手动备份和自动备份
    // 执行清理逻辑
    // 验证手动备份被保留，自动备份按策略删除
}

#[test]
fn backup_interval_calculates_next_run_correctly() {
    // 设置间隔为 daily
    // 验证时间计算正确
}
```

```typescript
// Vue 组件测试
describe('AutoBackupSettings.vue', () => {
  it('enables auto backup when toggle is on');
  it('disables auto backup when toggle is off');
  it('updates interval when dropdown changes');
  it('displays last backup time when available');
});
```

### Previous Story Intelligence

从 STAB-001 学到的经验:
- 前端组件使用 `ref()` 响应式状态管理
- 遵循现有的错误处理模式 `Result<T, String>`
- Tasks/Subtasks 必须标记为 `[x]` 表示完成，否则会造成状态混淆
- 前端组件测试使用 Vitest + @vue/test-utils

从 DATA-005 学到的经验:
- 恢复前自动备份已实现（rollback 机制）
- 使用 manifest.json 存储备份元数据

### References

- [Source: architecture.md#5.2] - settings 表结构
- [Source: architecture.md#6] - Tauri Commands 表
- [Source: backup/mod.rs] - 现有备份模块完整实现
- [Source: BackupModal.vue] - 备份 UI 组件参考
- [Source: STAB-001.md] - 前一个 Story 的实现模式
- [Source: epics.md#Epic 11] - 数据增强与稳定性 Epic

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List
