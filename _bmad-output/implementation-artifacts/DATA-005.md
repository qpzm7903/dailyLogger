# Story 4.5: 数据备份与恢复

Status: passed

## Story

As a DailyLogger 用户,
I want 一键备份和恢复应用数据,
so that 我可以保护我的工作记录免受数据丢失风险，并在需要时恢复历史数据.

## Acceptance Criteria

1. **AC1: 备份到指定位置**
   - Given 用户选择备份功能, When 用户选择备份位置, Then 将数据库和截图打包备份到指定目录
   - 备份文件名格式: `dailylogger-backup-YYYY-MM-DD-HHMMSS.zip`
   - 备份内容包含: `local.db` 数据库文件 + `screenshots/` 目录

2. **AC2: 从备份恢复**
   - Given 用户选择恢复功能, When 用户选择备份文件, Then 验证备份完整性并恢复数据
   - 恢复前显示备份文件包含的记录数量和截图数量
   - 恢复过程中显示进度指示

3. **AC3: 恢复前自动备份**
   - Given 用户确认恢复操作, When 开始恢复前, Then 自动备份当前数据到临时位置
   - 恢复失败时自动回滚到备份状态
   - 临时备份保留 24 小时后自动清理

4. **AC4: 备份历史管理**
   - Given 用户打开备份管理界面, When 查看备份历史, Then 显示最近 10 个备份文件
   - 显示每个备份的时间、大小、记录数
   - 支持删除旧备份文件

## Tasks / Subtasks

- [x] Task 1: 后端备份 API (AC: 1, 2, 3)
  - [x] 1.1 创建 `src-tauri/src/backup/mod.rs` 模块
  - [x] 1.2 实现 `create_backup` 函数 - 打包数据库和截图
  - [x] 1.3 实现 `restore_backup` 函数 - 解压并恢复数据
  - [x] 1.4 实现 `list_backups` 函数 - 列出可用备份
  - [x] 1.5 实现 `get_backup_info` 函数 - 获取备份详情
  - [x] 1.6 实现 `delete_backup` 函数 - 删除备份文件
  - [x] 1.7 在 `main.rs` 注册新 Tauri 命令
  - [x] 1.8 在 `lib.rs` 导出模块
  - [x] 1.9 添加单元测试

- [x] Task 2: 前端备份组件 (AC: 1, 2, 4)
  - [x] 2.1 创建 `BackupModal.vue` 组件
  - [x] 2.2 实现备份位置选择 (使用 Tauri dialog)
  - [x] 2.3 实现备份进度显示
  - [x] 2.4 实现备份历史列表
  - [x] 2.5 实现恢复确认对话框 (显示备份详情)
  - [x] 2.6 实现恢复进度显示

- [x] Task 3: 集成入口 (AC: 全部)
  - [x] 3.1 在 App.vue 或 SettingsModal 添加备份管理入口
  - [x] 3.2 集成 BackupModal
  - [x] 3.3 添加组件测试

## Dev Notes

### 架构约束

1. **数据库操作**: 使用现有的 `DB_CONNECTION` 全局 Mutex，恢复时需要正确处理连接
2. **Tauri 命令**: 所有新命令必须在 `main.rs` 的 `generate_handler![]` 中注册
3. **前端风格**: 使用 TailwindCSS，遵循 `bg-dark`、`bg-darker`、`text-primary` 主题色
4. **文件操作**: 使用 `zip` crate 进行压缩/解压，确保跨平台兼容

### 关键代码参考

**应用数据目录** (`src-tauri/src/memory_storage/mod.rs:10-18`):
```rust
fn get_app_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("DailyLogger")
}

fn get_db_path() -> PathBuf {
    get_app_data_dir().join("data").join("local.db")
}
```

**截图目录** (`src-tauri/src/auto_perception/mod.rs`):
```rust
// 截图保存到: {app_data_dir}/screenshots/screenshot_YYYYMMDD_HHMMSS.png
```

**文件写入模式** (`src-tauri/src/synthesis/mod.rs:226-231`):
```rust
let output_dir = PathBuf::from(&obsidian_path);
std::fs::create_dir_all(&output_dir)
    .map_err(|e| format!("Failed to create output directory: {}", e))?;
let output_path = output_dir.join(&filename);
std::fs::write(&output_path, summary).map_err(|e| format!("Failed to write: {}", e))?;
```

### 新增 API 设计

