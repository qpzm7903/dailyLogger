# Dev Log

Key technical decisions, problems encountered, and conventions from story implementations.

---

## SMART-002 Task 3 - 2026-03-15

### 技术决策

1. **数据库字段设计**：`auto_adjust_silent INTEGER DEFAULT 1`（默认开启），`silent_adjustment_paused_until TEXT DEFAULT NULL`（暂停截止时间）。理由：符合 AC3 手动覆盖需求，用户可关闭自动调整或临时暂停。

2. **幂等迁移模式**：使用 `let _ = conn.execute("ALTER TABLE ...")` 忽略列已存在错误。理由：与项目现有迁移模式一致，支持增量升级和首次安装。

3. **Settings 结构体字段类型**：`auto_adjust_silent: Option<bool>` 和 `silent_adjustment_paused_until: Option<String>`。理由：与现有字段类型保持一致，支持 NULL 值表示未配置状态。

4. **静默模式历史表**：选择不创建独立 `silent_patterns` 表，复用 Task 1 的内存存储方案。理由：Task 1 已实现 `SilentPatternTracker` 内存滑动窗口，无需额外持久化。

5. **测试覆盖**：添加 8 个测试用例覆盖默认值、读写持久化、RFC3339 格式验证。理由：确保数据库迁移正确性和 API 稳定性。

### 遇到问题

**多测试文件同步问题**：`manual_entry/mod.rs` 和 `synthesis/mod.rs` 中有独立的测试辅助函数创建 settings 表，未包含新字段导致测试失败。解决：同步更新所有测试辅助函数。

### 后续约定

- **新增 Settings 字段清单**：1) ALTER TABLE 迁移 2) Settings 结构体字段 3) get_settings_sync SELECT 列 4) save_settings_sync UPDATE 参数 5) 所有测试模块的 setup_test_db_with_settings
- **布尔字段默认值**：自动调整类功能默认开启（DEFAULT 1），用户主动关闭后可暂停
- **RFC3339 时间存储**：使用 TEXT 类型存储时间戳字符串，便于跨时区处理

---

## SMART-002 Task 2 - 2026-03-15

### 技术决策

1. **算法核心逻辑**：`calculate_optimal_silent_minutes()` 基于静默超时比例决定阈值调整方向。比例 >= 0.7 表示深度工作（提高阈值），<= 0.3 表示活跃工作（降低阈值），中间值保持不变。理由：符合 AC2 的行为模式检测需求。

2. **渐进式调整**：每次调整上限为 5 分钟，避免阈值剧烈变化影响用户体验。理由：Story Dev Notes 明确要求渐进式策略。

3. **阈值边界限制**：MIN_THRESHOLD = 10 分钟，MAX_THRESHOLD = 60 分钟。理由：过短会导致频繁捕获，过长可能遗漏重要工作内容。

4. **数据充分性检查**：少于 10 次捕获时返回默认阈值而非当前阈值。理由：数据不足时不应根据偶然数据调整，默认值更安全。

5. **常量暴露方式**：阈值相关常量使用 `pub const` 暴露，便于后续其他模块使用和测试验证。理由：与项目现有常量暴露模式一致。

### 遇到问题

无重大问题。TDD 流程先写 16 个测试用例覆盖各种边界场景，实现后全部通过。

### 后续约定

- **算法阈值常量**：`MIN_THRESHOLD = 10`, `MAX_THRESHOLD = 60`, `DEFAULT_THRESHOLD = 30`, `MAX_ADJUSTMENT = 5`
- **比例阈值**：`HIGH_SILENT_RATIO = 0.7`, `LOW_SILENT_RATIO = 0.3`
- **测试边界覆盖**：需测试最小/最大阈值限制、边界比例值、不足数据处理、渐进式调整验证

---

## SMART-002 Task 1 - 2026-03-15

### 技术决策

1. **独立模块设计**：`silent_tracker` 作为独立顶级模块而非 `auto_perception` 子模块。理由：`auto_perception` 依赖 `screenshot` feature，而静默模式跟踪不需要截图功能，应始终可用。

2. **内存滑动窗口**：使用 `Vec<HourlyStats>` 存储最近 7 天的每小时统计数据，自动裁剪过期条目。理由：避免频繁磁盘 IO，内存占用可控（7天 × 24小时 = 最多 168 条记录）。

