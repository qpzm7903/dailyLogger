# Story 2.2: 静默时段智能调整

Status: ready-for-dev

## Story

作为一个 DailyLogger 用户，
我希望系统能根据我的工作习惯自动调整静默阈值（max_silent_minutes），
以便在深度工作时减少不必要的截图，在活跃工作时确保记录完整性，同时保留手动覆盖的选项。

## Acceptance Criteria

### AC1 - 学习用户工作模式
- Given 自动捕获功能运行超过 3 天
- When 系统收集了足够的工作模式数据
- Then 计算出用户的典型静默时段分布（如上午专注编码、下午频繁会议）
- Given 系统检测到用户持续活跃（屏幕频繁变化）
- When 活跃持续时间超过当前阈值 50%
- Then 记录此行为模式用于调整

### AC2 - 自动调整 max_silent_minutes
- Given 用户处于深度工作状态（屏幕长期无变化）
- When 系统检测到连续多次因静默超时触发捕获
- Then 自动提高 max_silent_minutes（如从 30 分钟提高到 45 分钟）
- Given 用户处于活跃工作状态（屏幕频繁变化）
- When 系统检测到多次捕获间隔都很短
- Then 自动降低 max_silent_minutes（如从 30 分钟降低到 15 分钟）
- Given 自动调整发生
- When 调整幅度超过 10 分钟
- Then 通过系统通知提醒用户新的静默阈值

### AC3 - 提供手动覆盖选项
- Given 用户在设置中配置了自定义 max_silent_minutes
- When 用户启用"手动模式"开关
- Then 系统不再自动调整静默阈值
- Given 用户处于自动模式
- When 用户手动修改 max_silent_minutes
- Then 系统尊重用户设置，暂停自动调整 24 小时
- Given 用户想要恢复自动调整
- When 用户禁用"手动模式"开关
- Then 系统恢复自动学习模式

## Tasks / Subtasks

- [x] Task 1: 实现用户行为模式数据收集 (AC: 1)
  - [x] 创建 `SilentPatternTracker` 结构体跟踪捕获行为
  - [x] 记录每次捕获的时间、原因（屏幕变化/静默超时）
  - [x] 存储最近 7 天的行为模式数据
  - [x] 实现内存中滑动窗口统计（避免频繁磁盘 IO）

- [-] Task 2: 实现智能阈值调整算法 (AC: 2)
  - [ ] 创建 `calculate_optimal_silent_minutes()` 函数
  - [ ] 分析静默超时触发频率 vs 屏幕变化触发频率
  - [ ] 实现渐进式调整策略（每次调整不超过 5 分钟）
  - [ ] 设置调整上下限（最小 10 分钟，最大 60 分钟）
  - [ ] 编写算法单元测试

- [ ] Task 3: 扩展数据库 Schema (AC: 2, 3)
  - [ ] 在 settings 表添加 `auto_adjust_silent` 字段（布尔值，默认开启）
  - [ ] 在 settings 表添加 `silent_adjustment_paused_until` 字段（时间戳）
  - [ ] 创建 `silent_patterns` 表存储行为模式历史（可选，或内存存储）
  - [ ] 更新 `Settings` 结构体添加新字段
  - [ ] 编写数据库迁移测试

- [ ] Task 4: 集成到捕获流程 (AC: 1, 2)
  - [ ] 修改 `capture_and_store()` 记录捕获原因
  - [ ] 在每次捕获后更新行为模式统计
  - [ ] 定期（每小时）评估是否需要调整阈值
  - [ ] 实现调整通知逻辑

- [ ] Task 5: 前端设置界面支持 (AC: 3)
  - [ ] 添加"自动调整静默阈值"开关
  - [ ] 显示当前静默阈值和学习状态
  - [ ] 添加手动覆盖输入框
  - [ ] 显示最近调整历史（可选）

- [ ] Task 6: 编写测试 (AC: 1, 2, 3)
  - [ ] 模式检测单元测试
  - [ ] 调整算法边界测试
  - [ ] 手动覆盖逻辑测试
  - [ ] 集成测试

## Dev Notes

### 技术需求

1. **行为模式分析** - 统计静默超时 vs 屏幕变化的触发比例
2. **渐进式调整** - 避免阈值剧烈变化影响用户体验
3. **持久化** - 跨会话保持学习数据（可选内存模式）
4. **通知机制** - 阈值变化时提醒用户

