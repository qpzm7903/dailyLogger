# Story 2.3: 工作时间自动识别

Status: ready-for-dev

## Story

作为一个 DailyLogger 用户，
我希望系统能自动识别我的工作时间模式，在非工作时间自动暂停捕获，
以便节省资源并避免记录无关的活动，同时支持我手动设置工作时间。

## Acceptance Criteria

### AC1 - 学习用户工作时段
- Given 自动捕获功能运行超过 7 天
- When 系统收集了足够的活动数据
- Then 计算出用户的典型工作时间（如 09:00-12:00, 14:00-18:00）
- Given 系统检测到用户在非工作时间有活动
- When 活动持续超过 30 分钟
- Then 识别为临时工作，不改变长期模式

### AC2 - 自动启停捕获
- Given 用户启用了"自动识别工作时间"功能
- When 当前时间在识别出的工作时间内
- Then 自动捕获功能正常运行
- Given 用户启用了"自动识别工作时间"功能
- When 当前时间在识别出的非工作时间
- Then 自动暂停捕获，托盘图标显示休眠状态
- Given 系统检测到用户在非工作时间开始工作
- When 用户手动触发捕获或窗口活动增加
- Then 自动恢复捕获直到工作结束

### AC3 - 支持手动设置工作时间
- Given 用户在设置中配置了自定义工作时间
- When 用户启用"使用自定义工作时间"选项
- Then 系统使用用户设置的时间段，不自动学习
- Given 用户未启用自定义工作时间
- When 系统学习模式运行
- Then 根据学习结果调整工作时间范围
- Given 用户想要查看学习结果
- When 用户打开设置界面
- Then 显示当前识别的工作时间和学习进度

## Tasks / Subtasks

- [ ] Task 1: 实现工作时间模式学习 (AC: 1)
  - [ ] 创建 `WorkTimePatternLearner` 结构体
  - [ ] 记录每次捕获的时间戳，按小时统计活动分布
  - [ ] 实现滑动窗口统计（最近 14 天）
  - [ ] 设计学习算法：识别高频活动时段
  - [ ] 编写学习算法单元测试

- [ ] Task 2: 扩展数据库 Schema (AC: 1, 2, 3)
  - [ ] 在 settings 表添加 `auto_detect_work_time` 字段（布尔值，默认开启）
  - [ ] 在 settings 表添加 `use_custom_work_time` 字段（布尔值）
  - [ ] 在 settings 表添加 `custom_work_time_start` 字段（时间字符串，如 "09:00"）
  - [ ] 在 settings 表添加 `custom_work_time_end` 字段（时间字符串，如 "18:00"）
  - [ ] 在 settings 表添加 `learned_work_time` 字段（JSON，存储学习结果）
  - [ ] 创建 `work_time_history` 表存储每日活动分布（可选）
  - [ ] 更新 `Settings` 结构体添加新字段
  - [ ] 编写数据库迁移测试

- [ ] Task 3: 实现工作时间判断逻辑 (AC: 2)
  - [ ] 创建 `is_work_time()` 函数
  - [ ] 实现时间段匹配逻辑（支持跨午夜）
  - [ ] 实现学习结果解析
  - [ ] 处理边界情况（午休、加班）
  - [ ] 编写边界测试

- [ ] Task 4: 集成到捕获流程 (AC: 1, 2)
  - [ ] 修改 `start_auto_capture()` 检查工作时间
  - [ ] 在捕获循环中检查当前时间是否在工作时间内
  - [ ] 实现自动暂停/恢复逻辑
  - [ ] 更新托盘图标状态显示休眠/活跃
  - [ ] 记录工作时间变化日志

- [ ] Task 5: 前端设置界面支持 (AC: 3)
  - [ ] 添加"自动识别工作时间"开关
  - [ ] 添加"使用自定义工作时间"开关
  - [ ] 添加自定义工作时间选择器（开始/结束时间）
  - [ ] 显示当前学习到的工作时间（只读显示）
  - [ ] 显示学习进度指示器（可选）