3. **连续捕获计数**：跟踪 `consecutive_silent_captures` 和 `consecutive_change_captures` 用于实时模式检测。理由：支持 AC1 要求的"检测到用户持续活跃"场景。

4. **should_capture 返回类型**：改为 `Option<CaptureReason>` 而非 `bool`，在决策时自动记录捕获原因。理由：将模式跟踪集成到现有流程，无需额外调用点。

5. **全局单例模式**：使用 `Lazy<Mutex<SilentPatternTracker>>` 实现全局跟踪器。理由：跨异步任务共享状态，与项目现有 `SCREEN_STATE` 模式一致。

### 遇到问题

测试隔离问题：并行运行时 `DB_CONNECTION` 全局状态可能被其他测试污染。解决：使用 `--test-threads=1` 运行测试。

### 后续约定

- **模式跟踪位置**：`src-tauri/src/silent_tracker.rs`
- **捕获原因枚举**：`CaptureReason::{ScreenChanged, SilentTimeout, ManualTrigger}`
- **测试命令**：`cargo test --no-default-features -- --test-threads=1`
- **连续计数重置规则**：ScreenChanged 或 ManualTrigger 重置 silent 计数；SilentTimeout 重置 change 计数

---

## AI-003 - 2026-03-14

### 技术决策

1. **日报 Prompt 模板库**：预设模板定义在前端静态数组中，默认模板内容运行时从后端获取。理由：保持默认 Prompt 的单一事实来源在后端代码中。

2. **导入验证策略**：导入 JSON 模板时验证 `{records}` 占位符是否存在。理由：确保导入的模板可以正常使用，避免运行时错误。

3. **模板导出格式**：采用标准 JSON 格式，包含 version、name、description、content、createdAt 字段。理由：便于用户备份和分享模板配置。

### 遇到问题

开发过程顺利，所有测试通过。TDD 流程先写测试后实现，后端 41 测试通过，前端 92 测试通过。

### 后续约定

- **模板占位符**：日报 Prompt 必须包含 `{records}` 占位符
- **导入错误提示**：JSON 解析失败、缺少 content 字段、缺少占位符分别提供具体错误信息

---

## AI-002 - 2026-03-14

### 技术决策

1. **默认 Prompt 查询**：使用 `get_default_analysis_prompt` Tauri command 返回静态常量，而非存储在 DB。理由：保持默认 Prompt 的单一事实来源在代码中，便于版本升级时自动更新。

2. **重置策略**：重置时清空 `analysis_prompt` 字段而非填入默认值。理由：后端在字段为空时自动使用 DEFAULT_ANALYSIS_PROMPT，避免前后端默认值不一致。

3. **Modal 展示**：用独立 Modal 展示默认 Prompt 而非 Tooltip。理由：Prompt 内容较长（~500 字符），Modal 提供更好的阅读体验和滚动支持。

### 遇到问题

无重大问题。开发过程顺利，TDD 流程先写测试后实现，全部 92 测试通过。

### 后续约定

- **常量暴露模式**：需向前端暴露只读常量时，创建无参数 Tauri command 返回静态值
- **重置模式**：字段重置优先清空而非填默认值，依赖后端 fallback 逻辑
- **命令注册**：新增 Tauri command 必须在 `main.rs` 的 `generate_handler![]` 中注册

---

## CORE-002 - 2026-03-14

### 技术决策

1. **视图状态管理**：使用简单 `ref('grid')` 管理视图状态，默认网格视图。理由：无需复杂状态管理，单一组件内状态足够。

2. **响应式网格布局**：采用 `grid-cols-1 md:grid-cols-2 lg:grid-cols-3` 实现三列响应式。理由：在不同屏幕尺寸下自适应，符合 Tailwind 最佳实践。

3. **按钮高亮模式**：活动按钮 `bg-primary text-white`，非活动 `bg-darker text-gray-400 hover:text-white`。理由：提供清晰的视觉反馈，保持与现有主题一致。

4. **列表视图时间格式**：新增 `formatTimeShort` 函数返回 HH:MM:SS 格式。理由：列表视图空间有限，短格式更紧凑。