**Rust 端** (`src-tauri/src/backup/mod.rs`):
```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 备份信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    pub path: String,              // 备份文件路径
    pub created_at: String,        // 创建时间 RFC3339
    pub size_bytes: u64,           // 文件大小
    pub record_count: usize,       // 记录数量
    pub screenshot_count: usize,   // 截图数量
}

/// 备份结果
#[derive(Debug, Serialize, Deserialize)]
pub struct BackupResult {
    pub path: String,              // 备份文件路径
    pub size_bytes: u64,           // 备份大小
    pub record_count: usize,       // 备份的记录数
    pub screenshot_count: usize,   // 备份的截图数
}

/// 恢复结果
#[derive(Debug, Serialize, Deserialize)]
pub struct RestoreResult {
    pub success: bool,
    pub record_count: usize,       // 恢复的记录数
    pub screenshot_count: usize,   // 恢复的截图数
    pub rolled_back: bool,         // 是否发生了回滚
}

/// 创建备份
#[command]
pub async fn create_backup(backup_dir: Option<String>) -> Result<BackupResult, String>

/// 获取备份信息
#[command]
pub async fn get_backup_info(backup_path: String) -> Result<BackupInfo, String>

/// 列出备份历史
#[command]
pub async fn list_backups() -> Result<Vec<BackupInfo>, String>

/// 恢复备份
#[command]
pub async fn restore_backup(backup_path: String) -> Result<RestoreResult, String>

/// 删除备份
#[command]
pub async fn delete_backup(backup_path: String) -> Result<(), String>
```

### 备份文件结构

```
dailylogger-backup-2026-03-14-103000.zip
├── manifest.json           // 元数据
│   {
│     "version": "1.0",
│     "created_at": "2026-03-14T10:30:00Z",
│     "record_count": 150,
│     "screenshot_count": 45
│   }
├── data/
│   └── local.db           // SQLite 数据库
└── screenshots/           // 截图目录
    ├── screenshot_20260314_090000.png
    └── ...
```

### 备份流程

```
用户点击"创建备份"
       ↓
选择备份位置 (可选，默认: Documents/DailyLogger/backups/)
       ↓
create_backup()
  1. 锁定 DB_CONNECTION
  2. 计算记录数和截图数
  3. 创建临时目录
  4. 复制 local.db 到临时目录
  5. 复制 screenshots/ 到临时目录
  6. 创建 manifest.json
  7. 打包为 zip
  8. 清理临时目录
  9. 返回 BackupResult
       ↓
前端显示成功信息 + 备份路径
```

### 恢复流程

```
用户点击"恢复备份"
       ↓
选择备份文件
       ↓
get_backup_info() → 显示备份详情
       ↓
用户确认恢复
       ↓
restore_backup()
  1. 创建当前数据的临时备份
  2. 关闭当前数据库连接
  3. 解压备份文件到临时目录
  4. 验证备份完整性
  5. 替换 local.db
  6. 替换 screenshots/
  7. 重新初始化数据库连接
  8. 验证恢复成功
  9. 清理临时文件
  10. 返回 RestoreResult
       ↓
失败时: 从临时备份回滚
成功时: 清理临时备份，显示成功信息
```

### 前端组件结构

```
BackupModal.vue
├─ Header: 备份管理 + 关闭按钮
├─ TabNavigation:
│  ├─ 备份 Tab
│  │  ├─ 备份位置选择 (默认: Documents/DailyLogger/backups/)
│  │  └─ 创建备份按钮
│  └─ 恢复 Tab
│     ├─ 选择备份文件按钮
│     ├─ 备份详情预览
│     └─ 确认恢复按钮
├─ BackupHistory:
│  └─ 最近备份列表 (时间、大小、记录数)
│     └─ 删除按钮
└─ ProgressDialog:
   ├─ 进度条
   └─ 状态文本
```

### 项目结构 Notes

- 新模块位置: `src-tauri/src/backup/mod.rs`
- 新组件位置: `src/components/BackupModal.vue`
- 后端修改: `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`
- 默认备份目录: `~/Documents/DailyLogger/backups/`
- 临时备份目录: `{app_data_dir}/temp-backup-{timestamp}/`
- 遵循现有命名规范: snake_case (Rust), PascalCase (Vue)

### 代码复用机会

1. **memory_storage 的 `get_app_data_dir`**: 复用应用数据目录获取逻辑
2. **memory_storage 的数据库路径**: 复用 `get_db_path()` 函数
3. **synthesis 模块的文件操作**: 复用目录创建和错误处理模式
4. **Tauri dialog 插件**: 用于文件/目录选择对话框

