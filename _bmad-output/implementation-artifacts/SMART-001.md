# Story 2.1: 应用窗口识别

Status: ready-for-dev

## Story

作为一个 DailyLogger 用户，
我希望系统能自动识别当前活动的应用窗口，记录窗口标题和进程名，
以便我在回顾工作记录时能更清晰地了解当时的工作上下文，并支持配置特定应用的捕获策略。

## Acceptance Criteria

### AC1 - 记录中包含当前活动窗口信息
- Given 自动捕获功能正在运行
- When 系统完成一次截图捕获
- Then 记录的 content JSON 中包含 `active_window` 字段，含 `title`（窗口标题）和 `process_name`（进程名）
- Given 用户查看今日记录列表
- When 某条记录有窗口信息
- Then 记录详情中显示应用图标（如能获取）和窗口标题

### AC2 - 支持窗口白名单/黑名单
- Given 用户在设置中配置了窗口白名单 ["VS Code", "IntelliJ IDEA"]
- When 当前活动窗口不在白名单中
- Then 跳过本次捕获，不记录、不调用 AI
- Given 用户在设置中配置了窗口黑名单 ["浏览器", "Slack"]
- When 当前活动窗口在黑名单中
- Then 跳过本次捕获
- Given 白名单和黑名单都未配置
- When 系统捕获
- Then 正常执行捕获流程

### AC3 - 可配置仅捕获特定应用
- Given 用户启用了"仅捕获白名单应用"选项
- When 当前活动窗口在白名单中
- Then 正常执行捕获
- Given 用户启用了"仅捕获白名单应用"选项
- When 当前活动窗口不在白名单中
- Then 跳过捕获
- Given 用户禁用了"仅捕获白名单应用"选项
- When 系统捕获
- Then 按黑名单规则过滤，无黑名单则正常捕获

## Tasks / Subtasks

- [x] Task 1: 实现跨平台窗口信息获取 (AC: 1)
  - [x] 添加 `active-win` 相关依赖或实现跨平台窗口获取
  - [x] Windows: 使用 Win32 API 获取前台窗口标题和进程名
  - [x] macOS: 使用 NSWorkspace/Accessibility API
  - [x] Linux: 使用 xdotool 或 x11 相关库
  - [x] 创建 `get_active_window()` 函数返回 `ActiveWindow` 结构体
  - [x] 编写单元测试（mock 各平台返回）

- [-] Task 2: 扩展数据库 Schema (AC: 1, 2, 3)
  - [ ] 在 settings 表添加 `window_whitelist` 字段（JSON 数组）
  - [ ] 在 settings 表添加 `window_blacklist` 字段（JSON 数组）
  - [ ] 在 settings 表添加 `use_whitelist_only` 字段（布尔值）
  - [ ] 更新 `Settings` 结构体添加新字段
  - [ ] 编写数据库迁移测试

- [ ] Task 3: 实现窗口过滤逻辑 (AC: 2, 3)
  - [ ] 创建 `should_capture_by_window()` 函数
  - [ ] 实现白名单匹配逻辑（模糊匹配窗口标题）
  - [ ] 实现黑名单匹配逻辑
  - [ ] 实现白名单优先级逻辑
  - [ ] 编写匹配逻辑单元测试

- [ ] Task 4: 集成到捕获流程 (AC: 1, 2, 3)
  - [ ] 修改 `capture_and_store()` 函数
  - [ ] 在截图前获取窗口信息
  - [ ] 应用窗口过滤规则
  - [ ] 将窗口信息写入记录 content JSON
  - [ ] 更新 `ScreenAnalysis` 结构体添加窗口字段

- [ ] Task 5: 前端设置界面支持 (AC: 2, 3)
  - [ ] 添加窗口白名单配置 UI（标签输入）
  - [ ] 添加窗口黑名单配置 UI（标签输入）
  - [ ] 添加"仅捕获白名单应用"开关
  - [ ] 显示当前活动窗口提示（可选）

- [ ] Task 6: 前端记录显示支持 (AC: 1)
  - [ ] 修改记录列表项显示应用图标和窗口标题
  - [ ] 在记录详情中展示窗口信息
  - [ ] 支持按应用筛选记录（可选）

## Dev Notes

### 技术需求