5. **日期范围边界处理**：start_date 取 00:00:00，end_date 取 23:59:59，确保同一天记录全包含。理由：用户期望日期筛选包含边界值，符合直觉。

6. **xcap 依赖重构**：将 xcap 从条件依赖改为 optional 依赖，在 feature 中显式声明。理由：修复 `screenshot` feature 编译问题，依赖声明更清晰。

7. **缩略图加载复用**：抽取 `loadThumbnails` 函数供筛选和默认加载共用。理由：避免代码重复，统一缩略图加载逻辑。

### 遇到问题

**依赖配置问题**：xcap 在 `screenshot` feature 下编译失败，原配置 `[target.'cfg(all(not(target_os = "windows"), feature = "screenshot"))'.dependencies]` 无法正确解析 feature 条件。解决：改为 optional 依赖 + feature 显式引用。

### 后续约定

- **切换按钮组模式**：活动按钮 `bg-primary text-white`，非活动 `bg-darker text-gray-400 hover:text-white`
- **响应式三列**：`grid-cols-1 md:grid-cols-2 lg:grid-cols-3`
- **列表分隔线**：使用 `divide-y divide-gray-700` 实现行分隔
- **测试分组**：按 AC 分组测试用例（如 `AC1 - View Toggle`）提高可读性
- **日期 API 格式**：前端传 `YYYY-MM-DD` 字符串，后端解析为本地时间边界再转 UTC
- **日期筛选测试**：覆盖边界值（00:00:00, 23:59:59）和跨日场景
- **测试隔离**：用 `.iter().any()` 定位记录，不依赖记录顺序或全局数量

### Task 3 技术决策

1. **快速预览复用模式**：直接复用现有 ScreenshotModal.vue 组件，通过 props 传递完整的 record 对象。理由：避免重复代码，保持组件职责单一。

2. **异步测试等待策略**：使用 `waitFor` 辅助函数等待 VM 状态更新而非固定次数的 nextTick。理由：测试更稳定，不依赖具体的异步操作数量。

3. **测试分组策略**：按 AC 分组测试用例（AC1 - View Toggle, AC3 - Quick Preview Modal）。理由：提高测试可读性，便于定位问题。

### Task 3 后续约定

- **异步测试等待**：使用 `waitFor(() => wrapper.vm.xxx)` 等待状态更新
- **预览模态框测试**：验证 record 传递、路径正确性、内容完整性

### Task 3 测试实现 - 2026-03-14

### 技术决策

1. **waitFor 辅助函数**：封装异步等待逻辑，条件检查 + 超时机制。理由：替代固定 nextTick 次数，测试更稳定可靠。

2. **Modal 测试策略**：通过 `findComponent({ name: 'ScreenshotModal' })` 定位子组件，验证 props 和事件。理由：直接访问组件实例，断言更精确。

### 遇到问题

原测试使用多个 nextTick 等待异步操作，在 CI 环境偶尔超时。解决：引入 waitFor 辅助函数，基于条件轮询而非固定次数。

### 后续约定

- **异步测试模式**：`waitFor(() => condition, timeout)` 替代多次 nextTick
- **Modal 测试清单**：1) 验证组件存在 2) 验证 props 传递 3) 验证事件触发 4) 验证状态重置

---

## Task 4 分页加载 - 2026-03-14

### 技术决策

1. **分页状态管理**：使用 `currentPage`, `pageSize=20`, `isLoadingMore` ref 状态。`paginatedScreenshots` 和 `remainingCount` 作为 computed 属性。理由：响应式计算，无需手动同步。

2. **滚动检测策略**：在滚动容器上监听 `@scroll` 事件，计算 `scrollHeight - scrollTop - clientHeight`，当小于 100px 时触发加载。理由：提前加载，用户体验更好，避免滚动到底部才加载的突兀感。

3. **加载指示器**：使用 `animate-pulse` Tailwind 类实现加载动画，显示 "加载中..." 文字。理由：简洁的视觉反馈，与现有 UI 风格一致。

4. **双重触发机制**：同时支持滚动自动加载和 "加载更多" 按钮点击。理由：兼容不同用户习惯，按钮作为备用触发方式。