### 测试要求

**Rust 测试** (`backup/mod.rs`):
- 测试备份创建成功
- 测试备份文件包含正确内容
- 测试恢复成功
- 测试恢复失败回滚
- 测试备份信息解析
- 测试空数据备份
- 测试备份文件损坏时的错误处理

**前端测试** (Vitest):
- 组件挂载时加载备份历史
- 备份按钮点击触发备份
- 恢复确认对话框显示
- 进度显示状态更新

### 边界条件考虑

- 空数据库备份: 生成有效但内容为空的备份
- 大量截图: 显示压缩进度，考虑异步处理
- 备份文件损坏: 验证完整性，显示错误信息
- 磁盘空间不足: 检查可用空间，提示用户
- 恢复中途失败: 自动回滚到临时备份
- 跨版本备份: manifest 包含版本号，支持未来迁移

### 性能考虑

- 使用流式压缩避免内存溢出
- 大文件异步处理，不阻塞 UI
- 进度反馈提升用户体验
- 数据库恢复时使用事务确保一致性

### 错误处理

- 备份目录不存在: 自动创建
- 备份目录无写入权限: 返回错误，提示用户选择其他位置
- 恢复文件无效: 验证失败，显示具体错误
- 恢复过程中断: 自动回滚，保护数据完整性

### 依赖添加

需要在 `Cargo.toml` 添加:
```toml
zip = "0.6"  # 或更新版本，用于 zip 压缩/解压
tempfile = "3"  # 临时目录管理
```

### References

- [Source: architecture.md#5.1] 数据库 schema 和索引
- [Source: architecture.md#6] API 端点设计模式
- [Source: architecture.md#7] 文件系统结构
- [Source: memory_storage/mod.rs] 数据库路径和连接管理
- [Source: epics.md#Epic 4] 数据管理 Epic 上下文
- [Source: PRD.md#7.2] 安全要求 - 数据丢失风险

## Dev Agent Record

### Agent Model Used

MiniMax-M2.5

### Debug Log References

无

### Completion Notes List

- 创建 `src-tauri/src/backup/mod.rs` 模块，实现备份/恢复核心功能
- 添加 `create_backup` - 创建备份到指定目录
- 添加 `get_backup_info` - 获取备份信息
- 添加 `list_backups` - 列出备份历史
- 添加 `delete_backup` - 删除备份
- 添加 `restore_backup` - 恢复备份（支持回滚）
- 在 `main.rs` 注册 5 个 Tauri 命令
- 在 `lib.rs` 导出 backup 模块
- 创建 `src/components/BackupModal.vue` 组件（备份/恢复/历史三个 Tab）
- 在 `App.vue` 添加备份入口按钮

### File List

- src-tauri/src/backup/mod.rs (新增)
- src-tauri/src/lib.rs (修改 - 添加模块导出)
- src-tauri/src/main.rs (修改 - 注册命令)
- src-tauri/Cargo.toml (修改 - 添加依赖)
- src/components/BackupModal.vue (新增)
- src/App.vue (修改 - 添加备份按钮和组件)

## Code Review Findings

### Review Date: 2026-03-15
### Reviewer: Claude Code
### Status: PASSED with 1 fix

### Quality Gate Results
- **Rust Format**: ✓ PASSED
- **Rust Clippy**: ✓ PASSED (269 tests)
- **Frontend Tests**: ✓ PASSED (191 tests)

### Issues Found

1. **[Critical] restore_backup 后未重新初始化数据库连接**
   - **Severity**: High
   - **Location**: `src-tauri/src/backup/mod.rs:315`
   - **Description**: 恢复数据库后，应用仍然持有旧数据库文件的连接，导致恢复的数据无法被应用使用
   - **Fix Applied**: 在 `restore_backup` 函数中添加了数据库连接重置逻辑：
     1. 恢复前关闭现有连接 (`DB_CONNECTION = None`)
     2. 恢复后重新调用 `init_database()` 初始化连接
     3. 如果初始化失败，保留回滚备份以便恢复

### Summary
All acceptance criteria implemented correctly. The code follows project conventions and passes all quality gates. One critical bug was identified and fixed during review.

### AC Verification
- ✓ AC1: 备份到指定位置 - 已实现
- ✓ AC2: 从备份恢复 - 已实现（显示备份详情、进度）
- ✓ AC3: 恢复前自动备份 - 已实现（回滚机制）
- ✓ AC4: 备份历史管理 - 已实现（最近10个备份）