- [ ] Task 6: 编写测试 (AC: 1, 2, 3)
  - [ ] 学习算法单元测试
  - [ ] 时间判断边界测试
  - [ ] 跨午夜时间段测试
  - [ ] 手动覆盖优先级测试
  - [ ] 集成测试

## Dev Notes

### ⚠️ CRITICAL: Code Reuse from SMART-002

**MUST REUSE the following patterns from `src-tauri/src/silent_tracker.rs`:**

1. **`HourlyStats` structure** - Already tracks captures per hour. EXTEND it to add work time detection:
   ```rust
   // Existing in silent_tracker.rs (lines 46-56):
   pub struct HourlyStats {
       pub date: NaiveDate,
       pub hour: u8,
       pub silent_captures: u32,
       pub change_captures: u32,
   }
   // ADD: pub is_work_hour: bool, // Determined by activity threshold
   ```

2. **Global tracker pattern** - Use the same `Lazy<Mutex<...>>` pattern:
   ```rust
   // Pattern from silent_tracker.rs (line 317-318):
   static SILENT_PATTERN_TRACKER: Lazy<Mutex<SilentPatternTracker>> =
       Lazy::new(|| Mutex::new(SilentPatternTracker::default()));
   ```

3. **Sliding window pruning** - Reuse `prune_old_entries()` logic (lines 184-187)

4. **Settings field migration pattern** - Follow SMART-002's approach:
   - `let _ = conn.execute("ALTER TABLE...")` for idempotent migration
   - Update Settings struct, SELECT query, UPDATE query, and test helpers

### 技术需求

1. **时间模式学习** - 统计每小时的活动频率，识别活跃时段
2. **跨午夜处理** - 支持夜班用户（如 22:00-06:00）
3. **渐进式学习** - 避免单日异常影响整体模式
4. **状态可视化** - 托盘图标反映当前捕获状态

### 架构合规要求

- 后端命令注册在 `main.rs` 的 `generate_handler![]`
- 使用 `memory_storage::get_settings_sync()` 获取设置
- 使用 `AUTO_CAPTURE_RUNNING` 原子变量控制捕获状态
- 错误消息使用中文

### 现有实现分析

**当前捕获循环（auto_perception/mod.rs:490-517）：**
```rust
tokio::spawn(async move {
    // Execute immediately on start
    if let Err(e) = capture_and_store().await {
        tracing::error!("Initial capture failed: {}", e);
    }

    loop {
        tokio::time::sleep(Duration::from_secs(interval_minutes * 60)).await;

        if !AUTO_CAPTURE_RUNNING.load(Ordering::SeqCst) {
            tracing::info!("Auto capture stopped");
            break;
        }

        if let Err(e) = capture_and_store().await {
            tracing::error!("Auto capture failed: {}", e);
        }
    }
});
```

**需要添加工作时间检查：**
```rust
loop {
    tokio::time::sleep(Duration::from_secs(interval_minutes * 60)).await;

    if !AUTO_CAPTURE_RUNNING.load(Ordering::SeqCst) {
        break;
    }

    // 新增：检查是否在工作时间内
    if !is_in_work_time() {
        tracing::debug!("Outside work time, skipping capture");
        continue;
    }

    if let Err(e) = capture_and_store().await {
        tracing::error!("Auto capture failed: {}", e);
    }
}
```

### 工作时间学习算法设计