5. **防重复加载**：`isLoadingMore` 状态锁防止加载过程中重复触发。理由：避免竞态条件导致页面跳跃。

### 遇到问题

测试中使用 `setTimeout` 模拟加载延迟，需要等待异步操作完成。解决：测试中使用 `await new Promise(resolve => setTimeout(resolve, 200))` 等待延迟完成。

### 后续约定

- **滚动阈值**：100px 为触发加载的标准阈值
- **加载延迟**：150ms 用于提供视觉反馈
- **分页测试等待**：`await new Promise(resolve => setTimeout(resolve, 200))` 等待 loadMore 完成

---

## Task 5 元信息显示 - 2026-03-14

### 技术决策

1. **时间戳格式统一**：网格视图和列表视图均使用 `formatTimeShort` 函数，返回 HH:mm:ss 格式。理由：AC5 要求时间戳格式统一，网格视图原有 `formatTime` 返回完整日期时间字符串过长，不适合缩略图卡片。

2. **省略号截断策略**：当内容超过 50 字符时，截断前 50 字符并添加 "..." 省略号。理由：用户需要明确知道内容被截断，提升 UX 可读性。

3. **移除冗余代码**：删除未使用的 `formatTime` 函数。理由：保持代码整洁，避免死代码。

### 后续约定

- **时间戳格式**：缩略图卡片统一使用 `formatTimeShort` 返回 HH:mm:ss 格式
- **文本截断**：超过限制长度时添加 "..." 省略号
- **测试断言**：截断测试验证长度和省略号同时存在

---

## CORE-005 - 2026-03-14

### 技术决策

1. **AtomicBool 状态查询**：新增 `is_auto_capture_running()` 读取 `AUTO_CAPTURE_RUNNING` 的 `Ordering::SeqCst`。理由：线程安全访问跨线程共享状态。

2. **动态托盘菜单**：右键点击时重建菜单显示当前状态（● 运行中 / ○ 已停止）。理由：Tauri 托盘菜单不支持响应式更新，需手动重建。

3. **Feature 条件编译**：托盘菜单构建函数分 `#[cfg(feature = "screenshot")]` 和 `#[cfg(not(feature = "screenshot"))` 两版。理由：截图功能可选，菜单项需相应变化。

4. **状态指示器**：使用 Unicode 字符 ●/○ 表示运行状态。理由：简洁直观，无需图标资源。

### 遇到问题

`stop_auto_capture` 为 async 函数，无法在同步的 `on_menu_event` 回调中直接调用。当前方案用 `run_on_main_thread` 占位，后续需实现完整的异步启动/停止。

### 后续约定

- **托盘菜单项 ID**：`capture_toggle`、`show`、`quit`
- **菜单分组**：用 `PredefinedMenuItem::separator()` 分隔操作组
- **AtomicBool 读写**：统一使用 `Ordering::SeqCst` 保证顺序一致性

---

## CORE-005 Task 2 - 2026-03-14

### 技术决策

1. **多窗口入口**：使用 Vite 多页面应用模式，创建独立的 `quick-note.html` 入口和 `quick-note.js` 启动脚本。理由：快速记录窗口是独立的小窗口，无需加载主应用的全部状态。

2. **窗口配置**：快速记录窗口设置为 400x280 像素、不可调整大小、始终置顶。理由：小窗口需要始终可见，方便用户快速输入，不干扰其他工作。

3. **保存机制**：使用 `tray_quick_note` 命令直接保存记录到数据库，保存成功后调用 `window.close()` 关闭窗口。理由：简化交互流程，保存即关闭符合"快速"的设计目标。

4. **快捷键支持**：Enter 保存、Esc 关闭。理由：提供键盘操作友好性，符合桌面应用习惯。

### 后续约定

- **快速记录窗口 ID**：`quick-note`
- **窗口 URL**：`quick-note.html`
- **菜单项 ID**：`quick_note`
- **同步命令模式**：为 async Tauri 命令创建 `*_sync` 版本便于测试
- **测试 DB 访问**：`DB_CONNECTION` 需 `pub` 以便测试模块访问

---

## CORE-005 Task 2 补充 - 2026-03-14

### 遇到问题

1. **测试隔离**：`DB_CONNECTION` 是私有静态变量，测试模块无法访问。解决：改为 `pub static`，并在测试中用 `setup_test_db()` 初始化内存数据库。