1. **跨平台窗口检测** - 需要针对不同平台实现
2. **模糊匹配** - 窗口标题支持部分匹配
3. **性能考量** - 窗口获取应该是毫秒级操作
4. **权限** - macOS 可能需要 Accessibility 权限

### 架构合规要求

- 后端命令注册在 `main.rs` 的 `generate_handler![]`
- 使用 `memory_storage::get_settings_sync()` 获取设置
- 窗口信息存储在记录的 `content` JSON 中，不修改 records 表结构
- 错误消息使用中文

### 现有实现分析

**当前捕获流程（auto_perception/mod.rs:437-472）：**
```rust
async fn capture_and_store() -> Result<(), String> {
    let settings = load_capture_settings();
    if settings.api_key.is_empty() { ... }
    let image_base64 = capture_screen()?;
    let fingerprint = compute_fingerprint(&image_base64)?;
    if !should_capture(...) { return Ok(()); }
    let screenshot_path = save_screenshot(&image_base64);
    let analysis = analyze_screen(&settings, &image_base64).await?;
    let content = serde_json::json!({
        "current_focus": analysis.current_focus,
        "active_software": analysis.active_software,
        "context_keywords": analysis.context_keywords
    }).to_string();
    memory_storage::add_record("auto", &content, screenshot_path.as_deref())?;
    Ok(())
}
```

**当前 ScreenAnalysis 结构体（auto_perception/mod.rs:104-109）：**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenAnalysis {
    pub current_focus: String,
    pub active_software: String,
    pub context_keywords: Vec<String>,
}
```

**需要扩展为：**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenAnalysis {
    pub current_focus: String,
    pub active_software: String,
    pub context_keywords: Vec<String>,
    pub active_window: Option<ActiveWindow>,  // 新增
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveWindow {
    pub title: String,
    pub process_name: String,
}
```

### 跨平台窗口获取方案

#### Windows 方案
```rust
#[cfg(target_os = "windows")]
fn get_active_window() -> Result<ActiveWindow, String> {
    use winapi::um::winuser::GetForegroundWindow;
    use winapi::um::winuser::GetWindowTextW;
    use winapi::psapi::GetWindowThreadProcessId;

    // 获取前台窗口句柄
    // 获取窗口标题
    // 获取进程 ID 和名称
    // 返回 ActiveWindow { title, process_name }
}
```

**推荐依赖**: `windows` crate (已用于截图，可复用)
```toml
[target.'cfg(target_os = "windows")'.dependencies]
windows = { version = "0.58", features = [
    "Win32_Foundation",
    "Win32_UI_WindowsAndMessaging",
    "Win32_System_Threading",
    "Win32_System_ProcessStatus"
] }
```

#### macOS 方案
```rust
#[cfg(target_os = "macos")]
fn get_active_window() -> Result<ActiveWindow, String> {
    // 使用 NSWorkspace 获取前台应用
    // 使用 Accessibility API 获取窗口标题
    // 注意：需要 Accessibility 权限
}
```

**推荐依赖**: `core-foundation` 和 `objc` crate
```toml
[target.'cfg(target_os = "macos")'.dependencies]
core-foundation = "0.9"
objc = "0.2"
```

#### Linux 方案
```rust
#[cfg(target_os = "linux")]
fn get_active_window() -> Result<ActiveWindow, String> {
    // 使用 xdotool 命令行工具
    // 或使用 x11rb 库直接查询
}
```

**推荐依赖**: `x11rb` 或调用外部 `xdotool`
```toml
[target.'cfg(target_os = "linux")'.dependencies]
x11rb = "0.13"
```

### 窗口过滤逻辑

```rust
fn should_capture_by_window(
    window: &ActiveWindow,
    whitelist: &[String],
    blacklist: &[String],
    use_whitelist_only: bool,
) -> bool {
    // 1. 白名单模式：仅白名单中的应用才捕获
    if use_whitelist_only && !whitelist.is_empty() {
        return matches_any(&window.title, whitelist)
            || matches_any(&window.process_name, whitelist);
    }

    // 2. 黑名单模式：黑名单中的应用跳过
    if !blacklist.is_empty() {
        if matches_any(&window.title, blacklist)
            || matches_any(&window.process_name, blacklist)
        {
            return false;
        }
    }

    // 3. 默认允许捕获
    true
}

fn matches_any(text: &str, patterns: &[String]) -> bool {
    patterns.iter().any(|p| text.to_lowercase().contains(&p.to_lowercase()))
}
```