```rust
/// 工作时间模式学习者
struct WorkTimePatternLearner {
    // 最近 14 天的每小时活动统计
    hourly_activity: Vec<HourlyActivity>,
}

#[derive(Debug, Clone)]
struct HourlyActivity {
    hour: u8,           // 0-23
    active_days: u32,   // 该小时有活动的天数
    total_days: u32,    // 统计的总天数
}

impl WorkTimePatternLearner {
    /// 判断某小时是否属于工作时间
    fn is_work_hour(&self, hour: u8, threshold: f64) -> bool {
        if let Some(activity) = self.hourly_activity.iter().find(|a| a.hour == hour) {
            let ratio = activity.active_days as f64 / activity.total_days as f64;
            ratio >= threshold // 默认阈值 0.6 (60% 的日子在该时段有活动)
        } else {
            false
        }
    }

    /// 获取识别的工作时间段
    fn get_work_periods(&self) -> Vec<TimePeriod> {
        // 连续的工作小时合并为时段
        let mut periods = Vec::new();
        let mut current_start: Option<u8> = None;

        for hour in 0..24 {
            if self.is_work_hour(hour, 0.6) {
                if current_start.is_none() {
                    current_start = Some(hour);
                }
            } else if let Some(start) = current_start.take() {
                periods.push(TimePeriod {
                    start,
                    end: hour,
                });
            }
        }

        // 处理跨午夜情况
        if let Some(start) = current_start {
            periods.push(TimePeriod { start, end: 24 });
        }

        periods
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TimePeriod {
    start: u8,  // 开始小时 0-23
    end: u8,    // 结束小时 0-24 (不含)
}
```

### 数据库 Schema 扩展

```sql
-- 在 settings 表添加新字段
ALTER TABLE settings ADD COLUMN auto_detect_work_time INTEGER DEFAULT 1;
ALTER TABLE settings ADD COLUMN use_custom_work_time INTEGER DEFAULT 0;
ALTER TABLE settings ADD COLUMN custom_work_time_start TEXT DEFAULT '09:00';
ALTER TABLE settings ADD COLUMN custom_work_time_end TEXT DEFAULT '18:00';
ALTER TABLE settings ADD COLUMN learned_work_time TEXT DEFAULT NULL;  -- JSON
```

**Settings 结构体扩展：**
```rust
pub struct Settings {
    // ... 现有字段 ...
    pub auto_detect_work_time: Option<bool>,
    pub use_custom_work_time: Option<bool>,
    pub custom_work_time_start: Option<String>,    // "HH:MM" 格式
    pub custom_work_time_end: Option<String>,
    pub learned_work_time: Option<String>,         // JSON: {"periods": [{"start": 9, "end": 12}, ...]}
}
```

**可选：每日活动历史表（用于详细学习）**
```sql
CREATE TABLE work_time_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL,              -- 日期 YYYY-MM-DD
    hour INTEGER NOT NULL,           -- 小时 0-23
    capture_count INTEGER DEFAULT 0, -- 该小时的捕获次数
    UNIQUE(date, hour)
);
```

### 工作时间判断函数

```rust
/// 判断当前时间是否在工作时间内
fn is_in_work_time() -> bool {
    let settings = match memory_storage::get_settings_sync() {
        Ok(s) => s,
        Err(_) => return true, // 获取设置失败时默认允许捕获
    };

    // 1. 如果禁用自动检测，始终允许
    if settings.auto_detect_work_time.unwrap_or(true) == false {
        return true;
    }

    // 2. 如果使用自定义时间
    if settings.use_custom_work_time.unwrap_or(false) {
        return is_in_custom_work_time(&settings);
    }

    // 3. 使用学习到的工作时间
    is_in_learned_work_time(&settings)
}

fn is_in_custom_work_time(settings: &Settings) -> bool {
    let now = chrono::Local::now();
    let current_minutes = now.hour() * 60 + now.minute();

    let start = parse_time(settings.custom_work_time_start.as_deref(), 9 * 60);
    let end = parse_time(settings.custom_work_time_end.as_deref(), 18 * 60);

    if start <= end {
        // 正常时间段（如 09:00-18:00）
        current_minutes >= start && current_minutes < end
    } else {
        // 跨午夜时间段（如 22:00-06:00）
        current_minutes >= start || current_minutes < end
    }
}

fn parse_time(time_str: Option<&str>, default: u32) -> u32 {
    time_str
        .and_then(|s| {
            let parts: Vec<&str> = s.split(':').collect();
            if parts.len() == 2 {
                let hours: u32 = parts[0].parse().ok()?;
                let minutes: u32 = parts[1].parse().ok()?;
                Some(hours * 60 + minutes)
            } else {
                None
            }
        })
        .unwrap_or(default)
}
```