2. **同步/异步分离**：`tray_quick_note` 是 async 命令，但测试需要同步验证。解决：抽取 `add_quick_note_sync()` 同步核心逻辑，async 命令调用它。

### 后续约定

- **命令测试模式**：async 命令抽取同步核心，分别测试核心逻辑和命令包装
- **菜单项新增流程**：1) 添加 MenuItem 2) 注册 on_menu_event 处理 3) 条件编译处理 feature 差异

---

## CORE-005 Task 3 - 2026-03-14

### 技术决策

1. **跨平台文件打开**：使用 `#[cfg(target_os)]` 条件编译，Windows 用 `explorer`，macOS 用 `open`，Linux 用 `xdg-open`。理由：桌面应用标准做法，系统默认文件管理器体验一致。

2. **路径校验链**：依次检查 None → 空字符串 → 纯空白 → 路径不存在。理由：提供具体错误信息，引导用户正确配置。

3. **Sync/Async 分离**：`open_obsidian_folder_sync()` 核心逻辑 + async 命令包装。理由：便于单元测试，延续项目既有模式。

### 遇到问题

无重大问题。TDD 先写 6 个测试用例覆盖边界条件，实现后全部通过。

### 后续约定

- **跨平台命令模式**：`#[cfg(target_os)]` 分平台实现，spawn() 启动子进程
- **路径校验**：`.filter(|p| !p.trim().is_empty())` 同时处理空字符串和纯空白
- **错误消息**：中文提示，明确问题原因（"请先在设置中配置" vs "路径不存在"）

---

## CORE-005 Task 4 - 2026-03-14

### 技术决策

1. **CheckMenuItem 替代 MenuItem**：使用 `CheckMenuItem::with_id()` 创建可勾选菜单项，通过 `checked` 参数显示勾选状态。理由：比 Unicode 字符 ●/○ 更符合原生桌面应用 UI 习惯，提供更好的视觉反馈。

2. **菜单结构简化**：移除 `generate_summary` 和 `settings` 菜单项，保留核心操作。理由：托盘菜单应保持简洁，常用操作优先；生成日报和设置功能可通过主窗口访问。

3. **菜单顺序重新组织**：状态 → 分隔线 → 快速记录 → 打开文件夹 → 分隔线 → 显示窗口 → 退出。理由：按功能分组，状态独立一组，操作一组，窗口控制一组。

### 遇到问题

无重大问题。编译通过，所有 63 个 Rust 测试和 136 个前端测试均通过。

### 后续约定

- **CheckMenuItem 用法**：`CheckMenuItem::with_id(app, id, text, enabled, checked, accelerator)`
- **菜单分组**：用 `PredefinedMenuItem::separator()` 分隔不同功能组
- **非截图模式菜单**：不包含 capture_toggle，只有快速记录、打开文件夹、显示窗口、退出

---

## CORE-005 Task 5 - 2026-03-14

### 技术决策

1. **测试验证策略**：运行 `cargo test --no-default-features` 跳过截图 feature 测试。理由：CI 环境缺少 libspa 依赖，截图相关测试无法编译。

2. **测试位置**：get_auto_capture_status 测试在 auto_perception/mod.rs；tray_quick_note 和 open_obsidian_folder 测试在 manual_entry/mod.rs。理由：跟随命令定义位置，便于维护。

### 遇到问题

CI 环境无法编译 screenshot feature 测试（libspa 依赖缺失）。解决：使用 `--no-default-features` 标志运行测试，跳过截图相关测试。

### 后续约定

- **CI 测试命令**：`cargo test --no-default-features` 适用于无 GUI 的 CI 环境
- **测试组织**：命令测试跟随命令定义模块，边界测试覆盖 None、空字符串、纯空白、路径不存在

---

## SMART-001 - 2026-03-14

### 技术决策

1. **跨平台窗口检测架构**：创建独立 `window_info` 模块，`ActiveWindow` 结构体含 `title` 和 `process_name` 字段。理由：封装窗口信息，便于后续扩展（如窗口句柄、PID）。