### 架构合规要求

- 后端命令注册在 `main.rs` 的 `generate_handler![]`
- 使用 `memory_storage::get_settings_sync()` 获取设置
- 使用现有的 `SCREEN_STATE` 跟踪捕获状态
- 错误消息使用中文

### 现有实现分析

**当前静默检测逻辑（auto_perception/mod.rs:69-102）：**
```rust
fn should_capture(fingerprint: &[u8], change_threshold: f64, max_silent_minutes: u64) -> bool {
    let mut state = SCREEN_STATE.lock().unwrap();
    let silent_exceeded = state.last_capture_time.elapsed() >= Duration::from_secs(max_silent_minutes * 60);
    let changed = match &state.last_fingerprint {
        None => true,
        Some(prev) => {
            let rate = calc_change_rate(prev, fingerprint);
            rate >= change_threshold
        }
    };
    if changed || silent_exceeded {
        state.last_fingerprint = Some(fingerprint.to_vec());
        state.last_capture_time = Instant::now();
        true
    } else {
        false
    }
}
```

**当前 SCREEN_STATE 结构（auto_perception/mod.rs:21-31）：**
```rust
struct ScreenState {
    last_fingerprint: Option<Vec<u8>>,
    last_capture_time: Instant,
}

static SCREEN_STATE: Lazy<Mutex<ScreenState>> = Lazy::new(|| {
    Mutex::new(ScreenState {
        last_fingerprint: None,
        last_capture_time: Instant::now(),
    })
});
```

**需要扩展为：**
```rust
struct ScreenState {
    last_fingerprint: Option<Vec<u8>>,
    last_capture_time: Instant,
    // 新增：捕获原因跟踪
    last_capture_reason: CaptureReason,
    consecutive_silent_captures: u32,  // 连续静默超时捕获次数
    consecutive_change_captures: u32,  // 连续屏幕变化捕获次数
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CaptureReason {
    ScreenChanged,
    SilentTimeout,
    ManualTrigger,
}
```

### 智能调整算法设计

```rust
/// 基于用户行为模式计算最佳静默阈值
fn calculate_optimal_silent_minutes(tracker: &SilentPatternTracker) -> u64 {
    const MIN_THRESHOLD: u64 = 10;   // 最小 10 分钟
    const MAX_THRESHOLD: u64 = 60;   // 最大 60 分钟
    const DEFAULT: u64 = 30;         // 默认 30 分钟

    // 分析最近 24 小时的捕获模式
    let stats = tracker.get_recent_stats(Duration::from_secs(24 * 60 * 60));

    // 如果数据不足，返回默认值
    if stats.total_captures < 10 {
        return DEFAULT;
    }

    // 计算静默超时触发比例
    let silent_ratio = stats.silent_timeout_captures as f64 / stats.total_captures as f64;

    // 高静默比例 = 用户专注工作，提高阈值
    // 低静默比例 = 用户活跃工作，降低阈值
    let adjustment = if silent_ratio > 0.7 {
        // 超过 70% 是静默超时，说明屏幕很少变化，用户可能在深度工作
        (stats.current_threshold + 5).min(MAX_THRESHOLD)
    } else if silent_ratio < 0.3 {
        // 少于 30% 是静默超时，说明屏幕变化频繁，用户活跃
        (stats.current_threshold.saturating_sub(5)).max(MIN_THRESHOLD)
    } else {
        // 平衡状态，保持当前阈值
        stats.current_threshold
    };

    adjustment
}
```

### 数据库 Schema 扩展

```sql
-- 在 settings 表添加新字段
ALTER TABLE settings ADD COLUMN auto_adjust_silent INTEGER DEFAULT 1;
ALTER TABLE settings ADD COLUMN silent_adjustment_paused_until TEXT DEFAULT NULL;
```

**Settings 结构体扩展：**
```rust
pub struct Settings {
    // ... 现有字段 ...
    pub auto_adjust_silent: Option<bool>,           // 是否自动调整
    pub silent_adjustment_paused_until: Option<String>, // 暂停截止时间 (RFC3339)
}
```