### 托盘图标状态更新

需要在 `main.rs` 中更新托盘图标以反映工作状态：

```rust
// 托盘图标状态
enum TrayIconState {
    Active,     // 捕获中
    Sleeping,   // 非工作时间，休眠中
    Paused,     // 用户手动暂停
}

fn update_tray_icon(state: TrayIconState) {
    let icon_path = match state {
        TrayIconState::Active => "icons/32x32.png",
        TrayIconState::Sleeping => "icons/sleeping.png",
        TrayIconState::Paused => "icons/paused.png",
    };
    // 更新托盘图标...
}
```

### 文件结构要求

**修改文件：**
```
src-tauri/src/
├── main.rs                    # 更新托盘图标逻辑
├── auto_perception/
│   ├── mod.rs                 # 添加工作时间判断
│   └── work_time.rs           # 新增：工作时间学习模块
└── memory_storage/
    └── mod.rs                 # 添加新设置字段

src/
├── components/
│   └── SettingsModal.vue      # 添加工作时间设置 UI
```

### 测试要求

**Rust 测试重点：**
1. `is_in_work_time()` 各种时间场景
2. 学习算法准确性测试
3. 跨午夜时间段测试
4. 手动覆盖优先级测试
5. 边界值测试（开始时间=结束时间）

**边界测试：**
1. 00:00 时刻的处理
2. 跨午夜时间段（23:00-01:00）
3. 全天工作时间（00:00-24:00）
4. 学习数据不足时的默认行为
5. 无效时间格式处理

### 学习进度显示

```typescript
// 前端显示学习进度
interface WorkTimeLearningStatus {
  daysLearned: number;       // 已学习天数
  minDaysRequired: number;   // 最小学习天数（7天）
  confidence: number;        // 学习置信度 0-1
  detectedPeriods: TimePeriod[];  // 检测到的时段
}
```

## Previous Story Intelligence

### 从 SMART-001 学习的经验

1. **跨平台代码**：使用 `#[cfg(target_os = "...")]` 条件编译
2. **设置扩展模式**：`ALTER TABLE` + 结构体更新
3. **窗口检测**：已实现 `get_active_window()` 可复用
4. **数据库迁移**：使用 `let _ = conn.execute()` 忽略已存在列错误

### 从 SMART-002 学习的经验

1. **行为模式学习**：滑动窗口统计模式
2. **自动调整逻辑**：渐进式调整，避免剧烈变化
3. **暂停恢复机制**：使用时间戳判断暂停状态
4. **通知机制**：使用 Tauri 事件系统

### 从 CORE-001 学习的经验

1. **设置保存模式**：成功后 800ms 自动关闭，显示绿色勾号
2. **错误处理**：使用 Toast 组件显示错误
3. **Tailwind 类名**：`text-red-400` 用于错误文字

### 从 CORE-003 学习的经验

1. **数据库迁移**：每个新字段单独 ALTER TABLE
2. **测试模式**：每个 AC 对应多个测试用例

### 从 CORE-005 学习的经验

1. **系统级操作**：托盘图标状态更新
2. **全局状态**：使用 `Lazy<Mutex<...>>` 模式

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
- `once_cell` - 用于 Lazy 静态变量

## References

- [Source: architecture.md#2.2 后端模块] - auto_perception 职责描述
- [Source: architecture.md#4. 关键设计决策] - 截图去重优化设计
- [Source: architecture.md#5. 数据库设计] - settings 表结构
- [Source: PRD.md#6.1 自动感知] - 自动捕获功能需求
- [Source: epics.md#Epic 2] - 所属 Epic 信息
- [Source: src-tauri/src/auto_perception/mod.rs:490-517] - 当前捕获循环
- [Source: src-tauri/src/memory_storage/mod.rs] - 设置存储
- [Source: CLAUDE.md] - 项目开发规范
- [Source: SMART-001.md] - 前序故事参考
- [Source: SMART-002.md] - 前序故事参考

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List