2. **平台特定实现**：使用 `#[cfg(target_os)]` 条件编译分平台实现。Windows 用 Win32 API（`windows` crate），macOS 用 AppleScript，Linux 用 `xdotool` 命令。理由：各平台原生 API 最可靠，外部命令作为备选。

3. **Windows 进程权限**：使用 `PROCESS_QUERY_LIMITED_INFORMATION (0x1000)` 最小权限打开进程。理由：安全最佳实践，仅需获取模块名，无需完全访问权限。

4. **优雅降级**：失败时返回空字符串的 `ActiveWindow::default()`，而非 panic 或 Result。理由：窗口信息是辅助功能，不应阻塞主流程。

### 遇到问题

无重大问题。Windows API 使用 `unsafe` 块需注意资源释放（CloseHandle）。macOS AppleScript 可能需要用户授权辅助功能权限。

### 后续约定

- **平台条件编译**：`#[cfg(target_os = "windows/macos/linux")]` 分平台实现
- **窗口信息模块**：`src-tauri/src/window_info/mod.rs`
- **Windows 依赖**：`windows` crate 需显式声明 features（Win32_Foundation, Win32_UI_WindowsAndMessaging 等）
- **Linux 依赖**：`xdotool` 需系统安装，文档需说明

---

## SMART-001 Task 2 - 2026-03-14

### 技术决策

1. **数据库迁移策略**：使用 `let _ = conn.execute(...)` 忽略 ALTER TABLE 错误。理由：列已存在时不会报错，实现幂等迁移，避免首次创建和增量迁移的重复逻辑。

2. **JSON 数组默认值**：`window_whitelist/blacklist` 默认 `'[]'`（空 JSON 数组字符串）。理由：前端可直接 `JSON.parse()`，无需额外 null 处理。

3. **布尔字段模式**：`use_whitelist_only` 用 INTEGER (0/1) 存储，读取时 `row.get::<_, Option<i32>>()?.map(|v| v != 0)`。理由：延续项目现有布尔字段惯例（如 `auto_capture_enabled`）。

### 遇到问题

新增字段需同步更新 4 处：Settings 结构体、SELECT 查询列、UPDATE 参数、测试辅助结构体。遗漏任一处导致编译错误或运行时 panic。

### 后续约定

- **Settings 字段新增清单**：1) ALTER TABLE 迁移 2) 结构体字段 3) SELECT 列 4) UPDATE 参数 5) 测试 helper
- **布尔转换模式**：存储 `map(|v| if v { 1 } else { 0 })`，读取 `map(|v| v != 0)`
- **幂等迁移**：`let _ = conn.execute(...)` 忽略"列已存在"错误

---

## SMART-001 Task 3 - 2026-03-15

### 技术决策

1. **窗口过滤函数位置**：将 `should_capture_by_window()` 和 `matches_any()` 放在 `window_info` 模块而非 `auto_perception`。理由：窗口过滤逻辑与窗口信息紧密相关，便于模块内测试和复用。

2. **模糊匹配实现**：使用 `to_lowercase()` 实现大小写不敏感的部分匹配。理由：窗口标题大小写不一致（如 "VS Code" vs "vs code"），部分匹配支持窗口标题包含额外信息（如 "VS Code - main.rs"）。

3. **优先级逻辑**：`use_whitelist_only=true` 时仅应用白名单逻辑，忽略黑名单；`use_whitelist_only=false` 时应用黑名单逻辑。理由：符合 AC3 要求，白名单模式更严格，优先级更高。

4. **空列表处理**：白名单为空且 `use_whitelist_only=true` 时允许所有窗口（相当于白名单模式未生效）。理由：避免用户误配置导致无法捕获任何窗口。

### 遇到问题

Clippy 警告嵌套 if 可折叠。解决：将 `if !blacklist.is_empty() { if matches... }` 折叠为 `if !blacklist.is_empty() && matches...`。

### 后续约定

- **模糊匹配模式**：`patterns.iter().any(|p| text.to_lowercase().contains(&p.to_lowercase()))`
- **过滤逻辑文档**：函数注释需说明 AC 对应关系

---

## SMART-001 Task 4 - 2026-03-15

### 技术决策