**可选：静默模式历史表（用于长期学习）**
```sql
CREATE TABLE silent_patterns (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL,                    -- 日期 YYYY-MM-DD
    hour INTEGER NOT NULL,                 -- 小时 0-23
    silent_captures INTEGER DEFAULT 0,     -- 静默超时捕获次数
    change_captures INTEGER DEFAULT 0,     -- 屏幕变化捕获次数
    avg_interval_minutes REAL,             -- 平均捕获间隔
    UNIQUE(date, hour)
);
```

### 行为模式追踪器设计

```rust
/// 内存中跟踪用户捕获行为模式
struct SilentPatternTracker {
    // 滑动窗口：最近 7 天的每小时统计
    hourly_stats: Vec<HourlyStats>,
    // 当前调整状态
    current_threshold: u64,
    last_adjustment: Option<Instant>,
}

struct HourlyStats {
    date: chrono::NaiveDate,
    hour: u8,
    silent_captures: u32,
    change_captures: u32,
}

impl SilentPatternTracker {
    fn record_capture(&mut self, reason: CaptureReason) { ... }
    fn get_recent_stats(&self, duration: Duration) -> CaptureStats { ... }
    fn should_adjust(&self) -> bool { ... }
    fn get_adjustment(&self) -> i32 { ... }  // 正数=增加，负数=减少
}
```

### 文件结构要求

**修改文件：**
```
src-tauri/src/
├── main.rs                    # 注册新命令（如有）
├── auto_perception/
│   ├── mod.rs                 # 添加 SilentPatternTracker
│   └── silent_tracker.rs      # 新增：行为模式追踪模块
└── memory_storage/
    └── mod.rs                 # 添加新设置字段

src/
├── components/
│   └── SettingsModal.vue      # 添加静默阈值设置 UI
```

### 测试要求

**Rust 测试重点：**
1. `calculate_optimal_silent_minutes()` 各种输入场景
2. 滑动窗口统计准确性
3. 调整上下限边界测试
4. 手动覆盖优先级测试

**边界测试：**
1. 无捕获数据时的默认行为
2. 单一触发类型（全静默/全变化）时的调整
3. 调整后立即检查是否在边界内
4. 暂停时间过期后恢复自动调整

### 通知实现

使用 Tauri 的事件系统发送通知：
```rust
// 在阈值调整时
app_handle.emit("silent-threshold-adjusted", ThresholdAdjustment {
    old_value: old_threshold,
    new_value: new_threshold,
    reason: adjustment_reason,
})?;
```

前端监听：
```typescript
import { listen } from '@tauri-apps/api/event';

listen('silent-threshold-adjusted', (event) => {
    // 显示 Toast 通知
});
```

## Previous Story Intelligence

### 从 SMART-001 学习的经验

1. **跨平台代码**：使用 `#[cfg(target_os = "...")]` 条件编译
2. **窗口检测**：已实现 `get_active_window()` 可复用
3. **设置扩展模式**：`ALTER TABLE` + 结构体更新

### 从 CORE-001 学习的经验

1. **设置保存模式**：成功后 800ms 自动关闭，显示绿色勾号
2. **错误处理**：使用 Toast 组件显示错误

### 从 CORE-003 学习的经验

1. **数据库迁移**：使用 `let _ = conn.execute()` 忽略已存在列错误
2. **测试模式**：每个 AC 对应多个测试用例

### 从 CORE-004 学习的经验

1. **错误处理模式**：使用 `src/utils/errors.js` 解析错误类型
2. **Toast 组件**：`src/components/Toast.vue` 支持重试功能

### 从 CORE-005 学习的经验

1. **系统级操作**：需要考虑权限和错误处理
2. **托盘集成**：可在托盘菜单显示当前状态

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
- `chrono` - 已用于时间处理
- `serde_json` - 已用于 JSON 处理

## References

- [Source: architecture.md#2.2 后端模块] - auto_perception 职责描述
- [Source: architecture.md#4. 关键设计决策] - 截图去重优化设计
- [Source: architecture.md#5. 数据库设计] - settings 表结构
- [Source: PRD.md#6.1 自动感知] - 自动捕获功能需求
- [Source: epics.md#Epic 2] - 所属 Epic 信息
- [Source: src-tauri/src/auto_perception/mod.rs:69-102] - should_capture 实现
- [Source: src-tauri/src/auto_perception/mod.rs:21-31] - ScreenState 结构
- [Source: src-tauri/src/memory_storage/mod.rs] - 设置存储
- [Source: CLAUDE.md] - 项目开发规范

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List