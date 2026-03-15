# Story 1.5: 系统托盘菜单完善

Status: done

## Story

作为一个 DailyLogger 用户，
我希望系统托盘菜单能显示自动捕获状态、支持快速记录和直接打开 Obsidian 文件夹，
以便我无需打开主窗口即可完成常用操作，提升使用效率。

## Acceptance Criteria

### AC1 - 托盘菜单显示启动/停止自动捕获状态
- Given 自动捕获正在运行
- When 用户右键点击托盘图标
- Then 托盘菜单显示"停止自动捕获"选项（带状态指示）
- Given 自动捕获未运行
- When 用户右键点击托盘图标
- Then 托盘菜单显示"启动自动捕获"选项

### AC2 - 支持快速记录（无需打开主窗口）
- Given 用户想要快速记录想法
- When 用户点击托盘菜单中的"快速记录"选项
- Then 弹出独立的速记输入窗口（小窗口），无需打开主窗口
- When 用户输入内容并保存
- Then 记录存入数据库，窗口自动关闭

### AC3 - 支持直接打开 Obsidian 文件夹
- Given 用户已配置 Obsidian 路径
- When 用户点击托盘菜单中的"打开 Obsidian 文件夹"选项
- Then 使用系统文件管理器打开该目录
- Given 用户未配置 Obsidian 路径
- When 用户点击该选项
- Then 显示提示"请先在设置中配置 Obsidian 路径"

## Tasks / Subtasks

- [x] Task 1: 实现动态托盘菜单更新 (AC: 1)
  - [x] 添加 Rust 后端 `get_auto_capture_status` 命令
  - [x] 创建托盘菜单状态管理结构
  - [x] 实现菜单项动态文本更新（启动/停止）
  - [x] 添加菜单状态指示图标或文字

- [x] Task 2: 实现快速记录窗口功能 (AC: 2)
  - [x] 添加 Rust 后端 `tray_quick_note` 命令
  - [x] 创建独立的快速记录窗口（非主窗口）
  - [x] 实现小窗口 UI：文本输入框 + 保存按钮
  - [x] 支持 Enter 快捷键保存
  - [x] 保存后自动关闭窗口

- [x] Task 3: 实现打开 Obsidian 文件夹功能 (AC: 3)
  - [x] 添加 Rust 后端 `open_obsidian_folder` 命令
  - [x] 从 settings 读取 obsidian_path
  - [x] 使用系统默认文件管理器打开目录
  - [x] 处理路径未配置的情况

- [x] Task 4: 重构托盘菜单结构 (AC: 1, 2, 3)
  - [x] 使用 CheckMenuItem 实现状态切换
  - [x] 组织菜单项顺序：状态 → 分隔线 → 快速记录 → 打开文件夹 → 分隔线 → 显示窗口 → 退出
  - [x] 实现菜单事件处理逻辑

- [x] Task 5: 编写测试 (All ACs)
  - [x] Rust 单元测试：get_auto_capture_status 命令
  - [x] Rust 单元测试：tray_quick_note 命令
  - [x] Rust 单元测试：open_obsidian_folder 命令
  - [x] 边界测试：路径不存在、路径为空

## Dev Notes

### 技术需求

1. **Tauri v2 托盘 API** - 使用 `tauri::tray` 模块
2. **动态菜单** - 使用 `Menu::with_items` 和 `MenuItem::with_id`
3. **文件系统操作** - 使用 `std::process::Command` 打开目录
4. **测试** - Rust cargo test

### 架构合规要求

- 后端命令注册在 `main.rs` 的 `generate_handler![]`
- 使用 `memory_storage::get_settings_sync()` 获取设置
- 使用 `AUTO_CAPTURE_RUNNING` 原子变量获取捕获状态
- 错误消息使用中文

### 现有实现分析

**当前托盘功能（main.rs:79-121）：**
```rust
let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
let show = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
let menu = Menu::with_items(app, &[&show, &quit])?;
```

**当前问题：**
1. 菜单只有两个选项（显示窗口、退出）
2. 无法显示自动捕获状态
3. 无快速记录功能
4. 无打开文件夹功能

**自动捕获状态获取方式：**
```rust
// src-tauri/src/auto_perception/mod.rs:9
static AUTO_CAPTURE_RUNNING: AtomicBool = AtomicBool::new(false);

// 使用方式
AUTO_CAPTURE_RUNNING.load(Ordering::SeqCst)
```

**Obsidian 路径获取方式：**
```rust
// src-tauri/src/memory_storage/mod.rs
let settings = get_settings_sync()?;
let obsidian_path = settings.obsidian_path;
```

### 关键技术实现

#### 1. 动态菜单更新

Tauri v2 支持运行时更新菜单项文本：
```rust
use tauri::menu::{Menu, MenuItem, CheckMenuItem};

// 创建可切换状态的菜单项
let capture_toggle = MenuItem::with_id(
    app,
    "capture_toggle",
    if running { "停止自动捕获" } else { "启动自动捕获" },
    true,
    None::<&str>
)?;
```