1. **ScreenAnalysis 扩展**：添加 `active_window: Option<ActiveWindow>` 字段，使用 `#[serde(skip_serializing_if = "Option::is_none")]` 避免序列化空值。理由：保持向后兼容，旧的 JSON 反序列化时没有 active_window 字段也不会报错。

2. **CaptureSettings 扩展**：添加 `window_whitelist`, `window_blacklist`, `use_whitelist_only` 三个字段。理由：将窗口过滤配置集成到捕获设置中，便于 `capture_and_store()` 统一访问。

3. **窗口过滤时机**：在截图之前获取窗口信息并应用过滤规则。理由：如果窗口被过滤，跳过截图和 AI 分析，节省资源和 API 调用。

4. **parse_window_patterns 函数**：解析 JSON 数组字符串为 `Vec<String>`，解析失败返回空 Vec。理由：提供优雅降级，配置错误不阻塞主流程。

5. **content JSON 格式**：直接在 JSON 中添加 `active_window` 字段，包含 `title` 和 `process_name`。理由：AC1 要求记录中包含窗口信息，前端可直接解析使用。

### 遇到问题

无重大问题。代码编译通过，所有 104 个测试通过，clippy 无警告。

### 后续约定

- **窗口过滤集成模式**：在异步函数开始时获取窗口信息，应用过滤后再执行耗时操作（截图、AI 分析）
- **配置解析模式**：JSON 字符串解析失败时返回默认值（空数组），避免配置错误导致功能完全失效
- **测试隔离**：`auto_perception` 模块测试依赖 `screenshot` feature，CI 使用 `--no-default-features` 跳过

---

## SMART-001 Task 5 - 2026-03-15

### 技术决策

1. **标签输入组件模式**：使用 ref 数组 (`whitelistTags`, `blacklistTags`) 管理标签列表，独立的 `newXxxTag` ref 管理输入值。理由：响应式数组支持动态添加/删除，输入值独立管理避免输入和显示状态混淆。

2. **标签持久化策略**：`loadSettings()` 时从 JSON 解析到数组，`saveSettings()` 时序列化数组为 JSON。理由：前端使用数组便于操作，后端使用 JSON 字符串保持数据库字段简洁。

3. **标签删除定位**：通过遍历 spans 过滤特定标签元素的 close 按钮。理由：避免全局按钮索引变化导致测试不稳定，提高测试精确性。

4. **UI 布局**：白名单标签使用 `bg-primary/20 text-primary` 样式，黑名单使用 `bg-red-500/20 text-red-400`。理由：颜色语义化区分两种过滤模式，提高用户识别度。

### 遇到问题

测试删除标签时，全局按钮索引可能变化导致误删。解决：改为通过标签文本定位特定标签元素，再点击其内部删除按钮。

### 后续约定

- **标签输入模式**：`v-model` 绑定输入值，`@keyup.enter` 触发添加，空值或重复值忽略
- **标签样式**：白名单 primary 色，黑名单 red 色，统一 `text-xs rounded-lg` 样式
- **测试定位**：通过元素内容或父元素 class 过滤特定组件，避免全局索引依赖

---

## SMART-001 Task 6 - 2026-03-15

### 技术决策

1. **JSON 解析容错**：`getWindowInfo()` 使用 try/catch 解析 content JSON，失败返回 null。理由：旧记录无 `active_window` 字段或格式异常时优雅降级，不阻塞 UI。

2. **图标映射函数**：`getWindowIcon()` 根据 process_name 返回应用专属 emoji，未知应用返回 🖥️。理由：视觉化区分应用类型，提升 UX 可识别度。

3. **条件渲染**：仅在 `active_window` 存在且 `title` 或 `process_name` 非空时显示窗口信息区。理由：避免空数据区域，保持 UI 简洁。

4. **组件复用问题**：App.vue 和 ScreenshotModal.vue 各自定义 `getWindowIcon()` 函数。理由：任务紧急未抽取公共函数，后续可优化为共享 util。

### 遇到问题

无重大问题。开发顺利，前端测试全部通过。

### 后续约定

- **窗口信息显示模式**：computed 属性解析 JSON + 图标函数 + 条件渲染
- **图标映射扩展**：新增应用在 `getWindowIcon()` 函数中添加判断分支
- **JSON 解析容错**：try/catch 返回 null/fallback，不抛异常阻塞流程