### 数据库 Schema 扩展

```sql
-- 在 settings 表添加新字段
ALTER TABLE settings ADD COLUMN window_whitelist TEXT DEFAULT '[]';
ALTER TABLE settings ADD COLUMN window_blacklist TEXT DEFAULT '[]';
ALTER TABLE settings ADD COLUMN use_whitelist_only INTEGER DEFAULT 0;
```

**Settings 结构体扩展：**
```rust
pub struct Settings {
    // ... 现有字段 ...
    pub window_whitelist: Option<String>,      // JSON 数组
    pub window_blacklist: Option<String>,      // JSON 数组
    pub use_whitelist_only: Option<bool>,
}
```

### 文件结构要求

**修改文件：**
```
src-tauri/src/
├── main.rs                    # 注册新命令（如有）
├── lib.rs                     # 可能添加辅助函数
├── auto_perception/
│   └── mod.rs                 # 添加窗口获取和过滤逻辑
└── memory_storage/
    └── mod.rs                 # 添加新设置字段

src/
├── App.vue                    # 可能需要更新记录显示
└── components/
    └── SettingsModal.vue      # 添加窗口过滤设置 UI
```

### 测试要求

**Rust 测试重点：**
1. `get_active_window()` 返回有效结构（mock 测试）
2. `should_capture_by_window()` 各种匹配情况
3. 白名单优先级测试
4. 黑名单排除测试
5. 数据库迁移测试

**边界测试：**
1. 窗口标题为空
2. 进程名无法获取
3. 白名单/黑名单包含特殊字符
4. 匹配时大小写处理

### 权限说明

| 平台 | 所需权限 | 说明 |
|-----|---------|------|
| Windows | 无特殊权限 | Win32 API 直接可用 |
| macOS | Accessibility 权限 | 获取窗口标题需要用户授权 |
| Linux | 无特殊权限 | 需要运行 X11 环境 |

**macOS 权限处理：**
- 首次调用时检查权限
- 无权限时返回进程名，窗口标题可能为空
- 提供用户引导提示

### 捕获流程修改

**修改后的 `capture_and_store()` 流程：**
```
1. 获取窗口信息
2. 应用窗口过滤规则
   - 如果不满足条件，跳过并记录日志
3. 截图
4. 计算指纹，判断是否需要捕获
5. 保存截图
6. AI 分析
7. 组合 content JSON（含窗口信息）
8. 存入数据库
```

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

### 从 CORE-005 学习的经验

1. **跨平台代码**：使用 `#[cfg(target_os = "...")]` 条件编译
2. **系统级操作**：需要考虑权限和错误处理

## Project Structure Notes

### 现有项目结构

```
src-tauri/src/
├── lib.rs                     # 应用入口，APP_STATE
├── main.rs                    # Tauri 主进程，托盘定义
├── auto_perception/
│   └── mod.rs                 # 自动感知（capture_screen, analyze_screen）
├── manual_entry/
│   └── mod.rs                 # 手动输入（add_quick_note）
├── memory_storage/
│   └── mod.rs                 # 数据存储（Settings）
└── synthesis/
    └── mod.rs                 # 日报生成

src/
├── App.vue                    # 主界面容器
├── components/
│   ├── SettingsModal.vue      # 设置模态框
│   └── ...
```

### 关键依赖

- `tauri = { version = "2", features = ["tray-icon"] }` - 已启用
- `windows` crate - 已用于截图（Windows），可扩展用于窗口信息
- `xcap` - 用于 macOS/Linux 截图，可能也提供窗口信息

## References

- [Source: architecture.md#2.2 后端模块] - auto_perception 职责描述
- [Source: architecture.md#4. 关键设计决策] - 跨平台处理模式
- [Source: architecture.md#5. 数据库设计] - settings 表结构
- [Source: PRD.md#6.1 自动感知] - 自动捕获功能需求
- [Source: epics.md#Epic 2] - 所属 Epic 信息
- [Source: src-tauri/src/auto_perception/mod.rs] - 当前捕获实现
- [Source: src-tauri/src/memory_storage/mod.rs] - 设置存储
- [Source: CLAUDE.md] - 项目开发规范

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List