**注意**：Tauri v2 的 MenuItem 在创建后无法直接修改文本。
**解决方案**：使用 `tray.set_menu()` 在状态变化时重建菜单，或使用 `CheckMenuItem`。

#### 2. 快速记录窗口

方案 A：使用独立的 WebviewWindow（推荐）
```rust
use tauri::WebviewWindowBuilder;

// 创建小型速记窗口
let quick_note_window = WebviewWindowBuilder::new(
    app,
    "quick-note",
    WebviewUrl::App("quick-note.html".into())
)
.title("快速记录")
.inner_size(400.0, 200.0)
.resizable(false)
.build()?;
```

方案 B：弹出主窗口的简化视图
- 复用主窗口，发送事件切换视图模式
- 更简单但需要修改前端状态管理

#### 3. 打开文件夹

跨平台打开目录：
```rust
#[cfg(target_os = "windows")]
std::process::Command::new("explorer")
    .arg(&path)
    .spawn()?;

#[cfg(target_os = "macos")]
std::process::Command::new("open")
    .arg(&path)
    .spawn()?;

#[cfg(target_os = "linux")]
std::process::Command::new("xdg-open")
    .arg(&path)
    .spawn()?;
```

### 文件结构要求

**修改文件：**
```
src-tauri/src/
├── main.rs                    # 重构托盘菜单，注册新命令
├── lib.rs                     # 可能添加辅助函数
├── auto_perception/
│   └── mod.rs                 # 添加 is_auto_capture_running 公开函数
└── manual_entry/
    └── mod.rs                 # 添加 tray_quick_note 命令

src/
├── quick-note.html            # 快速记录窗口页面（如果使用独立窗口）
└── QuickNoteWindow.vue        # 快速记录组件
```

### 测试要求

**Rust 测试重点：**
1. `is_auto_capture_running` 返回正确状态
2. `tray_quick_note` 正确保存记录
3. `open_obsidian_folder` 正确处理各种路径情况

**边界测试：**
1. 路径为 None 或空字符串
2. 路径指向不存在的目录
3. 路径包含特殊字符或空格

### 托盘菜单设计

```
┌─────────────────────┐
│ ● 自动捕获运行中    │  ← 状态显示（动态切换）
│   停止自动捕获      │  ← 点击切换
├─────────────────────┤
│ 快速记录...         │  ← 打开速记窗口
│ 打开 Obsidian 文件夹│  ← 打开目录
├─────────────────────┤
│ 显示窗口            │
│ 退出                │
└─────────────────────┘
```

**状态指示方案：**
- 运行中：菜单项文字前加 "●" 或 "▶"
- 已停止：菜单项文字前加 "○" 或 "■"

## Previous Story Intelligence

### 从 CORE-001 学习的经验

1. **设置保存模式**：成功后 800ms 自动关闭，显示绿色勾号
2. **错误处理**：使用 Toast 组件显示错误
3. **Tailwind 类名**：`text-red-400` 用于错误文字

### 从 CORE-002 学习的经验

1. **组件复用**：优先修改现有组件而非创建新组件
2. **状态管理**：使用 ref() 管理组件状态

### 从 CORE-003 学习的经验

1. **数据库迁移**：使用 ALTER TABLE 添加新字段
2. **测试模式**：每个 AC 对应多个测试用例

### 从 CORE-004 学习的经验

1. **错误处理模式**：使用 `src/utils/errors.js` 解析错误类型
2. **Toast 组件**：`src/components/Toast.vue` 支持重试功能
3. **状态管理**：`src/stores/toast.js` 管理通知队列

## Project Structure Notes

### 现有项目结构

```
src-tauri/src/
├── lib.rs                     # 应用入口，APP_STATE
├── main.rs                    # Tauri 主进程，托盘定义
├── auto_perception/
│   └── mod.rs                 # 自动感知（AUTO_CAPTURE_RUNNING）
├── manual_entry/
│   └── mod.rs                 # 手动输入（add_quick_note）
├── memory_storage/
│   └── mod.rs                 # 数据存储（Settings）
└── synthesis/
    └── mod.rs                 # 日报生成

src/
├── App.vue                    # 主界面容器
├── components/
│   ├── QuickNoteModal.vue     # 速记模态框（可参考）
│   └── ...
```

### 关键依赖

- `tauri = { version = "2", features = ["tray-icon"] }` - 已启用
- `tauri-plugin-shell` - 已安装，可用于打开目录

## References

- [Source: architecture.md#2.2 后端模块] - auto_perception 职责描述
- [Source: architecture.md#7. 文件系统] - Obsidian 路径
- [Source: PRD.md#6.5 系统托盘] - 托盘功能需求
- [Source: epics.md#Epic 1] - 所属 Epic 信息
- [Source: src-tauri/src/main.rs:79-121] - 现有托盘实现
- [Source: src-tauri/src/auto_perception/mod.rs:9] - AUTO_CAPTURE_RUNNING
- [Source: src-tauri/src/memory_storage/mod.rs] - get_settings_sync
- [Source: CLAUDE.md] - 项目开发规范

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List